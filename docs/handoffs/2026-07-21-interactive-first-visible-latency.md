# Interactive First-Visible Latency Repair

- title: Interactive first-visible latency repair and production redeployment
- status: in_progress
- created_at: 2026-07-21
- updated_at: 2026-07-22
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-core/src/{agent.rs,tool_effect.rs}`, `crates/hone-channels/src/agent_session/{core.rs,emitter.rs,helpers.rs,restore.rs,tests.rs,types.rs}`, `crates/hone-channels/src/{investment_response_guard.rs,prompt.rs,turn_builder.rs}`, `crates/hone-channels/src/runners/{types.rs,tool_reasoning.rs}`, `crates/hone-web-api/src/routes/{chat.rs,public.rs}`, `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs: `docs/adr/0004-agent-owned-research-loop.md`, `docs/decisions.md#d-2026-07-21-01-commit-only-delivered-interactive-prefixes-and-remove-synchronous-ttft-work`, `docs/decisions.md#d-2026-07-22-01-bound-interactive-finance-fan-out-and-ack-a-typed-web-prefix`, `docs/current-plans/ticker-resolution-architecture.md`, `docs/runbooks/backend-deployment.md`
- related_prs: N/A
- verification: first-phase exact build/deploy completed and its exact-query canary exposed unbounded finance fan-out; second-phase complete local Rust/Web/Worker/CI-safe gates pass, while exact commit/build/redeployment and production acceptance remain in progress
- risks: phantom prefix recording, post-ACK admission of an unregistered/read-write/malformed call, invalid DataFetch activating deferred ACK, explicit identity routes bypassing the first-batch six-route ceiling, a model final that does not preserve the typed prefix, incomplete research at a hard budget, and a deployment that does not use the exact verified revision

## Incident

The production Web turn `大A有没有类似CRWV、Nebius这样的数据中心的标的` entered the backend at `2026-07-21 19:50:19.973` and completed at `19:51:59.427`, roughly `99.45s` later. The user experienced this as almost two minutes before the first visible word, followed by the answer arriving progressively.

Trace attribution separated the delay:

- synchronous pre-run SessionCompress: `14.377s`;
- five model calls: about `76.5s` total;
- ten business-tool calls: about `8.45s` total;
- final model stream: about `39.45s`, buffered by the runner before publication;
- Web SSE and client rendering: immediate after backend emission, so not the bottleneck.

This was not fixed by restarting the existing binary. The running immutable deployment was `100f5608`, and the later `main` revision initially had no TTFT runtime diff relative to that build. Redeployment was deliberately paused until the repair was restored, tested, committed, and built from an exact revision.

## First Deployment And Failed Canary

First-phase commit `b06de76a` was pushed, built into immutable `target/deploy-b06de76a`, and gracefully deployed. Web/console/Discord/Feishu processes, ports `8077/8088`, cloud storage authority, and active-run health were normal. The previous `target/deploy-100f5608` runtime remained intact.

Fresh direct actor `codex-canary-b06de76a-exact-1784656112`, run `ab697de9-d942-482b-9590-906eaaf73d6f`, replayed the exact incident query. It produced no assistant delta within 240 seconds and completed at `246.037s`:

- 11 model calls consumed `200.365s`;
- 105 tools consumed `45.271s`: 44 DataFetch searches, 42 quotes, 2 profiles, and 17 Web calls;
- the route ledger expanded from the two user anchors to 26 keys, 24 still unresolved, so evidence-floor tool choice remained `Required`;
- SSE immediately carried run/progress/tool events, but no assistant delta/reset/error/finish appeared before the delayed final;
- the persisted assistant was 3,731 characters with hash prefix `19b5c7b`, and active chats returned to zero.

The first-phase fast context path worked, but batching guidance and final-only streaming could not control a provider that kept discovering aliases and candidates. The remaining defect was unbounded finance research fan-out, not SSE delivery or one slow external call.

## Current Repair Boundary

