# 推送/蒸馏成本整治：消灭重复计算 + 按公司共享润色

- status: `planned`
- created_at: `2026-08-16`
- owner: `Claude`（协调）+ `Codex`（实现）
- 执行范围：**P0 + P1**（用户 2026-08-16 决策）；P2 为已设计、待决策的后续项
- related_files:
  - `crates/hone-event-engine/src/global_digest/mainline_cron.rs`
  - `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-event-engine/src/router/dispatch.rs`
  - `crates/hone-event-engine/src/router/renderer.rs`
  - `crates/hone-event-engine/src/polisher.rs`
- related_docs:
  - `docs/current-plans/scheduler-runtime-hardening-2026-08-15.md`（role=all CPU 根因链）
  - `docs/conventions/periodic_tasks.md`

## Goal

把事件引擎的 LLM 成本从「随用户数线性增长、且反复计算未变化的输入」收敛为
「只为变化付费、共享产物按公司复用」。P0 消灭主线蒸馏的重复计算（**零个性化损失**）；
P1 让即时推送的 LLM 润色从每持有人一次降为每事件一次（个性化降级到模板层，
仓位/主线内容不丢，只丢"LLM 把仓位揉进行文"）。

## 背景与生产实测（2026-08-16，直接查生产 PG）

| 口径 | 数字 |
|---|---|
| 持仓行 / 去重公司 / 有持仓 actor | 749 / 323 / 61（去重比 2.3×） |
| 头部重叠 | RKLB 32 人持有，TEM 25、MU 24、SNDK 22 |
| 长尾 | 240 只票仅 1 人持有 |
| profile.md 数 / 内容去重后 | **472 / 472，零重复**（画像=用户私有论文，跨用户去重收益为零） |
| 有画像的 actor | 28 / 61；最大持仓用户（187 只）0 画像 |

**主线蒸馏永动机**：`should_trigger`（`mainline_cron.rs:60`）在 holdings 有任一 ticker
缺主线时每 6h 触发，而触发后 `distill_and_persist_one` 对该 actor **全部匹配画像重蒸**。
生产上几乎没有 actor 画像覆盖完整（20/27、16/24、0/187 …）⇒ 6h 重试**永远**触发
⇒ 每天 ~4 轮 × ~500 次 grok-4.3 调用（472 主线 + 28 style），输入与产出与上一轮
**一字不差**。role=all 试开时 51% 日志行来自本路径；PG 读放大部分已由
`dc68eedf`（批量读取）修复，LLM 重复计算部分即本方案 P0。

**润色扇出**：`deliver_high_immediate`（`router/dispatch.rs:631`）先把仓位注解、
主线关联拼进正文，再 `polisher.polish(event, body)`——正文人人不同导致同一条
RKLB High 事件要跑 32 次 LLM 润色，内容实质相同。

## P0 哈希增量蒸馏（零语义损失）

### 改法

1. `NotificationPrefs`（`prefs.rs`）新增两个字段，均 `#[serde(default)]` 向后兼容：
   - `mainline_source_hashes: Option<HashMap<String, String>>` —— ticker → 产出当前主线
     的 profile.md 内容 md5。
   - `mainline_distill_fingerprint: Option<String>` —— 上次蒸馏时的
     `md5(model_id + prompt模板)`。模型或 prompt 升级时全量重蒸的通道，不能丢。
2. `distill_from_profiles`（`mainline_distill.rs:314`）改为增量：
   - 对 scan 结果逐个算 md5；**待蒸集合** = hash 与存量不符的 + `mainline_by_ticker`
     未覆盖的；fingerprint 不匹配 ⇒ 全量。
   - 未变化的 ticker 直接沿用旧主线文本，**不发 LLM 调用**。
   - style：仅当待蒸集合非空或画像集合（hash 集）有变化时重算，否则沿用。
   - **最终 map 与现行为等价**：现行为是"用本轮结果整体覆盖"，增量版输出同一个
     map（未变化项取旧值），画像被删则对应 ticker 照旧从 map 消失。差异只有
     LLM 调用次数。
3. 触发节奏（`should_trigger`）**不改**：6h/周更节流保留。改完后"永动机"退化为
   每 6h 一次批量读（track-A 后每 actor 1 条查询、~KB 级）+ 内存 hash 比对 +
   通常 0 次 LLM——不需要再动触发语义，复杂度留在蒸馏层一处。
