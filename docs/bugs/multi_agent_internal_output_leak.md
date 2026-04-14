# Bug: 多代理内部思考与工具协议文本泄漏到用户回复

- **发现时间**: 2026-04-14
- **Bug Type**: Business Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近修复提交: `12a5352 fix: sanitize leaked internal agent output`
  - 关联归档: `docs/archive/plans/multi-agent-output-sanitization.md`

## 端到端链路

1. 用户从 Feishu / Telegram / Discord / iMessage 等渠道发起一次 multi-agent 问答。
2. 搜索阶段或思考流阶段生成了 `<think>`、`<tool_call>`、`<tool_result>`、`[TOOL_CALL]` 等内部协议内容。
3. 旧实现会把这些中间产物直接当成用户可见正文继续流式渲染，甚至落入 compact summary 与历史会话。
4. 用户最终收到的不是纯净答案，而是带有内部推理或伪工具调用残渣的回复。

## 期望效果

- 用户只能看到清洗后的最终答案，内部思考、工具调用协议和工作草稿必须全部隐藏。
- 即便历史会话中曾混入脏内容，后续 restore / compact 也应阻断其再次污染 prompt。

## 当前实现效果（问题发现时）

- 用户可能直接看到 `<think>...</think>`、`<tool_call>...</tool_call>` 或类协议正文。
- Feishu 等流式链路会把这些内部内容实时展示出来，造成明显的产品失真。
- 历史污染还可能被继续写回 compact summary，导致后续会话反复被旧污染放大。

## 用户影响

- 直接泄露系统内部工作稿，破坏产品可信度与专业感。
- 在群聊或外部沟通场景中尤其敏感，会让终端用户误以为系统把“脑内推理”和“工具协议”当正式答复发送。
- 一旦污染历史会话，后续多轮回复都可能继续异常。

## 根因判断

- multi-agent 搜索阶段与统一运行时缺少“用户可见输出”和“内部协议输出”的严格边界。
- 会话恢复和 compact 路径此前没有对历史 assistant 内容做统一净化。

## 修复线索

- `12a5352` 已补充统一输出净化、历史污染清洗，以及渠道侧 `<think>` 隐藏渲染。
- 当前 bug 台账先以 `New` 登记，等待人工确认是否按现网状态转 `Fixed` / `Closed`。
