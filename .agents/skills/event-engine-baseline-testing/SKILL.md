---
name: event-engine-baseline-testing
description: Use in the honeclaw repository when Codex needs to validate event-engine notification quality changes, run the event-engine/core/web-api regression subset, rerun the saved news classifier baseline against OpenRouter models, or add new real-news samples to the baseline fixture without overfitting.
---

# Event Engine Baseline Testing

Use this skill for event-engine push quality work, especially changes touching:

- `crates/hone-event-engine/src/pollers/*`
- `crates/hone-event-engine/src/router.rs`
- `crates/hone-event-engine/src/digest.rs`
- `crates/hone-event-engine/src/news_classifier.rs`
- `crates/hone-core/src/config/event_engine.rs`
- `crates/hone-web-api/src/lib.rs`
- `tests/fixtures/event_engine/news_classifier_baseline_*.json`

Use repository-native commands exactly as written in this skill. Do not add local wrapper prefixes.

## Start Checklist

Before changing code or fixtures, write a short todo that includes:

1. Goal: which behavior is being changed or verified.
2. Files: implementation files plus baseline/test files.
3. Verification: exact commands, including whether a live LLM run is needed.
4. Documentation: update the active plan/memory when the work affects the event-engine quality track; otherwise state why no doc sync is needed.

Check the worktree first:

```bash
git status --short --branch
```

Do not commit if `main` is behind, or if staged/unrelated files would make an isolated commit unsafe.

## Test Matrix

Run the smallest useful subset first, then broaden.

For event-engine logic:

```bash
cargo test -p hone-event-engine --lib
```

For config schema/default changes:

```bash
cargo test -p hone-core --lib
```

For web-api assembly changes, such as sink/classifier model wiring:

```bash
cargo check -p hone-web-api
```

Always finish Rust work with:

```bash
cargo fmt --all -- --check
```

If formatting fails, run:

```bash
cargo fmt --all
cargo fmt --all -- --check
```

## Existing News Baseline

The current reusable baseline is:

- `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`
- `tests/regression/manual/test_event_engine_news_classifier_baseline.sh`

The fixture stores real FMP titles/sites/symbols and expected post-tuning route decisions. It intentionally does **not** store full FMP article bodies. Some entries therefore have both:

- `expected_llm_after_engine`: original live run result with FMP text in prompt.
- `expected_llm_title_only_after_engine`: expected result for the saved title-only rerun script.

The CI-safe source/kind drift test is in:

- `crates/hone-event-engine/src/pollers/news.rs`
- test: `live_news_classifier_baseline_source_policy_is_stable`

Run it directly when touching source classification or transcript splitting:

```bash
cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib
```

## Rerun Baseline

Offline check, no network/API cost:

```bash
bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh
```

Live model drift check against the saved title-only samples:

```bash
env RUN_EVENT_ENGINE_LLM_BASELINE=1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh
```

To explicitly check the current recommended model:

```bash
env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh
```

To compare another model:

```bash
env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=x-ai/grok-4.1-fast bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh
```

To collect a non-blocking drift report:

```bash
env RUN_EVENT_ENGINE_LLM_BASELINE=1 ALLOW_EVENT_ENGINE_LLM_BASELINE_DRIFT=1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh
```

Interpretation:

- Any drift on noisy/title-only samples is a prompt/model review signal, not automatically a bug.
- Drift that promotes opinion/list/preview/social noise should usually be fixed in source classification, router gating, or prompt rules.
- Drift that demotes a concrete hard event should usually be fixed in prompt rules or deterministic fallback.
- Do not update the expected baseline just to make a new model pass. First decide whether the new answer is better.

## Daily Push Calibration Export

Daily Telegram calibration should start from the actual delivery evidence already stored in
`data/events.sqlite3`. Use the read-only exporter to produce an ignored local JSON/Markdown
snapshot for one actor and one local day:

```bash
python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-23 --actor telegram::::8039067465
```

The default output directory is `data/exports/event-engine-calibration/`, which is ignored by git.
The JSON report includes blank `calibration_label` and `calibration_note` fields. The Markdown
report is for quick human review.

Suggested labels:

- `useful`
- `noise`
- `should_immediate`
- `should_digest`
- `should_filter`
- `baseline_candidate`

When the user marks a stable reusable case, copy only the durable public fields into
`tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json` or a newer fixture. Do not
commit daily exports, private runtime DB files, full copyrighted article bodies, or one-off labels
that only explain a single noisy day.

## Add New Baseline Samples

Add a new baseline when live logs or user feedback exposes a reusable decision case, such as:

- a noisy source that should stay Low/digest;
- a concrete hard event that should reach Medium digest or High immediate;
- a transcript/earnings item that should be independently controllable;
- a route timing issue that should not regress after future prompt/model changes.

Workflow:

1. Pull or extract the real sample, but do not store API keys, private actor data beyond the stable actor key, or full copyrighted article bodies.
2. Save only durable fields needed for regression:
   - `symbol`
   - `site`
   - `title`
   - expected source class
   - expected event kind
   - old/new model answers when known
   - expected route after engine
3. If a result depends on article body that is not stored, add `expected_llm_title_only_after_engine` for the manual script.
4. Update summary counts in the fixture (`items`, LLM item count, yes/no counts).
5. Extend or adjust the CI-safe unit test if the fixture schema changes.
6. Run:

```bash
python3 -m json.tool tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json >/dev/null
bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh
cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib
```

7. If the user explicitly asked for live model validation, also run the live script and report cost/latency/drift.

## Preserve Semantics

Keep these invariants unless the user explicitly changes product policy:

- Source classification and deterministic router behavior must be CI-safe and mockable.
- Live LLM calls are manual validation only, never default CI gates.
- `earnings_call_transcript` is a standalone controllable kind, not generic `news_critical`.
- Legal ads, shareholder alerts, listicles, valuation commentary, and earnings previews should not become immediate alerts by LLM enthusiasm.
- High/immediate routing must be explainable through kind, severity, actor preference, cap/cooldown, and sink result.

## Closeout

Before final response:

- Re-run the relevant test subset and report exact pass/fail results.
- Update the active event-engine plan or automation memory if this work is part of the ongoing push-quality track.
- Mention whether live LLM baseline was run, skipped, or only run offline.
