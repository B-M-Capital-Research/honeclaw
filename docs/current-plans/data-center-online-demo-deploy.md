# 3D 数据中心在线试玩入口上线

- title: 3D 数据中心在线试玩入口上线
- status: in_progress
- created_at: 2026-09-07
- updated_at: 2026-09-07
- owner: Codex
- related_files: packages/app/src/pages/chat.tsx, packages/app/e2e/public-data-center.spec.ts
- related_docs: docs/runbooks/backend-deployment.md, docs/repo-map.md

## Goal

将聊天框桌面/手机的 3D 数据中心入口切换至 NEXUS 官方在线试玩，新标签打开并保留聊天草稿，部署生产。

## Scope

- [x] 替换入口并更新浏览器回归。
- [ ] 验证并提交代码，构建精确 revision 的 public bundle。
- [ ] 推送生产分支，验证 Pages 与 origin fallback 的入口和目标 URL。

## Validation

- [x] Web 类型检查通过；桌面/手机新标签与草稿保留回归通过。
- [ ] Web 单元测试与 public 构建。
- 已运行相关 E2E：7/9 通过，另 2 项旧行业标题断言与现有 ontology 不一致（光通信与 AI 互连、电力与供电），与此次入口无关。
- [ ] 生产静态资源 hash、安全响应头与浏览器验收。

## Documentation Sync

- [x] 更新 docs/repo-map.md 入口说明。
- [ ] 在 docs/handoffs/2026-09-07-data-center-online-demo.md 留存部署 revision、产物与回滚入口。
- [ ] 完成后计划移至 docs/archive/plans/，更新 docs/archive/index.md 并移除活跃索引。

## Risks / Open Questions

- 外部试玩由独立 GitHub Pages 项目提供。
- 仅更新前端；使用稳定 public-web symlink 更新 origin fallback，无需重启后端。
