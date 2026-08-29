use std::path::PathBuf;

use hone_core::config::SecCompanyFactsConfig;
use hone_event_engine::{EventStore, SecCompanyFactsBackfiller};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./data/events.sqlite3"));
    let symbols = std::env::args()
        .nth(2)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["SNDK".into(), "MU".into(), "MSFT".into()]);
    let config = SecCompanyFactsConfig {
        enabled: true,
        symbols,
        history_filings: 12,
        refresh_hours: 24,
        user_agent: std::env::var("SEC_USER_AGENT")
            .unwrap_or_else(|_| "honeclaw event-engine ops@honeclaw.local".into()),
    };
    let store = EventStore::open(&database)?;
    let report = SecCompanyFactsBackfiller::new(config)?
        .backfill_into_store(&store)
        .await?;
    println!("{report:#?}");
    Ok(())
}
