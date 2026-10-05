use super::{YEAR, inputs, markets, wad};
use crate::format::pct;
use crate::types::{CliError, Context, FactJson, Reads};

/// Net native APY and performance fee.
pub fn facts(reads: &Reads, _ctx: &Context) -> Result<Vec<FactJson>, CliError> {
    let markets = markets(reads)?;
    let performance_fee = wad(reads.uint("performanceFee")?);
    let earned: f64 = markets.iter().map(|market| f64::from(market.supply) * market.supply_apy).sum();
    let gross = (earned / reads.float("totalAssets")?).min(wad(reads.uint("maxRate")?) * YEAR);
    let net = gross * (1.0 - performance_fee) - wad(reads.uint("managementFee")?) * YEAR;
    let formula = "min(Σ(supply_i × supplyApy_i) / totalAssets, maxRate × 31536000) × (1 − performanceFee) − managementFee × 31536000; supplyApy = (exp(borrowRate × 31536000) − 1) × utilization × (1 − marketFee); excludes rewards";
    let inputs = inputs(&["totalAssets", "maxRate", "performanceFee", "managementFee"], 0..markets.len(), &["supply", "state", "rate"]);
    Ok(vec![
        FactJson::derived("apy.net", format!("{net:.6}"), "ratio per year", pct(net), formula, inputs),
        FactJson::rpc("fees.performance", reads.read("performanceFee")?, Some(18), Some("WAD"), pct(performance_fee)),
    ])
}
