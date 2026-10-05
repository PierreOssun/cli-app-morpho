# Vault advice

- Evidence file: `outputs/vault_evidence_Sentora_RLUSD_Main_Steakhouse_Prime_USDC_block26127202.json`
- A: Sentora RLUSD Main, `0x6dc58a0fdfc8d694e571dc59b9a52eeea780e6bf`, deposit asset RLUSD
- B: Steakhouse Prime USDC, `0xbeef088055857739c12cd3765f20b7679def0f51`, deposit asset USDC
- Block: 26127202
- Block timestamp: 1791215783

## Recommendation

**Prefer B — Steakhouse Prime USDC**

At block 26127202, B's net APY is 4.27% against 2.83% for A, and B takes a 0.00% performance fee against A's 10.00%. Realized yield confirms the lead over both windows: 4.43% against 2.85% over 30 days, and 4.08% against 2.75% over 90 days. B could also pay out a larger share of its deposits through the normal withdrawal path (33.74% against 13.41%). The main trade-off is that the two vaults take different assets (USDC for B, RLUSD for A), so this compares rates and not like-for-like deposits, and B is somewhat more concentrated in its top market (55.96% against 50.42%).

## Key evidence

| Metric | A · Sentora RLUSD Main | B · Steakhouse Prime USDC | Evidence IDs |
| --- | --- | --- | --- |
| Deposit asset | RLUSD | USDC | A.asset.symbol, B.asset.symbol |
| Net APY | 2.83% | 4.27% | A.apy.net, B.apy.net |
| Realized 30d | 2.85% | 4.43% | A.perf.realized.30d, B.perf.realized.30d |
| Realized 90d | 2.75% | 4.08% | A.perf.realized.90d, B.perf.realized.90d |
| Performance fee | 10.00% | 0.00% | A.fees.performance, B.fees.performance |
| Total deposits (different tokens, not comparable) | 466,180,605.69 RLUSD | 108,190,806.86 USDC | A.tvl.totalAssets, B.tvl.totalAssets |
| 30d deposit change | 33.26% | 15.00% | A.tvl.change.30d, B.tvl.change.30d |
| Available liquidity (ratio) | 13.41% | 33.74% | A.liquidity.availableRatio, B.liquidity.availableRatio |
| Top market | kBTC, share 50.42%, utilization 90.49%, LLTV 86.00% | cbBTC, share 55.96%, utilization 89.26%, LLTV 86.00% | A.exposure.markets, B.exposure.markets, A.exposure.topMarketShare, B.exposure.topMarketShare |

## How much to trust it

Confidence: **Medium**

**The numbers.** Every fact is an on-chain read, or a deterministic formula over on-chain reads, at block 26127202, and can be replayed with `cast call` from its `source` or `inputs`; every fact in the file has status `ok`. Net APY is an approximation, and A's is the staler of the two: A's top market last updated at 1791131843 while B's top market last updated at 1791214751, against a block timestamp of 1791215783 [A.apy.net, B.apy.net, A.exposure.markets, B.exposure.markets].

**The recommendation.** Net APY and both realized windows rank the vaults the same way, and no red flag stands against B [A.apy.net, B.apy.net, A.perf.realized.30d, B.perf.realized.30d, A.perf.realized.90d, B.perf.realized.90d]. It is held at Medium, not High, because the vaults are not like-for-like: they take different assets, so preferring B also means holding USDC instead of RLUSD, which the evidence does not assess [A.asset.symbol, B.asset.symbol].

What could change the advice:

- **Different assets.** Amounts cannot be compared between the vaults, only ratios. If you must stay in RLUSD, B is not an option [A.asset.symbol, B.asset.symbol, A.tvl.totalAssets, B.tvl.totalAssets].
- **B depends on one market.** A single cbBTC market is 55.96% of B, B holds 0.00% idle, and its available liquidity comes from that same market's cash, which is shared with the market's other lenders [B.exposure.topMarketShare, B.exposure.markets, B.liquidity.availableNow].
- **A's liquidity is its idle cash only.** A's available 13.41% equals its idle share; nothing from its markets is counted in the normal withdrawal path [A.liquidity.availableRatio, A.liquidity.availableNow, A.exposure.markets].
- **Yield is variable.** Every market listed for both vaults runs at 85.96% utilization or higher, and B's current net APY (4.27%) sits between its 90-day (4.08%) and 30-day (4.43%) realized figures [A.exposure.markets, B.exposure.markets, B.apy.net, B.perf.realized.30d, B.perf.realized.90d].
- **The fee gap is as of this block.** Part of B's lead is its 0.00% performance fee against A's 10.00% [A.fees.performance, B.fees.performance].

## Scope

Based only on on-chain state at block 26127202. Yield is variable; rewards and governance were not checked.

```json
{ "block": 26127202, "recommendation": "Prefer B", "confidence": "Medium", "claims": [
  { "claim": "B has the higher net APY (4.27% against 2.83%)", "evidence": ["A.apy.net", "B.apy.net"] },
  { "claim": "B takes a 0.00% performance fee against A's 10.00%", "evidence": ["A.fees.performance", "B.fees.performance"] },
  { "claim": "B has the higher realized yield over 30 days (4.43% against 2.85%)", "evidence": ["A.perf.realized.30d", "B.perf.realized.30d"] },
  { "claim": "B has the higher realized yield over 90 days (4.08% against 2.75%)", "evidence": ["A.perf.realized.90d", "B.perf.realized.90d"] },
  { "claim": "B could pay out a larger share of its deposits through the normal withdrawal path (33.74% against 13.41%)", "evidence": ["A.liquidity.availableRatio", "B.liquidity.availableRatio"] },
  { "claim": "The vaults take different assets (RLUSD for A, USDC for B), so amounts are not comparable", "evidence": ["A.asset.symbol", "B.asset.symbol", "A.tvl.totalAssets", "B.tvl.totalAssets"] },
  { "claim": "B is more concentrated in its top market (55.96% against 50.42%)", "evidence": ["A.exposure.topMarketShare", "B.exposure.topMarketShare"] },
  { "claim": "The top markets have the same LLTV (86.00%), and B's has the lower utilization (89.26% against 90.49%)", "evidence": ["A.exposure.markets", "B.exposure.markets"] },
  { "claim": "Deposits grew in both vaults over 30 days (33.26% for A, 15.00% for B)", "evidence": ["A.tvl.change.30d", "B.tvl.change.30d"] },
  { "claim": "Every fact has status ok; net APY is an approximation and A's top market totals are staler than B's (last updates 1791131843 and 1791214751)", "evidence": ["A.apy.net", "B.apy.net", "A.exposure.markets", "B.exposure.markets"] },
  { "claim": "B holds 0.00% idle and its available liquidity comes from its top cbBTC market, whose cash is shared with other lenders", "evidence": ["B.exposure.markets", "B.exposure.topMarketShare", "B.liquidity.availableNow"] },
  { "claim": "A's available liquidity (13.41%) is its idle cash only", "evidence": ["A.liquidity.availableRatio", "A.liquidity.availableNow", "A.exposure.markets"] },
  { "claim": "Every listed market runs at 85.96% utilization or higher, and B's net APY sits between its 90-day and 30-day realized figures", "evidence": ["A.exposure.markets", "B.exposure.markets", "B.apy.net", "B.perf.realized.30d", "B.perf.realized.90d"] }
] }
```
