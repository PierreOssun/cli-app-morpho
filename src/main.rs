mod catalogue;
mod format;
mod rpc;
mod types;

use alloy::primitives::Address;
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use std::collections::BTreeMap;
use std::io::Write;
use types::{CliError, Context, EvidenceJson, VaultJson};

#[tokio::main]
async fn main() {
    match run().await {
        Ok(path) => {
            println!("DONE! {path}");
            // Edge case: without Node.js the viewer is not opened.
            let _ = std::process::Command::new("node").args(["view.mjs", &path]).status();
        }
        Err(e) => println!("Error! {}", e.0),
    }
}

/// Prompts for two vaults, builds their evidence and writes it to `outputs/`. Returns the file path.
///
/// # Errors
/// Bad input, missing `ETH_RPC_URL`, RPC failure or write failure.
async fn run() -> Result<String, CliError> {
    println!("Build evidence data to compare 2 Morpho vaults.");
    let vault_a = ask("Please enter the first vault address: ")?;
    let vault_b = ask("Please enter the second vault address: ")?;
    let provider = ProviderBuilder::new().connect_http(rpc_url()?.parse()?).erased();
    let ctx = context(&provider).await?;
    // Reads are sequential on purpose: free RPC tiers rate-limit concurrent calls.
    let a = catalogue::evidence(&provider, &ctx, "A", vault_a).await?;
    let b = catalogue::evidence(&provider, &ctx, "B", vault_b).await?;
    let path = format!("outputs/vault_evidence_{}_{}_block{}.json", file_name(&a), file_name(&b), ctx.block);
    let evidence = EvidenceJson {
        schema: "vault-evidence/1",
        chain_id: ctx.chain_id,
        block: ctx.block,
        block_timestamp: ctx.block_timestamp,
        blocks: BTreeMap::from([("30d", ctx.block_30d), ("90d", ctx.block_90d)]),
        vaults: BTreeMap::from([("A", a), ("B", b)]),
    };
    std::fs::create_dir_all("outputs")?;
    std::fs::write(&path, serde_json::to_string_pretty(&evidence)?)?;
    Ok(path)
}

/// The vault's on-chain name with every non-alphanumeric character replaced, safe in a file name.
fn file_name(vault: &VaultJson) -> String {
    let name = vault.reads.iter().find(|read| read.name == "name");
    name.map(|read| read.result[0].replace(|c: char| !c.is_alphanumeric(), "_")).unwrap_or_default()
}

/// The RPC URL from the `ETH_RPC_URL` environment variable, else from `.env`.
///
/// # Errors
/// Neither has `ETH_RPC_URL`.
fn rpc_url() -> Result<String, CliError> {
    if let Ok(url) = std::env::var("ETH_RPC_URL") {
        return Ok(url);
    }
    let env = std::fs::read_to_string(".env").unwrap_or_default();
    let url = env.lines().find_map(|line| line.trim().strip_prefix("ETH_RPC_URL=")).ok_or("ETH_RPC_URL not set (export it or add it to .env)")?;
    Ok(url.trim().trim_matches('"').to_string())
}

/// Asks for an Ethereum address on stdin.
///
/// # Errors
/// The input is not a valid address.
fn ask(prompt: &str) -> Result<Address, CliError> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line.trim().parse()?)
}

/// Reads the chain id, the latest block and the blocks about 30 and 90 days earlier.
///
/// Edge case: past blocks assume 7200 blocks per day; their real timestamps are stored.
async fn context(provider: &DynProvider) -> Result<Context, CliError> {
    let timestamp = async |number: u64| -> Result<u64, CliError> { Ok(provider.get_block_by_number(number.into()).await?.ok_or("block not found")?.header.timestamp) };
    let block = provider.get_block_number().await?;
    let (block_30d, block_90d) = (block - 30 * 7200, block - 90 * 7200);
    Ok(Context {
        chain_id: provider.get_chain_id().await?,
        block,
        block_timestamp: timestamp(block).await?,
        block_30d,
        block_30d_timestamp: timestamp(block_30d).await?,
        block_90d,
        block_90d_timestamp: timestamp(block_90d).await?,
    })
}
