//! Bounded, source-verified operating-KPI backfill.
//!
//! The manifest contains only short candidate claims and official URLs.  This
//! tool downloads each source into memory, independently verifies the emitted
//! issuer name/definition/evidence against the source and validates the normal
//! event-engine admission contract. V2 manifests pin the source SHA-256 and,
//! when writing, archive the exact bytes under an immutable content-addressed
//! object path so future review can replay the evidence. It is dry-run by
//! default; set
//! `HONE_OPERATING_KPI_BACKFILL_WRITE=1` to insert idempotent events.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use hone_event_engine::{
    EventKind, EventStore, MarketEvent, OPERATING_KPI_BACKFILL_EVENT_SCHEMA_VERSION,
    OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION, OperatingKpiClaimInput,
    OperatingKpiSourceArtifact, Severity, operating_kpi_catalog_for_symbol,
    operating_kpi_claims_from_event, operating_kpi_input_is_supported_for_symbol,
    operating_kpi_input_is_verbatim_in_source, operating_kpi_source_artifact_is_valid,
};
use reqwest::Url;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

const DEFAULT_MANIFEST: &str =
    "./tests/fixtures/event_engine/operating_kpi_backfill_storage_v1.json";
const DEFAULT_DATABASE: &str = "./data/events.sqlite3";
const DEFAULT_ARTIFACT_ROOT: &str = "./data/investment_decisions/source-artifacts/operating-kpi";
const MAX_DOCUMENTS: usize = 12;
const MAX_SOURCE_BYTES: usize = 5 * 1024 * 1024;
const OFFICIAL_HOSTS: &[&str] = &[
    "investor.sandisk.com",
    "investors.micron.com",
    "www.gevernova.com",
    "www.sec.gov",
];

#[derive(Debug, Deserialize)]
struct Manifest {
    schema_version: String,
    documents: Vec<ManifestDocument>,
}

#[derive(Debug, Deserialize)]
struct ManifestDocument {
    event_id: String,
    symbol: String,
    occurred_at: DateTime<Utc>,
    period: String,
    document_type: String,
    format: String,
    source_url: String,
    #[serde(default)]
    source_sha256: String,
    #[serde(default)]
    source_time_precision: String,
    claims: Vec<OperatingKpiClaimInput>,
}

struct VerifiedDocument {
    event: MarketEvent,
    artifact: Option<OperatingKpiSourceArtifact>,
    source_bytes: Vec<u8>,
}

struct FetchedSource {
    source_bytes: Vec<u8>,
    source_text: String,
    source_sha256: String,
    extracted_text_sha256: String,
}

#[derive(Debug, Serialize)]
struct DocumentResult {
    event_id: String,
    symbol: String,
    period: String,
    source_url: String,
    source_sha256: String,
    source_byte_length: u64,
    admitted_claims: usize,
    artifact_archived: bool,
    inserted: bool,
}

