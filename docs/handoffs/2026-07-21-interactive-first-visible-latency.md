# Interactive First-Visible Latency Repair

- title: Interactive first-visible latency repair and production redeployment
- status: done
- created_at: 2026-07-21
- updated_at: 2026-07-22
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-core/src/{agent.rs,tool_effect.rs}`, `crates/hone-channels/src/agent_session/{core.rs,emitter.rs,helpers.rs,restore.rs,tests.rs,types.rs}`, `crates/hone-channels/src/{investment_response_guard.rs,prompt.rs,turn_builder.rs}`, `crates/hone-channels/src/runners/{types.rs,tool_reasoning.rs}`, `crates/hone-web-api/src/routes/{chat.rs,public.rs}`, `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs: `docs/adr/0004-agent-owned-research-loop.md`, `docs/decisions.md#d-2026-07-21-01-commit-only-delivered-interactive-prefixes-and-remove-synchronous-ttft-work`, `docs/decisions.md#d-2026-07-22-01-bound-interactive-finance-fan-out-and-ack-a-typed-web-prefix`, `docs/current-plans/ticker-resolution-architecture.md`, `docs/runbooks/backend-deployment.md`
- related_prs: N/A
- verification: first-phase exact build/deploy exposed unbounded finance fan-out; second-phase `820a7240` achieved `182ms` TTFT but exposed provider-leading-whitespace/prefix alignment and was rolled back; follow-up `2563f7ad` passed the complete repository gates, immutable-build manifest, zero-active-chat restart, runtime health checks, and exact-query production acceptance with `179ms` TTFT and byte-identical successful persistence
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
- Hidden-reasoning stripping may leave provider whitespace before an otherwise exact final prefix. The Session removes only such leading Unicode whitespace when the first non-whitespace content begins with the byte-exact ACKed prefix; any non-whitespace preamble or real prefix mismatch remains a failure.
- Separate telemetry records first provider delta, first runner commitment, first successful Web delta send, and final completion.

## Second-Phase First Deployment And Rollback

Commit `820a724023c86cb6bce387cebf69f91bd8eaf0b9` was pushed to `main`, built from a clean detached worktree into immutable `target/deploy-820a7240`, verified against a deployment manifest, and gracefully deployed after two zero-active-chat checks. Process cwd/executables, ports `8077/8088`, PostgreSQL, OSS, cloud authority, static entry hash, and local/origin/public anonymous auth boundaries were healthy.

Fresh direct actor `codex-canary-820a7240-exact-1784662888`, run `c751af17-7e21-4f68-a755-217663cab319`, replayed the exact incident query:

- first `assistant_delta` arrived `182ms` after POST and contained the exact typed line, reducing the old roughly two-minute first-visible delay to sub-second;
- the loop stayed inside the intended bounds: four model calls, three finance tool batches, ten tools total (four DataFetch and six Web), two accepted identity routes, and no route/tool-budget overflow;
- the fourth same-Agent model call completed a substantive 1,948-character direct final, but its provider body began with two newline bytes before the exact typed prefix;
- the Agent stream compatibility check had used `trim_start()` while the Session's irreversible-prefix check still required raw `starts_with`, so the otherwise valid final failed byte alignment at the Session boundary;
- the user saw the already ACKed prefix plus the explicit research-failure suffix at `89.415s`, persisted history matched those visible bytes, and the stream closed once as partial with no reset/error flash.

Because the exact-query correctness gate failed, the new CLI supervisor was drained and SIGINT-stopped immediately. Production was restored to immutable `target/deploy-b06de76a`; its new CLI PID was `46752`, ports and cloud authority were healthy, and active chats returned to zero. The follow-up strips only leading Unicode whitespace when the first non-whitespace content begins with the exact committed prefix; an arbitrary preamble or true mismatch still fails closed.

## Follow-up Deployment And Successful Canary

Follow-up commit `2563f7ad7db7129d82479653dc91644f0b33dafd` was pushed to `main`, built from a clean detached worktree into immutable `target/deploy-2563f7ad`, and verified against its manifest. The manifest pins that source revision plus the five runtime binary hashes, 27 skill files, `soul.md`, ignored production config, public `index.html`, and `/assets/index-o0hmDXxE.js`. After two zero-active-chat checks, the `b06de76a` supervisor was drained and SIGINT-stopped; the new CLI started from the repository root so it loaded the reviewed ignored `.env`. CLI PID `65914`, console PID `65917`, ports `8077/8088`, runtime-root child cwd, PostgreSQL, OSS, cloud authority, static hash, and local/origin/public anonymous `401` JSON boundaries were healthy.

Fresh direct actor `codex-canary-2563f7ad-exact-1784664084`, run `8d0fdad9-0b76-4bec-a386-345f7c109c18`, replayed the exact incident query:

- the exact typed first line arrived `179ms` after POST;
- four model calls and three finance tool batches completed; 14 calls actually executed—eight DataFetch and six Web—with two accepted identity routes, while the Web ceiling rejected further execution and the fourth same-Agent call received `tools=[]`;
- the provider again began its valid natural final with two newline bytes before the exact prefix, proving the follow-up path was exercised rather than merely covered by a unit fixture;
- the Session removed only those leading whitespace bytes, emitted the final suffix once, and closed with exactly one `run_finished success=true`; there was no partial, reset, error, second generation, or research-failure suffix;
- completion arrived at `117.189s`, but first-visible latency remained `179ms` throughout the research window;
- concatenated SSE deltas and the persisted assistant were byte-identical at 8,167 bytes with SHA-256 `f0c2279619a70c79366c6d723b53f7c07875381d605aaa0a17b6d4e8c1fe3621`; history contained exactly the user and assistant rows, and active chats returned to zero.

The latency subtask is therefore accepted in production. Immutable `target/deploy-b06de76a` remains available as the immediate rollback artifact.

## Verification

Completed acceptance evidence:

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

- workspace `cargo check` passed; workspace `cargo test` passed, including Agent `124/124`, Channels `670/670`, Web API `128/128` with two credentialed tests ignored by contract, Core `124/124`, and Tools `143` passed with one optional renderer test ignored;
- Web tests passed `280/280`; Public Community Edge typecheck and tests passed `45/45`;
- finance static contracts passed `39/39`; the complete `tests/regression/run_ci.sh`, changed-file rustfmt, shell syntax, and `git diff --check` passed;
- regressions include forced bounded final, T0 Web-only/intrinsic Web caps, first-batch route admission/merge, invalid identity zero-network/non-poisoning, post-ACK registered/read-only/shape safety, unsupported/missing-target/budget-rejected DataFetch no-ACK, exact success-tail equality, and explicit failure-suffix partial equality;
- second-phase `820a7240` was pushed, exactly built, deployed, canaried, and rolled back as recorded above; follow-up `2563f7ad` passed focused positive/negative and end-to-end byte-equality coverage, complete repository gates, exact immutable build verification, controlled production deployment, and the successful exact-query canary.

Next entry point: the latency subtask needs no further deployment work. The umbrella ticker plan remains `in_progress` because the separately tracked scheduler `800G` / `NAND` / `AST` / `SEC` entity-guard P2 is still open; do not archive that plan or this shared plan file until that remaining work is complete.
