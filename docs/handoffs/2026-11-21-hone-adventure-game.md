# HONE 历险记肉鸽割草游戏开发与上线

- title: HONE 历险记肉鸽割草游戏开发与上线
- status: done
- created_at: 2026-11-21
- updated_at: 2026-11-21
- owner: Antigravity
- related_files: packages/app/public/hone-adventure/index.html, packages/app/public/hone-adventure/app.js, packages/app/public/hone-adventure/world.js, packages/app/public/hone-adventure/data.js, packages/app/public/hone-adventure/art.js, packages/app/public/hone-adventure/audio.js, packages/app/public/hone-adventure/fx.js, packages/app/public/hone-adventure/poster.js, packages/app/public/hone-adventure/styles.css, packages/app/public/hone-adventure/sprites.webp, packages/app/public/hone-adventure/floor.webp, packages/app/public/hone-adventure/atlas-coords.js, packages/app/src/pages/chat.tsx, packages/app/src/lib/public-content.ts, packages/app/src/lib/hone-adventure.test.ts, packages/app/public/_worker.js, packages/app/public/_worker.test.js, packages/app/e2e/public-data-center.spec.ts, packages/app/vite.config.ts
- related_docs: docs/repo-map.md, docs/archive/plans/hone-adventure-game.md, docs/runbooks/backend-deployment.md
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
