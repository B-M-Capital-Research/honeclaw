# HONE 历险记 V3：去模板化、后期深度构筑、通俗化描述与特工/产线深度交互上线

- title: HONE 历险记 V3：去模板化、后期深度构筑、通俗化描述与特工/产线深度交互上线
- status: done
- created_at: 2026-11-24
- updated_at: 2026-11-24
- owner: Antigravity
- related_files: packages/app/public/hone-adventure/index.html, packages/app/public/hone-adventure/data.js, packages/app/public/hone-adventure/world.js, packages/app/public/hone-adventure/art.js, packages/app/public/hone-adventure/fx.js, packages/app/public/hone-adventure/audio.js, packages/app/public/hone-adventure/poster.js, packages/app/public/hone-adventure/app.js, packages/app/public/hone-adventure/styles.css, packages/app/public/_worker.js, packages/app/src/lib/public-content.ts, packages/app/src/components/public-nav.tsx, packages/app/src/pages/public-home.tsx, packages/app/src/components/public-login-form.tsx, packages/app/src/pages/public-data-center.tsx, packages/app/src/pages/public-data-center.css, packages/app/src/lib/hone-adventure.test.ts, packages/app/e2e/public-data-center.spec.ts
- related_docs: docs/repo-map.md, docs/archive/plans/hone-adventure-v3-deep-overhaul.md, docs/handoffs/2026-11-21-hone-adventure-game.md
- related_prs: direct push to `main` (Cloudflare Pages production deployment)

## Summary

完成《HONE 历险记》（`/hone-adventure/`）V3.0.0 全量升级并发布至线上：彻底清除了所有残留的基版机制与视听痕迹，保留现有 AI 硬件武器标题并将全部武器/超武/被动/联动/圣物说明改写为直观通俗的中文，同时深度重构了后期构筑上限、5 名特工主动技能与专精挂载体系、以及 6 大算力产线的局内外切换与地图机关交互。

## What Changed

- **彻底清除基版痕迹**：
  - 移除大厅右上角「领曜金皮」礼包按钮，替换为三档工况难度切换（`标准巡检` / `满载压测 ×1.45` / `极限流片 ×2.0`）；曜金形态独立为第 5 名算力特工「星枢」。
  - 删除开局 12 秒脚本化 36 怪圆环 + 白送清屏核弹的旧机制，改为贯穿全局的「洁净室产线交互基站」（`ZONE_KIND.NODE`）空间占点充能玩法。
  - 移除 `L3 缓存环刃` 周期性熄火停转的旧设定（改为全程常驻护体 + 周期性磁暴脉冲）；重写 `audio.js` 为 138 BPM 8 小节 B-Dorian 赛博洁净室电子合成配乐（支持随 6 大产线移调），替换原 130 BPM 4 小节循环；重构 8 条每日算力工况词条。
- **武器扩充与全量通俗中文描述**：
  - 保留 AI 硬件标题（`HBM 带宽飞梭`、`硅光分光束`、`张量矩阵炮` 等），新增第 9/10 把武器 `NVLink 光网矩阵 → 全互联超算穹顶` 与 `液氮超频迫击 → 极寒坍缩奇点`（配对新被动 `超导供电轨`），并将全部主武器、超武、每级提升、被动模块、联动与圣物说明改写为一目了然的直白中文。
- **后期深度玩法与多维升级路径**：
  - 槽位从 `4+4` 扩充至 `5+5`（可通过圣物扩至 `6+6`），彻底消除满槽后只剩 3 张枯燥兜底卡的断层。
  - 新增 **6 项双武器联动协议**（`⚡ 双武联动`）、**超武突破 `★+1 ~ ★+5`**（超武成型后可继续突破 5 阶）、**8 张终局圣物卡**（`✦ 终局圣物`，含自动充能护盾、16% 血线斩杀、多核齐射 +1 弹道、连环电浆爆破、击杀回血、扩容主板等）、**4 类词条精英怪**（护盾/狂飙/狂暴/裂变）与 **终局双巨兽同屏压测**，以及 **4 分支 × 4 节点 = 16 节点基建矩阵**（双里程碑 `6pt` / `10pt`）和 **12 阶晋升路线**。
