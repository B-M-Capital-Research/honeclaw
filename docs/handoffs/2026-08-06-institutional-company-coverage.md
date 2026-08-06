# Institutional Company Coverage Local Chain Handoff

- title: Institutional company coverage and earnings delivery chain
- status: done
- created_at: 2026-08-06
- updated_at: 2026-08-06
- owner: Codex / Hone Product
- related_files:
  - config.example.yaml
  - crates/hone-core/src/config/event_engine.rs
  - crates/hone-event-engine/src/engine.rs
  - crates/hone-event-engine/src/earnings_document.rs
  - crates/hone-event-engine/src/earnings_continuity.rs
  - crates/hone-event-engine/src/subscription.rs
  - crates/hone-event-engine/examples/earnings_quality_models.rs
  - crates/hone-event-engine/examples/earnings_continuity_models.rs
  - crates/hone-event-engine/src/digest/buffer.rs
  - crates/hone-event-engine/src/pollers/earnings_surprise.rs
  - crates/hone-event-engine/src/pollers/earnings_quality.rs
  - crates/hone-event-engine/src/pollers/corp_action.rs
  - crates/hone-event-engine/src/renderer.rs
  - crates/hone-event-engine/src/router/dispatch.rs
  - crates/hone-event-engine/src/sinks/multi.rs
  - crates/hone-event-engine/src/store.rs
  - memory/src/company_profile/
  - tests/fixtures/event_engine/earnings_continuity_baseline_2026-08-06.json
  - tests/regression/manual/test_event_engine_earnings_continuity_baseline.sh
- related_docs:
  - docs/current-plans/institutional-company-coverage.md
  - docs/decisions.md#d-2026-08-06-01-treat-earnings-as-actor-scoped-coverage-updates
  - docs/invariants.md
- related_prs: []

## Summary

The first two local implementation slices turn a reviewed earnings event from a terse shared summary into one structured fact card plus actor-scoped mainline context, then make same-document delivery converge safely across immediate and digest paths. They correct the SNDK-class timing, duplicate-notification, restart-review, and false-channel-acknowledgement failure modes and expand new company portraits with the minimum institutional coverage fields. The overall plan remains active because historical four-quarter continuity and a live earnings-season trial have not been completed.

## What Changed

- Earnings quality output now retains conclusion, up to three key facts, up to two counterpoints, explicit unknowns, and named follow-up questions. The SNDK regression requires all fields to survive into the event body.
- Final routing reads `mainline_by_ticker` only for the resolved actor. Plain, Discord, Telegram, and Feishu outputs all show the same shared facts plus that actor's read-only mainline. A missing mainline is labelled as generic, and structured earnings cards bypass the generic plain-text polisher.
- Earnings surprise discovery runs every ten minutes by default during common US pre/post-market release windows. FMP SEC timestamps are parsed as DST-aware US/Eastern, an earnings-looking 8-K is preferred over a later unrelated filing, and the selected disclosure timestamp becomes the earnings event time.
- An 8-K URL identified as an earnings/press-release document is demoted to digest material, leaving the structured earnings card as the default single immediate notification.
- A shared canonical earnings-document key now links the SEC support item and structured card across pollers, routing, actor delivery history, and digest buffering. Both arrival orders converge on one structured actor delivery after success; queued SEC support is removed and later same-document SEC events are audited as superseded.
- Failed or quiet-held structured sends preserve the SEC digest fallback. A queued structured card replaces the same-document SEC item as the richer pending digest item, so there is still one deliverable fallback. A real channel failure remains an error even when the diagnostic log sink records a copy; only an intentionally unregistered channel path reports `dryrun`.
- Store-backed reviewed-event checks prevent repeated SEC/LLM work after restart, while insertion failures remain retryable because the poller does not mark the in-memory cache before durable pipeline persistence.
- A paid, delivery-free model diagnostic now runs the production SEC extractor and earnings prompt against SNDK, AMD, and BE, emitting one JSON result per model/sample with contract score, latency, tokens, and current OpenRouter price-derived cost. It is opt-in and writes no event or delivery state.
- The earnings-quality default and local effective profile now use `x-ai/grok-4.5` after the user's explicit model preference and a live current-generation comparison. The prompt requires original B/M units and forbids unsupported non-EPS consensus language.
- Company-profile metadata adds A/B/C coverage tier and investment horizon. The template adds expectation baseline, valuation scenarios, management commitments, catalyst calendar, open questions with validation dates, and decision log. Legacy tracking metadata defaults to C/long-term without migration.

## Verification

- `cargo test -p hone-event-engine --lib`: 554 passed, 13 ignored, 0 failed.
- `cargo test -p hone-memory --lib`: 134 passed, 0 failed.
- `cargo test -p hone-core --lib`: 137 passed, 0 failed.
- `cargo check -p hone-web-api`: passed with one unrelated existing dead-code warning in `feishu_direct_actor_contact_targets_from_records`.
- `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`: 43 fixture items loaded, including 15 saved LLM-classified items; passed without a live provider call.
- `cargo run -p hone-event-engine --example earnings_quality_models`: live OpenRouter comparison completed on public SNDK/AMD/BE SEC exhibits. The final Grok 4.5 pass produced 3/3 valid 10/10 contract objects with source-preserved B/M units; observed estimated cost was `$0.01150–$0.01511` per event and the full multi-model exercise used about `$0.252` of the authorized `$20`.
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed; the largest suites included 716/717, 554/567, 161/162, and 161/163 with only declared ignored tests.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed with the same unrelated Web API dead-code warning.
- `bun run test:web`: 334 passed, 0 failed.
- `workers/public-community-edge`: TypeScript check passed; 45 tests passed, 0 failed.
- `bash tests/regression/run_ci.sh`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed. The repository-wide formatter made two format-only adjustments in the pre-existing `crates/hone-channels/src/agent_session/artifacts.rs` diff so the full gate is clean.
- The saved news-classifier baseline remained offline because that model/profile did not change. Earnings-quality live calls were explicitly authorized and run through the dedicated example.

