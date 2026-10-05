# Morpho vault comparison CLI

Compares the indicative yield of two Morpho Vault V2 vaults.

It only supports Ethereum mainnet.

## How it works

The work is split in two process, so that the numbers and the opinion never mix.

1. **Evidence (Rust CLI).** Reads both vaults at one pinned block through an archive RPC and saves the evidence as a json file in `outputs/vault_evidence__….json`
2. **Advice (Claude Code skill `vault-advice`).** Reads only the evidence json file and writes one recommendation with a High / Medium / Low confidence grade to `outputs/vault_advice_….md`

A small viewer (`viewer.html`, served by `view.mjs`) shows the two vaults side by side with the reads behind each fact.

## Run it

Requirements: Rust, an Ethereum mainnet **archive** RPC URL, and optionally Node.js for the viewer.

```sh
cp .env.example .env        # then set ETH_RPC_URL
cargo run --release
```

### First Process: Run the CLI
The CLI asks for two vault addresses. A pair that works well (different assets, both with 90 days of history):

For example:
- Sentora RLUSD Main: `0x6dC58a0FdfC8D694e571DC59B9A52EEEa780E6bf`
- Steakhouse Prime USDC: `0xbeef088055857739c12cd3765f20b7679def0f51`

It prints the evidence file path and opens the viewer if Node.js is installed (`node view.mjs [file]` reopens it)

### Second process: run the advice agent

For the recommendation, open the project in Claude Code and run `/vault-advice` with the name of the evidence file.
For example: `/vault-advice outputs/vault_evidence_Sentora_RLUSD_Main_Steakhouse_Prime_USDC_block<N>.json`, where `<N>` is the block the CLI printed.

## What is compared

| Group | Facts | Question |
| --- | --- | --- |
| Metadata | `meta.name`, `asset.symbol` | |
| APY | `apy.net`, `fees.performance` | APY and fees |
| Total deposits | `tvl.totalAssets`, `tvl.change.30d` | total asset and if it's growing |
| Liquidity | `liquidity.availableNow`, `liquidity.availableRatio` | Checks exitability |
| Exposure | `exposure.markets`, `exposure.topMarketShare` | checks backing and concetration |
| Performance | `perf.realized.30d`, `perf.realized.90d` | Performance last 30 and 90 days |

Formulas, their sources in the Morpho docs and contracts, and the data rules are in [`.claude/skills/vault-advice/references/evidence-schema.md`](.claude/skills/vault-advice/references/evidence-schema.md). That file is the contract between the CLI and the skill.

## Code layout

| File | Role |
| --- | --- |
| `src/main.rs` | Prompts, pins the block, writes the evidence file |
| `src/rpc.rs` | One `eth_call` from a cast-style signature, result as strings |
| `src/types.rs` | The JSON types, and `Reads`: every read of one vault, by name |
| `src/catalogue/mod.rs` | Reads a vault once (`reads`, `walk`), then runs every fact group on those reads |
| `src/catalogue/<group>.rs` | One pure `facts(&Reads, &Context)` function per group |
| `src/format.rs` | Amount and percent display strings |

Each vault is read once. Fact groups never touch the network: they are pure functions over the reads, and each derived fact lists the reads its formula uses.

## Extending it

To add a fact group:

1. Add the reads it needs to `reads` in `src/catalogue/mod.rs`.
2. Write `src/catalogue/<group>.rs` with a `facts` function and list it in `GROUPS`.
3. Document the facts in `evidence-schema.md` and add a row to `CATALOGUE` in `viewer.html`.

## Decisions and known limits

- **Mainnet, Vault V2, `MorphoMarketV1AdapterV2` only.** Another adapter type, or a market without collateral, stops the CLI with `Error!`.
- **APY is an approximation.** `borrowRateView` averages the rate since the market's last update, and market totals are as of that update. Rewards are excluded.
- **Past blocks are estimated** at 7200 blocks per day. Realized yield uses the blocks' real timestamps, so only the "30d" and "90d" labels are approximate.
- **The block is `latest`**, not `finalized`: a reorg could make a file non-replayable.
- **A failed past-block read is reported as "vault not deployed".** A rate limit or a non-archive RPC looks the same.
- **An empty vault is not handled**: with zero total assets the ratios come out as `NaN`.
- **Reads are sequential**, about 160 calls for the demo pair. Free RPC tiers rate-limit concurrent calls.
- **No tests yet.** The fact groups are pure functions, so they can be tested from a saved evidence file's reads.

## AI tools

Built with Claude Code.

- **Scope and design** were decided by hand: mainnet only, the split between evidence and advice, the choice of facts, and yield mattering more than a due-diligence checklist.
- **Claude wrote** most of the Rust, the viewer and the `vault-advice` skill, under review.
- **Claude reviewed** the catalogue formulas against the Morpho contract sources (`VaultV2.sol`, `MorphoMarketV1AdapterV2.sol`, `AdaptiveCurveIrm.sol`) and against live mainnet runs, where every derived fact was recomputed independently from the file's raw reads.
- **The recommendation itself is produced by an AI agent**, which is why it is only based on the evidence file.
