# evidence.json — schema `vault-evidence/1`

This is the contract between the evidence CLI (process 1, `cargo run --release`) and the `vault-advice` skill (process 2). The CLI reads Ethereum mainnet only, through an archive RPC, and writes `outputs/vault_evidence_<nameA>_<nameB>_block<N>.json`. Every fact can be re-checked by replaying its reads (`to`, `call`, `args`, `block`), for example with `cast call`.

Demo scope: two facts per group, chosen for investor relevance. Twelve facts per vault.

## File shape

```json
{
  "schema": "vault-evidence/1",
  "chainId": 1,
  "block": 26126476,
  "blockTimestamp": 1791206471,
  "blocks": { "30d": 25910476, "90d": 25478476 },
  "vaults": {
    "A": { "address": "0x…", "reads": [ … ], "facts": [ … ] },
    "B": { "address": "0x…", "reads": [ … ], "facts": [ … ] }
  }
}
```

- `block` is the pinned block and `blockTimestamp` its unix time in seconds. `blocks` are the past blocks used for history, estimated at 7200 blocks per day; both vaults use the same ones, and realized yield uses their real timestamps.
- `reads` lists each vault's RPC reads once, by unique `name`:

```json
{ "name": "m0.supply", "to": "0x…", "call": "expectedSupplyAssets(bytes32)(uint256)", "args": ["0x…"], "block": 26126476, "result": ["…"] }
```

`result` holds one string per return value. Read names:

| Name | Read |
| --- | --- |
| `name` | vault `name()` |
| `asset`, `decimals`, `symbol` | vault `asset()`, then the token's `decimals()` and `symbol()` |
| `totalAssets`, `totalAssets@30d`, `totalAssets@90d` | vault `totalAssets()` at `block` and each past block |
| `idle` | token `balanceOf(vault)` |
| `performanceFee`, `managementFee`, `maxRate` | vault fees (WAD; management fee per second) and the cap on total-assets growth (WAD per second) |
| `liquidityAdapter`, `liquidityData` | the adapter that serves withdrawals, and the ABI-encoded market params of its one market |
| `adaptersLength`, `adapter<i>`, `adapter<i>.morpho`, `adapter<i>.marketIdsLength` | the vault's adapters |
| `m<k>.id`, `.supply`, `.params`, `.state`, `.collateral`, `.rate` | market `k` in walk order: id, `expectedSupplyAssets`, `idToMarketParams`, `market`, collateral `symbol()`, IRM `borrowRateView` |
| `shareDecimals`, `sharePrice`, `sharePrice@30d`, `sharePrice@90d` | vault `decimals()` and `convertToAssets(10^(shareDecimals + 12))` at `block` and each past block |

A past-block read is absent when it failed (the vault did not exist then).

## Fact shape

A raw read (`kind: "rpc"`). `source` is the read itself. `decimals` and `unit` appear when they apply:

```json
{
  "id": "A.tvl.totalAssets",
  "status": "ok",
  "kind": "rpc",
  "value": "112653256666716",
  "decimals": 6,
  "unit": "USDC",
  "display": "112,653,256.66 USDC",
  "source": { "name": "totalAssets", "to": "0x…", "call": "totalAssets()(uint256)", "args": [], "block": 26126476, "result": ["112653256666716"] }
}
```

A derived value (`kind: "derived"`). The CLI computes it deterministically; the agent never does. `formula` says how, in words; `inputs` names only the reads the formula uses, each found in the vault's `reads`:

```json
{
  "id": "A.apy.net",
  "status": "ok",
  "kind": "derived",
  "value": "0.044978",
  "unit": "ratio per year",
  "display": "4.50%",
  "formula": "Σ(supply_i × supplyApy_i) / totalAssets × (1 − performanceFee) − managementFee × 31536000; …",
  "inputs": ["totalAssets", "maxRate", "performanceFee", "managementFee", "m0.supply", "m0.state", "m0.rate", "…"]
}
```

A fact that could not be established:

```json
{ "id": "B.perf.realized.90d", "status": "error", "kind": "derived", "reason": "vault not deployed at blocks.90d" }
```

- `value` is a string: a raw integer for token amounts and WAD values, a decimal string for ratios (`0.044978` = 4.50%). Token amounts are never floats. `exposure.markets` is the one list fact; its entries hold display strings (see below).
- `display` is the human-readable form to quote.
- `status` is `ok` or `error`; an error carries `reason` and no value.
- IDs are `<A|B>.<group>.<name>`.

## Fact catalogue

Nothing is hardcoded: Morpho Blue's address comes from each adapter's `morpho()` and each market's IRM from its market params. Signatures were checked against `morpho-org/vault-v2` (commit `30ca75f`).

APY, liquidity and exposure come from one walk of the vault's markets: vault `adaptersLength()` / `adapters(i)` → adapter `morpho()`, `marketIdsLength()` / `marketIds(j)`, `expectedSupplyAssets(marketId)` (the vault's assets in that market, accrued to `block`) → Morpho `idToMarketParams(id)`, `market(id)` → collateral `symbol()` → IRM `borrowRateView(marketParams, market)`. Only `MorphoMarketV1AdapterV2` adapters are supported: a vault with another adapter type (`MorphoVaultV1Adapter`, `MidnightAdapter`) makes the CLI stop with `Error!`.

Units: a vault's share decimals are the asset's decimals plus an offset, so share price uses the vault's own `decimals()`. It is quoted for 10^12 whole shares: one whole share of a 6-decimal asset is worth only 7 digits, too few to compare two vaults' realized yields. Only the ratio of two share prices is used. WAD values are 1e18 = 100%.

