# 3D 数据中心与行业分析导航

- title: 3D 数据中心与行业分析导航
- status: in_progress
- created_at: 2026-09-05
- updated_at: 2026-09-05
- owner: Codex
- related_files: `packages/app/src/pages/public-data-center.tsx`; `packages/app/src/pages/public-data-center.css`; `packages/app/src/components/data-center-scene.tsx`; `packages/app/src/lib/data-center-{model,geometry}.ts`; `packages/app/src/lib/industry-map-navigation.ts`; `packages/app/src/pages/{chat,public-research,public-industry-map}.tsx`; `packages/app/src/app.tsx`; `packages/app/src/lib/route-prefetch.ts`; `crates/hone-web-api/src/routes/industry_map.rs`
- related_docs: `docs/current-plans/3d-data-center-explorer.md`; `docs/repo-map.md`; `docs/invariants.md`; `docs/decisions.md#d-2026-09-05-01-ai-infrastructure-explorer-and-authenticated-industry-reading`
- related_prs: none; local implementation only, no commit/push/tag/deployment

## Summary

聊天页「持仓分析」快捷导航已替换为「3D 数据中心」，新增 `/data-center` 页面；研究台也提供常驻的 3D / 完整行业入口，不将这两个静态入口计入每日更新产品。用户确认所有登录用户都可查看完整行业，仍仅管理员可编辑。

## What Changed

- 六个空间/逻辑区：芯、存、光、电、冷、AI 软件与云平台。浮窗解释位置、组件和研究关注点，并关联现有八个行业；不引入第二套行业本体或实时行情。
- 使用本地三维坐标、正交投影和 SVG 多边形；支持拖动、旋转按钮、80–135% 缩放及复位。转向限制为 18–66°，设备面按深度绘制，地板单独先绘制，避免遮住后方供电柜。
- 稳定 SVG 节点随投影更新，不在每个拖动帧重建所有多边形；无闲置渲染循环、WebGL 引擎、外部模型资源或新增生产依赖。此次构建页面 JS 17.40 kB / gzip 7.79 kB，CSS 10.63 kB / gzip 2.88 kB（不含共享壳层）。
- 容器宽度不足时采用 44px 短标签，避免侧栏挤压后的窄 PC / 平板标签重叠。手机为底部浮窗，桌面为侧边浮窗；原生 dialog 提供焦点圈定、Escape 与关闭后焦点恢复，支持深色与减少动画偏好。
- `/industry-map?industry=<id>` 由 URL 驱动选中，支持刷新、前进后退、动态新增/删除行业和非法参数回退。
- GET 使用 `require_public_session_user`，未付费但已登录用户也能阅读。POST 保留管理员校验；普通用户不返回 `recent_edits` 内的管理员身份/内部备注，前端也仅在服务端 `is_admin=true` 时显示编辑及日志。

## Verification

- `PATH=/Users/fengming2/.bun/bin:$PATH bun run typecheck:web`：通过。
- `PATH=/Users/fengming2/.bun/bin:$PATH bun run test:web`：当前工作树 564 项通过；其中本任务新增行业映射/链接、相机及地板遮挡相关单测 13 项。工作树还有独立 community 任务的变更，整体数量包含它的测试。
- `PATH=/Users/fengming2/.bun/bin:$PATH bun run build:web:public`：通过；既有共享大分块警告不影响构建。
- 在 `packages/app` 执行 `PATH=/Users/fengming2/.bun/bin:$PATH bun x playwright test public-data-center.spec.ts --project=public --workers=1`：8 项全部通过，最终运行 27.1s。覆盖 320 / 390 / 900 / 1440px、触摸点击模拟、热点与卡片、浮窗全部行业链接、键盘/焦点、拖动/复位、最小最大相机、深色/减弱动画、行业 URL 历史、读写权限和登录失效。
- PostgreSQL 环境变量按仓库约定指向 `127.0.0.1:5433` 后，`cargo test -p hone-web-api --lib industry_map::tests`：11 项通过。真实 handler 回归证明匿名读写 401、未付费登录用户可读、普通用户写入 403 且无日志副作用、管理员可写，以及普通响应不含管理员 ID/独有内部备注。
- Rust 改动文件 `rustfmt --edition 2024 --config skip_children=true --check` 与 `git diff --check` 通过。
- 人工查看最终 PC、手机、窄屏极限相机和深色截图，供电柜不再被地板遮挡，标签可见，复位后几何与颜色恢复。截图由上述 E2E 生成于 `packages/app/test-results/public-data-center-*/`，包含 `data-center-1440.png`、`data-center-390.png`、`data-center-320-camera-limit.png`、`data-center-tablet-900.png`、`data-center-dark.png` 与浮窗截图；测试代码保留生成步骤。
- 验证所用临时 PostgreSQL 已关闭、专属 `/tmp/hone-industry-map-pg.b5x3yK` 已清理；没有变更业务数据。当前会话预览服务为 `http://127.0.0.1:4319/data-center`。

