pub mod apy;
pub mod exposure;
pub mod liquidity;
pub mod metadata;
pub mod performance;
pub mod total_deposit;

use crate::rpc::call;
use crate::types::{CliError, Context, FactJson, Read, Reads, VaultJson};
use alloy::primitives::{Address, U256};
use alloy::providers::DynProvider;

/// Seconds per year, as in the Morpho docs.
pub const YEAR: f64 = 31_536_000.0;

/// A fact group: builds its facts from one vault's reads.
type Group = fn(&Reads, &Context) -> Result<Vec<FactJson>, CliError>;

/// Every fact group, in output order: to add one, create `<group>.rs` in this folder with a
/// `facts` function, declare it with `pub mod` above, list it here, and add any new read it
/// needs to `reads`.
const GROUPS: &[Group] = &[metadata::facts, apy::facts, total_deposit::facts, liquidity::facts, exposure::facts, performance::facts];

/// Reads one vault once, then builds every fact from those reads, ids prefixed with `label`.
pub async fn evidence(provider: &DynProvider, ctx: &Context, label: &str, vault: Address) -> Result<VaultJson, CliError> {
    let reads = reads(provider, ctx, vault).await?;
    let mut facts = vec![];
    for group in GROUPS {
        facts.extend(group(&reads, ctx)?);
    }
    for fact in &mut facts {
        fact.id = format!("{label}.{}", fact.id);
    }
    Ok(VaultJson { address: vault, reads: reads.0, facts })
}

/// Makes every read the fact groups need for one vault.
///
/// Edge case: a past-block read fails when the vault did not exist then, and is left out.
async fn reads(provider: &DynProvider, ctx: &Context, vault: Address) -> Result<Reads, CliError> {
    let block = ctx.block;
    let mut reads = Reads(vec![]);
    for (name, signature) in [
        ("name", "name()(string)"),
        ("asset", "asset()(address)"),
        ("totalAssets", "totalAssets()(uint256)"),
        ("performanceFee", "performanceFee()(uint96)"),
        ("managementFee", "managementFee()(uint96)"),
        ("maxRate", "maxRate()(uint64)"),
        ("liquidityAdapter", "liquidityAdapter()(address)"),
        ("liquidityData", "liquidityData()(bytes)"),
        ("shareDecimals", "decimals()(uint8)"),
    ] {
        reads.0.push(call(provider, name, vault, signature, &[], block).await?);
    }
    let token = reads.text("asset")?.parse()?;
    reads.0.push(call(provider, "decimals", token, "decimals()(uint8)", &[], block).await?);
    reads.0.push(call(provider, "symbol", token, "symbol()(string)", &[], block).await?);
    reads.0.push(call(provider, "idle", token, "balanceOf(address)(uint256)", &[vault.to_string()], block).await?);

    // Share price is quoted for 10^(shareDecimals + 12) shares: one whole share of a 6-decimal
    // asset is worth only 7 digits. Only the ratio of two share prices is used.
    let shares = [format!("1{}", "0".repeat(reads.text("shareDecimals")?.parse::<usize>()? + 12))];
    reads.0.push(call(provider, "sharePrice", vault, "convertToAssets(uint256)(uint256)", &shares, block).await?);
    for (window, past_block) in [("30d", ctx.block_30d), ("90d", ctx.block_90d)] {
        if let (Ok(price), Ok(total)) = (
            call(provider, &format!("sharePrice@{window}"), vault, "convertToAssets(uint256)(uint256)", &shares, past_block).await,
            call(provider, &format!("totalAssets@{window}"), vault, "totalAssets()(uint256)", &[], past_block).await,
        ) {
            reads.0.extend([price, total]);
        }
    }

    reads.0.extend(walk(provider, vault, block).await?);
    Ok(reads)
}

