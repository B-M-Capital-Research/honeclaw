# Bug: Web 直聊公司画像沉淀后向用户暴露内部相对文件路径

- **发现时间**: 2026-06-02 11:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-02 10:58-11:00 CST
  - session_id: `Actor_web__direct__web-user-14f4cadb069f`
  - 用户输入摘要：`avgo财报如何看`
  - ACP 事件显示该轮有 `session/prompt`、公司画像文件写入 tool update，以及最终 `response stopReason=end_turn`，说明 Web direct 回复链路已收口。
  - 最终用户可见正文末尾包含：`我已把 AVGO 财报前框架沉淀到 company_profiles/AVGO.md，后续财报出来可以直接对照更新。`

## 端到端链路

1. Web direct 用户询问 AVGO 财报怎么看。
2. runner 校验 Broadcom 财报时间、Q1 / Q2 指引、AI revenue、行情和新闻，并写入 actor sandbox 下的公司画像文件。
3. 最终回复完成了 AVGO 财报前判断、Bull / Bear、动作建议、看点和来源，并以 `stopReason=end_turn` 收口。
4. 回复末尾把内部长期画像相对文件路径 `company_profiles/AVGO.md` 直接展示给 Web 用户。

## 期望效果

- 对外回复可以说明“已为后续跟踪沉淀本轮 AVGO 财报前框架”。
- 不应把 `company_profiles/<ticker>.md` 这类内部文件组织路径作为用户可见结论的一部分。
- 若 Web 产品要暴露画像入口，应使用前端可点击的业务入口、附件或自然语言说明，而不是 runner sandbox 的内部目录名。

## 当前实现效果

- 主分析内容完整，用户可以基于正文理解 AVGO 财报前看点。
- 但最终回复把 `company_profiles/AVGO.md` 作为沉淀位置告诉用户；该相对路径不是 Web 用户可直接使用的稳定产品入口。
- 本轮没有看到 `/Users/...`、`data/agent-sandboxes/...`、`/var/folders/...` 等绝对路径进入最终正文；绝对路径只出现在 ACP tool update 诊断事件中。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 它暴露了公司画像的内部文件组织方式，降低 Web 回复的产品感，也可能让用户误以为自己能直接访问该路径。
- 本轮 AVGO 财报分析已完成、文件写入也成功、最终 `response` 正常 `end_turn`，没有未回复、错投、数据损坏或投递失败证据。
- 因此它不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 初步判断是公司画像沉淀流程把 runner 原生文件路径作为“沉淀完成”的证明写入最终用户回复。
- 既有 `feishu_company_profile_absolute_path_leak.md` 修复覆盖的是绝对路径、本地 Markdown 链接和 sandbox 标识脱敏；本轮新增证据是 Web direct 最终正文里的内部相对路径，属于相邻但独立的用户态文案边界。
- 该问题也不同于 `web_direct_tool_call_raw_output_leak`：本轮最终正文没有 raw JSON、工具协议或 provider 报错外泄。

## 下一步建议

- 在公司画像 / 长期跟踪最终回复模板或共享出站净化层中，将 `company_profiles/<ticker>.md`、`events/*.md` 等内部相对路径改写为自然语言。
- 对 Web / Feishu direct 增加一条回归：当 runner 成功写入公司画像文件时，最终用户可见文本只说明已沉淀，不包含内部文件路径。
- 后续巡检继续区分两类证据：绝对路径 / sandbox 标识泄漏应回看既有路径脱敏缺陷；仅相对内部路径进入自然语言回复时按本单跟踪。
