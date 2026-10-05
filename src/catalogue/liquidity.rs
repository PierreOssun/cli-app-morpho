use super::{inputs, markets};
use crate::format::{amount, pct};
use crate::rpc::decode;
use crate::types::{CliError, Context, FactJson, Reads};
use alloy::primitives::{Address, U256};

/// Assets a normal withdrawal could pay out now, as a ratio and an amount.
pub fn facts(reads: &Reads, _ctx: &Context) -> Result<Vec<FactJson>, CliError> {
    // Edge case: with no liquidity adapter set, only idle cash is withdrawable.
    let params = if reads.text("liquidityAdapter")? == Address::ZERO.to_string() { vec![] } else { decode("(address,address,address,address,uint256)", reads.text("liquidityData")?)? };
    let markets = markets(reads)?;
    let liquidity_market = markets.iter().position(|market| market.params == params);
    let available = reads.uint("idle")? + liquidity_market.map_or(U256::ZERO, |k| markets[k].supply.min(markets[k].cash));
    let ratio = f64::from(available) / reads.float("totalAssets")?;
    let (decimals, symbol) = (reads.text("decimals")?.parse()?, reads.text("symbol")?);
    let formula = "idle + min(supply_l, marketSupply_l − marketBorrow_l), l = the market in liquidityData of the liquidityAdapter (normal withdraw path: idle, then the liquidity adapter)";
    let inputs = inputs(&["idle", "liquidityAdapter", "liquidityData"], liquidity_market, &["params", "supply", "state"]);
    let ratio_inputs = [inputs.clone(), vec!["totalAssets".into()]].concat();
    Ok(vec![
        FactJson::derived("liquidity.availableRatio", format!("{ratio:.6}"), "ratio", pct(ratio), &format!("({formula}) / totalAssets"), ratio_inputs),
        FactJson::derived("liquidity.availableNow", available.to_string(), symbol, amount(available, decimals, symbol), formula, inputs),
    ])
}
