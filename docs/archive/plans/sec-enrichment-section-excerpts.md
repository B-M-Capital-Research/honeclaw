# SEC Enrichment Section Excerpts

- title: SEC Enrichment Section Excerpts
- status: done
- created_at: 2026-05-07
- updated_at: 2026-05-07
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/pollers/sec_enrichment.rs
  - docs/bugs/sec_enrichment_openrouter_max_tokens_402.md
- related_docs:
  - docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md
  - docs/archive/index.md

## Goal

Fix the second SEC enrichment OpenRouter `HTTP 402` mode where the filing input itself exceeded the current key's prompt-token budget. The fix must preserve useful filing semantics by selecting important excerpts before the LLM call, not by blind truncation.

## Scope

- Use real TEM/AMD/COHR 10-Q and TEM 8-K filings to identify where long-term-investor signals live.
- Add a deterministic `selected SEC filing excerpts` layer before the LLM call.
- Prioritize MD&A, business/recent-development sections, strategic customer or agreement windows, acquisition/debt/capital-allocation windows, and risk/legal changes.
- Drop table-of-contents, routine GAAP table overflow, exhibit indexes, and hidden inline XBRL header/resource noise.

## Validation

- POC: TEM/AMD/COHR 10-Q filings reduced from roughly 32k-54k prompt-token equivalents to about 3.3k-3.9k rough tokens in selected excerpts.
- Live smoke: TEM 10-Q selected excerpt request to `x-ai/grok-4.1-fast` succeeded with 3,170 prompt tokens, 798 completion tokens, and reported cost about `$0.0010`.
- Passed: `cargo test -p hone-event-engine sec_enrichment --lib`

## Documentation Sync

- Updated `docs/bugs/sec_enrichment_openrouter_max_tokens_402.md` with the input-token failure evidence and section-aware fix.
- Updated `docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md` with the follow-up implementation notes.
- Added this archived plan and linked it from `docs/archive/index.md`.

## Risks / Open Questions

- The extractor does not diff current Risk Factors against the prior filing. It can only include explicit "no material changes" language or current-period risk/legal excerpts.
- S-1 and DEF 14A are handled by a generic front narrative plus keyword-window path; they were not part of the 2026-05-07 real-data POC sample.
