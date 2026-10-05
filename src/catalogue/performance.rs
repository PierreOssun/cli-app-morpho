use super::{YEAR, no_history};
use crate::format::pct;
use crate::types::{CliError, Context, FactJson, Reads};

/// Realized net yield over 30 and 90 days.
pub fn facts(reads: &Reads, ctx: &Context) -> Result<Vec<FactJson>, CliError> {
    let mut facts = vec![];
    for (window, past_timestamp) in [("30d", ctx.block_30d_timestamp), ("90d", ctx.block_90d_timestamp)] {
        let id = format!("perf.realized.{window}");
        if let Some(reason) = no_history(reads, window)? {
            facts.push(FactJson::error(&id, &reason));
            continue;
        }
        let past_price = format!("sharePrice@{window}");
        let elapsed = ctx.block_timestamp - past_timestamp;
        let realized = (reads.float("sharePrice")? / reads.float(&past_price)?).powf(YEAR / elapsed as f64) - 1.0;
        let formula = format!("(sharePrice / sharePrice@{window})^(31536000 / {elapsed}) − 1; elapsed = block timestamp − blocks.{window} timestamp");
        let inputs = vec!["sharePrice".into(), past_price, format!("totalAssets@{window}")];
        facts.push(FactJson::derived(&id, format!("{realized:.6}"), "ratio per year", pct(realized), &formula, inputs));
    }
    Ok(facts)
}
