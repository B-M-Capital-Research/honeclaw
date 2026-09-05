# 3D 数据中心与行业分析导航

- title: 3D 数据中心与行业分析导航
- status: done
- created_at: 2026-09-05
- updated_at: 2026-09-05
- owner: Codex
- related_files: `packages/app/src/pages/public-data-center.tsx`; `packages/app/src/pages/public-data-center.css`; `packages/app/src/components/data-center-scene.tsx`; `packages/app/src/lib/data-center-{model,geometry}.ts`; `packages/app/src/lib/industry-map-navigation.ts`; `packages/app/src/pages/{chat,public-research,public-industry-map}.tsx`; `packages/app/src/app.tsx`; `packages/app/src/lib/route-prefetch.ts`; `crates/hone-web-api/src/routes/industry_map.rs`
- related_docs: `docs/archive/plans/3d-data-center-explorer.md`; `docs/repo-map.md`; `docs/invariants.md`; `docs/decisions.md#d-2026-09-05-01-ai-infrastructure-explorer-and-authenticated-industry-reading`
- related_prs: direct main deployment; [505cf737](https://github.com/B-M-Capital-Research/honeclaw/commit/505cf737170e8a80715d41c75fc05d794ce5c7c8); no release tag

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

- 前后端已部署到 `505cf737`，session-only GET 与管理员日志隔离已生效。实际生产切换及验证边界见下方生产阶段。
- 浏览器验证使用 Chromium，包括移动视口与触屏模拟；未安装 WebKit，未声称 Safari 或 iPhone 真机已测。部署阶段补跑全 Rust workspace，结果和已知基线失败见下方；未执行外部行情准确性实测。
- 模型表示产业结构，设备比例简化，AI 软件为逻辑层。冷却链接电力，软件链接云厂与 AI 平台 / 新云，设备制造是机房外上游。完整行业编辑后如删除 canonical ID，旧链接按已有无效链接回退策略处理。
- 实现提交基于已提交的 community 基线；没有暂存并行任务未完成的代码或文档。生产后端与 community 合并为一次重启；其后续 PDF 前端修复独立发布，可能将 public fallback 推进到兼容的新 Web revision。

## Next Entry Point

线上入口为 [3D 数据中心](https://hone-claw.com/data-center)；布局/几何从 `data-center-scene.tsx` 进入，说明与行业关系从 `data-center-model.ts` 进入。后续可补实际 Safari / iPhone 验收；独立前端发布与回滚遵循 `docs/runbooks/backend-deployment.md`。

## Production Deployment — 2026-09-05 (done)

用户随后授权上线。沿用同一任务，部署前本地完成记录保留；目标为包含已提交 community 基线的新精确 revision，前后端协同切换。部署期间计划曾恢复到活跃目录，完成后已重新归档。

### Pre-deploy verification

- 基于已推送 `e0278ed1`；保留生产 `HONE_APP_COMMUNITY_EDGE_DISCOVERY=1`。当前生产模式构建入口 `index-DABemvNv.js`（SHA-256 `990dfa0bf96bcab09e8b25675005bff2fd37a4cb212e027cf89ad8e7a7476dc9`），3D 分块 `public-data-center-DopGnpJS.js`（SHA-256 `69a5e1c7fa7eada3bca56d3a7d7066db423c8987903a9617c8e1b276c7405aa5`）。
- Web 类型检查、564 项单测、Worker 类型检查/45 项测试、public build 全通过。Rust workspace all-targets check 通过；test --no-fail-fast 共 2824 passed / 113 ignored / 3 failed，失败为已有 `hone-agent` 两个 streaming fixture 和 `hone-core` soul prompt contract，与 community 基线逐项一致，非本次新增；行业相关路径单独验证。CI-safe 中 finance_automation_contracts 的九项历史断言同样与基线一致。未绕过或删除这些检查。

- 生产构建上的 3D + community Playwright 共 10/10 通过（23.8s，mock API），保留当前社区行为；行业专项 core 10/API 11 全通过。CI-safe 共覆盖 22 脚本，21 通过，唯一失败为上述既有 finance contracts，主入口截断后的 14 脚本已补跑。部署继续以相关回归全部通过、旧失败明确留证为依据，没有将全仓结果描述为全绿。

### Exact artifacts and Pages acceptance

- 实现提交 `505cf737170e8a80715d41c75fc05d794ce5c7c8` 已推送 main；未打 release tag。干净 worktree 在该 revision 以 `HONE_APP_COMMUNITY_EDGE_DISCOVERY=1` frozen install/public build，关键入口与 3D JS/CSS 和已通过 E2E 的产物字节完全一致。
- Cloudflare Pages deployment `bcf78b4c-1721-4171-9ae6-b11a8c58a425` 成功。`https://hone-claw.com/data-center` 返回 200，线上 entry / 3D chunk SHA 与上述精确构建一致；HSTS、CSP frame-ancestors、DENY、nosniff 和 referrer-policy 正常。匿名行业 API 与 auth/me 返回 JSON 401。
- [Runtime Image run 33939864158](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/33939864158) 成功，部署绑定 `ghcr.io/b-m-capital-research/honeclaw-runtime@sha256:7fc791b9a3fafb14bc753b3e861d51ac9f7639a16d3f2eaab480320daa9d0c57`。
- Public fallback 使用独立目录，完整归档 469 文件。归档 SHA-256 `8a34301e0c47605a3b00e6832dcf5b4a611f54a9f1a83574d9b83a6c85b6876c`；逐文件 size/SHA manifest SHA-256 `fb8a6c122301ea3437d6700946170db49b21cedca35f5711ada04cf0d5d9ae7e`。不向严格校验的 runtime bundle 添加静态文件。
- 真实已登录管理员浏览器：从聊天快捷入口进入模型、打开光互联浮窗并进入光通信行业；390px 线上手机视口打开 AI 软件浮窗，确认 hyperscaler/neocloud 两个跳转，页面无横向溢出；随后恢复桌面视口。未创建生产测试账号或修改行业数据。普通/未付费用户覆盖来自真实 PostgreSQL handler 回归，不等同于线上普通账号实测。

### Joint runtime cutover and acceptance — 10:58 CST

- 与并行 community owner 协调后合并为一次重启。实际生产前序为 `a2d76ea44d04ef307740e7d599d360f65dd3b6bc`；`e0278ed1` 只预备了 binary/Web，没有作为中间版本重启。community 完成资源 publisher 与 managed secret 配置后明确交接，由本任务唯一切换。
- 切换前校验 505、a2d、e0278 三份 runtime bundle；505 Web 469 文件、e0278 Web 465 文件均完整通过。受控环境文件权限、既有 working directory `/var/lib/hone` 检查通过；剩余磁盘约 3.10 GB。两次活跃对话检查间隔 3 秒，均为 0。
- `/opt/hone/current` 指向 `/opt/hone/releases/505cf737170e8a80715d41c75fc05d794ce5c7c8-ghcr-runtime`；`/opt/hone/public-web/current` 指向 `/opt/hone/public-web/releases/505cf737170e8a80715d41c75fc05d794ce5c7c8/dist-public`，随后仅重启既有 `hone-web.service`。本任务没有修改 managed env、skills 或业务数据，没有启用任何 channel worker。
- `/api/meta` 实际 build SHA 为 505 完整 revision，`cloud_mode=cloud`、cloud storage authoritative、PostgreSQL/OSS health 均正常、local durable dependency count=0。活跃对话仍为 0，channel active set 仍为空。匿名 community edge-session 返回 401。
- origin loopback `8088/chat` 返回新 HTML，入口与 3D chunk 的实际响应 size/SHA 和 Pages/干净构建一致。切换后真实管理员浏览器再次从芯片浮窗进入 AI 芯片详情，确认详情读取与「编辑本体」开关存在；没有触发编辑保存。
- 保留回滚组合：实际旧 binary `a2d76ea44d04ef307740e7d599d360f65dd3b6bc-ghcr-runtime` + 已校验的 e0278 public `dist-public`；该 Web 的 `BUILD_MANIFEST.json` SHA-256 为 `f2d7e717fd1f9366d78ddb0483806f6031106498e99e75cd2c968f4155bf025c`。e0278 binary 另作备用。回滚需保留现有 env/secret、在 idle 时分别恢复两条 symlink 后重启；若 API 全部不可达可走故障恢复，不应把无法确认活跃会话误报为零。
- 本次临时 joint helper 的 SHA-256 为 `5bf6d1b382cfea5a67906c6052bdde38f5ef3fba33a709edc9394c7d675f9d27`，Python 3.9 编译、25 项文件校验/异常恢复及 11 项联合配对流程隔离检查通过。证据目录 `/tmp/hone-3d-deploy-validation-20260905/` 包含 `joint-cutover.log`、`runtime-stage.log`、`web-stage.log`、`live-public.json` 与 `deployment-artifacts.json`；关键结果已在本 handoff 留存，后续按通用 runbook 操作。
- 全仓已知失败为 hone-agent 的 `deferred_prefix_ignores_structurally_invalid_datafetch_activation`、`first_batch_identity_route_limit_executes_only_six_valid_routes` 和 hone-core 的 `soul_prompt_keeps_the_full_investment_contract`，另有 CI-safe finance 的九项旧断言。未将本轮验证描述为全仓全绿。

- 最终复检确认 `hone-web` active、`NRestarts=0`，实际进程读取 `HONE_PUBLIC_WEB_DIST_DIR=/opt/hone/public-web/current`，匿名行业接口 401。community owner 完成联合 canary：最新 feed 与源/PG 对齐，图片 HEAD/GET/304、PDF GET/304 和完整 SHA 均通过；关联报告为 `data/community-imports/2026-09-05/production-edge-canary-505-complete-report.jsonl`，其完整恢复范围与 PDF 预览后续工作见 `docs/handoffs/2026-09-05-community-freshness-assets-latency.md`。
