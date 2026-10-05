---
name: vault-advice
description: >
  Use this when the user wants investment advice on two Morpho Vault V2 vaults
  on Ethereum mainnet from an evidence JSON produced by the evidence CLI
  (outputs/vault_evidence_*.json). Digests the evidence into a clear
  recommendation that answers "Which vault currently looks more attractive?"
  and "How much should we trust it?", citing evidence IDs for every claim.
---

# Vault advice from evidence

You read one evidence file and give an investor a clear answer to two questions:

1. **Which vault currently looks more attractive?** A recommendation, backed by cited evidence.
2. **How much should we trust it?** A confidence level, with the reasons that raise or lower it.

This is not a due-diligence report. Do not walk through every fact or score categories. Digest the evidence, decide, and show only what drives the decision.

The input is `outputs/vault_evidence_<nameA>_<nameB>_block<N>.json` (below: `evidence.json`). Use the file the user names; otherwise the newest one in `outputs/`, and say which. It describes vaults **A** and **B** at one Ethereum mainnet block, twelve facts per vault. Read [references/evidence-schema.md](references/evidence-schema.md) before the first report: it explains each fact and its limits.

## Citation rules

- **Every fact or judgment is backed by evidence IDs** that exist in `evidence.json`, for example `[A.apy.net, B.apy.net]`: inline in the trust section, and always in the claims block.
- **`evidence.json` is the only source.** No web, API, MCP, or memory of the vault, curator, token, or news. Never infer quality from a name.
- **No new numbers.** Quote values as the fact's `display` shows them (for `exposure.markets`, the entry's fields). You may say which is higher or lower. Differences, ratios and USD conversions are not facts: do not compute them.
- **Missing is `UNVERIFIED`**, not zero, safe, or bad: `UNVERIFIED (missing: <ID or data needed>)`. A fact with `status: "error"` is missing; quote its `reason`.
- **One block.** Values are at `block`. Never present them as live or as a forecast.

## 1. Load and check