| Group | ID | Kind | Source / formula | Investor question |
| --- | --- | --- | --- | --- |
| Metadata | `meta.name` | rpc | vault `name()` (display only) | Which vault is this? |
| Metadata | `asset.symbol` | rpc | token `symbol()`; the token address is `source.to`, its decimals in `decimals` | What do I deposit? Are the two vaults like-for-like? |
| APY | `apy.net` | derived | Per market: borrow APY = `exp(borrowRate × 31536000) − 1`, supply APY = `borrowAPY × utilization × (1 − marketFee)`. Gross = Σ (vault assets in market × supply APY) / `totalAssets`, idle earning 0, capped at `maxRate × 31536000`. Net = `gross × (1 − performanceFee) − managementFee × 31536000`. Excludes rewards | What do I earn now? |
| APY | `fees.performance` | rpc | vault `performanceFee()` (WAD) | How much of the yield does the curator keep? |
| Total deposits | `tvl.totalAssets` | rpc | vault `totalAssets()` at `block` | How big is it, and would my ticket dominate it? |
| Total deposits | `tvl.change.30d` | derived | `totalAssets / totalAssets@30d − 1`. Subject to the history rule | Is money arriving or leaving? |
| Liquidity | `liquidity.availableRatio` | derived | `liquidity.availableNow / totalAssets` | What share of the vault could exit now? |
| Liquidity | `liquidity.availableNow` | derived | `idle + min(supply_l, marketSupply_l − marketBorrow_l)`, where `l` is the market in `liquidityData`. That is the normal withdrawal path: idle first, then the liquidity adapter's one market. Money in other markets needs an allocator or a permissionless `forceDeallocate` (in-kind exit, with a penalty), which this fact does not count. Market cash is shared with the market's other lenders, including other vaults | Could my ticket exit today? |
| Exposure | `exposure.markets` | derived | One entry per market at or above the dust threshold: `marketId`, `collateralSymbol`, `lltv`, `utilization`, `supplyApy`, `share` (percent strings such as `"86.00%"`) and `marketLastUpdate` (unix seconds). Then one `{ "marketId": "idle", "share": … }` entry. `display` counts the markets and the dust omitted | What backs my deposit? |
| Exposure | `exposure.topMarketShare` | derived | Largest market `share`, idle excluded | How concentrated is it? |
| Performance | `perf.realized.30d` | derived | `(sharePrice / sharePrice@30d)^(31536000 / elapsed) − 1`, `elapsed` from the two blocks' real timestamps (written in `formula`). Net of fees; negative means a loss. Subject to the history rule | What did depositors actually earn recently? |
| Performance | `perf.realized.90d` | derived | Same over `blocks.90d` | Does the yield hold up over time? |

## Sources

| Formula | Source |
| --- | --- |
| Borrow APY, supply APY, utilization, `secondsPerYear` = 31,536,000 | Morpho docs: [Interest Rate Model](https://docs.morpho.org/learn/concepts/irm) and [Interest Rate Models contract](https://docs.morpho.org/developers/contracts/irm) |
| Total assets = idle + Σ adapter `realAssets()`, capped by `maxRate`; share price grows with total assets | Morpho docs: [Yield](https://docs.morpho.org/curate/concepts/yield); `VaultV2.sol` `accrueInterestView` |
| Performance fee on interest, management fee as an annual % of total assets | Morpho docs: [Fees](https://docs.morpho.org/curate/concepts/fee) |
| Withdrawals served from idle, then the liquidity adapter's one market (`liquidityData`) | Morpho docs: [Liquidity Curation](https://docs.morpho.org/curate/concepts/liquidity), [Liquidity Curation tutorial](https://docs.morpho.org/curate/tutorials-v2/liquidity-curation); `VaultV2.sol` `exit()` |
| Vault assets per market (`expectedSupplyAssets`), adapter market list | `MorphoMarketV1AdapterV2.sol` (`morpho-org/vault-v2`) |
| Realized yield annualization | No Morpho formula found; standard compounding of the on-chain share-price ratio |

Docs read on 2026-10-05. `borrowRateView` returns the IRM's average rate since the market's last update, so APY is a close approximation of the rate at `block`, not an exact one.

## Data rules

Found on the first mainnet runs (blocks 26125193 to 26126476).

- **History rule.** A past-block read that fails gives `reason: "vault not deployed at blocks.<w>"`. If `totalAssets` at the past block is below one whole unit of the asset, the vault was effectively empty (only its creation deposit), and the reason is `insufficient history: vault empty at blocks.<w>`. Both apply to `tvl.change.30d` and `perf.realized.<w>`. Without this rule, a vault that grew from 1e-9 RLUSD reported a 30-day deposit change of 2.5e19 %.
- **Dust threshold.** Markets below 0.01% of `totalAssets` are left out of `exposure.markets` and counted in its `display`. Adapters keep markets holding 1 wei in their list. Dust markets still count in `apy.net`, where they are negligible.
- **Staleness.** `market(id)` totals are stored at the market's last interaction, not accrued to `block`, so `utilization`, `supplyApy` and market cash can be slightly stale. Each `exposure.markets` entry carries `marketLastUpdate`.

## Not in the demo

Rewards, USD prices (so deposit sizes of vaults with different tokens can't be compared), gross APY and management fee as separate facts, force-exit penalties, `eligibility.*` (official factory, exit gates) and `gov.*` (owner, curator, timelocks, pending actions). The skill reports these as `UNVERIFIED`.