#[derive(Debug, Serialize)]
struct BackfillReport {
    schema_version: &'static str,
    manifest_version: String,
    write_enabled: bool,
    documents: usize,
    admitted_claims: usize,
    inserted_events: usize,
    archived_artifacts: usize,
    results: Vec<DocumentResult>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manifest_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MANIFEST));
    let database_path = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATABASE));
    let artifact_root = std::env::var("HONE_OPERATING_KPI_SOURCE_ARCHIVE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_ARTIFACT_ROOT));
    let write_enabled = std::env::var("HONE_OPERATING_KPI_BACKFILL_WRITE")
        .ok()
        .as_deref()
        == Some("1");
    let manifest = read_manifest(&manifest_path).await?;
    let is_v2 = manifest.schema_version == "hone-operating-kpi-backfill-manifest-v2";
    if !is_v2 && manifest.schema_version != "hone-operating-kpi-backfill-manifest-v1" {
        bail!("unsupported operating KPI backfill manifest version");
    }
    if write_enabled && !is_v2 {
        bail!("writes require a v2 manifest with a pinned source SHA-256");
    }
    if manifest.documents.is_empty() || manifest.documents.len() > MAX_DOCUMENTS {
        bail!("manifest must contain 1..={MAX_DOCUMENTS} documents");
    }
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; HONE operating-KPI research)")
        .timeout(Duration::from_secs(90))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("build source client")?;
    let store = write_enabled
        .then(|| EventStore::open(&database_path))
        .transpose()?;
    let manifest_version = manifest.schema_version;
    let mut results = Vec::new();
    for document in manifest.documents {
        let verified = build_verified_event(&client, &document, is_v2).await?;
        let admitted_claims = operating_kpi_claims_from_event(&verified.event);
        if admitted_claims.len() != document.claims.len() {
            bail!(
                "{} admitted {}/{} claims after deterministic validation",
                document.event_id,
                admitted_claims.len(),
                document.claims.len()
            );
        }
        let artifact_archived = if write_enabled {
            let artifact = verified
                .artifact
                .as_ref()
                .context("v2 write is missing source artifact metadata")?;
            archive_source_object(&artifact_root, artifact, &verified.source_bytes).await?;
            true
        } else {
            false
        };
        let inserted = store
            .as_ref()
            .map(|store| store.insert_event(&verified.event))
            .transpose()?
            .unwrap_or(false);
        results.push(DocumentResult {
            event_id: document.event_id,
            symbol: document.symbol,
            period: document.period,
            source_url: document.source_url,
            source_sha256: verified
                .artifact
                .as_ref()
                .map(|artifact| artifact.source_sha256.clone())
                .unwrap_or_else(|| sha256_hex(&verified.source_bytes)),
            source_byte_length: verified.source_bytes.len() as u64,
            admitted_claims: admitted_claims.len(),
            artifact_archived,
            inserted,
        });
    }
    let report = BackfillReport {
        schema_version: "hone-operating-kpi-backfill-report-v2-source-artifact",
        manifest_version,
        write_enabled,
        documents: results.len(),
        admitted_claims: results.iter().map(|item| item.admitted_claims).sum(),
        inserted_events: results.iter().filter(|item| item.inserted).count(),
        archived_artifacts: results.iter().filter(|item| item.artifact_archived).count(),
        results,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn read_manifest(path: &Path) -> anyhow::Result<Manifest> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("read {}", path.display()))?;
    if bytes.len() > 512 * 1024 {
        bail!("manifest exceeds 512 KiB");
    }
    serde_json::from_slice(&bytes).context("parse operating KPI manifest")
}

async fn build_verified_event(
    client: &reqwest::Client,
    document: &ManifestDocument,
    require_pinned_digest: bool,
) -> anyhow::Result<VerifiedDocument> {
    validate_manifest_document(document, require_pinned_digest)?;
    let fetched = fetch_source(client, document).await?;
    if require_pinned_digest && fetched.source_sha256 != document.source_sha256 {
        bail!(
            "{} source SHA-256 changed: expected {}, received {}",
            document.event_id,
            document.source_sha256,
            fetched.source_sha256
        );
    }
    for (index, claim) in document.claims.iter().enumerate() {
        if !operating_kpi_input_is_verbatim_in_source(claim, &fetched.source_text) {
            let source = normalize_source_for_diagnostic(&fetched.source_text);
            let mut missing = [
                ("issuer_metric_name", claim.issuer_metric_name.as_str()),
                ("issuer_definition", claim.issuer_definition.as_str()),
                ("value_text", claim.value_text.as_str()),
                ("evidence_quote", claim.evidence_quote.as_str()),
            ]
            .into_iter()
            .filter_map(|(field, value)| {
                (!source.contains(&normalize_source_for_diagnostic(value))).then_some(field)
            })
            .collect::<Vec<_>>();
            if !matches!(
                claim.claim_kind,
                hone_event_engine::OperatingKpiClaimKind::ReportedFact
            ) && !source.contains(&normalize_source_for_diagnostic(&claim.speaker))
            {
                missing.push("speaker");
            }
            bail!(
                "{} claim {index} is not verbatim in the official source; missing fields: {}",
                document.event_id,
                missing.join(", "),
            );
        }
    }
    let artifact = require_pinned_digest.then(|| OperatingKpiSourceArtifact {
        schema_version: OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION.into(),
        source_sha256: fetched.source_sha256.clone(),
        extracted_text_sha256: fetched.extracted_text_sha256.clone(),
        byte_length: fetched.source_bytes.len() as u64,
        format: document.format.clone(),
        object_path: format!("objects/{}.{}", fetched.source_sha256, document.format),
    });
    if artifact
        .as_ref()
        .is_some_and(|artifact| !operating_kpi_source_artifact_is_valid(artifact))
    {
        bail!(
            "{} produced invalid source artifact metadata",
            document.event_id
        );
    }
    let (kind, payload) = event_shape(document, artifact.as_ref())?;
    Ok(VerifiedDocument {
        event: MarketEvent {
            id: document.event_id.clone(),
            kind,
            severity: Severity::Low,
            symbols: vec![document.symbol.clone()],
            occurred_at: document.occurred_at,
            title: format!(
                "{} {} operating KPI source",
                document.symbol, document.period
            ),
            summary: format!(
                "{} source-verified issuer operating KPI claim(s); training only",
                document.claims.len()
            ),
            url: Some(document.source_url.clone()),
            source: if document.document_type.starts_with("sec_filing") {
                "sec.operating_kpi_point_in_time".into()
            } else {
                "company_ir.operating_kpi_point_in_time".into()
            },
            payload,
        },
        artifact,
        source_bytes: fetched.source_bytes,
    })
}

