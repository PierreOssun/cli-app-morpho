use crate::types::{CliError, Context, FactJson, Reads};

/// Vault name and deposit asset.
pub fn facts(reads: &Reads, _ctx: &Context) -> Result<Vec<FactJson>, CliError> {
    Ok(vec![
        FactJson::rpc("meta.name", reads.read("name")?, None, None, reads.text("name")?.into()),
        FactJson::rpc("asset.symbol", reads.read("symbol")?, Some(reads.text("decimals")?.parse()?), None, reads.text("symbol")?.into()),
    ])
}
