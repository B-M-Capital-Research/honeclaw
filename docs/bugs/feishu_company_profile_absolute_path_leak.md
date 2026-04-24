# Bug: Feishu 公司画像建档成功后向用户暴露本机绝对路径与内部文件落点

- **发现时间**: 2026-04-24 19:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_feishu__direct__ou_5f6ac070b0b574f2bc3ba49f9678b675a3`
  - `2026-04-24T18:56:59.303665+08:00` 用户发送：`帮我建ccld公司画像`
  - `2026-04-24T18:59:02.075860+08:00` assistant final 明确返回：
    - `主画像：[/Users/ecohnoch/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__ou_5f6ac070b0b574f2bc3ba49f9678b675a3/company_profiles/ccld/profile.md](...)`
    - `初始化事件：[/Users/ecohnoch/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__ou_5f6ac070b0b574f2bc3ba49f9678b675a3/company_profiles/ccld/events/2026-04-24-init-profile.md](...)`
  - `sessions.last_message_preview` 同样保留 `/Users/ecohnoch/Desktop/honeclaw/data/agent-sandboxes/...` 绝对路径，说明这不是单次渲染误差，而是最终出站文本本身带了本机路径。
  - `data/runtime/logs/acp-events.log` 同时显示该轮正常 `stopReason=end_turn` 收口，没有证据表明系统把这类路径识别为内部实现细节并在出站前剥离。

## 端到端链路

1. Feishu 直聊用户要求“帮我建 ccld 公司画像”。
2. 系统完成画像建档，并在最终回复里同时汇报“已创建两份文件”。
3. 汇报内容直接包含本机绝对文件路径和 Markdown 文件链接，路径中暴露了本地仓库根目录、渠道 sandbox 结构和用户 open_id 作用域。
4. 用户虽然知道“画像已建好”，但拿到的是只对本机调试者有意义的内部路径，而不是面向 Feishu 用户的业务结果摘要。

## 期望效果

- 对外回复应只说明“画像已创建/已初始化事件”，并给出用户可理解的后续动作，例如继续分析、补交易计划、财报后更新。
- 本机绝对路径、sandbox 目录结构、open_id 作用域、Markdown 本地文件链接不应直接暴露给 Feishu 用户。
- 如果需要保留文件定位能力，也应只在桌面/app 内部使用，不应进入渠道用户可见文本。

## 当前实现效果

- 画像建档主功能已成功完成，但最终回复把内部文件落点当成用户可见结果的一部分发送。
- Feishu 用户无法使用这些 `/Users/...` 路径，且路径内容会暴露本地工程目录与运行时存储布局。
- 该问题属于输出质量与信息边界问题，不是“任务没做完”；因此当前按独立质量缺陷跟踪。

## 用户影响

- 用户会看到对自己没有操作意义的本机绝对路径，回复可读性下降，也暴露了不该面向终端用户公开的实现细节。
- 当前证据里，画像文件实际已创建，后续继续分析的主功能链路仍可继续使用，没有出现错误投递、任务失败或数据损坏。
- 因此该问题不影响主功能完成度，按 `P3` 定级，而不是 `P1/P2` 的功能性故障。

## 根因判断

- 公司画像建档链路很可能直接复用了桌面/app 场景下的 Markdown 文件链接输出模板，没有区分“本机可点击路径”与“渠道用户可见文本”。
- 出站净化当前主要关注 `<think>`、原始报错、工具协议等污染文本，但没有把本地绝对路径和 markdown file-link 视为需要剥离的内部实现细节。
- 这与已有“空回复 fallback”或“过早收口”不是同一根因；本轮是建档成功后的结果组织与出站边界问题。

## 下一步建议

- 排查公司画像创建成功后的用户态总结模板，确认是否直接复用了桌面 markdown link 生成逻辑。
- 为 Feishu / Telegram / Discord 等外部渠道增加本地路径净化规则：命中 `/Users/...`、`data/agent-sandboxes/...`、Markdown 本地文件链接时改写为用户可理解的纯文本摘要。
- 在缺陷修复前，后续巡检继续关注“建画像/写报告/生成文件”类真实会话，确认是否还有同类本地路径外泄样本。