fn normalize_source_for_diagnostic(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn validate_manifest_document(
    document: &ManifestDocument,
    require_pinned_digest: bool,
) -> anyhow::Result<()> {
    if document.event_id.is_empty()
        || document.event_id.len() > 240
        || document.event_id.contains(['/', '\\'])
    {
        bail!("invalid event id");
    }
    if operating_kpi_catalog_for_symbol(&document.symbol).is_none() {
        bail!(
            "{} is outside the shared operating KPI catalog",
            document.symbol
        );
    }
    if document.period.trim().is_empty() || document.claims.is_empty() || document.claims.len() > 6
    {
        bail!("{} has an invalid period or claim count", document.event_id);
    }
    let url = Url::parse(&document.source_url).context("parse source URL")?;
    if url.scheme() != "https"
        || !url
            .host_str()
            .is_some_and(|host| OFFICIAL_HOSTS.contains(&host))
    {
        bail!(
            "{} is not an allowlisted official HTTPS URL",
            document.event_id
        );
    }
    if !matches!(document.format.as_str(), "pdf" | "html") {
        bail!("unsupported source format");
    }
    if require_pinned_digest && !valid_sha256(&document.source_sha256) {
        bail!(
            "{} must pin one lowercase source SHA-256",
            document.event_id
        );
    }
    if require_pinned_digest
        && !matches!(
            document.source_time_precision.as_str(),
            "exact" | "date_only_conservative_end_of_day"
        )
    {
        bail!("{} has invalid source time precision", document.event_id);
    }
    for (index, claim) in document.claims.iter().enumerate() {
        if !operating_kpi_input_is_supported_for_symbol(&document.symbol, claim) {
            bail!(
                "{} claim {index} is outside the symbol-scoped KPI catalog",
                document.event_id
            );
        }
    }
    Ok(())
}

async fn fetch_source(
    client: &reqwest::Client,
    document: &ManifestDocument,
) -> anyhow::Result<FetchedSource> {
    let mut request = client.get(&document.source_url);
    if Url::parse(&document.source_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .as_deref()
        == Some("www.sec.gov")
    {
        let sec_user_agent = std::env::var("SEC_USER_AGENT")
            .unwrap_or_else(|_| "honeclaw research ops@honeclaw.local".into());
        request = request.header(reqwest::header::USER_AGENT, sec_user_agent);
    }
    let response = request
        .send()
        .await
        .with_context(|| format!("fetch {}", document.event_id))?
        .error_for_status()
        .with_context(|| format!("official source status for {}", document.event_id))?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if response
        .content_length()
        .is_some_and(|length| length > MAX_SOURCE_BYTES as u64)
    {
        bail!("{} exceeds the source byte limit", document.event_id);
    }
    let bytes = response.bytes().await.context("read official source")?;
    if bytes.len() > MAX_SOURCE_BYTES {
        bail!("{} exceeds the source byte limit", document.event_id);
    }
    if document.format == "pdf" && (!bytes.starts_with(b"%PDF-") || !content_type.contains("pdf")) {
        bail!("{} did not return an authenticated PDF", document.event_id);
    }
    if document.format == "html"
        && (!content_type.contains("html") || bytes.iter().any(|byte| *byte == 0))
    {
        bail!("{} did not return UTF-8 HTML", document.event_id);
    }
    let source_bytes = bytes.to_vec();
    let source_sha256 = sha256_hex(&source_bytes);
    let extraction_bytes = source_bytes.clone();
    let format = document.format.clone();
    let text = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
        match format.as_str() {
            "pdf" => {
                std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(&extraction_bytes))
                    .map_err(|_| anyhow::anyhow!("PDF parser panicked"))?
                    .map_err(|error| anyhow::anyhow!("PDF extraction failed: {error}"))
            }
            "html" => {
                let html =
                    String::from_utf8(extraction_bytes).context("official HTML is not UTF-8")?;
                let document = Html::parse_document(&html);
                let body =
                    Selector::parse("body").map_err(|_| anyhow::anyhow!("build body selector"))?;
                Ok(document
                    .select(&body)
                    .flat_map(|node| node.text())
                    .collect::<Vec<_>>()
                    .join(" "))
            }
            _ => bail!("unsupported source format"),
        }
    })
    .await
    .context("join source extraction")??;
    if text.trim().is_empty() {
        bail!("{} yielded no source text", document.event_id);
    }
    Ok(FetchedSource {
        source_bytes,
        source_sha256,
        extracted_text_sha256: sha256_hex(text.as_bytes()),
        source_text: text,
    })
}

