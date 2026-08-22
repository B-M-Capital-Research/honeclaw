- title: 明确公司投研的行情优先取证
- status: done
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-tools/src/web_search.rs`
  - `crates/hone-tools/src/registry.rs`
- related_docs:
  - `docs/invariants.md`
  - `docs/archive/plans/market-data-source-priority.md`
  - `docs/archive/plans/financial-report-data-verification-guidance.md`
  - `docs/archive/plans/market-data-financial-guidance-production-rollout.md`
- related_prs: none; direct `main` implementation commit `3678558483628b605aa927cfa168539a22eca84a`

## Summary

明确公司或证券的交互式投研现在先把结构化行情作为第一事实来源：实体解析后优先读取 `snapshot`，不适用时组合 `quote/profile`，涉及盘前盘后时补 `extended_hours`；开放 Web 搜索随后补公告、关系、事件和因果。该顺序是 Agent 软引导和工具展示信号，不是缺数据即拒答的内容门禁。实现已作为精确 revision `3678558483628b605aa927cfa168539a22eca84a` 发布到生产后端。

## What Changed

- 全局金融提示、严格 function-calling 的发现/取证/自然收口提示，以及 DataFetch/WebSearch 工具描述统一表达 `search → snapshot/quote → Web` 的优先顺序。
- `ToolRegistry::get_tools_schema` 只调整相对优先级：`data_fetch` 在首位，`web_search` 在普通业务工具之后，其它工具之间保留原顺序；所有工具仍完整暴露，执行能力不变。
- 明确 provider 无覆盖或失败时不得反复补取、拒绝终稿或停止回答，继续用已有证据和公开来源并披露具体缺口。
- 严格 function-calling 的市场涨跌幅证据从 provider `changesPercentage` 改为服务端 `hone_change_basis.pct`；AAOI 回归固定 `129.10 → 124.82 = -3.32%`，拒绝把原始 `-3.46%` 当权威值。

## Follow-up: 财报数字时效与准确性软核对

- 回答引用营收、净利润、EPS、EBIT、EBITA、EBITDA、利润率、现金流或资产负债表数字前，Agent 先确认最新已披露的 `date` / `period`，并明确使用单季、`hone_ttm.period_ends` 还是 `hone_forward.forward_period_ends`。"最新" 不得指向尚未披露的季度。
- EBIT、EBITA、EBITDA 与营业利润视为不同指标，不得互相替代，也不得在来源没有给出时自行补算。季度、年度、TTM 与 forward 不得混用。
- 关键数字的 provider 窗口疑似滞后或来源冲突时，只做一次针对性的公司 IR、财报公告或监管文件复核。官方二次来源不可得时，标明截至日、来源层级和具体缺口后继续回答。
- 该行为通过全局提示、严格终稿提示、证券预取上下文和 `hone_financials_policy` 共同引导；没有新增逐数字双来源、内容 validator、缺项拒答、反复搜索或自动重写门禁。

## Verification

- `cargo test -p hone-tools data_fetch::tests`: 57 passed.
- `cargo test -p hone-tools web_search::tests`: 19 passed.
- `cargo test -p hone-tools registry::tests`: 5 passed.
- `cargo test -p hone-agent`: 153 passed.
- `cargo test -p hone-channels finance_policy_prioritizes_structured_market_data_without_a_completion_gate`: 1 passed.
- `cargo check -p hone-channels`: passed.
- Financial-report guidance targeted tests: DataFetch bundle 1/1, function-calling prompt 1/1, global channel policy 1/1, pre-turn financial guidance 1/1.
- `rustfmt --edition 2024 --config skip_children=true ...` and `git diff --check`: passed.
- Broader PostgreSQL-backed test subsets could not complete on this host: `scripts/dev_pg.sh up` reported Docker unavailable. Before the environment failure, `hone-tools` reported 169 passing / 26 PostgreSQL-dependent failures, and the broad `hone-channels prompt` filter reported 47 passing / 26 PostgreSQL-dependent failures.

## Production Deployment

- Implementation commit `3678558483628b605aa927cfa168539a22eca84a` was pushed directly to `main`; no PR, formal release, or `v*` tag was created.
- GitHub Runtime Image run `32548881694` published and verified immutable digest `sha256:fc6029b42f04e2ce58b944bc6f4c9acb5fa654f808d804e9b3c4ed0d7e662676`. Secret Scan passed; the CI frontend lane passed Web tests plus Public Community Edge typecheck/tests, and the Rust lane passed changed-file formatting and workspace compile.
- The CI Rust test lane stopped at the pre-existing unchanged `soul.md` fixed-character-budget assertion (`soul_prompt_keeps_the_full_investment_contract`): the target reported 161 passed / 1 failed in `hone-core`, and its parent revision failed the same test at the same assertion. Per the repository generative-workflow invariant, the release did not delete prompt rules or raise a mechanical content threshold to manufacture a green result.
- Production initially had about 1.78 GiB free, below the 2 GiB staging floor. Exact old releases `2a738b12`, `253421df`, and `69933303` were individually bundle-verified, proven non-current/non-rollback with no open process references, and confirmed rebuildable from immutable GHCR tags before removal. No user data, database, skill, current runtime, or retained rollback was removed; staging began with about 5.27 GiB free.
- The target bundle was staged by exact digest, then independently verified against its embedded revision and payload checksums. Deployment tools and the protected runtime-environment checker were copied only into validated `/tmp` directories because the remote operations checkout lacked them; their SHA-256 values matched the reviewed local revision, and the temporary directories were deleted after use. The remote checkout was not pulled or overwritten.
- The first cutover reached lightweight readiness but `/api/meta` exceeded the initial 30-second acceptance timeout, so the prepared rollback restored `e08bb460…` automatically. The rollback served application JSON `401`, had zero active chats and no critical logs. A bounded second cutover passed its first complete meta attempt and is the accepted deployment.
- Production now points `/opt/hone/current` to `36785584…-ghcr-runtime` and `/opt/hone/previous` to `e08bb460…-ghcr-runtime`. Two pre-cutover idle reads and all post-cutover/soak reads were zero; `hone-web.service` is active with `NRestarts=0`, the integrated Feishu stream emitted reconnect markers, and recent critical-log count is zero.
- `/api/meta` reports exact `build.git_sha=36785584…`, `build.source=ghcr_linux_oci`, `cloud_mode=cloud`, healthy PostgreSQL and OSS, `cloud_storage_authoritative=true`, and `local_durable_dependency_count=0`. Loopback and public unauthenticated auth probes return application JSON `401`; production timezone remains `Asia/Shanghai`.
- The running binary contains the structured-market-priority, financial-period verification, and `hone_change_basis` policy markers. An existing authenticated Chrome tab loaded the production chat and history successfully; no canary message was sent because sending a user-visible message requires action-time confirmation.

## Risks / Follow-ups

- Stronger source ordering can add one model/tool round; preferring the aggregate `snapshot` is intended to contain that cost.
- Schema order is a model hint. It does not guarantee every provider/model will choose the preferred tool, so production acceptance should inspect actual traces rather than infer success from prompt text.
- With explicit confirmation to create visible test messages, run three fresh canaries: a named-company overview, a two-company relationship question, and an AAOI-style current/after-hours move plus a latest-period financial metric. Confirm each trace resolves identity, attempts structured market data before Web, uses the service-computed change basis, binds financial figures to the disclosed period, and still answers naturally when one provider component is unavailable.

## Next Entry Point

Start with `crates/hone-channels/src/prompt.rs` for cross-runner policy and `agents/function_calling/src/lib.rs` for strict fallback behavior. For runtime rollback, require two zero-active-chat reads, atomically restore `/opt/hone/current` to the retained `e08bb460…-ghcr-runtime`, restart `hone-web.service`, and repeat exact meta/cloud/public probes. Do not convert the priority into a missing-data validator or forced retry loop.