## Risks / Follow-ups

- Initial earnings-release classification still uses conservative URL signals. Generic exhibit names can escape classification before the shared document identity is attached; accession/source identity remains the stronger future improvement.
- Store-backed restart dedup prevents repeated review for durably inserted events, but it intentionally does not claim exactly-once external delivery. Router idempotency and the retained SEC fallback remain the safety boundary for channel failures.
- The live corpus still contains only three strong positive releases. AMD received Grok confidence `0.86` and therefore remains digest under the current `0.90` immediate threshold; do not lower that threshold until mixed and negative releases are added and scored.
- A/B/C metadata and template fields exist, but routing cadence and service levels are not yet driven by coverage tier.
- Management commitments, open questions, expectation baselines, and event-card updates are not yet reconciled across quarters; this is the main next stage rather than a claimed completion.
- No commit, push, source deployment, or production notification was performed. The local `main` branch is 77 commits behind `origin/main`, so integration should first reconcile the branch without mixing unrelated worktree changes.

## Next Entry Point

Continue from `docs/current-plans/institutional-company-coverage.md`: add transcript/earnings-call material to the same research object for at least eight historical events, prepare the 14-point blind-review packet, establish the user's real A-level pilot portraits, then run one forward earnings season. Do not treat the automated 18-point structure score as a substitute for user decision-utility review.

## Phase 4 Update: Durable Four-Quarter Research Continuity

### What Changed

- Explicitly tracked company profiles now join portfolio holdings as event subscriptions. Coverage tiers drive service depth: A gets full symbol coverage, B gets material filings/news/earnings/corporate actions with reduced interruption, and C gets earnings discovery.
- Company-profile events can append structured open-question and management-commitment updates. Stable IDs and storage validation preserve the first statement; later quarters update state/evidence without rewriting history. The folded ledger is actor-scoped and works for local files and cloud-backed logical files.
- After an admitted A-tier T0 earnings delivery, the router schedules actor continuity in the background. The reconciler compares verified event facts with the saved profile and active ledger, proposes a thesis effect without mutating the thesis, writes one idempotent research event, and carries omitted/invalid updates forward.
- Attention is bounded at eight active questions and six active commitments, with at most two new items of each kind per quarter. Immediate quality and continuity use separate LLM providers/budgets (1,800 vs 3,600 tokens); compact current-state prompts prevent fourth-quarter truncation.
- The earnings-quality path now rejects unsafe comma-B values and deterministically normalizes the headline and list limits. This closes the COST table-unit failure and the live BE evidence-list drift found during replay.
- The repository now contains a 24-event fixture covering SNDK, AMD, BE, SNOW, COST, and MU over four sequential quarters each, plus an offline contract test and an opt-in paid SEC/OpenRouter manual regression.

### Verification

- `cargo test -p hone-memory --lib`: 136 passed.
- `cargo test -p hone-core --lib`: 137 passed.
- `cargo test -p hone-event-engine --lib`: 561 passed, 13 ignored.
- `cargo check -p hone-web-api`: passed with the existing unrelated Feishu dead-code warning.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed with the same existing warning.
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed; relevant final suites included hone-memory 136, hone-core 137, hone-event-engine 561 plus 13 declared ignored, hone-channels 716 plus one ignored, and hone-web-api 162 plus two ignored.
- `bash tests/regression/manual/test_event_engine_earnings_continuity_baseline.sh`: offline 6-company/24-event SEC fixture contract passed without a provider call.
- `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`: offline 43-item fixture loaded, including 15 saved LLM items; no paid classifier call was made because that route did not change.
- `bun run test:web`: 334 passed; Public Community Edge typecheck and 45 tests passed.
- `bash tests/regression/run_ci.sh`: all CI-safe regression scripts passed.
- Final paid `x-ai/grok-4.5` replay: 24/24 successful, every sample scored 18/18 on the automated quality-plus-continuity contract; 48 calls, 137,764 prompt tokens, 48,639 completion tokens, about `$0.567362`, and about 306 seconds elapsed.
- COST FY25 Q4 emitted `$13,335M` operating cash flow and `$14,161M` cash; the final 24-result file contained no comma-formatted B-unit amount.
- All paid runs in this iteration, including the earlier model comparison, failure diagnosis, and remediated reruns, used about `$2.41` of the authorized `$20` budget.
- `cargo fmt --all -- --check`, `git diff --check`, and the reviewed-diff credential-shape scan passed.

### Risks / Follow-ups

- This fixture proves earnings-release continuity, not transcript continuity. At least eight call transcripts still need to update the same research object and answer previously registered questions.
- The 18-point replay score is a deterministic schema/chain contract. The plan's 14-point fact quality and decision-usefulness score still needs blind human review, especially for mixed/negative and apparently strong-but-low-quality quarters.
- The background task is non-blocking and retries provider transport internally, but there is not yet a durable continuity-job queue for a process crash after T0. Add replayable pending/failed job state before claiming operational exactly-once completion.
- The structured ledger is visible in portrait event Markdown, but the public professional-investor UI does not yet provide a dedicated open-question/commitment/decision-history view.
- No commit, push, deployment, restart, or production notification was performed. The active plan stays `in_progress`; no archive action is appropriate.