async fn archive_source_object(
    root: &Path,
    artifact: &OperatingKpiSourceArtifact,
    bytes: &[u8],
) -> anyhow::Result<()> {
    if !operating_kpi_source_artifact_is_valid(artifact)
        || artifact.source_sha256 != sha256_hex(bytes)
        || artifact.byte_length != bytes.len() as u64
    {
        bail!("source bytes do not match the immutable artifact metadata");
    }
    let path = root.join(&artifact.object_path);
    let parent = path.parent().context("source artifact has no parent")?;
    tokio::fs::create_dir_all(parent)
        .await
        .with_context(|| format!("create {}", parent.display()))?;
    match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
    {
        Ok(mut file) => {
            file.write_all(bytes)
                .await
                .with_context(|| format!("write {}", path.display()))?;
            file.sync_all()
                .await
                .with_context(|| format!("sync {}", path.display()))?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = tokio::fs::read(&path)
                .await
                .with_context(|| format!("read existing {}", path.display()))?;
            if existing != bytes {
                bail!("existing source artifact bytes do not match their digest path");
            }
        }
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn event_shape(
    document: &ManifestDocument,
    artifact: Option<&OperatingKpiSourceArtifact>,
) -> anyhow::Result<(EventKind, Value)> {
    let claims = serde_json::to_value(&document.claims)?;
    let shaped: anyhow::Result<(EventKind, Value)> = match document.document_type.as_str() {
        "earnings_call_transcript" => Ok((
            EventKind::EarningsCallTranscript,
            json!({
                "point_in_time_backfill": true,
                "earnings_transcript_operating_kpi_source_verified": true,
                "earnings_transcript_review": {"operating_kpi_claims": claims}
            }),
        )),
        "earnings_release" => Ok((
            EventKind::EarningsReleased,
            json!({
                "point_in_time_backfill": true,
                "earnings_quality_operating_kpi_source_verified": true,
                "earnings_quality_review": {"operating_kpi_claims": claims}
            }),
        )),
        "investor_presentation" => Ok((
            EventKind::NewsCritical,
            json!({
                "point_in_time_backfill": true,
                "operating_kpi_source_verified": true,
                "operating_kpi_source": {
                    "document_type": "investor_presentation",
                    "source_tier": "company_primary"
                },
                "operating_kpi_claims": claims
            }),
        )),
        "sec_filing_10k" => Ok((
            EventKind::SecFiling {
                form: "10-K".into(),
            },
            json!({
                "point_in_time_backfill": true,
                "operating_kpi_source_verified": true,
                "operating_kpi_claims": claims
            }),
        )),
        "sec_filing_10q" => Ok((
            EventKind::SecFiling {
                form: "10-Q".into(),
            },
            json!({
                "point_in_time_backfill": true,
                "operating_kpi_source_verified": true,
                "operating_kpi_claims": claims
            }),
        )),
        "sec_filing_8k" => Ok((
            EventKind::SecFiling { form: "8-K".into() },
            json!({
                "point_in_time_backfill": true,
                "operating_kpi_source_verified": true,
                "operating_kpi_claims": claims
            }),
        )),
        _ => bail!("unsupported document type"),
    };
    let (kind, mut payload) = shaped?;
    if let Some(artifact) = artifact {
        payload["operating_kpi_backfill_schema_version"] =
            Value::String(OPERATING_KPI_BACKFILL_EVENT_SCHEMA_VERSION.into());
        payload["operating_kpi_source_artifact"] = serde_json::to_value(artifact)?;
        payload["operating_kpi_source_time_precision"] =
            Value::String(document.source_time_precision.clone());
    }
    Ok((kind, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gev_claim() -> OperatingKpiClaimInput {
        serde_json::from_value(json!({
            "claim_kind": "contract_milestone",
            "kpi_id": "generation_equipment_backlog",
            "issuer_metric_name": "backlog",
            "issuer_definition": "remaining performance obligation (RPO)",
            "period": "FY2025",
            "numeric_value": 150.0,
            "unit": "USD_billions",
            "value_text": "We increased our backlog to $150 billion",
            "measurement_scope": "GE Vernova total backlog at year-end 2025",
            "comparison_basis": "point_in_time",
            "speaker": "GE Vernova CEO Scott Strazik",
            "evidence_quote": "Defined as remaining performance obligation (RPO)",
            "source_locator": "4Q 2025 earnings release · highlights and footnote 1",
            "definition_changed": false,
            "disposition": "active"
        }))
        .unwrap()
    }

    fn gev_document(source_sha256: String) -> ManifestDocument {
        ManifestDocument {
            event_id: "operating-kpi-backfill:GEV:FY2025:earnings-release".into(),
            symbol: "GEV".into(),
            occurred_at: DateTime::parse_from_rfc3339("2026-01-28T23:59:59Z")
                .unwrap()
                .with_timezone(&Utc),
            period: "FY2025".into(),
            document_type: "earnings_release".into(),
            format: "pdf".into(),
            source_url:
                "https://www.gevernova.com/sites/default/files/gev_webcast_pressrelease_01282026.pdf"
                    .into(),
            source_sha256,
            source_time_precision: "date_only_conservative_end_of_day".into(),
            claims: vec![gev_claim()],
        }
    }

    fn artifact_for(bytes: &[u8]) -> OperatingKpiSourceArtifact {
        let digest = sha256_hex(bytes);
        OperatingKpiSourceArtifact {
            schema_version: OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION.into(),
            source_sha256: digest.clone(),
            extracted_text_sha256: "a".repeat(64),
            byte_length: bytes.len() as u64,
            format: "pdf".into(),
            object_path: format!("objects/{digest}.pdf"),
        }
    }

    #[test]
    fn v2_requires_a_pinned_digest_and_symbol_scoped_kpi() {
        assert!(validate_manifest_document(&gev_document(String::new()), true).is_err());
        let mut document = gev_document("a".repeat(64));
        validate_manifest_document(&document, true).unwrap();
        document.claims[0].kpi_id = "token_or_call_volume".into();
        assert!(validate_manifest_document(&document, true).is_err());
    }

    #[test]
    fn v2_event_binds_the_artifact_to_every_admitted_claim() {
        let document = gev_document("a".repeat(64));
        let artifact = OperatingKpiSourceArtifact {
            schema_version: OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION.into(),
            source_sha256: "a".repeat(64),
            extracted_text_sha256: "b".repeat(64),
            byte_length: 123,
            format: "pdf".into(),
            object_path: format!("objects/{}.pdf", "a".repeat(64)),
        };
        let (kind, payload) = event_shape(&document, Some(&artifact)).unwrap();
        let event = MarketEvent {
            id: document.event_id,
            kind,
            severity: Severity::Low,
            symbols: vec![document.symbol],
            occurred_at: document.occurred_at,
            title: "source".into(),
            summary: "source".into(),
            url: Some(document.source_url),
            source: "company_ir.operating_kpi_point_in_time".into(),
            payload,
        };
        let claims = operating_kpi_claims_from_event(&event);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].source_artifact.as_ref(), Some(&artifact));
    }

    #[tokio::test]
    async fn content_addressed_archive_is_idempotent_and_rejects_tampering() {
        let directory = tempfile::tempdir().unwrap();
        let bytes = b"%PDF-1.7\nsource bytes";
        let artifact = artifact_for(bytes);
        archive_source_object(directory.path(), &artifact, bytes)
            .await
            .unwrap();
        archive_source_object(directory.path(), &artifact, bytes)
            .await
            .unwrap();
        let path = directory.path().join(&artifact.object_path);
        assert_eq!(tokio::fs::read(&path).await.unwrap(), bytes);
        tokio::fs::write(&path, b"changed").await.unwrap();
        assert!(
            archive_source_object(directory.path(), &artifact, bytes)
                .await
                .is_err()
        );
    }
}
