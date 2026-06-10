# Bug: Feishu 直聊通俗化改写回复外露本地技能文件路径不可读

## 发现时间

- 2026-06-10 11:03 CST

## Bug Type

- Business Error

## 严重等级

- P3

## 状态

- New

## GitHub Issue

- 无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-06-10 07:03-11:03 CST。
  - session `Actor_feishu__direct__ou_5fe40dc70caa78ad6cb0185c21b53c4732`。
  - user `ordinal=324` / `timestamp=2026-06-10T07:56:48.677153+08:00`：用户要求把一段黄金技术分析改写成普通小白可理解的表达，并使用生活比喻。
  - assistant `ordinal=325` / `timestamp=2026-06-10T07:57:48.016141+08:00`：final 开头写出“本地技能文件路径不可读，我继续按你给的原文做通俗化改写；这轮不需要实时行情，所以不做价格结论。”
  - 同一 assistant final 随后完成了通俗化改写与生活比喻，用户请求本身没有被阻断。
- 同窗巡检摘要：
  - 最近四小时共有 25 个 user turn 与 26 个 assistant 记录；Feishu direct 用户请求均有 assistant final 收口。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、`company_profiles/...`、raw tool 字段、思维痕迹、provider 原始错误、quota、panic 或 stream disconnect。
  - 本轮问题集中在用户可见 final 外露内部技能 / 本地文件状态，而不是 Feishu 投递失败、runner 未收口或数据破坏。

## 端到端链路

1. Feishu direct 用户发送一段黄金技术分析，要求改写成小白可理解版本。
2. 系统进入直聊回答链路，并尝试使用技能 / 本地上下文辅助回答。
3. 某个本地技能文件路径不可读或技能加载状态异常。
4. final 没有把该内部状态净化掉，而是直接写入用户可见回复开头。
5. 回复继续完成业务改写并正常落库 / 投递。

## 期望效果

- 用户可见回复应直接完成通俗化改写，或用业务语言说明“我将基于你提供的原文改写”。
- 不应暴露“本地技能文件路径不可读”这类内部技能、本地文件或运行时状态。
- 如果技能不可用，系统可以静默降级到原文改写能力，但 final 不应让用户看到本地执行环境细节。

## 当前实现效果

- Feishu direct final 先暴露内部本地技能文件状态，再继续回答用户问题。
- 业务回答本身基本完成，结构正常，没有空回复、错投、重复投递或 runner 中断证据。
- 该问题与 `feishu_direct_image_attachment_not_readable_skill_phrase_exposed.md` 不同：本轮没有图片附件，也没有图片理解链路阻断；它是普通文本改写直聊中外露本地技能文件状态。
- 该问题与 `web_direct_internal_skill_and_local_store_terms_exposed.md` 不同：本轮发生在 Feishu direct，且外露的是“本地技能文件路径不可读”这一更具体的本地文件 / 技能状态。

## 用户影响

- 用户仍收到可用的通俗化改写结果，Feishu 直聊主功能链路没有被阻断。
- 没有证据显示投递失败、空回复、错投、跨用户数据暴露、绝对路径泄漏或敏感信息泄漏。
- 影响集中在产品感、信任感和内部实现边界：用户看到本地技能文件状态，会把一次普通改写理解为系统运行异常。
- 因为不影响主功能链路，只是用户可见文案质量与内部状态外露，按规则定级为 `P3`。

## 根因判断

- 共享用户可见输出净化层已覆盖部分“技能未加载 / 图片理解工具没成功激活”等措辞，但没有覆盖“本地技能文件路径不可读”这种普通文本直聊中的内部状态句。
- Feishu direct answer 阶段可能允许 runner 把本地技能读取失败的自我说明作为 final 首句输出。
- 需要同时补 prompt guard 和 sanitizer：prompt 避免模型主动解释本地技能 / 文件状态，sanitizer 兜底改写或删除同类句子。

## 下一步建议

- 扩展 `sanitize_user_visible_output(...)` 或等效共享净化层，覆盖“本地技能文件路径不可读”“技能文件不可读”“本地技能路径无法读取”等同义句。
- 对 Feishu direct 文本改写类请求增加回归：当内部技能读取失败时，final 只保留“我将基于你提供的原文改写”之类用户态说明，不出现本地文件 / 技能路径状态。
- 检查 answer prompt 是否明确禁止向用户描述本地技能文件、技能加载状态、路径可读性和内部降级原因。
