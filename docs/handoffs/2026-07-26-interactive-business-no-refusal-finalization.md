# Interactive Business No-Refusal Finalization

- title: Interactive business no-refusal finalization
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-core/src/tool_effect.rs`, `crates/hone-channels/src/agent_session/`, `crates/hone-channels/src/attachments/`, `crates/hone-channels/src/prompt.rs`, `crates/hone-web-api/src/routes/public.rs`, `skills/image_understanding/SKILL.md`, `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs: `docs/archive/plans/interactive-business-no-refusal-finalization.md`, `docs/decisions.md#d-2026-07-26-06-buffer-interactive-answers-and-recover-without-generic-business-refusal`, `docs/adr/0004-agent-owned-research-loop.md`, `docs/bugs/web_direct_image_attachment_not_readable_internal_debug_leak.md`, `docs/runbooks/backend-deployment.md`
- related_prs: none; implementation/runtime commit `75ca1957ebceaa3f8d564d423bcb6f8940b5c74e`
- verification: complete repository gates, real-host Apple Vision OCR, 501-file immutable manifest, controlled production restart, cloud/auth/channel health, and three fresh production replays
- risks: whole-answer buffering increases first-visible latency; Apple Vision OCR is macOS-specific; total infrastructure loss remains an explicit technical failure rather than fabricated business output

## Summary

The repeated screenshot failure had four independent causes. Web published the neutral finance header before the answer was usable, so any later error became irreversible. The Agent then treated non-executable `skill_tool(image_understanding)` prompt loading as an unknown/persistent effect and failed the whole turn. The actual MiniMax request was text-only and received a local path but no image bytes, while the image skill falsely assumed multimodal access. A separate prompt and Session shortcut also hard-refused ordinary non-finance questions.

Exact production commit `75ca1957` removes those business-refusal exits without changing the established finance first-line or answer-section format. It is deployed for Web and Feishu; Discord remains quarantined because its configured gateway credential is invalid.

## What Changed

- Web finance keeps the byte-exact Session-time prefix contract but buffers the whole natural answer until the same Agent completes a usable final. The early header ACK and fixed `本轮研究未能完成，暂未形成可供参考的标的结论。` production suffix are gone.
- A finance/read-only batch containing an unregistered, malformed, unknown-effect, or write-capable call executes zero calls before framing/observer/registry/network. The same Agent receives one tools-disabled continuation from evidence already in context. One-off empty/protocol/provider/step-timeout failures receive the same bounded recovery; a second failure remains an infrastructure failure.
- `skill_tool` is argument-aware: prompt loading without script execution is known read-only; `execute_script=true` remains unsafe. Persistent failures and uncertain mutations still stop without replay or false success.
- Downloaded macOS images are OCRed by a bounded local Apple Vision helper. Normalized rows enter a current-turn `【图片文字提取】` block grouped by filename; unsupported hosts and empty extraction never fabricate fields. The image skill no longer claims a path-only text model can see the file.
- Ordinary questions reach the configured Agent and answer directly. They do not inherit finance tools, the finance first line, or stale ticker context.
- The answer prompt's finance formatting contract was preserved. Only publication timing and refusal/recovery behavior changed.

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  - Agent `135/135`
  - Channels `680 passed`, one explicit host OCR ignore
  - Web API `140 passed`, two credentialed ignores
- `bun run test:web`: `294/294`
- Public Community Edge typecheck plus `45/45`
- finance contracts `43/43`; complete `bash tests/regression/run_ci.sh`
- changed-file rustfmt, pre-push gitleaks, and diff checks
- Real Apple Vision OCR read the user feedback screenshot's time header, exact fixed-failure sentence, and follow-up text. The earlier production holding screenshot test also read `CRWV`, `NBIS`, `ARM`, `MU`, holdings, and cash fields.
- Exact `target/deploy-75ca1957` contains 501 payload files and a fully verified SHA-256 manifest. The live CLI supervisor, console and Feishu processes use that package from the repository-root working directory. Ports `8077/8088`, `cloud_mode=cloud`, authoritative cloud storage, healthy PostgreSQL/R2, zero local durable dependencies, and local/origin/public `401 application/json` auth boundaries passed.
- Fresh production actors:
  - ordinary CPU question: about `7.7s`, direct concise answer, no finance format;
  - `美股为什么大跌`: about `39.6s`, unchanged finance first line, verified SPY/QQQ/IWM premise correction, one successful terminal, no generic failure;
  - real-OCR feedback attachment-block replay: about `16.1s`, read and explained the screenshot substantively instead of refusing.
- Every replay had exactly one `assistant_delta` and one successful `run_finished`, exactly two persisted rows, byte-identical visible/history SHA-256, and zero active chats after completion.

## Risks / Follow-ups

- Whole-answer buffering deliberately gives up the former sub-second neutral-header display. Correctness wins over a stranded header plus canned refusal; the prompt format and final bytes remain unchanged.
- Apple Vision is available on the production macOS host. Other hosts report a narrow OCR capability gap and must answer from other evidence or ask for the one missing field.
- The signed-in Chrome production page was healthy, but its very long history caused browser automation to time out while opening the native file chooser. No login, SMS, account setting, or existing conversation was modified. Public upload-to-shared-ingest behavior is covered by Web API tests; the host OCR and online downstream attachment replay were verified separately.
- A complete provider/infrastructure outage cannot yield a truthful research answer. The runtime now avoids canned business refusal copy and never claims an unexecuted mutation, but it still exposes a sanitized technical failure when the bounded same-Agent recovery also fails.
- Keep `target/deploy-b2897ec9` as the immediate prior runtime and `target/deploy-84ca1f21` as the older known-good market-move rollback package until ordinary production sampling confirms stability.

## Next Entry Point

Use D-2026-07-26-06 and this handoff for future Interactive refusal/image regressions. Reopen the fixed bug if a normal question again returns only generic research failure, if a successful image upload reaches the Agent without the current-turn OCR block on the production host, or if visible output and persisted history diverge. Continue the separate scheduler entity-guard P2 in `docs/current-plans/ticker-resolution-architecture.md`; it is not part of this closed task.
