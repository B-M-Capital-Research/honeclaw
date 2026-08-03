- title: SNDK current-listing evidence and GCE rollout
- status: done
- created_at: 2026-08-03
- updated_at: 2026-08-03
- owner: Codex
- related_files: `crates/hone-tools/src/data_fetch.rs`, `agents/function_calling/src/lib.rs`, `crates/hone-channels/src/{agent_session/tests.rs,prompt.rs}`, `crates/hone-core/src/config/agent.rs`, `config.example.yaml`, `skills/stock_research/SKILL.md`, `soul.md`, `docs/invariants.md`
- related_docs: `docs/current-plans/ticker-resolution-architecture.md`, `docs/decisions.md#d-2026-08-03-02-reject-malformed-read-only-calls-individually-and-require-listing-evidence`, `docs/runbooks/backend-deployment.md`
- related_prs: none

## Summary

第一次 production-like 的“用户问 SNDK 财报前瞻”会话中，DataFetch 已成功取得 SNDK 当前行情，但回答仍依据陈旧模型记忆声称 SNDK 没有上市，并错误引导用户查看 WDC。第一阶段给结构化结果补充当前上市证据并部署了 `116dc54b`，一次 canary 通过；然而北京时间 12:52 的真实 Web 请求随后再次发布同类错误，证明单次成功 canary 没覆盖另一条实际模型调用分支，第一阶段不能视为问题已经关闭。

当前 provider 的 exact search、quote 与 profile 均把该证券识别为 `SNDK` / Sandisk、NASDAQ 上市、正价格且 `isActivelyTrading=true`。历史事实也已变化：Western Digital 在 2025-02 完成 Flash 业务分拆，Sandisk 随后以 `SNDK` 在 Nasdaq 开始 regular-way trading。因此，2016 年收购事实不能作为 2026 年“未上市”的依据。

## What Changed

- `snapshot` 与 `earnings_outlook` 都返回 `hone_security_listing_evidence`，并在财报前瞻中同时抓取 profile。
- 只有同代码 quote/profile、正价格、非空交易所及 provider 明示 active 时才输出 `active_listing`；显式 inactive 仅在无正价格且同代码 profile 明示 inactive 时输出，其余情况一律 `unverified`。
- Agent、channel prompt、soul 和 stock-research skill 明确规定：当前同代码 `active_listing` 证据优先于陈旧并购、退市或改名记忆；若当前官方 filing 与 provider 冲突则继续核验并披露冲突。
- 增加 SNDK snapshot、SNDK earnings-outlook 与 quote/profile 冲突回归，避免用单一弱信号断言上市状态。

## Verification

- Passed: SNDK snapshot/outlook, conflicting listing evidence, snapshot aggregation, partial earnings-outlook coverage, Agent prompt, and channel soul-contract focused Rust tests.
- Passed: `cargo check -p hone-tools -p hone-agent -p hone-channels --tests`.
- Passed: all 44 finance contract checks, changed Rust-file `rustfmt --check`, and `git diff --check`.
- Commit `116dc54b3540e30b8420aaacf007ede33f0b9f5d` passed the pre-push rustfmt and gitleaks hooks and was pushed to `main`.
- GCE built all six production binaries from that exact commit with `profile=release`, `source=workspace`; the immutable release manifest verified six binaries plus `soul.md` and the stock-research skill.
- Before cutover, Web was active, Feishu was already inactive, the protected runtime-env validator passed, cloud PostgreSQL/S3 authority passed, and two separate active-chat reads returned zero. The atomic Web-only cutover became ready on probe 2 with about three seconds of downtime; Feishu remained inactive.
- After cutover, `/api/meta` reported the exact SHA, release/workspace provenance, cloud mode, healthy PostgreSQL/S3, authoritative cloud storage, and zero local durable dependencies. Ports 8077/8088 were listening, both public unauthenticated boundaries returned `401`, Web warning-level journal lines were zero, and active chats were zero.
- Fresh direct actor `codex-canary-116dc54b-sndk-1785732115` replayed `sndk财报前瞻`. It completed in 92 seconds with current-turn DataFetch search/quote/profile/earnings-outlook/news plus Web evidence, exactly one start, one assistant answer, one successful finish, zero reset/error, byte-identical SSE/history, explicit SNDK earnings coverage, no “未上市/改看 WDC” denial, and active chats returning to zero.
- Temporary remote clone, canary files, and the temporary build swap were removed. Disk returned to 8.5 GiB free. The old immutable release and protected asset rollback copy remain available.
- Replacement commit `aed36044` passed the focused SNDK pre-turn/listing regressions, complete Hone Tools/Core/Agent/Channels library suites, all 44 finance contracts, workspace `cargo check`, CI-safe regression suite, changed-file rustfmt and push hooks. It was pushed to `main` and remained an ancestor of the deployed source revision.
- GCE fetched the then-current exact `main` revision `5028870dcb341476e17b57fdfa84d72624b04200` without modifying the dirty operational checkout, built all six release entries from a detached isolated worktree with the repository-mandated checkout-local `target/source-runtime`, and verified the embedded Git SHA plus every file in `SHA256SUMS` before publishing immutable release `/opt/hone/releases/5028870dcb341476e17b57fdfa84d72624b04200-sndk-quota100-20260803`.
- The production config backup is `/srv/honeclaw/.rollbacks/config-pre-5028870d-20260803T151518Z.yaml`. The staged `daily_conversation_limit: 100` passed the candidate CLI's `config validate`; after restart both canonical config and `/srv/honeclaw/data/runtime/effective-config.yaml` report 100.
- Pre-cutover runtime-env validation passed and two active-chat reads three seconds apart returned zero. The config and release symlink were replaced atomically, only `hone-web.service` restarted, and already-inactive Feishu/Discord units stayed inactive. Journal timestamps show stop at `15:21:12Z` and both 8077/8088 listening at `15:21:14Z`, approximately two seconds of API interruption.
- Post-cutover `/api/meta` reports exact `5028870d`, `source=direct_source_runtime`, healthy PostgreSQL/S3, authoritative cloud storage and zero local durable dependencies. The main process resolves into the new immutable release, unauthenticated public auth on 8088 remains `401`, and active chats returned to zero.
- Two fresh strict Web actors independently replayed `SNDK 财报前瞻` and `SNDK财报前瞻`. Both pre-turn paths executed Web search plus two DataFetch calls before `agent.run`; the live tool loop then executed current SNDK quote/earnings/news, with the second canary explicitly executing `quote SNDK` and `snapshot SNDK`. Both requests exited 0 with one successful answer recognizing SanDisk/闪迪; automated final-text checks found zero instances of “已退市”, “未上市”, or “无法提供当前财报前瞻”.
- After acceptance, the explicit candidate Cargo target was cleaned (`4.9 GiB` rebuildable cache removed). The running immutable release and prior release were retained, and root disk free space returned to 11 GiB.