## Risks / Follow-ups

- 尚未提交、部署或发布。上线需要同时交付前端权限/UI 和后端 session-only GET / 管理员日志隔离；旧后端仍可能对未付费读者拒绝。
- 浏览器验证使用 Chromium，包括移动视口与触屏模拟；未安装 WebKit，未声称 Safari 或 iPhone 真机已测。未运行全 Rust workspace / 外部行情实测；本次是前端交互与行业读取权限改动，完成了对应后端路由回归。
- 模型表示产业结构，设备比例简化，AI 软件为逻辑层。冷却链接电力，软件链接云厂与 AI 平台 / 新云，设备制造是机房外上游。完整行业编辑后如删除 canonical ID，旧链接按已有无效链接回退策略处理。
- 本次未覆盖并行 community/cloud 任务的代码或文档修改，也未将它们提交。

## Next Entry Point

本地体验 `/data-center`；布局/几何从 `data-center-scene.tsx` 进入，说明与行业关系从 `data-center-model.ts` 进入。发布前按仓库发布/部署流程验证精确 revision，并补实际 Safari / iPhone 验收。

## Production Deployment — 2026-09-05 (in_progress)

用户随后授权上线。沿用同一任务，部署前本地完成记录保留；目标为包含已提交 community 基线的新精确 revision，前后端协同切换。计划暂恢复到 `docs/current-plans/3d-data-center-explorer.md`，完成后归档。

### Pre-deploy verification

- 基于已推送 `e0278ed1`；保留生产 `HONE_APP_COMMUNITY_EDGE_DISCOVERY=1`。当前生产模式构建入口 `index-DABemvNv.js`（SHA-256 `990dfa0bf96bcab09e8b25675005bff2fd37a4cb212e027cf89ad8e7a7476dc9`），3D 分块 `public-data-center-DopGnpJS.js`（SHA-256 `69a5e1c7fa7eada3bca56d3a7d7066db423c8987903a9617c8e1b276c7405aa5`）。
- Web 类型检查、564 项单测、Worker 类型检查/45 项测试、public build 全通过。Rust workspace all-targets check 通过；test --no-fail-fast 共 2824 passed / 113 ignored / 3 failed，失败为已有 `hone-agent` 两个 streaming fixture 和 `hone-core` soul prompt contract，与 community 基线逐项一致，非本次新增；行业相关路径单独验证。CI-safe 中 finance_automation_contracts 的九项历史断言同样与基线一致。未绕过或删除这些检查。

- 生产构建上的 3D + community Playwright 共 10/10 通过（23.8s，mock API），保留当前社区行为；行业专项 core 10/API 11 全通过。CI-safe 共覆盖 22 脚本，21 通过，唯一失败为上述既有 finance contracts，主入口截断后的 14 脚本已补跑。部署继续以相关回归全部通过、旧失败明确留证为依据，没有将全仓结果描述为全绿。
