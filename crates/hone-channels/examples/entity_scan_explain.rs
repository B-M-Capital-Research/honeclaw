//! Explain the deterministic entity scan without provider or model calls.
//!
//! Usage:
//!   cargo run -p hone-channels --example entity_scan_explain -- \
//!     --origin scheduled "股票代码 AAPL 现在多少钱"

use anyhow::{Result, bail};
use hone_channels::AgentTurnOrigin;

fn parse_origin(value: &str) -> Result<AgentTurnOrigin> {
    match value {
        "interactive" => Ok(AgentTurnOrigin::Interactive),
        "scheduled" => Ok(AgentTurnOrigin::Scheduled),
        "heartbeat" => Ok(AgentTurnOrigin::Heartbeat),
        _ => bail!("--origin must be interactive, scheduled, or heartbeat"),
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--origin") {
        bail!("usage: entity_scan_explain --origin <interactive|scheduled|heartbeat> <input>");
    }
    let origin = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing value for --origin"))
        .and_then(|value| parse_origin(&value))?;
    let input = args.collect::<Vec<_>>().join(" ");
    if input.trim().is_empty() {
        bail!("missing input text");
    }

    println!(
        "{}",
        hone_channels::diagnostics::explain_entity_scope(&input, origin)
    );
    Ok(())
}