## Production Web Regression After Initial Rollout

- 北京时间 2026-08-03 12:52 的真实公共 Web 请求 `sndk财报前瞻` 在精确 `116dc54b` 上仍回复 “SNDK 已退市/无法提供/改看 WDC”。日志确认它经公共用户严格 `function_calling` runner，而非通过第一次成功 canary 的幸运工具序列。
- 开启审计的隔离非管理员 Web canary 精确复现：首批同时生成有效 DataFetch search、格式错误的 `earnings_outlook`（使用 `query` 且早于 search 结果）以及把 SNDK 错联到 Western Digital 的 Web 搜索。旧安全门因一个参数错误取消整个批次，然后关闭工具并允许模型直接发布陈旧退市记忆。
- 新修复仅改变已注册、已知只读调用的失败隔离：格式错误调用得到 `invalid_read_only_tool_shape` / `rejected_before_execution` 结果，不触发 observer/registry/provider；有效 search/Web siblings 继续。未知、未注册、写入或持久化调用仍让全批零执行。
- 新增上市发布边界：明确证券代码只有获得当前同代码 `inactive_listing` 才能断言已退市/未上市；`active_listing` 或同代码正报价加 active/exchange profile 会触发一次同-Agent 纠正，第二次仍矛盾则返回证据缺口，不允许用历史母公司替代。完整 SNDK 回归模拟了真实首批调用、后续 active outlook 证据和故意错误终稿。
- 本阶段同时把 GCE 生产 `agent.daily_conversation_limit` 从 12 调至 100。replacement commit `aed36044` 随当前主线精确 revision `5028870d` 完成构建、零会话合并切换与两次独立严格 Web canary；该生产回归现已关闭。

## Risks / Follow-ups

- `active_listing` is intentionally conservative and provider-derived; absence of the marker must not be interpreted as delisted.
- A future provider schema change could omit `exchange` or `isActivelyTrading`; the result will degrade to `unverified` instead of making a false claim.
- The listing-regression canary passed, but the same generated earnings answer contained a separate temporal-tense inconsistency: at Beijing 2026-08-03 it described a 2026-08-05 result as already released. This rollout does not claim that independent earnings-date wording issue fixed; treat it as a follow-up evidence-ordering defect rather than weakening the listing proof.
- Immediate GCE rollback is `/opt/hone/releases/116dc54b3540e30b8420aaacf007ede33f0b9f5d-sndk-listing-20260803` together with `/srv/honeclaw/.rollbacks/config-pre-5028870d-20260803T151518Z.yaml`. The older `c4c21723` release and `/srv/honeclaw/.rollbacks/pre-116dc54b-20260803` asset copy remain a second rollback tier.

## Next Entry Point

For the separate earnings-date tense defect, start from the exact SNDK canary evidence and add a focused current-time versus earnings-event-date regression without weakening current-listing precedence. For rollback, drain active chats twice, restore the `116dc54b` release symlink and pre-`5028870d` config, restart Web, then repeat the same meta/auth/authority/quota gates.
