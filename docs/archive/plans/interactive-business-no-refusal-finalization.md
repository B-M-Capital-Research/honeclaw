# Interactive Business No-Refusal Finalization

- title: Remove generic refusal exits from ordinary Interactive business turns
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-core/src/tool_effect.rs`, `crates/hone-channels/src/{attachments/,agent_session/,prompt.rs,response_finalizer.rs,runtime.rs}`, `crates/hone-tools/src/skill_tool.rs`, `skills/image_understanding/SKILL.md`, `tests/regression/ci/`, `docs/bugs/web_direct_image_attachment_not_readable_internal_debug_leak.md`
- related_docs: `docs/current-plan.md`, `docs/invariants.md`, `docs/repo-map.md`, `docs/decisions.md`, `docs/handoffs/`, `docs/archive/index.md`, `docs/current-plans/ticker-resolution-architecture.md`

## Goal

Ordinary Interactive user questions must end with a useful natural answer or a narrow, actionable clarification. A blocked, malformed, unavailable, or conservatively classified tool must not replace a finance/portfolio/image-analysis turn with a generic research-failure sentence. Preserve the existing investment answer prefix, answer structure, evidence rules, and execute-once safety for real mutations.

## Scope

- Reproduce the 2026-07-26 Web direct portfolio-image incident from the exact Session rows and runtime failure key.
- Audit the Interactive path for fixed refusal copy, generic failure suffixes, non-finance refusal instructions, empty-output handling, post-prefix tool admission, and outer retry/finalization behavior.
- Distinguish durable business mutations from idempotent skill-invocation metadata. A `skill_tool` call that does not execute a script may be retried/read as a non-business-state operation; executable skill scripts remain unsafe unless a separately audited read-only contract exists.
- Keep every rejected unknown, malformed, unregistered, or write-capable tool outside the assistant tool frame, observer, registry, and network boundary. Instead of failing the whole read-only business turn, give the same Agent one bounded tools-disabled continuation that must answer from already available evidence and disclose only the relevant missing fact.
- Keep the byte-exact existing Web finance header and body format, but buffer the whole answer until the same Agent has completed a usable response. Do not publish an irreversible header before any image, tool, or model-dependent work is complete, and remove the canned research-failure suffix from the production path.
- Materialize readable image text before the model turn where the host supports it, carry the extraction result in the trusted attachment context, and keep a narrow product-level fallback when extraction genuinely yields no text.
- Remove the stale prompt instruction that rejects every non-finance question; greetings and ordinary questions remain short, while finance-specific formatting and evidence requirements activate only for finance.
- Add focused unit and CI-safe regressions proving the incident no longer reaches a generic refusal, unsafe tools still execute zero times, image context is readable, and the established investment answer format is unchanged.

## Validation

- Focused `hone-core`, `hone-tools`, `hone-channels`, FunctionCalling Agent, and Web attachment tests.
- A static CI regression that rejects new generic Interactive refusal/failure exits while allowing explicit mutation-state and infrastructure disclosure at their narrow boundaries.
- Repository formatting, changed-file format check, workspace compile/test, Web tests, Edge Worker checks, and CI-safe regressions per `AGENTS.md`.
- Exact-commit immutable build, zero-active-chat controlled restart, storage/auth/channel health checks, and fresh Web replays for:
  - the incident portfolio screenshot turn;
  - an ordinary text portfolio question;
  - a broad-market question;
  - an ordinary non-finance question;
  - a write-capable request proving no duplicate or unconfirmed mutation.
- Production acceptance requires exactly one successful terminal, no generic research failure, no error/reset flash, byte-identical visible/history content, and active chats returning to zero.

## Completion

- Exact implementation commit `75ca1957ebceaa3f8d564d423bcb6f8940b5c74e` was rebased onto the latest `origin/main`, pushed, built into `target/deploy-75ca1957`, and verified against a 501-file SHA-256 manifest.
- Web and Feishu were drained at zero active Web chats and restarted from the exact package. The live supervisor and Feishu working directories are the repository root; ports `8077/8088`, cloud authority, PostgreSQL, object storage, local durable dependency count, local/origin/public auth boundaries, and zero-active-chat health all passed. Discord remains intentionally offline because its configured gateway credential is invalid.
- The unchanged finance first-line contract was observed in production for `美股为什么大跌`; the answer completed successfully in about 39.6 seconds, corrected the broad-market premise from verified SPY/QQQ/IWM evidence, emitted no canned refusal, and matched its two-row persisted history byte-for-byte.
- A non-finance CPU question completed directly in about 7.7 seconds without finance formatting. The supplied feedback screenshot was read by real Apple Vision OCR on the production host, including the exact fixed-failure sentence and user feedback. An online Web Agent replay of that extracted attachment block completed substantively in about 16.1 seconds with one successful terminal and byte-identical two-row history.
- Repository verification passed: workspace check/test excluding Apple clients, Agent `135/135`, Channels `680 passed` plus one explicit host OCR ignore, Web API `140 passed` plus two credentialed ignores, Web `294/294`, Edge Worker typecheck and `45/45`, finance contract `43/43`, complete CI-safe regressions, changed-file formatting, diff checks, and real-host OCR.
- Handoff: `docs/handoffs/2026-07-26-interactive-business-no-refusal-finalization.md`.

## Documentation Sync

- Record the revised tool-effect and same-Agent continuation boundary in `docs/decisions.md`, `docs/invariants.md`, and `docs/repo-map.md`.
- Update the existing image-attachment bug rather than creating a duplicate.
- Update `docs/bugs/README.md`, this plan, and `docs/current-plan.md` as implementation and production verification progress.
- On completion, write one reusable handoff, move this plan to `docs/archive/plans/`, remove the active index entry, and add the archive entry.

## Risks / Open Questions

- A blanket “all skill calls are read-only” rule would permit arbitrary script effects; classification must remain argument-aware.
- A blanket retry after an uncertain persistent call could duplicate user-state changes; execute-once and failed-trace protections remain mandatory.
- Local OCR may be host-specific. Unsupported hosts must not fabricate image facts, and CI must validate the pure parsing/formatting contract without requiring Apple frameworks.
- Whole-answer buffering intentionally gives up the earlier sub-second typed-header ACK. This is a correctness tradeoff: the established finance answer bytes and structure stay unchanged, but no business prefix may become a stranded partial/refusal when later work fails.
- The product cannot manufacture a correct answer when every provider and local evidence source is unavailable. The boundary can still preserve a useful evidence-bounded response and must never misreport an unexecuted mutation as successful.
- Whole-answer buffering intentionally moves first visible business text to answer completion; the production broad-market canary took about 39.6 seconds. This is the accepted integrity tradeoff for eliminating stranded headers and fixed refusal tails without changing the prompt format.
