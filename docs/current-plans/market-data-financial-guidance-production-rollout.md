- title: 行情优先级与财报数字软核验生产发布
- status: in_progress
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

## Goal

把结构化行情优先于开放搜索、AAOI 涨跌幅统一使用服务端 `hone_change_basis.pct`，以及财报数字的最新报告期/口径/准确性软核验发布到生产后端。

## Scope

- 审查并提交当前六个实现文件和对应文档，不夹带其它工作树内容。
- 运行本机可执行的发布门禁；PostgreSQL 依赖门禁若本机环境不可用，则必须等待 GitHub CI 对目标 revision 给出结果。
- 等待目标 revision 的不可变 GHCR runtime image，按 digest 在生产主机 staging；两次零活跃会话后原子切换并保留即时回滚版本。
- 验证精确 revision、cloud authority、PostgreSQL/OSS、服务与渠道状态、公共 API，并运行不涉及真实用户写入的投研 canary。
- 本次不创建 `v*` tag，不修改生产数据库或配置，不把生成型质量引导升级为硬门禁。

## Verification

- Changed-file rustfmt、workspace check/test、Web tests、Public Community Edge tests、CI-safe regression 与 diff/secret checks，按本机能力执行。
- GitHub CI、Runtime Image、Secret Scan 与相关 workflow 对目标 revision 成功。
- 生产 `/api/meta` 精确匹配目标 Git SHA，cloud/PostgreSQL/OSS 权威状态健康，active chats 切换前后为零。
- `hone-web.service` 和已配置渠道 worker 健康且无新增重启；公共未登录 API 返回应用 JSON `401`。
- 真实 canary 检查结构化行情优先、服务端涨跌口径和财报报告期/指标口径引导，不发送证券交易指令或修改用户数据。

## Documentation Sync

- 完成后追加既有 `docs/handoffs/2026-08-22-market-data-source-priority.md` 的生产发布证据。
- 从 `docs/current-plan.md` 移除本任务，将本计划归档到 `docs/archive/plans/`，并更新 `docs/archive/index.md` 既有条目。

## Risks

- 生产切换必须以不可变 GHCR digest 为部署身份，不能使用 mutable `main` tag。
- 若 GitHub CI 或 runtime image 失败，停止发布；不得为了通过固定内容阈值而删改研究规则。
- 若生产磁盘低于 runbook floor、活跃会话无法归零、精确 revision 或 cloud authority 不匹配，停止切换或回滚。
