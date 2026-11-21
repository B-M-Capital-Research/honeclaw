# HONE 历险记肉鸽割草游戏开发与上线

- title: HONE 历险记肉鸽割草游戏开发与上线
- status: done
- created_at: 2026-11-21
- updated_at: 2026-11-21
- owner: Antigravity
- related_files: packages/app/public/hone-adventure/index.html, packages/app/public/hone-adventure/app.js, packages/app/public/hone-adventure/world.js, packages/app/public/hone-adventure/data.js, packages/app/public/hone-adventure/art.js, packages/app/public/hone-adventure/audio.js, packages/app/public/hone-adventure/fx.js, packages/app/public/hone-adventure/poster.js, packages/app/public/hone-adventure/styles.css, packages/app/public/hone-adventure/sprites.webp, packages/app/public/hone-adventure/floor.webp, packages/app/public/hone-adventure/atlas-coords.js, packages/app/src/pages/chat.tsx, packages/app/src/lib/public-content.ts, packages/app/src/lib/hone-adventure.test.ts, packages/app/public/_worker.js, packages/app/public/_worker.test.js, packages/app/e2e/public-data-center.spec.ts, packages/app/vite.config.ts
- related_docs: docs/repo-map.md, docs/archive/plans/hone-adventure-game.md, docs/archive/plans/hone-adventure-v2-datacenter-overhaul.md, docs/runbooks/backend-deployment.md
- related_prs: direct push to `main` (no `v*` release tag)

## Summary

参考单页肉鸽生存架构全新设计并交付了 HONE 原生世界观肉鸽割草游戏《HONE 历险记》（`/hone-adventure/`），将入口挂载到 `/chat` 输入框「3D 数据中心」旁（桌面端 chip 与移动端「添加与工具」菜单同步提供，新标签页 `noopener,noreferrer` 打开且保留对话与草稿），并推送至 `main` 完成 Cloudflare Pages 线上发布。

## What Changed

- **独立游戏本体（`packages/app/public/hone-adventure/`）**：
  - 全新黑曜墨玉 + 熔金/翡翠赛博投研视觉体系，重绘打包 34 张高清角色/怪物/Boss/道具透明精灵图集（`sprites.webp`）与无缝电路地板纹理（`floor.webp`），内置 WebGL 4096-quad 批处理渲染器与 Canvas 2D 自动回退、自适应帧率画质调节。
  - 重构全套角色与敌群 IP：4 名可操控 HONE 特工（`小砺` 含免费可切换限定皮 `曜金小砺`、`芯核`、`光瞳`、`量闪`）、9 类市场噪音怪与 4 大多阶段深渊巨兽 Boss（`伪研报之王`、`高频收割机`、`幻觉魔眼主脑`、`流动性黑天鹅`）。
  - 重构局内与局外升级路径：8 把投研/算力武器 × 9 项被动因子 × 8 项满级配对超维进化，支持升级三选一锁卡（`toggleLockChoice`）与因子重抽；局外采用「火力因子 / 风控装甲 / 投研效率」三分支 9 项算力天赋矩阵（每分支满 6 点激活里程碑共鸣）。
  - 保留双模式与留存分享闭环：常规无尽历险 + 7 词条每日全网同种子行情挑战（FNV-1a 日期哈希 + Mulberry32 确定性 PRNG）、100% WebAudio 实时合成音效与 4 阶动态电子配乐、8 档研究员段位、20 枚荣誉勋章、本机 Top 10 荣耀殿堂、`1080×1440`（3:4）高清战报海报下载/剪贴板复制，以及兼容浏览器新标签与原生 WebView 的「返回 HONE」导航。
- **`/chat` 入口挂载与双语契约**：
  - `packages/app/src/pages/chat.tsx`：在 `Composer` 的 `actions` 列表中紧随 `data-center` 新增 `hone-adventure` 动作与专属图标，以 `window.open("/hone-adventure/", "_blank", "noopener,noreferrer")` 打开。
  - `packages/app/src/lib/public-content.ts`：补齐 `zh` / `en` 的 `hone_adventure_tip` 与 `hone_adventure_hint`，满足 `public-language-contract.test.ts` 零硬编码中文与键值对齐要求。
- **边缘路由保护与本地开发目录映射**：
  - `packages/app/public/_worker.js`：为 `/hone-adventure/*` 增加静态资源防 SPA 回退守卫（缺失子资源返回 `404`，入口未安装 `<meta name="hone-game" content="hone-adventure">` 返回 `503`），并保持 `X-Frame-Options: DENY`、`Content-Security-Policy: frame-ancestors 'none'` 与 `Strict-Transport-Security: max-age=31536000`。
  - `packages/app/vite.config.ts`：增加开发服务器 `/hone-adventure/` 目录到 `/hone-adventure/index.html` 的静态重写，保证 `vite dev` 与线上 Pages 路径行为一致。

