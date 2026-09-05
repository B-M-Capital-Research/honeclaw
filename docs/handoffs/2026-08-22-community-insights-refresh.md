# Community Insights Refresh 2026-08-22

- title: Community Insights Refresh 2026-08-22
- status: `done`
- created_at: `2026-08-22`
- updated_at: `2026-09-05`
- owner: `Codex`
- related_files:
  - `bins/hone-cli/src/cloud.rs`
  - `bins/hone-cli/src/main.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `docs/runbooks/backend-deployment.md`
  - `docs/repo-map.md`
- related_docs:
  - `docs/archive/plans/community-insights-refresh-2026-08-22.md`
  - `docs/handoffs/2026-07-19-public-community-edge-delivery.md`
- related_prs: none; local uncommitted implementation and production data operation

## September 5 correction

The August results below were recovered from the earlier task record, but its claim that the inspected database was the live authority was incorrect. The managed production API still had 718 topics ending July 31 on September 5, while the local automation had updated another database and the shared private R2 projection. The August bytes and captures remain useful recovery evidence; its successful CLI results do not prove production freshness. The uncommitted implementation and runbook were also absent from main until this recovery. See [the September 5 recovery](2026-09-05-community-freshness-assets-latency.md) for the production-bound wrapper, source reconciliation, deployment and current limitations. This correction supersedes the production-status claims below.

## Summary

The user-authorized Knowledge Planet account was signed in through Chrome after an SMS/CAPTCHA flow, and the `巴芒科技` timeline was read continuously from the canonical anchor `content_id=769` / `2026-07-16 23:43` through the latest visible topic `2026-08-22 00:22`. The contiguous delta contains 164 topics, 120 file references, and 107 image references. All 164 topic bodies and 227 resource metadata rows are now in the authoritative PostgreSQL community archive.

## What Changed

- Added read-only `hone-cli cloud community-inspect` to establish an exact canonical cutoff.
- Added dry-run-first `hone-cli cloud community-append` with a bounded newest-first manifest, stable topic identifiers, exact anchor validation, one-transaction insertion, and all-existing replay idempotence.
- Retained the ignored operator artifacts under `data/community-imports/2026-08-22/`: the 201,996-byte append manifest has SHA-256 `c657ff1d3d8591df64c4430b73bd16feaa1ee1ad3795b4308656986cacffa833`; the asset manifest has SHA-256 `a6d26aabba632d01359fdec0d8d605632e369ef565022328ade961472a307082`.
- Promoted 103 legitimately captured assets through the existing verified backfill workflow: 102 rendered Knowledge Planet image variants plus the 8-page, 4,272,631-byte `GOOGLE_财报前瞻.pdf` (SHA-256 `b507349ac48cf60c5f4f0a0fb2138ed61c3d6eb3ff081e8ec1f7bd9af3771987`). R2 upload/readback and PostgreSQL promotion completed without conflict.
- The remaining 119 file references and 5 image references remain metadata-only / application-protected. A clean automation tab was blocked by Chrome at `files.zsxq.com` with `ERR_BLOCKED_BY_CLIENT`; no source protection, browser interstitial, cookie, or signed source URL was bypassed or persisted.
- Updated the repository map and deployment runbook with the recurring `community-inspect → community-append → community-assets → community-publish` sequence.

## Verification

- Append manifest structure: 164 items, 227 resources, 164 unique stable IDs, contiguous indices, newest `2026-08-22 00:22`, oldest delta `2026-07-20 10:21`, exact anchor match.
- First append dry-run: `anchor_matched=true`, `existing=0`, `would_insert=164`.
- Append apply: `inserted=164`; replay dry-run: `existing=164`, `would_insert=0`, all actions `already_present`.
- Asset dry-run: `validated=103`, `would_upload=103`, `would_update=103`, no conflicts. Apply and replay: 103 immutable objects current, all replay actions `already_current`.
- Canonical readback: newest row `2026-08-22 00:22`; newest ten entries contain the expected text/resources and promoted images.
- Publisher dry-run before apply: `ok=true`, `content_count=826`, `resource_count=1060`, `page_count=42`, `conflicts=[]`.
- Publisher apply completed full-byte/SHA-256 verification for 822 edge resources, retained 238 legacy resources, wrote 146 of 865 planned private objects, updated `latest` last, and reported `conflicts=[]`.
- Final publisher dry-run: `ok=true`, `no_op=true`, `would_write=0`, `conflicts=[]`.
- `cargo fmt --all -- --check` and `git diff --check` passed.
- `cargo test -p hone-core community_append --lib`: 4 passed.
- `cargo test -p hone-cli cli_parses_cloud_community -- --nocapture`: 4 passed.
- `cargo check -p hone-cli` passed.
- `bash tests/regression/ci/test_community_forum_research_boundary.sh` passed.

## Risks / Follow-ups

- The 102 images are the legitimately observed rendered variants, commonly 380px wide, not asserted to be source originals. Re-capture only through a future authorized official export/connector or a browser flow that exposes the originals without bypassing controls.
- The 124 metadata-only resources should stay on the legacy/application-protected path until verified bytes can be obtained. Do not infer or fabricate file contents.
- This task changed production community data and its private snapshot only. It did not deploy code, restart services, enable the unfinished Edge route, change Worker variables, or activate frontend discovery.
- The implementation is an uncommitted local change set alongside unrelated user-owned worktree changes; preserve unrelated files when committing later.

## Next Entry Point

Use `docs/runbooks/backend-deployment.md` Step 5 for future authorized refreshes. Start with `community-inspect`, capture only the contiguous delta, require append dry-run/apply/replay, then promote verified bytes and republish the private snapshot.
