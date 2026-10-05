use super::{inputs, markets};
use crate::format::pct;
use crate::types::{CliError, Context, FactJson, Reads};
use serde_json::{Value, json};

/// Markets backing the deposit and the largest single share.
pub fn facts(reads: &Reads, _ctx: &Context) -> Result<Vec<FactJson>, CliError> {
    let total = reads.float("totalAssets")?;
    let markets = markets(reads)?;
    // Edge case: markets below 0.01% of total assets are dust and left out.
    let kept: Vec<_> = markets.iter().map(|market| (market, f64::from(market.supply) / total)).filter(|(_, share)| *share >= 0.0001).collect();
    let idle = reads.float("idle")? / total;
    let mut list: Vec<Value> = kept
        .iter()
        .map(|(m, share)| json!({ "marketId": m.id, "collateralSymbol": m.collateral, "lltv": pct(m.lltv), "utilization": pct(m.utilization), "supplyApy": pct(m.supply_apy), "share": pct(*share), "marketLastUpdate": m.last_update }))
        .collect();
    list.push(json!({ "marketId": "idle", "share": pct(idle) }));
    let top = kept.iter().map(|(_, share)| *share).fold(0.0, f64::max);
    let display = format!("{} markets ({} dust omitted), idle {}", kept.len(), markets.len() - kept.len(), pct(idle));
    let all = 0..markets.len();
    let list_inputs = inputs(&["totalAssets", "idle"], all.clone(), &["id", "supply", "params", "state", "collateral", "rate"]);
    Ok(vec![
        FactJson::derived("exposure.markets", list, "list", display, "per market: supply_i / totalAssets, utilization = marketBorrow / marketSupply", list_inputs),
        FactJson::derived("exposure.topMarketShare", format!("{top:.6}"), "ratio", pct(top), "max(supply_i / totalAssets)", inputs(&["totalAssets"], all, &["supply"])),
    ])
}