## Verification

- `bun run typecheck:web`：通过。
- `bun run test:web`：703 passed, 0 failed（含 `hone-adventure.test.ts`、`public-language-contract.test.ts`、`_worker.test.js`）。
- `bun run build:web:public`：通过，`dist-public/hone-adventure/` 完整产出全部静态模块与 WebP 纹理资源。
- `HONE_E2E_ADMIN_PORT=4283 HONE_E2E_PUBLIC_PORT=4284 bun run test:e2e -- --project=public e2e/public-data-center.spec.ts -g "chat opens the online demo"`：桌面 `1440px` 与移动 `390px` 两项全链路回归均通过，验证「HONE 历险记」紧邻「3D 数据中心」、新标签 `window.opener === null`、`__honeAdventure.state().sheetLoaded === true`、无 `pageerror` 且保留 `/chat` 输入草稿。
- 独立无头 Chromium 交互与截图巡检：覆盖主页 AI 演示、曜金皮肤切换、特工档案、三分支算力天赋矩阵、每日挑战、Boss 战 HUD、升级锁卡与重抽、结算战报海报、投研图鉴及 `390×844` 移动端安全边距，零控制台报错。
- 干净隔离 worktree 构建、`.githooks/pre-push`（`gitleaks`）扫描、推送 `origin/main` 及 Cloudflare Pages `https://hone-claw.com/`、`/chat`、`/hone-adventure/` HTTP 状态与安全响应头核验通过。本次仅涉及纯前端与公开静态资产，无后端 Rust 代码或数据库变更。

## Risks / Follow-ups

- 游戏存档（`localStorage` 键 `hone_adventure_save_v1`）为纯浏览器本地持久化，不读写后端账号与业务数据库；清除站点数据会重置本机局外进度。
- 回滚方式：如需下线入口或回滚静态包，可在 `main` revert 本次前端提交触发 Cloudflare Pages 重新发布，或在 Cloudflare Pages 控制台将上一成功 deployment 提升为生产版本；无需重启后端服务。

## Next Entry Point

任务已上线并归档。后续若扩展新特工、超武或全服排行榜接口，入口在 `packages/app/public/hone-adventure/data.js`、`world.js` 与 `app.js`，并同步更新 `packages/app/src/lib/hone-adventure.test.ts` 与 `packages/app/e2e/public-data-center.spec.ts`。

## V2 阶段：浅色洁净室 AI 数据中心、六大算力产线关卡与弹道/晋升体系全面重构

### V2 Summary

根据用户体验反馈对《HONE 历险记》进行了 V2 彻底重构并重新发布上线：全面移除 V1 残留的深海暗色光影（焦散、气泡、阳光束、暗色隧道晕影、水波扭曲与海洋形态怪物），切换为浅色高对比度 **Cleanroom AI Data Center（洁净室 AI 数据中心）** 工程视觉（`#EFF4FA` 架空机房地板 + 低干扰母线网格，零大块遮挡、不损伤走位可读性）；新增支持大厅自由切换与局内 75 秒递进跃迁的 **6 大算力产线关卡**（`存储阵列`、`光模块`、`CPU 中枢`、`GPU 矩阵`、`先进封装`、`超算总机房`）；全面重做 **8 套硬件弹道与超武**、**9 类硬件故障怪与 5 大产线巨兽 Boss**、**10 阶带永久被动特权的算力架构师晋升路线**与 **4 维 12 节点算力基建矩阵**；左上角统一改为跳转 `/` 的「回到首页」按钮，并在结算页与战报图内置醒目的 `https://github.com/B-M-Capital-Research/honeclaw` GitHub Star 卡片。

### V2 What Changed

- **浅色洁净室 AI 数据中心视觉与美术重绘（`sprites.webp` / `floor.webp` / `art.js` / `fx.js`）**：
  - 重新绘制并打包 `floor.webp`（浅银白架空防静电地板与低对比蓝灰母线纹）及 32 张深石板色（`#0F172A`）硬朗勾边的硬件异常怪（RJ45 丢包蛛、时钟抖动蜂、热失控铜管魔像、光纤串扰炮、浪涌变压器牛、PCB 内存泄漏魔方、爆浆短路电容、几何幻觉幽灵、坏扇区螨虫）、5 大产线 Boss（`闪存冷阵列·坏块巨像`、`硅光耦合主脑·色散棱镜`、`乱序超标量·死锁仲裁者`、`显存墙霸主·张量熔炉`、`极紫外晶圆母机·EUV 巨神`）与机柜/晶圆盒/光交换机/液冷舱道具。
  - WebGL 与 Canvas 2D 双渲染管线统一移除浅底易泛白的 additive `lighter` 混色，改为饱和实心色核 + `#0F172A` 描边弹道与跳字，张量矩阵采用超低透明度（4.5%）线框对准框并绘制于敌群下层，确保密集战斗零遮挡。
