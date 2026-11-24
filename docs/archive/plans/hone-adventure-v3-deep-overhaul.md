# HONE 历险记 V3：彻底去模板化、后期深度构筑玩法、通俗化描述与关卡/特工深度交互重构

- title: HONE 历险记 V3：彻底去模板化、后期深度构筑玩法、通俗化描述与关卡/特工深度交互重构
- status: archived
- created_at: 2026-11-24
- updated_at: 2026-11-24
- owner: Antigravity
- related_files: packages/app/public/hone-adventure/index.html, packages/app/public/hone-adventure/data.js, packages/app/public/hone-adventure/world.js, packages/app/public/hone-adventure/art.js, packages/app/public/hone-adventure/fx.js, packages/app/public/hone-adventure/audio.js, packages/app/public/hone-adventure/poster.js, packages/app/public/hone-adventure/app.js, packages/app/public/hone-adventure/styles.css, packages/app/src/lib/hone-adventure.test.ts, packages/app/e2e/public-data-center.spec.ts
- related_docs: docs/repo-map.md, docs/handoffs/2026-11-24-hone-adventure-v3.md, docs/handoffs/2026-11-21-hone-adventure-game.md

## Goal

全面排查并清除《HONE 历险记》中残留的《胖鱼不好惹》基版痕迹（包括右上角一键领金皮挂件、12 秒 36 怪圆环送清屏核弹脚本、完全一致的 4 角色属性/解锁数字模板、7 条旧每日词条镜像、L3 环刃间歇停转旧机制、4+4 槽位封顶后三选一枯燥兜底卡、以及 130 BPM 原四小节和弦循环）；在保留现有 AI 硬件武器标题的基础上，将全部武器、超武、被动、升级项、角色与圣物描述改写为直观易懂的通俗中文；深度重构**特工选择体系**（5 名特工、专属主动战术大招、特工专精等级与自由更换起手主武器）、**产线关卡切换体系**（3 档超频难度、6 大产线专属地图交互基站、产线星级评定与局内可控产线跃迁门）以及**后期深度玩法与升级路径**（扩充至 10 把武器×10 超武、5+5 基础槽位、6 项双武协同协议、超武突破 `★+1~★+5`、8 张终局架构圣物卡、词条精英与双巨兽终局波次、四维 16 节点基建矩阵），验证通过后发布上线。

## Scope

- [x] **彻底清除基版痕迹**：
  - 移除右上角「领曜金皮」礼包挂件，替换为「工况难度切换（标准巡检 / 满载压测 / 极限流片）」；曜金形态纳入特工编队与第 5 名独立特工「星枢」。
  - 移除开局 12 秒脚本化 36 怪包围圈 + 白送全屏脉冲弹的旧教程痕迹，改为贯穿全局的「洁净室产线交互基站」空间玩法。
  - 重写 `audio.js` 电子合成配乐（138 BPM 8 小节赛博洁净室旋律与随产线移调，彻底替换原 130 BPM 4 小节和弦循环）及调整波次/数值节奏常数。
  - 消除 `L3 缓存环刃` 间歇熄火的旧珍珠护卫痕迹（改为常驻护体 + 周期性磁力脉冲），扩充至 **10 把硬件主武器 × 10 超武**（新增 `NVLink 光网矩阵` 与 `液氮超频迫击`）。
  - 重构 8 条全新数据中心每日工况词条，彻底替换旧 7 词条镜像。
- [x] **武器与模块描述全面通俗化**：保留现有 AI 硬件标题（`HBM 带宽飞梭`、`硅光分光束`、`张量矩阵炮` 等），将所有主武器、超武、每级提升、被动模块、协同协议与终局圣物的说明改写为一读即懂的直白中文。
- [x] **人物选择深度逻辑重构**：
  - 扩充至 **5 名差异化特工**（`小砺`、`芯核`、`光瞳`、`量闪`、`星枢`），每名特工拥有独立**主动战术技能**（`Space` 键或 HUD 技能按钮释放）、专属成长被动与主场产线加成。
  - 新增**特工专精等级（Lv.1~Lv.5）**与**起手主武器自由挂载**（特工专精达 Lv.2 后，可自由选择任意已点亮武器作为开局主武器）。
- [x] **关卡切换深度逻辑重构**：
  - 大厅支持 **6 大算力产线切换 × 3 档工况难度** + **路线模式切换（自动轮换 / 驻守本关）**，展示每关专属地图机关与通关星级（`★0~★3`）。
  - 局内实装 **6 类产线专属地图交互基站**（踩点充能触发：晶核喷泉、光速聚焦、冷却超频塔、全屏矩阵轰炸、无尘修复护盾、超导母线脉冲）及 **HUD 可控产线跃迁面板**（充能就绪或击败 Boss 后可随时在局内任选下一条产线跃迁并获得跃迁超频增益）。
- [x] **后期玩法与多维升级路径全面升级**：
  - 局内构筑从 `4+4` 扩容为 `5+5`（可通过圣物扩至 `6+6`），新增 **6 项双武器协同协议**、**超武突破 `★+1 ~ ★+5`**（超武成型后仍可继续突破 5 次）、**8 张终局架构圣物卡**（多核 +1 弹道、16% 斩杀线、能量护盾、连环电浆爆破、击杀回血、槽位扩容等）。
  - 后期怪群新增**词条高危精英**（护盾 / 极速 / 狂暴 / 裂变）与 **终局双巨兽同屏压测**；局外基建扩充为 **4 分支 × 4 节点 = 16 节点算力基建矩阵**（含 `6pt` 与 `10pt` 双重里程碑）与 **12 阶晋升路线**。
- [x] **验证与上线**：更新单元测试与 Playwright E2E，执行无头浏览器全界面截图巡检，在干净 `git worktree` 中通过门禁后推送到 `main` 并核验线上 `https://hone-claw.com/hone-adventure/`。

## Validation

- [x] `bun run typecheck:web`
- [x] `bun run test:web`（覆盖 10 武器、主动大招、超武突破 `★+1~+5`、双武协同协议、终局圣物、产线交互基站、局内自选产线跃迁、5 特工专精与起手武器切换）
- [x] `bun run build:web:public`
- [x] Playwright E2E `packages/app/e2e/public-data-center.spec.ts` 与无头 Chromium 多场景截图巡检
- [x] 线上 `https://hone-claw.com/hone-adventure/` 生产部署核验

## Documentation Sync

- [x] 更新 `docs/repo-map.md` 中 `/hone-adventure/` V3 深度构筑、主动技与关卡交互说明
- [x] 新增 `docs/handoffs/2026-11-24-hone-adventure-v3.md` 记录 V3 改造与验证结论
- [x] 完成后将本计划归档至 `docs/archive/plans/hone-adventure-v3-deep-overhaul.md`，清理 `docs/current-plan.md` 并补充 `docs/archive/index.md`

## Risks / Open Questions

- 存档向后兼容：升级存档结构时完整保留玩家已有的晶核（`cores`）、最高时长（`best`）、已解锁特工与图鉴，无损迁移旧字段。
- 浅色可读性红线：新增的 2 把武器（`NVLink 光网矩阵`、`液氮超频迫击`）、产线交互基站与护盾/主动技特效延续浅色洁净室高对比描边与低透明底色规范，无走位视野遮挡。