/// Reads every market of the vault's adapters, named `m<k>.<key>` in walk order.
///
/// Edge case: only `MorphoMarketV1AdapterV2` adapters and markets with collateral are
/// supported; anything else fails a read.
async fn walk(provider: &DynProvider, vault: Address, block: u64) -> Result<Vec<Read>, CliError> {
    let adapters = call(provider, "adaptersLength", vault, "adaptersLength()(uint256)", &[], block).await?;
    let adapter_count = adapters.result[0].parse::<u64>()?;
    let mut reads = vec![adapters];
    let mut k = 0;
    for i in 0..adapter_count {
        let adapter = call(provider, &format!("adapter{i}"), vault, "adapters(uint256)(address)", &[i.to_string()], block).await?;
        let adapter_address = adapter.result[0].parse()?;
        let morpho = call(provider, &format!("adapter{i}.morpho"), adapter_address, "morpho()(address)", &[], block).await?;
        let morpho_address = morpho.result[0].parse()?;
        let markets = call(provider, &format!("adapter{i}.marketIdsLength"), adapter_address, "marketIdsLength()(uint256)", &[], block).await?;
        let market_count = markets.result[0].parse::<u64>()?;
        reads.extend([adapter, morpho, markets]);
        for j in 0..market_count {
            let id = call(provider, &format!("m{k}.id"), adapter_address, "marketIds(uint256)(bytes32)", &[j.to_string()], block).await?;
            let key = [id.result[0].clone()];
            let supply = call(provider, &format!("m{k}.supply"), adapter_address, "expectedSupplyAssets(bytes32)(uint256)", &key, block).await?;
            let params = call(provider, &format!("m{k}.params"), morpho_address, "idToMarketParams(bytes32)(address,address,address,address,uint256)", &key, block).await?;
            let state = call(provider, &format!("m{k}.state"), morpho_address, "market(bytes32)(uint128,uint128,uint128,uint128,uint128,uint128)", &key, block).await?;
            let collateral = call(provider, &format!("m{k}.collateral"), params.result[1].parse()?, "symbol()(string)", &[], block).await?;
            let rate_args = [format!("({})", params.result.join(",")), format!("({})", state.result.join(","))];
            let rate = call(provider, &format!("m{k}.rate"), params.result[3].parse()?, "borrowRateView((address,address,address,address,uint256),(uint128,uint128,uint128,uint128,uint128,uint128))(uint256)", &rate_args, block).await?;
            reads.extend([id, supply, params, state, collateral, rate]);
            k += 1;
        }
    }
    Ok(reads)
}

/// One market the vault supplies to.
pub struct Market {
    pub id: String,
    /// `idToMarketParams`: loan token, collateral token, oracle, IRM, LLTV.
    pub params: Vec<String>,
    /// The vault's assets in the market.
    pub supply: U256,
    /// Assets the market holds that are not borrowed.
    pub cash: U256,
    pub utilization: f64,
    pub supply_apy: f64,
    pub lltv: f64,
    pub collateral: String,
    pub last_update: String,
}

/// The markets read by `walk`. Supply APY follows the Morpho docs:
/// `(exp(borrowRate × YEAR) − 1) × utilization × (1 − marketFee)`.
///
/// # Errors
/// A market read is missing or unparsable.
pub fn markets(reads: &Reads) -> Result<Vec<Market>, CliError> {
    let mut markets = vec![];
    for k in (0..).take_while(|k| reads.find(&format!("m{k}.id")).is_some()) {
        let name = |key: &str| format!("m{k}.{key}");
        let params = reads.read(&name("params"))?.result.clone();
        // market(): totalSupplyAssets, totalSupplyShares, totalBorrowAssets, totalBorrowShares, lastUpdate, fee.
        let state = reads.read(&name("state"))?.result.iter().map(|field| field.parse()).collect::<Result<Vec<U256>, _>>()?;
        let (total_supply, total_borrow) = (state[0], state[2]);
        let utilization = f64::from(total_borrow) / f64::from(total_supply);
        let borrow_apy = (wad(reads.uint(&name("rate"))?) * YEAR).exp_m1();
        markets.push(Market {
            id: reads.text(&name("id"))?.into(),
            supply: reads.uint(&name("supply"))?,
            cash: total_supply.saturating_sub(total_borrow),
            utilization,
            supply_apy: borrow_apy * utilization * (1.0 - wad(state[5])),
            lltv: wad(params[4].parse()?),
            collateral: reads.text(&name("collateral"))?.into(),
            last_update: state[4].to_string(),
            params,
        });
    }
    Ok(markets)
}

/// A WAD integer (1e18 = 100%) as a ratio.
pub fn wad(value: U256) -> f64 {
    f64::from(value) / 1e18
}

/// Read names behind a derived fact: the vault-level `names`, then the `keys` reads of each
/// market index in `markets` (`m<k>.<key>`).
pub fn inputs(names: &[&str], markets: impl IntoIterator<Item = usize>, keys: &[&str]) -> Vec<String> {
    let per_market = markets.into_iter().flat_map(|k| keys.iter().map(move |key| format!("m{k}.{key}")));
    names.iter().copied().map(String::from).chain(per_market).collect()
}

/// Why the past window (`30d`, `90d`) can't be used, if it can't.
///
/// Edge case: the vault was not deployed (its past read failed) or was empty (below one asset
/// unit) at the past block.
pub fn no_history(reads: &Reads, window: &str) -> Result<Option<String>, CliError> {
    let past = format!("totalAssets@{window}");
    if reads.find(&past).is_none() {
        return Ok(Some(format!("vault not deployed at blocks.{window}")));
    }
    let one_unit = U256::from(10).pow(reads.uint("decimals")?);
    Ok((reads.uint(&past)? < one_unit).then(|| format!("insufficient history: vault empty at blocks.{window}")))
}