- **6 大可切换算力产线关卡与 8 套全新硬件弹道（`data.js` / `world.js`）**：
  - 大厅底部控制台可直接切换 `storage` / `optics` / `cpu` / `gpu` / `packaging` / `hyperscale` 任一起手产线（各自携带产线全局 Buff、地板色温、敌群偏置与镇守 Boss），局内每 75 秒自动跃迁至下一产线。
  - 8 套硬件弹道与超武：`HBM 带宽飞梭 → 3D 堆叠数据洪流`（多通道平行矩形总线帧）、`硅光分光束 → 1.6T 波分相干阵列`（命中折射三/五波长子光束）、`张量矩阵炮 → Rubin 脉动方阵`（3×3 方形线框脉动矩阵与四角子方阵）、`L3 缓存环刃 → CoWoS 晶圆护盾`（总线串联八角硅片环）、`PCB 母线电弧 → 吉瓦级超导馈线`（45°/90° 电路板折线路由与过孔焊盘）、`EUV 光刻扫描 → High-NA 全场曝光`（洁净室横向扫描梁）、`ASIC 推理蜂群 → 主权算力集群`（三角翼无人机曲率追击）、`浸没液冷喷射 → 超流态绝对零度`（双层六角冰晶冷却板）。
- **10 阶晋升路线、四维基建矩阵、回到首页与 GitHub Star 闭环（`app.js` / `styles.css` / `poster.js`）**：
  - 升级为 `L1 · 见习机房巡检员` 至 `L10 · HONE 首席算力官` 十阶晋升路线（每阶解锁永久全局属性加成与首达晶核奖励，配独立晋升路线弹窗）及「存算互联 / 光刻制程 / 液冷供电 / 智能调度」四维 12 节点基建矩阵（存档升级至 `hone_adventure_save_v2` 并保留旧晶核兼容迁移）。
  - 大厅、战斗 HUD、暂停弹窗与结算页左上角均放置 `<a class="home-pill" href="/">回到首页</a>`；结算页与 `1080×1440` 浅色工程战报底部均展示 `https://github.com/B-M-Capital-Research/honeclaw` 地址与一键点 Star 入口（读取或拉取仓库实时 Star 数并附赠首次致敬晶核）。

### V2 Verification

- `bun run typecheck:web`：通过。
- `bun run test:web`：通过（`hone-adventure.test.ts` 覆盖 V2 6 大产线关卡切换与跃迁、8 套硬件武器与超武、5 大产线 Boss、10 阶晋升属性聚合、4 维 12 节点基建矩阵与 `HONECLAW_GITHUB_URL`）。
- `bun run build:web:public`：通过。
- Playwright E2E `packages/app/e2e/public-data-center.spec.ts`（`-g "chat opens the online demo"`）：`390px` 与 `1440px` 全部通过，验证 `/chat` 打开游戏、`v2.0.0` 资源加载、关卡切换、结算页 `honeclaw` GitHub Star 卡片及左上角「回到首页」回跳 `/`。
- 独立无头 Chromium 8 场景截图巡检（大厅关卡切换、十阶晋升路线、四维基建矩阵、密集弹道与张量线框防遮挡 Boss 战、升级锁卡、结算 GitHub Star 卡片、硬件图鉴、`390×844` 移动端适配）零报错通过，并完成线上 `https://hone-claw.com/hone-adventure/` 生产部署核验。

## V2.0.1 阶段：后期密集战斗屏幕抖动收敛与开关

- **问题根因**：后期超武成型（如 `3D 堆叠数据洪流`、`Rubin 脉动方阵`、`吉瓦级超导馈线`）每秒触发数十次普通击杀（`FX_EVT.KILL`）与范围爆炸（`FX_EVT.MATRIX` / `FX_EVT.BOOM`），原 `fx.shake` 采用累加模型（`trauma += v`）且每次暴击击杀附带 `0.03s` 顿帧，导致后期 `trauma` 长期顶满 `1.0`（`±8.2px` 高频连续震屏）。
- **修复方案**（`fx.js` / `app.js` / `hone-adventure.test.ts`）：
  - 移除普通小怪击杀、暴击击杀微顿帧、常规武器矩阵/范围爆破与连击播报的所有镜头抖动，高频割草全程保持镜头零抖动。
  - 将 `fx.shake` 从累加模型改为**非叠加峰值包络**（`Math.max(fx.trauma, Math.min(cap, v))`），仅保留玩家受击、精英/Boss 击破、超频清屏与流片进化等低频关键事件的瞬时轻反馈，加快衰减速率（`4.2/s`）并将最大像素振幅收敛至 `< 2.5px`；同时在设置与暂停面板新增「关键事件轻微震屏」开关（关闭后全局 100% 零震屏）。
