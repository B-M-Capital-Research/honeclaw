# HONE 历险记肉鸽割草游戏开发与上线

- title: HONE 历险记肉鸽割草游戏开发与上线
- status: archived
- created_at: 2026-11-21
- updated_at: 2026-11-21
- owner: Antigravity
- related_files: packages/app/src/pages/chat.tsx, packages/app/src/lib/public-content.ts, packages/app/public/_worker.js, packages/app/public/_worker.test.js, packages/app/public/hone-adventure/index.html, packages/app/src/lib/hone-adventure.test.ts, packages/app/e2e/public-data-center.spec.ts, packages/app/vite.config.ts
- related_docs: docs/repo-map.md, docs/handoffs/2026-11-21-hone-adventure-game.md, docs/runbooks/backend-deployment.md

## Goal

参考《胖鱼不好惹》单文件高性能肉鸽割草架构，完整设计并实现 HONE 原生世界观肉鸽生存游戏《HONE 历险记》（HONE Adventure），重构全套美术风格、角色/怪物/道具图像、武器进化与三分支算力天赋升级路径，挂载到 `/chat` 输入框「3D 数据中心」游戏入口旁，并完成构建验证与线上发布。

## Scope

- [x] 重新设计并生成《HONE 历险记》全套美术图像资源（4 位 HONE 特工含限定金皮、9 类市场噪音怪、4 大资本/算力巨兽 Boss、场景道具与深色墨玉算力地板纹理）及高精 Canvas/WebGL 双轨精灵图集。
- [x] 实现完整游戏本体 `packages/app/public/hone-adventure/`（WebGL 批处理渲染器 + Canvas 2D 回退、WebAudio 实时合成音效与 4 阶动态电子配乐、8 武器×8 超武进化、9 被动、3 分支 9 项局外算力天赋树、常规无尽模式 + 7 词条每日同种子挑战、生涯战绩与本地 Top 10 荣耀殿堂、3:4 战报海报导出与返回 HONE 导航）。
- [x] 在 `packages/app/src/pages/chat.tsx` 与 `packages/app/src/lib/public-content.ts`（中英双语）中将「HONE 历险记」挂载到「3D 数据中心」入口旁，桌面端 chip 与移动端「+」面板同步可达且保留聊天草稿。
- [x] 在 `packages/app/public/_worker.js` 增加 `/hone-adventure/` 静态资源防 SPA 误回退保护与安全响应头测试。
- [x] 完成类型检查、单测、Playwright 端到端验证、`build:web:public` 构建，提交并推送到 `main` 触发 Cloudflare Pages 线上发布与公网探针验收。

## Validation

- [x] `bun run typecheck:web`
- [x] `bun run test:web`（含语言契约、样式契约、`_worker.test.js` 及新增游戏/入口回归单测）
- [x] `bun run build:web:public`（校验 `dist-public/hone-adventure/index.html` 与 entry hash）
- [x] Playwright 桌面与移动端验证 `/chat` 中「HONE 历险记」位于「3D 数据中心」旁、新标签打开 `/hone-adventure/` 正常渲染可玩且保留草稿、无 `pageerror`
- [x] 线上 `https://hone-claw.com/`、`/chat`、`/hone-adventure/` HTTP 状态、安全头与静态 bundle hash 校验

## Documentation Sync

- [x] 更新 `docs/repo-map.md` 中 `/chat` 工具栏 `3D 数据中心` 旁的 `HONE 历险记` 游戏入口说明
- [x] 新增 `docs/handoffs/2026-11-21-hone-adventure-game.md` 记录上线范围、验证结果与回滚路径
- [x] 任务完成后将本计划移入 `docs/archive/plans/hone-adventure-game.md`，从 `docs/current-plan.md` 移除活跃索引并追加 `docs/archive/index.md`

## Risks / Open Questions

- 工作区存在其它任务的未提交改动；通过独立临时 `git worktree` 限定本任务文件提交与推送，避免混入无关 WIP。
- `chat.tsx` 受 `public-language-contract.test.ts` 约束，禁止硬编码中文字符，所有文案均走 `CONTENT` 中英双语词典。
- 按 CD 契约推送 `main` 上线 Cloudflare Pages 生产站点，未创建 `v*` tag。
