# Plan

- title: 3D 数据中心与行业分析导航
- status: archived
- created_at: 2026-09-05
- updated_at: 2026-09-05
- owner: Codex
- related_files: packages/app/src/pages/chat.tsx; packages/app/src/pages/public-data-center.tsx; packages/app/src/components/data-center-scene.tsx; packages/app/src/lib/data-center-model.ts; packages/app/src/pages/public-industry-map.tsx; packages/app/src/app.tsx
- related_docs: docs/repo-map.md; docs/handoffs/2026-09-05-3d-data-center-explorer.md

## Goal

将聊天页「持仓分析」快捷入口换成「3D 数据中心」，用可转动、可缩放的机房模型呈现芯、存、光、电、冷和 AI 软件层；点击标签查看浮窗并直达对应行业分析。

## Scope

- [x] 复用行业树 canonical ID 与研究范围，建立说明和行业关联。
- [x] 实现轻量 3D 几何场景、移动/桌面布局、触控与键盘操作。
- [x] 接入快捷导航、新页面路由、行业 URL 选择和历史导航。
- [x] 明确行业读权限范围；保留管理员写权限。

## Validation

- [x] 内容关联与投影/相机边界单元测试。
- [x] Web 类型检查、单元测试与 public 构建。
- [x] Playwright 实测手机/PC：模型渲染、浮窗关闭/焦点、缩放/旋转、快捷入口、行业跳转和回退。
- [x] 检查窄屏无横向溢出、深色模式和减少动画偏好。

## Documentation Sync

- [x] 更新 docs/repo-map.md 入口与映射关系。
- [x] 写 docs/handoffs/2026-09-05-3d-data-center-explorer.md 留存验证和限制。
- [x] 完成时从 docs/current-plan.md 移除本任务、将计划移到 docs/archive/plans/ 并更新 docs/archive/index.md。

## Risks / Open Questions

- 用户已明确同意所有登录用户阅读、仅管理员编辑；GET 与 UI 已同步，并隔离管理员内部改动日志。
- 工作树已有 community 任务的修改（api.ts、对应计划与测试）；本任务不覆盖这些修改。
- 3D 场景为产业结构示意，不表示真实机房比例或实时经营数据；软件层关联现有 hyperscaler/neocloud，避免虚构独立行业。

## Completion

2026-09-05 本地实现、类型检查、564 项 Web 单测、11 项后端行业回归、8 项响应式浏览器验证与 public 构建完成；完整证据与未测边界见 handoff。未提交或部署。

## Production Deployment — 2026-09-05

用户已授权本轮直接上线。沿用原任务计划，不新开重复主题；此前本地完成记录保留。

- [x] 隔离本任务变更并核对当前生产基线，与并行 community 上线协调切换顺序。
- [x] 完成仓库 push 门禁：Rust check/test、格式、Web/Worker 单测、CI-safe 回归，记录已有基线失败。
- [x] 提交本任务、推送 main，等精确 revision 的 Pages 与 GHCR 构建（505cf737；Pages 与镜像均成功）。
- [x] 验证 runtime/env/磁盘/两次零活跃会话，保留回滚版本后原子切换；与 community 合并为一次 a2d76ea4 → 505cf737 重启。
- [x] 验证线上静态资源、3D 入口/浮窗/链接和服务健康；真实管理员线上读取，普通/未付费读权限与写隔离由 PostgreSQL handler 回归证明。
- [x] 在 docs/handoffs/2026-09-05-3d-data-center-explorer.md 追加生产证据，更新 docs/archive/index.md 并重新归档计划。

2026-09-05 10:58 CST 完成生产部署：Pages 与 backend 505cf737；实际服务和 origin fallback 验收通过，旧 binary/Web 回滚组合保留。后续 community PDF 前端独立发布见对应 handoff。
