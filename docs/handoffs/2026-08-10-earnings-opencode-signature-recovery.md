# Earnings OpenCode Signature Recovery — 2026-08-10

- title: Earnings OpenCode signature recovery and production rollout
- status: in_progress
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files: `crates/hone-channels/src/tool_trace.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-channels/src/agent_session/tests.rs`
- related_docs: `docs/current-plans/earnings-opencode-signature-recovery.md`, `docs/runbooks/opencode-setup.md`, `docs/runbooks/backend-deployment.md`
- related_prs: direct `main` commits `4dd76971d7b9985e281c3632db17b2936e0f91ce` and `185504bc03d8be32bfcc1f851200e411ed8a8238`; no PR, release, or tag

## Summary

After the OpenRouter account was recharged, the RKLB earnings-preview request progressed through evidence collection but failed with Gemini/OpenRouter `400 Corrupted thought signature`. The first repair added a narrow OpenCode earnings replay boundary and went live as `4dd76971`. The owner's first real canary then reached the renderer twice, but both reports were rejected before any write; a failed OpenCode built-in `glob` kept that otherwise safe trace outside the generic PDF-validation recovery predicate. The second repair extends only the dedicated OpenCode earnings predicate and is live as `185504bc`; one new authenticated RKLB canary is still required for the PDF closure.

## What Changed

- Added a dedicated retry-safety classifier that accepts Hone's existing read-only calls plus exact OpenCode built-ins `read`, `grep`, and `glob`.
- Accepts OpenCode's `invalid` record only when its structured arguments explicitly prove `Model tried to call unavailable tool '<name>'`; the rejected target was never dispatched.
- Keeps real `bash`, `task`, ambiguous `invalid`, arbitrary external tools, executable skills, and persistent mutations outside automatic replay.
- The phase-one exception applies only to verified `opencode_acp` dedicated earnings turns. Ordinary conversations, other runners, context ownership, model routing and generic signature retry behavior are unchanged.
- Added an OpenCode-specific safe PDF-validation predicate. It accepts the same exact built-in read records while still requiring every renderer rejection to report `success=false`, `render_success=false`, `side_effect_status=not_started`, zero artifacts, and a nonempty validation error.
- The post-run persistent-effect normalizer now preserves this exact safe renderer failure so AgentSession can use its existing one-shot `fresh_session_after_safe_pdf_validation_failure` path. Real shell, unknown tools, completed/partial artifacts and uncertain results remain non-replayable.

## Verification

- Production evidence before the fix: the recharged account had about `$99.78` available; the exact route was `openrouter/google/gemini-3.1-pro-preview`; the failed attempt collected data/search evidence, performed OpenCode `read`/`grep`, recorded the unavailable `bash` as non-executed `invalid`, then failed on thought signature after about 105 seconds.
- Phase-one focused safety-classifier and end-to-end AgentSession recovery tests passed. Full relevant suites passed: `hone-channels` 790/790 non-ignored and `hone-web-api` 209/209 non-ignored; format and both crate checks passed.
- GitHub Runtime Image run `31352981377` published revision `4dd76971d7b9985e281c3632db17b2936e0f91ce` at digest `sha256:a7d5178a8c2b5f3db8a505ee79611322e3e2fcb0268a8c7e33f978efd697cabd`; bundle verification passed before cutover.
- Preflight had about 6.9 GiB free, canonical config validated the earnings OpenCode/Gemini route and credential presence, and two consecutive active-chat reads were zero.
- `/opt/hone/current` was atomically switched and only `hone-web.service` restarted. Production `/api/meta` reports exact `4dd76971`, `source=ghcr_linux_oci`, healthy PostgreSQL/S3, cloud-authoritative storage and zero local durable dependencies. The service is active with `NRestarts=0`, active chats are zero, loopback/public auth return expected JSON `401`, and the post-cutover warning journal was empty.
- The real `4dd76971` RKLB canary collected three finance payloads and two web-search batches, then attempted the official renderer twice. Both calls exited before writing (`side_effect_status=not_started`, zero artifacts); preflight rejected 14 then 19 issues around the required opening judgment/numeric reason, expectation comparison, investor-material context, institution details and company-operating news classification. OpenCode itself ended normally and billed about `$0.4023`; no PDF existed. HONE then displayed the generic uncertain-state message because the failed built-in `glob` prevented the safe PDF-validation predicate from matching.
- Phase two added production-shaped `glob + renderer not_started` tests and kept real `bash` rejected. `hone-channels` passed 791 non-ignored tests and `hone-web-api` passed 209; targeted recovery/normalizer/signature tests, crate checks, format and diff checks passed.
- Runtime Image run `31354569937` published `185504bc03d8be32bfcc1f851200e411ed8a8238` at digest `sha256:f69549d581edbd783153bee695ea7f79ace14b643bbb017bb4cbff62eaed5f6e`. The bundle verifier passed; two zero-active-chat reads preceded the atomic cutover and only `hone-web.service` restarted.
- Production now executes the exact `185504bc` release. `/api/meta` reports `source=ghcr_linux_oci`, healthy PostgreSQL/S3, cloud-authoritative storage and zero local durable dependencies. `NRestarts=0`, active chats are zero, loopback/public auth are JSON `401`, and warning count since cutover is zero.

## Risks / Follow-ups

- The provider may still emit a corrupted signature; the repair is one bounded fresh-session retry, not suppression or an unbounded loop. A second signature failure remains visible and stops charging further retries.
- A second real authenticated RKLB rerun was not submitted by Codex because it spends account credits and generates a user artifact. The owner has been asked to retry exactly once from the existing UI; acceptance requires a persisted/downloadable PDF. If the first model attempt fails preflight, logs must show `fresh_session_after_safe_pdf_validation_failure` before the single clean retry.
- `hone-cli config get llm.providers.openrouter` currently prints the real API key despite its help text promising redaction. No key was committed, but the command should be fixed and the exposed key rotated through the protected canonical-config path.
- Immediate rollback is the retained `/opt/hone/releases/4dd76971d7b9985e281c3632db17b2936e0f91ce-ghcr-runtime`; require two zero-active-chat reads before atomically restoring it and restarting only `hone-web.service`.

## Next Entry Point

Ask the owner to retry RKLB once. If it fails, correlate the new message with `agent.run.retry`: absence after a proven pre-write renderer rejection means a new trace classification boundary; one safe-PDF retry followed by another preflight rejection means the report-generation prompt still cannot satisfy the validator and should be fixed separately; a repeated signature error is upstream. Do not broaden the allowlist without structured proof that the specific tool never executed or is inherently read-only.