1. Stop and say why if `schema` is not `vault-evidence/1`, `chainId` is not `1`, or `vaults.A` / `vaults.B` is missing.
2. Note `block` and `blockTimestamp` for the header.
3. Compare `A.asset.symbol` and `B.asset.symbol` (token address is the fact's `source.to`). If they differ, the vaults are not like-for-like: amounts (`tvl.totalAssets`, `liquidity.availableNow`) cannot be compared, only ratios.

## 2. Decide: which vault looks more attractive?

Yield decides, risk can overrule. Work in this order:

1. **Yield now.** Who has the higher `apy.net` (native, after fees, excludes rewards)?
2. **Does the yield hold up?** Compare `perf.realized.30d` and `perf.realized.90d`. They are what depositors actually earned. If the realized ranking matches the `apy.net` ranking, the yield lead is real. If it flips, the snapshot may be flattering. A missing window (`insufficient history`, `vault not deployed`) means the vault is unproven over that window.
3. **Red flags that overrule yield.** Any of these against the higher-yield vault can flip the answer or make it a no-call:
   - a negative `perf.realized.*` (depositors lost value);
   - a clearly lower `liquidity.availableRatio` (less of the vault can exit now);
   - a clearly higher `exposure.topMarketShare`, or its top market in `exposure.markets` showing higher utilization, higher LLTV or a riskier collateral than the other vault's;
   - deposits leaving (`tvl.change.30d` negative) while the APY is high.
4. **Choose exactly one recommendation:**
   - **Prefer A** / **Prefer B**: one vault wins on yield and no red flag overrules it, or it wins on risk while yields are close.
   - **No clear winner**: yields are close and the trade-offs point both ways. Say what would tip it (for example "prefer A if exit liquidity matters more than yield").
   - **Cannot advise**: `apy.net` is missing for either vault, or the asset mismatch makes the comparison meaningless for the user's purpose.
5. **Ticket size.** If the user gives a ticket size, compare it with each `liquidity.availableNow` display value and say whether it could exit through the normal path at `block`. Market cash is shared with other lenders; when both vaults' liquidity reads point to the same market, say they draw on the same cash.

## 3. Grade trust: how much should we trust it?

Separate the two kinds of trust; investors confuse them.

**Trust in the numbers.** High by construction: every fact is an on-chain read, or a deterministic formula over on-chain reads, at a pinned block, replayable with `cast call` from its `source` or `inputs`. Say this in one sentence. Lower it only for what the schema documents: facts with `status: "error"`, `apy.net` being an approximation (`borrowRateView` averages since the market's last update), and stale market totals (`marketLastUpdate` in `exposure.markets`).

**Trust in the recommendation.** Grade it **High**, **Medium**, or **Low** with these rules:

| Grade | When |
| --- | --- |
| High | All facts behind the decision are `ok`; `apy.net`, `perf.realized.30d` and `perf.realized.90d` rank the vaults the same way; no red flag against the recommended vault |
| Medium | The ranking agrees on `apy.net` and one realized window but the other is missing or disagrees; or a trade-off exists that is not a red flag; or the recommendation is **No clear winner** |
| Low | `apy.net` and realized yield rank the vaults differently; or the recommended vault lacks 30d history; or a red flag stands against it; or a fact behind the decision has `status: "error"` |

Then list **what could change the advice**: only risks the evidence itself shows, each with its IDs (for example variable yield across overlapping markets, short history, shared liquidity, a single dominant market). Do not list data the evidence does not cover; the scope line already says what was not checked.

## Report

Keep it to one screen. Write in plain language for an investor. Outside the recommendation, IDs go in brackets at the end of sentences.

1. **Header** (one line each): evidence file, A and B with `meta.name` and address, `asset.symbol`, `block`, `blockTimestamp` (unix seconds, as-is).
2. **Recommendation**: the decision in bold (**Prefer A — <meta.name>**, etc.), then two to four sentences: the yield reason, whether realized performance confirms it, and the main trade-off. Phrase as "At block N, …". No inline evidence IDs here: the key evidence table and the claims block carry them, and every claim in this section must appear in the claims block.
3. **Key evidence** table, only the facts that drove the decision: metric | A | B | evidence IDs. Default rows: net APY, realized 30d, realized 90d, performance fee, total deposits, 30d deposit change, available liquidity (ratio), top market (collateral, share, utilization).
4. **How much to trust it**: `Confidence: High | Medium | Low`, then one sentence on trust in the numbers, one or two on why the recommendation got its grade, and the bulleted "what could change the advice" list.
5. **Scope** (one line): "Based only on on-chain state at block N. Yield is variable; rewards and governance were not checked."
6. A fenced `json` block for mechanical checking:

```json
{ "block": 26126476, "recommendation": "Prefer A", "confidence": "Medium", "claims": [
  { "claim": "A has the higher net APY", "evidence": ["A.apy.net", "B.apy.net"] }
] }
```

Write it to `outputs/vault_advice_<nameA>_<nameB>_block<N>.md` (same names and block as the evidence file), print the recommendation and confidence in the chat, and give the path.

## Third-party opinion (optional)

Only if the user asks. You may run the Philidor `vault-due-diligence` skill on the same two addresses and add a separate section "Third-party opinion — Philidor (not evidence)", noting its data comes from Philidor's index, not from `block`. It never supplies an evidence ID and never changes the recommendation or confidence.

## Done when

- The report answers both questions: one recommendation from the list above, and one confidence grade with its reasons.
- Every fact or judgment is covered by a claim in the claims block (and, outside the recommendation, cited inline) with IDs that exist in `evidence.json`, and every number matches a cited fact's `display` (or `exposure.markets` field).
- "What could change the advice" lists only risks shown by cited evidence. Rewards and governance are never presented as zero or safe.
- The claims block lists every claim, and no source other than `evidence.json` was used apart from a labeled Philidor section the user asked for.