- Only a self-contained strict Interactive request whose current input contains a deterministic explicit security seed uses the fast first attempt. Referential turns such as `第二个再详细点` or `继续分析它` keep the full history/compaction path. Actual context overflow still forces compaction/retry; manual `/compact` and noninteractive behavior stay unchanged.
- Fast-path active context and up to four earlier user-only reference utterances come from one durable Session snapshot. Selection uses final durable user-row order, stops before the current row, and filters automation, failed groups, slash commands, summaries, assistants, and tool payloads without a same-text exemption. Previously invoked skill prompts are restored separately from Session metadata, remain subject to current activation, and do not consume the four-user-reference budget.
- Independent candidate work is still requested in one model batch, but hard runtime limits now dominate prompt compliance: at most three finance tool batches, 24 total calls, 20 DataFetch calls, six Web calls, and six accepted identity routes. T0 ACK starts those counters immediately, so Web-only batches before DataFetch activation still consume the batch, total-call, and Web-call budgets. The executor remains serial. Only a valid identity search can create a route; an explicit route without a valid call-scoped `identity_match` and any seventh new route, including in the first batch, are rejected before observer/registry/network access without ledger pollution. An unknown alias whose symbol uniquely matches an accepted route merges into it.
- When a batch or call ceiling is reached, the next iteration of the same Agent receives `tools=[]` and writes one natural final from its existing current-turn evidence. Missing candidates, quotes, or business evidence are disclosed as concrete gaps. This is not `finish_research`, a handoff, locator correction, terminal audit, recovery role, or second synthesis.
- Web finance configures exactly one typed line: `数据时间：北京时间 {answer_time}；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露`. It contains no market conclusion. A deterministic explicit-security seed may request ACK before the first model call. A name-only finance turn waits until a budget-accepted batch includes a registered DataFetch call with supported `data_type`, every required target, and valid identity shape, while every call in that batch is registered, structurally valid, and known read-only. Unsupported/missing-target/malformed/rejected DataFetch never activates research or deferred ACK. Ordinary non-finance turns never ACK it, and the Agent final must reproduce it byte-for-byte.
- The prefix becomes committed only after the unique downstream sink accepts it. Cancellation, a closed/rejected receiver, ambiguity, or backpressure creates no visible/persisted bytes. After ACK, every call in a batch must name an active registered tool, have parseable structurally valid arguments, and be classified known read-only; otherwise the whole batch fails before assistant tool framing, observer notification, registry execution, or network access.
- A successful final publishes only the suffix after the already visible prefix. A later failure appends `本轮研究未能完成，暂未形成可供参考的标的结论。`, persists exactly the prefix plus suffix with partial/failure metadata, and closes once without reset or an error-card flash.
- Separate telemetry records first provider delta, first runner commitment, first successful Web delta send, and final completion.

## Verification

Required before production acceptance:

- focused Agent/channel tests for the three-batch tools-disabled final including T0 Web-only batches, intrinsic total/DataFetch/Web limits without caller budgets, first-batch six-route admission, missing/invalid `identity_match` pre-network rejection and non-poisoning, unique-symbol merge, unsupported/missing-target/zero-budget DataFetch no-ACK, pre-model/deferred/rejected ACK, post-ACK registered + structurally-valid + known-read-only enforcement, exact success-tail equality, and exact failure-suffix partial persistence;
- finance static contract suite plus complete workspace Rust/Web/Worker/CI-safe gates;
- formatting and diff checks;
- exact commit pushed to `main`, immutable `target/deploy-<sha>` build, binary/skill/soul/config/public-asset manifest verification;
- zero-active-chat SIGINT restart from the repository root with cloud authority, PostgreSQL, object storage, ports, process cwd/executable, and anonymous auth boundaries healthy;
- fresh actor replay of the exact incident query, recording first `assistant_delta`, `run_finished`, model/tool/route counts, first visible line, exact visible/persisted bytes, two-row history, and active chats returning to zero. Acceptance requires first visibility under 60 seconds (target under five seconds for this deterministic seed), no more than four model calls, no more than 24 tools/six routes, and one terminal without reset/error flash.

## Rollback

Keep both `target/deploy-b06de76a` and `target/deploy-100f5608` intact. If the follow-up process fails health, correctness, one-output/history, or side-effect checks, drain active chats, SIGINT the new CLI supervisor, and restart `b06de76a` with the same reviewed repository-root environment; use `100f5608` only if the first-phase runtime is itself unusable. Whole-final buffering is an acceptable temporary fallback, but unbounded route/tool growth, phantom prefix recording, post-ACK admission without registered + structurally-valid + known-read-only proof, invalid DataFetch activation, synchronous first-attempt compaction, or a second synthesis/publication gate is not.

## Handoff State

First-phase complete repository gates and exact deployment are retained as evidence, but its production canary failed the latency requirement. The second-phase implementation and local verification are complete:

- workspace `cargo check` passed; workspace `cargo test` passed, including Agent `119/119`, Channels `667/667`, Web API `128/128` with two credentialed tests ignored by contract, Core `124/124`, and Tools `143` passed with one optional renderer test ignored;
- Web tests passed `280/280`; Public Community Edge typecheck and tests passed `45/45`;
- finance static contracts passed `39/39`; the complete `tests/regression/run_ci.sh`, changed-file rustfmt, shell syntax, and `git diff --check` passed;
- regressions include forced bounded final, T0 Web-only/intrinsic Web caps, first-batch route admission/merge, invalid identity zero-network/non-poisoning, post-ACK registered/read-only/shape safety, unsupported/missing-target/budget-rejected DataFetch no-ACK, exact success-tail equality, and explicit failure-suffix partial equality;
- no second-phase commit, push, immutable build, restart, or production canary has been claimed yet.

Next entry point: finish the channel suite, run every repository gate, review the exact diff/docs, commit and push, build `target/deploy-<sha>`, drain/restart, then replay the exact query with a fresh actor. The umbrella ticker plan remains `in_progress` after this latency phase because the separately tracked scheduler `800G` / `NAND` / `AST` / `SEC` entity-guard P2 is still open; do not archive that plan when this deployment completes.
