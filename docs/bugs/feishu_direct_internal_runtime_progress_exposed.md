# Bug: Feishu direct 投研回复外露本机命令与内部工具流程

## 发现时间

- 2026-06-17 23:02 CST

## Bug Type

- Business Error

## 严重等级

- P3

## 状态

- New

## GitHub Issue

- 无，非 P1

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-17 19:00-23:02 CST。
  - session_id: `Actor_feishu__direct__ou_5f6ac070b0b574f2bc3ba49f9678b675a3`。
  - ACP 事件在 2026-06-17T13:22:56Z 以 `stopReason=end_turn` 收口，说明 Feishu direct 链路完成。
  - 用户前序要求追问老铺黄金 / `06181.HK` 财报数据来源；assistant final 完成结构化财务口径纠偏，但开头连续写出本机执行过程与内部能力名，包括“本机没有 `python` 命令，我改用 `python3` 继续查”“已加载股票研究流程”“现在用 Hone 的实时检索工具再查一遍”“我会把数据补进老铺黄金画像”等。
- 同窗复核：
  - `data/sessions.sqlite3` 的 `session_messages` 仍停在 `2026-06-17T10:37:37.202464+08:00`，因此本轮用户可见文本证据来自 ACP 流式日志重构。
  - 19:00-23:02 CST `acp-events.log` 有 55 个 ACP session 启动、55 个 prompt、220 个 response、55 个 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误进入本轮候选。
  - 同窗最近 `data/sessions/*.json` 有 5 个会话文件在 20:00-21:21 CST 更新，但 JSON 会话源没有覆盖该 21:20 新会话，进一步说明本轮需要以 ACP 流式日志作为真实会话证据。

## 端到端链路

1. Feishu direct 用户追问前一轮为何没有拿到老铺黄金财报数据。
2. runner 进入投研回答链路，尝试搜索交易所 / 公司公告、调用行情财务数据与本地画像沉淀。
3. 部分执行过程、命令选择和内部工具 / 流程名被模型写入 assistant final。
4. final 仍正常完成财务数据纠偏和回答，并以 `end_turn` 收口。
5. 用户可见文本同时看到业务结论和本机执行细节。

## 期望效果

- Feishu 用户只应看到业务化说明，例如“我改用更直接的公告与行情财务来源重新核验”。
- 不应暴露本机命令可用性、`python` / `python3` 切换、内部工具名、内部研究流程名或画像写入过程。
- 如果官方 PDF 仍未稳定打开，应只说明数据口径边界，不展示执行路径。

## 当前实现效果

- 回复主体回答了用户问题，并明确区分“结构化财务口径”和“官方 PDF 直链未稳定打开”。
- 但 final 前段把多句执行过程当作用户态正文输出，暴露本机命令状态、内部工具名和画像沉淀流程。
- 这不是链路失败：没有未回复、空回复、错投、重复投递、原始 provider 错误或内部 prompt 泄露证据。

## 用户影响

- 用户仍拿到可用的老铺黄金财务核验结果，Feishu direct 主功能链路没有被阻断。
- 问题主要影响产品感、信任感和实现边界：普通投研回答显得像调试日志或 agent 中间过程。
- 因为业务回答完成、投递收口正常，且没有造成数据写坏或消息投递异常，所以不影响功能链路，按规则定级为 `P3`。

## 根因判断

- 共享用户可见输出净化已覆盖部分“技能未加载 / 本地技能文件不可读 / 本地 data 口径”等短语，但没有覆盖自然语言形式的本机命令切换、内部研究流程和 Hone 工具名。
- Feishu direct answer 阶段允许模型把工具规划和执行进度原样合并到 final，而不是只保留业务结论。
- 该问题不同于 `feishu_direct_local_skill_file_path_unreadable_exposed.md`：本轮不是技能文件不可读，而是本机命令与内部工具流程外露。
- 该问题不同于 `web_direct_internal_skill_and_local_store_terms_exposed.md`：本轮发生在 Feishu direct，且外露形态包括本机命令可用性与投研流程。

## 下一步建议

- 扩展共享用户可见净化或 Feishu direct final guidance，过滤 / 改写“本机没有 python / 改用 python3”“Hone 的实时检索工具”“已加载股票研究流程”“补进公司画像”等内部执行说明。
- 对 Feishu direct 投研问答增加回归：当 runner 需要切换命令或内部工具时，最终回复只保留数据来源和口径边界，不出现命令、工具名或画像写入过程。
- 后续巡检若只在 tool update / rawOutput 中看到这些词，但 final 没有外露，不应补充为本缺陷复发。