- **特工编队与产线切换深度交互**：
  - 5 名特工（`小砺`、`芯核`、`光瞳`、`量闪`、`星枢`）均配备独立**主动战术大招**（`Space`/`E` 或右下角按钮释放）、成长型被动、主场产线加成、**专精等级 Lv.1~Lv.5**，且专精达 Lv.2 后可自由切换任意已点亮武器作为开局主武器。
  - 大厅支持 **全产线轮换 / 驻守本关** 切换、产线星级评定（`★~★★★`）与主场特工提示；局内实装 **6 大产线专属交互基站** 与 **HUD 可控产线跃迁门**（击败 Boss 或充能完成后可在局内自选下一条产线跃迁，立即获得 10 秒跨产线超频增益）。

### 阶段节点 2：全站免登录直达入口开放（2026-11-24）

- **问题根因**：`/hone-adventure/` 游戏页面本身不校验登录态，但此前站内唯一可见入口挂在已登录 `/chat` 输入框工具栏内；未登录访客从官网 `/` 点击进入 `/chat` 或 `/research` 时会被 `PublicLoginForm` 登录页拦截，导致外部用户无法免登录找到入口。
- **修复方案**：
  - 官网首页（`public-home.tsx`）Hero 主行动区新增原生 `<a href="/hone-adventure/">HONE 历险记 · 免登录畅玩</a>` 按钮。
  - 全站顶部公开导航与移动端抽屉菜单（`public-nav.tsx`）新增原生 `/hone-adventure/` 免登录直达链接（绕开 SPA 通配路由回跳）。
  - 未登录验证码卡片（`public-login-form.tsx`，覆盖未登录访问 `/chat`、`/research`、`/me`）底部新增「免登录公开体验」快捷栏，一键直达 `HONE 历险记` 与 `3D 数据中心`。
  - 公开免登录页面 `/data-center`（`public-data-center.tsx`）头部同步提供 `HONE 历险记 ↗` 姊妹入口；`_worker.js` 显式保证 `/hone-adventure`（无尾斜杠）308 跳转 `/hone-adventure/`。

## Verification

- `bun run typecheck:web`：通过。
- `bun run test:web`：通过（`packages/app/src/lib/hone-adventure.test.ts` 全量覆盖 10 武器、主动大招、超武突破 `★+1~+5`、双武联动、终局圣物、产线交互基站、局内跃迁、特工专精与去模板化断言）。
- `bun run build:web:public`：通过，`dist-public/hone-adventure/` 自动生成最新 SHA-256 `?v=<hash>` 防缓存指纹。
- 无头 Chromium 多场景截图巡检（桌面 `1440x920` 与移动端 `390x844`）及 Playwright E2E `packages/app/e2e/public-data-center.spec.ts`：零报错、浅色洁净室无视野遮挡、无持续屏幕抖动。
- 生产环境核验：`https://hone-claw.com/hone-adventure/` 返回 `version === "3.0.0"`，最新哈希子资源 200 生效，零运行时异常。

## Risks / Follow-ups

- 本地存档由 `migrateSave()` 自动从 `v1`/`v2` 无损升级至 `v3`，保留玩家既有晶核、最高存活记录、已解锁特工与图鉴进度。
- 如后续继续扩充新武器或新特工，只需在 `data.js`、`art.js` 与 `world.js` 同步注册即可，`vite.config.ts` 的 `closeBundle` 会自动刷新线上缓存指纹。

## Next Entry Point

- 游戏主逻辑与数据定义：`packages/app/public/hone-adventure/data.js`、`packages/app/public/hone-adventure/world.js`、`packages/app/public/hone-adventure/app.js`
- 单元与 E2E 回归：`packages/app/src/lib/hone-adventure.test.ts`、`packages/app/e2e/public-data-center.spec.ts`
