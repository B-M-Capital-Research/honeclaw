# Hone 个人交易管理工具接入

- status: archived
- created_at: 2026-11-16
- updated_at: 2026-11-16
- owner: Codex / engineering handoff
- upstream_base: f7d87aff034507a529ccf34b84de2aff7879f5ec
- goal: 在公共聊天工作流菜单新增个人交易管理入口，复用 Hone 登录身份，以服务端账户为隔离边界保存独立账本；交付代码，不部署。
- related_files: packages/app/src/pages/chat.tsx; packages/app/public/trade-manager/; crates/hone-web-api/src/routes/trade_manager.rs; docs/runbooks/trade-manager.md
- verification: 前端构建及聚焦测试；模型回归；账户隔离、未登录拒绝、并发冲突、恢复点和请求重放测试；Rust 编译及必要 DB 测试；浏览器虚构数据验收。
- documentation: 更新 docs/repo-map.md、docs/invariants.md、docs/decisions.md；新增 runbook 和 handoff；完成后归档本计划并更新 docs/archive/index.md。
- risks: 本地公开源码快照与生产版本可能不同；需工程人员在目标分支审查补丁，并完成真实登录和生产迁移前验收。

## Todo

- [x] 检查现有工具入口、身份与持久化边界，使用独立源码副本。
- [x] 实现原生菜单入口、认证 API 和用户独立账本。
- [x] 将现有交易管理台改为异步服务端保存，保持原有核算及复盘能力。
- [x] 完成隔离、冲突、失败路径与界面验证。
- [x] 同步上下文文档，打包补丁、可覆盖源码和工程说明。
- [x] 归档计划，交付，不上线。
