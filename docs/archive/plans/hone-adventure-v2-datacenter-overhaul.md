# HONE 历险记 V2：浅色 AI 数据中心氛围、算力产线关卡与弹道/晋升体系全面重构

- title: HONE 历险记 V2：浅色 AI 数据中心氛围、算力产线关卡与弹道/晋升体系全面重构
- status: archived
- created_at: 2026-11-21
- updated_at: 2026-11-21
- owner: Antigravity
- related_files: packages/app/public/hone-adventure/index.html, packages/app/public/hone-adventure/styles.css, packages/app/public/hone-adventure/data.js, packages/app/public/hone-adventure/world.js, packages/app/public/hone-adventure/art.js, packages/app/public/hone-adventure/fx.js, packages/app/public/hone-adventure/poster.js, packages/app/public/hone-adventure/app.js, packages/app/public/hone-adventure/sprites.webp, packages/app/public/hone-adventure/floor.webp, packages/app/public/hone-adventure/atlas-coords.js, packages/app/src/lib/hone-adventure.test.ts, packages/app/e2e/public-data-center.spec.ts
- related_docs: docs/repo-map.md, docs/handoffs/2026-11-21-hone-adventure-game.md

## Goal

彻底去除《HONE 历险记》V1 残留的深海暗色氛围与旧弹道痕迹，将整体视觉与美术全面升级为**浅色洁净室 AI 数据中心（Cleanroom AI Data Center）**工程风格；新增可自由切换与递进跃迁的 **6 大算力产线关卡（存储阵列 / 光模块走廊 / CPU 调度中枢 / GPU 张量矩阵 / 先进封装工坊 / 超算液冷总机房）**；全面重做 **8 套硬件工程弹道与超武、9 类硬件异常怪与 5 大产线巨兽 Boss、10 阶带永久增益的算力架构师晋升路线与 4 维 12 节点算力基建升级矩阵**；左上角改为返回 HONE 主站首页（`/`）的「回到首页」按钮，并在结算页提供醒目的 `honeclaw` GitHub 仓库地址与一键跳转点 Star 入口，验证通过后重新发布上线。

## Scope

- [x] 重新生成并打包浅色背景高对比 AI 数据中心洁净室地板纹理（`floor.webp`）及全套深色勾边硬件异常怪（9 种）、5 大产线 Boss（存储/光模块/CPU/GPU/EUV 封装）与机柜/光模块/晶圆盒场景道具贴图（`sprites.webp`），彻底清除气泡/焦散等深海视觉残留，保证浅色背景下零遮挡与高可读性。
- [x] 实现 6 大算力产线关卡（`storage` 存储、`optics` 光模块、`cpu` CPU、`gpu` GPU、`packaging` 先进封装、`hyperscale` 超算总机房），支持大厅一键切换任意起手产线关卡及局内定时跃迁轮换，每关具备专属浅色洁净室母线地标、产线特性、敌群配比与镇守 Boss。
- [x] 全面重构 8 套硬件武器弹道与 8 项超维进化（多通道 HBM 数据包齐射、WDM 三波长分光棱镜束、3×3 方形张量脉动矩阵印章、八角 L3 缓存硅片环、浸没液冷六角冰晶场、EUV 横向全屏光刻扫描梁、自主飞行 ASIC 推理无人机蜂群、45°/90° PCB 布线高压母线电弧），确保视觉与手感跟原游戏完全脱胎换骨。
- [x] 全面升级 5 大产线 Boss 专属多阶段技能（坏块扇区墙、色散多棱光束与高速冲刺、乱序十字流水线锁死、显存过热矩阵格、EUV 光刻十字锁定）及浅色高对比预警几何框。
- [x] 升级「10 阶算力架构师晋升路线」（每阶附带永久属性特权与晋升晶核奖励，支持路线图预览）与「四维 12 节点算力基建矩阵」（存算互联 / 光刻制程 / 液冷供电 / 智能调度）。
- [x] 重构浅色工程控制台 UI 布局：左上角固定「回到首页」（跳转 `/`），结算页与战报展示醒目 `https://github.com/B-M-Capital-Research/honeclaw` 地址与一键点 Star 卡片。
- [x] 更新单测与 Playwright E2E，完成 `typecheck:web`、`test:web`、`build:web:public`、无头浏览器多屏截图巡检，推送 `main` 并完成线上 `https://hone-claw.com/hone-adventure/` 验收。

## Validation

- [x] `bun run typecheck:web`
- [x] `bun run test:web`（覆盖 6 大关卡切换、新 8 武器弹道/超武、5 大 Boss、10 阶晋升路线与 4 维 12 节点基建矩阵）
- [x] `bun run build:web:public`
- [x] Playwright 桌面与移动端 E2E（验证 `/chat` 打开游戏、左上角「回到首页」跳转 `/`、关卡切换、局内高对比浅色弹道渲染、结算页 `honeclaw` GitHub Star 链接可达且无 `pageerror`）
- [x] 线上 `https://hone-claw.com/hone-adventure/` 生产部署与无头浏览器实机冒烟验证

## Documentation Sync

- [x] 更新 `docs/repo-map.md` 中 `/hone-adventure/` V2 浅色 AI 数据中心关卡与架构说明
- [x] 在既有 `docs/handoffs/2026-11-21-hone-adventure-game.md` 追加 V2 阶段升级节点与验证结论（遵循同一天同主题追加规则）
- [x] 完成后将本计划归档至 `docs/archive/plans/hone-adventure-v2-datacenter-overhaul.md`，清理 `docs/current-plan.md` 并更新 `docs/archive/index.md`

## Risks / Open Questions

- 浅色背景下若沿用白色加法混色（additive glow）或浅白字会导致弹道发灰看不清；已将弹道、预警圈、伤害跳字与拾取物全部改为高饱和实心色核 + 深石板色描边（`#0F172A`），张量矩阵采用 4.5% 线框对准框绘制于敌群下层，地板纹理对比度控制在低干扰区间且无大块遮挡物。
- 工作区仍有其它任务的未提交改动；继续使用独立临时 `git worktree` 隔离提交与推送，按 CD 契约不自动打 `v*` tag。
