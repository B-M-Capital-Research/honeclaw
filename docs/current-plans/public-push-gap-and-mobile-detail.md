# Public 推送缺口审计与移动详情弹窗修复

- title: Public 推送缺口审计与移动详情弹窗修复
- status: in_progress
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - packages/app/src/components/public-push-center.tsx
  - packages/app/src/pages/public-site.css
  - crates/hone-web-api/src/routes/public_pushes.rs
  - crates/hone-core/src/persistence.rs
  - production public push/scheduler runtime data (read-only audit)
- related_docs:
  - docs/current-plan.md
  - docs/handoffs/2026-08-09-public-push-gap-and-mobile-detail.md

## Goal

查明指定 Web 账号的“英伟达每日消息”为什么在 2026-07-25 后没有出现在推送中心，并修正 iPhone Safari 上推送详情弹窗横向贴边、视觉歪斜的问题。

## Scope

- 审计窗口：北京时间 2026-07-26 00:00 至 2026-08-09 当前时刻，并回看 2026-07-25 最后一条成功消息作为边界样本。
- 只读核对账号映射、订阅启用状态、cron/scheduler 执行、消息生成、投递日志、public push 存储与 list API 查询。
- 区分“任务未执行 / 执行失败 / 生成但未投递 / 已投递但未入库 / 已入库但前端漏显”。
- 修复移动端详情弹窗在动态视口和安全区下的居中、边距、最大高度与滚动边界。
- 仅在证据确认 durable failure 后改业务代码；不提交私有运行数据或完整消息正文。

## Validation

- 生产数据只读 SQL/诊断命令，给出每天的任务、消息和投递闭环证据。
- 弹窗布局 contract/组件测试、TypeScript、Web 全量测试与 public production build。
- 如需后端修复，增加相应 Rust 回归并执行相关 crate/workspace 验证。
- 部署后用真实账号和 iPhone 尺寸浏览器验收弹窗与最近消息。

## Documentation Sync

- 实施期间更新 `docs/current-plan.md` 与本计划。
- 完成后归档计划，补 handoff 和 `docs/archive/index.md`。
- 若根因改变长期调度/留存语义，更新 `docs/decisions.md` 或相应 runbook；仅样式修复与恢复既有契约则说明无需 ADR。

## Risks / Open Questions

- 生产消息可能包含用户私有内容，只输出时间、状态、任务 ID 等最小必要证据。
- 如果缺口来自外部模型、行情源或机器停机，需要区分可补发与不可安全重放的日期。
- 补发历史推送属于生产数据写入，必须先定位精确缺口和幂等边界，不在诊断阶段擅自执行。
