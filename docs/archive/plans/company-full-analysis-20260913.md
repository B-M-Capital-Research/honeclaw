# 公司完整分析与任务进度

- title: 公司完整分析（原 Dify 全跑完-美链路）
- status: archived
- created_at: 2026-09-13
- updated_at: 2026-09-14
- owner: Codex
- related_files: packages/app/src/pages/chat.tsx; crates/hone-web-api/src/routes/company_analysis/; skills/earnings-research/scripts/
- related_docs: docs/repo-map.md; docs/invariants.md; docs/decisions.md

## Goal

新增管理员专用的公司完整分析入口，仅输入公司名称。遵从原工作流的全跑完-美完整链路及核心 Prompt，提供 task id、真实进度和最终完整 PDF，并以 TEM 完整验收。

## Scope

- [x] 读取原节点、Prompt、输入输出及进度回调协议，记录可复核来源。
- [x] 实现管理员入口、服务端授权、独立任务及 actor 归属。
- [x] 接入原研究链路、进度上报/读取、完整 PDF 持久化和下载。

## Validation

- [x] 自动化覆盖权限、task id/进度隔离、失败/恢复及 PDF 附件闭环。
- [x] 运行相关 Rust / Web / 回归验证；对既有失败单独归因。
- [x] TEM 真实跑完，检查进度及最终 PDF 内容和排版，保留证据。

## Documentation Sync

- [x] 同步 docs/repo-map.md、docs/invariants.md、docs/decisions.md 与必要 runbook。
- [x] 完成 handoff，归档本计划并更新 docs/current-plan.md 与 docs/archive/index.md。

## Risks / Open Questions

- 原工作流包含外部研究与回调服务；接入方式以实际节点协议为准。
- 核心 Prompt 保持原文；仅保留身份、协议、路径、进程、产物等确定性技术边界，不加入机械内容门禁。
- 主工作区存在用户修改；本任务在独立 worktree 执行。

## Implementation decision

采用 HONE 本地分阶段任务执行器：逐节点绑定原 Prompt 与输入，保持搜索/财务子流程、研究并行依赖和完整组装顺序。任务及阶段检查点保存在 PostgreSQL；进度由实际阶段回调驱动，按 task id 读取；PDF 持久化到 actor 所属 OSS 后才完成。采用独立管理员 API 与弹窗，避免原 Prompt 与普通聊天策略混合。重启或技术失败允许从已完成阶段继续，不采用内容 validator。

## Validation follow-through

- TEM 首轮和最终轮均完成，最终轮 202.5 秒、20 页、9 个模型节点，原 Prompt 不变。
- 发现修改 `routes/mod.rs` 会让 changed-file rustfmt 脚本递归检查未修改子模块；同步修复脚本遵守其“仅改动文件”契约，补独立临时仓库回归证明。
- 完成前增加 Linux PDF 验证、不可变产物部署及线上入口检查。

## Production follow-through

- Runtime/frontend/renderer revision `9a2f27c7` 已部署，健康检查确认 PostgreSQL/R2 正常；Cloudflare Pages 管理员入口已实际打开。
- 线上浏览器已启动 TEM task `f9ba885f-ec35-40cc-87d5-ffdfc5ac2460`，验证关闭重开仍读取同一 task id；已到 100% 并从页面下载完整 20 页 PDF，逐页排版检查通过。
- 隔离 canary 本地服务、搜索隧道及临时 PostgreSQL 已停止，临时凭据已删除。

## Completion

所有实现、验证、文档同步与线上验收完成。最终交接：`docs/handoffs/2026-09-14-company-full-analysis.md`；已知全仓历史测试失败单独留档。