4. 调用侧签名不变（`distill_and_persist_one` 需要把现存 prefs 传进
   `distill_from_profiles` 或在内部先 load——实现取其一，但 prefs 读取不得多于现状）。

### 验收（每条都要 mutation 验证：注释掉 hash 比对逻辑，对应测试必须转红）

- 同一画像连续两个 tick：第二个 tick `distill_mainline` 与 `distill_style`
  调用数均为 **0**。
- 只改一份画像：恰好该 ticker 重蒸 1 次，其余 0 次；style 重算。
- fingerprint 变化：全量重蒸。
- 持仓含无画像 ticker（生产常态）：已覆盖 ticker **不**被重蒸——这条直接杀死永动机。
- 旧 prefs JSON（无新字段）加载后首轮全量蒸一次，之后进入增量。文件后端与 PG
  后端各验一次。

### 生产可观测判定

`cloud_llm_audit_records` 按 model=mainline_short 的日调用量：部署前基线
（role=all 试开窗口折算 ~2000/天）→ 部署后应降到**个位数/天**（≈画像编辑次数）。

## P1 润色按事件共享（个性化降到模板层）

### 改法

1. `deliver_high_immediate`（`router/dispatch.rs:631`）渲染顺序调整：
   - 先渲染**通用正文**（不含仓位注解、不含主线关联的事件原文）→
     `polish(event, generic_body)`；
   - 润色结果按 `(event.id, fmt)` 在 dispatcher 内做有界 memo（固定容量
     `Mutex<HashMap>`，容量 256、按插入序逐出即可，不引新依赖）；
   - 个性化内容（仓位注解、主线摘录、跨票关联）由 renderer 以模板**追加在润色后**
     —— 这些本来就是模板拼的数字/文本，从不需要 LLM。
2. `polish_levels` 门控、Plain-only 门控、polish 失败回退默认正文的语义全部不变。
3. 接受的损失（用户已批准）：润色不再能把"你持有 3.2% 仓位"揉进行文，仓位以
   附注行呈现。仓位感知（item 2）、主线关联（item 6）的**信息本身不丢**。

### 验收

- 3 个 actor 持有同一 symbol、同一 High 事件：counting stub polisher 恰好被调 **1** 次；
  三人正文各含自己的仓位数字与主线摘录。
- polish 失败：三人都拿到"通用正文 + 各自附注"，不丢投递。
- memo 逐出后再来同一事件：允许重新 polish（正确性不依赖缓存命中）。
- `replay_push_quality_audit`（ignored 测试）重放：投递条数、去重、
  delivery_log 关联与重构前逐条一致；正文允许措辞差异但必须包含仓位/主线要素
  （断言要素存在而非全文相等）。

## 通用验收门禁

```bash
cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
cargo test -p hone-memory --all-targets -- --ignored
bash tests/regression/run_ci.sh   # PATH 无 rg 条件下
```

无 SQL/schema 变更（prefs 是 JSONB 整体读写），不触发老库迁移演练要求。

## 排期与依赖（关键）

**必须等 track-B（存储 API async 化，`honeclaw-b` worktree）合并后再动工。**
track-B 正在重写 `prefs.rs` 的 PrefsProvider 调用链和 `router/dispatch.rs` 的
async 级联——与 P0/P1 的改动面直接重叠，并行必撞。track-B 合并 → 全量门禁绿
→ Codex 按本文实施（P0、P1 各自独立 commit，可单独回滚）。

## P2（已设计，暂不执行——留给用户后续决策）

digest 管线 company-major 化：per-ticker 共享简报（缓存键 = date/slot/ticker，
只算当天有新闻的票）+ 用户侧零 LLM 组装（holdings 交集 + 规则打分替代 LLM 排序），
pass2 personalize 仅保留给有自写画像的 actor（现 28 人，可演化为付费能力）。
成本曲线从 O(用户数) 变为 O(有新闻公司数)（被 323 universe 封顶），新增用户边际
LLM 成本≈0。**隐私红线：共享简报只能用公开数据，任何用户画像内容不得进入其他
用户可见的产物。** 待 P0/P1 生产数据落定后再决策。

## 本轮不做

- 不合并/互喂用户画像（隐私红线）。
- 不动 digest slot、轮询间隔、缺失 section 三项生产对齐（用户已知悉、单独决策）。
- 不降蒸馏模型档位——P0 之后蒸馏量≈编辑频率，降档收益无意义。
- 不改 `should_trigger` 触发语义（见 P0 改法第 3 条的理由）。
