- title: 行情优先级与财报数字软核验生产发布
- status: archived
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-tools/src/registry.rs`
  - `crates/hone-tools/src/web_search.rs`
- related_docs:
  - `docs/archive/plans/market-data-source-priority.md`
  - `docs/archive/plans/financial-report-data-verification-guidance.md`
  - `docs/handoffs/2026-08-22-market-data-source-priority.md`
  - `docs/runbooks/backend-deployment.md`
- related_prs: none; direct `main` implementation commit `3678558483628b605aa927cfa168539a22eca84a`

## Goal

把结构化行情优先于开放搜索、AAOI 涨跌幅统一使用服务端 `hone_change_basis.pct`，以及财报数字的最新报告期/口径/准确性软核验发布到生产后端。

## Completed Scope

- 审查、提交并推送六个实现文件及长期留档，没有创建正式版本 tag。
- 等待并验证精确 revision 的不可变 Linux runtime image；按 digest staging、两次零会话读取、原子 symlink 切换与完整 cloud-authority 验收发布。
- 生产磁盘低于 staging floor 时，只清理三个已验证、未引用且可从 GHCR 重建的旧 runtime；保留当前、即时回滚和次级回滚，不触碰用户数据、数据库或技能。
- 首次切换因完整 meta 超过 30 秒自动回滚；确认旧版健康后进行一次有界重试，成功切换并通过 soak。
- 未修改生产配置或数据库，未把生成型质量引导升级为内容门禁；未在没有 action-time 确认时发送用户可见 canary 消息。

## Verification

- Local: relevant DataFetch/WebSearch/registry tests, four financial-guidance tests, `hone-agent` 153/153, workspace check, rustfmt, diff check, and pre-push gitleaks passed. Docker/PostgreSQL and Bun were unavailable locally.
- GitHub: Runtime Image `32548881694` and Secret Scan passed; frontend tests plus Public Community Edge typecheck/tests passed; Rust format and compile passed.
- The only CI Rust-test failure was the unchanged parent-baseline `soul.md` fixed-character-budget assertion. It stopped `hone-core` at 161 passed / 1 failed; no prompt content or threshold was changed to satisfy that mechanical gate.
- Runtime image: revision `3678558483628b605aa927cfa168539a22eca84a`, digest `sha256:fc6029b42f04e2ce58b944bc6f4c9acb5fa654f808d804e9b3c4ed0d7e662676`, embedded metadata and every payload checksum verified.
- Production: exact meta revision/source, cloud mode, PostgreSQL, OSS, authority and zero local dependencies passed; active chats remained zero; Web stayed active with zero restarts; public and loopback auth returned JSON `401`; recent critical logs remained zero; integrated Feishu reconnect markers were present.
- Read-only UI acceptance: an existing authenticated Chrome tab loaded production chat/history/input successfully. No message was transmitted.

## Documentation Sync

- Deployment evidence, rollback path, CI baseline exception and canary boundary were appended to the existing same-day handoff.
- The existing archive-index entry now points to all three plans and the deployed revision/digest.

## Risks / Follow-up

- Prompt/tool ordering is a model hint; only explicitly authorized visible canaries can prove the production model's trace choice on fresh turns.
- Retained immediate rollback is `e08bb460…-ghcr-runtime`; rollback still requires two zero-active-chat reads and full meta/cloud/public acceptance.
- The fixed `soul.md` length assertion remains an independent CI baseline defect and should be addressed under generative-workflow governance, not by deleting rules or silently raising a number.
