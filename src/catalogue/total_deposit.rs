use super::no_history;
use crate::format::{amount, pct};
use crate::types::{CliError, Context, FactJson, Reads};

/// Total deposits and their 30-day change.
pub fn facts(reads: &Reads, _ctx: &Context) -> Result<Vec<FactJson>, CliError> {
    let (decimals, symbol) = (reads.text("decimals")?.parse()?, reads.text("symbol")?);
    let total = FactJson::rpc("tvl.totalAssets", reads.read("totalAssets")?, Some(decimals), Some(symbol), amount(reads.uint("totalAssets")?, decimals, symbol));
    let change = if let Some(reason) = no_history(reads, "30d")? {
        FactJson::error("tvl.change.30d", &reason)
    } else {
        let change = reads.float("totalAssets")? / reads.float("totalAssets@30d")? - 1.0;
        let inputs = vec!["totalAssets".into(), "totalAssets@30d".into()];
        FactJson::derived("tvl.change.30d", format!("{change:.6}"), "ratio", pct(change), "totalAssets / totalAssets@30d − 1", inputs)
    };
    Ok(vec![total, change])
}
