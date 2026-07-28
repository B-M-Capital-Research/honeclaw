# Codex ACP Discord Runtime Recovery

- title: Codex ACP Discord Runtime Recovery
- status: done
- created_at: 2026-07-29
- updated_at: 2026-07-29
- owner: shared
- related_files:
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runners/tests.rs`
  - `crates/hone-channels/src/attachments/ingest.rs`
  - `crates/hone-channels/src/attachments/vision.rs`
  - `docs/current-plan.md`
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/archive/index.md`
- related_docs:
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
  - `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`
- related_prs: none

## Summary

A real Discord admin turn on `codex-acp 1.1.7` exposed two independent failures in sequence. Hone first replaced the adapter's valid `gpt-5.6-sol[xhigh]` session selection with a bare `gpt-5.6-sol` `session/set_model` request, which the adapter rejected before `session/prompt`. After that was fixed, a second image turn showed that optional Apple Vision OCR helper compilation blocked the first Discord placeholder for 95.4 seconds even though Codex ACP could read both images natively.

Both paths are now fixed and deployed from the source checkout. This follow-up did not change Hone/Kimi session compaction.

## What Changed

- Codex ACP model selection now emits the adapter-required `model[effort]` selector.
  - Bare model plus configured variant becomes `gpt-5.6-sol[xhigh]`.
  - Legacy `model/effort` and bracketed `model[effort]` inputs normalize to the same shape.
  - An ambiguous bare model with no effective effort no longer sends a known-invalid override.
- Admin actors using the configured `codex_acp` runner skip redundant local image OCR pre-extraction and pass the materialized image path to Codex ACP's native image flow.
- Strict public fallback actors retain the existing local OCR behavior.
- Optional OCR helper and Swift compiler subprocesses now use kill-on-drop so a timeout cannot leave the child running after its future is dropped.
- Image prompt guidance now tells native-image-capable runners to read the attachment path directly when no server-side OCR text is present.

## Verification

- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/attachments/ingest.rs crates/hone-channels/src/attachments/vision.rs crates/hone-channels/src/runners/codex_acp.rs crates/hone-channels/src/runners/tests.rs`
- `cargo test -p hone-channels attachments::ingest::tests --lib`
  - `25 passed`
- `cargo test -p hone-channels configured_codex_model_id --lib`
  - `5 passed`
- `cargo test -p hone-channels codex_acp --lib`
  - `5 passed`
- `cargo test -p hone-channels --lib`
  - `695 passed`, `1 ignored`
- `cargo check -p hone-channels --tests`
- `cargo build --bin hone-cli --bin hone-console-page --bin hone-discord --bin hone-mcp`
- A no-side-effect admin-scoped ACP probe returned `ACP_MODEL_OK` with zero tool calls. Its ACP trace showed successful `initialize`, `session/new`, `session/set_model(gpt-5.6-sol[xhigh])`, `session/prompt`, and final response.
- The real Discord image turn that motivated the OCR fix:
  - was sent at `2026-07-29 00:36:37.913 +08:00`;
  - created its pre-fix placeholder only at `00:38:13.282`, proving the 95.4-second pre-ACP delay;
  - then used Codex ACP native image reads for both attachments;
  - completed successfully after 19 tool calls with a 671-character final reply;
  - had its placeholder edited to the final Discord reply at `00:43:11`.
- The rebuilt source launchd job restarted cleanly (`runs=6`, PID `94657` at handoff validation). Discord re-authenticated as `Hone-TEST`, ports `8077` and `8088` listened, stderr was empty, and no `swiftc` or `hone-image-ocr` process remained.
- The first post-deployment Discord follow-up was sent at `00:46:28.111`; Discord created the bot placeholder at `00:46:29.232`, restoring a `1.121s` initial acknowledgement. ACP again accepted `gpt-5.6-sol[xhigh]` and entered `session/prompt` successfully.
- That post-deployment follow-up then completed successfully in `112473ms` after 53 tool calls (`portfolio`, `data_fetch`, and web search) and produced a 680-character final reply. Discord API readback confirmed that `Hone-TEST` edited the original placeholder at `00:48:22.320`, the final content was present, and no processing-placeholder text remained.

## Risks / Follow-ups

- The fix removes the admin Codex ACP turn's 95-second pre-placeholder OCR stall. It does not make a complex `gpt-5.6-sol[xhigh]` portfolio reconciliation instantaneous: the observed ACP work itself took about 296 seconds while reading two images, validating multiple securities, checking the existing portfolio, and writing the requested update.
- The post-deployment text follow-up likewise spent about 112 seconds performing 53 market-data and portfolio operations. This is model/tool execution latency after the immediate acknowledgement, not channel ingress, OCR, model selection, or Discord delivery failure.
- A future live image resend should be used to measure the post-deployment placeholder latency from the Discord snowflake timestamp. The code path and regression tests prove OCR is skipped, but no new user image was sent after the final restart in this handoff.
- Strict public fallback image turns still use local OCR. On a host with a broken Swift toolchain, the compiler child is now cleaned up on timeout, but the optional extraction can still consume its timeout budget. If public image TTFA becomes a requirement, move placeholder delivery before enrichment or make OCR a background capability with an explicit budget.
- No transcript, compact, module-boundary, or architecture decision changed, so `docs/repo-map.md`, `docs/invariants.md`, `docs/decisions.md`, and the ACP ADR did not require edits.
- The four unrelated pre-existing event-engine working-tree modifications were preserved and were not staged or altered.

## Next Entry Point

- Parent plan: `docs/current-plans/acp-runtime-refactor.md`
- Model selector: `crates/hone-channels/src/runners/codex_acp.rs`
- Native-image OCR routing: `crates/hone-channels/src/attachments/ingest.rs`
- Optional OCR subprocess lifecycle: `crates/hone-channels/src/attachments/vision.rs`
