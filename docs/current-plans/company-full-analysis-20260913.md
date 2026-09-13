# 公司完整分析与任务进度

- title: 公司完整分析（原 Dify 全跑完-美链路）
- status: in_progress
- created_at: 2026-09-13
- updated_at: 2026-09-13
- owner: Codex
- related_files: packages/app/src/pages/chat.tsx; crates/hone-web-api/src/routes/public.rs; skills/
- related_docs: docs/repo-map.md; docs/invariants.md; docs/decisions.md

## Goal

新增管理员专用的公司完整分析入口，仅输入公司名称。遵从原工作流的全跑完-美完整链路及核心 Prompt，提供 task id、真实进度和最终完整 PDF，并以 TEM 完整验收。

## Scope

- [x] 读取原节点、Prompt、输入输出及进度回调协议，记录可复核来源。
- [ ] 实现管理员入口、服务端授权、独立任务及 actor 归属。
- [ ] 接入原研究链路、进度上报/读取、完整 PDF 持久化和下载。

## Validation

- [ ] 自动化覆盖权限、task id/进度隔离、失败/恢复及 PDF 附件闭环。
- [ ] 运行相关 Rust / Web / 回归验证；对既有失败单独归因。
- [ ] TEM 真实跑完，检查进度及最终 PDF 内容和排版，保留证据。

## Documentation Sync

- [ ] 同步 docs/repo-map.md、docs/invariants.md、docs/decisions.md 与必要 runbook。
- [ ] 完成 handoff，归档本计划并更新 docs/current-plan.md 与 docs/archive/index.md。

## Risks / Open Questions

- 原工作流包含外部研究与回调服务；接入方式以实际节点协议为准。
- 核心 Prompt 保持原文；仅保留身份、协议、路径、进程、产物等确定性技术边界，不加入机械内容门禁。
- 主工作区存在用户修改；本任务在独立 worktree 执行。

## Implementation decision

采用 HONE 本地分阶段任务执行器：逐节点绑定原 Prompt 与输入，保持搜索/财务子流程、研究并行依赖和完整组装顺序。任务及阶段检查点保存在 PostgreSQL；进度由实际阶段回调驱动，按 task id 读取；PDF 持久化到 actor 所属 OSS 后才完成。采用独立管理员 API 与弹窗，避免原 Prompt 与普通聊天策略混合。重启或技术失败允许从已完成阶段继续，不采用内容 validator。
