# Bug: Feishu 公司画像建档成功后向用户暴露本机绝对路径与内部文件落点

- **发现时间**: 2026-04-24 19:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **证据来源**:
  - `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_feishu__direct__ou_5f6ac070b0b574f2bc3ba49f9678b675a3`
  - `2026-04-24T18:56:59.303665+08:00` 用户发送：`帮我建ccld公司画像`
  - `2026-04-24T18:59:02.075860+08:00` assistant final 明确返回：
    - `主画像：[/Users/ecohnoch/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__ou_5f6ac070b0b574f2bc3ba49f9678b675a3/company_profiles/ccld/profile.md](...)`
    - `初始化事件：[/Users/ecohnoch/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__ou_5f6ac070b0b574f2bc3ba49f9678b675a3/company_profiles/ccld/events/2026-04-24-init-profile.md](...)`
  - `sessions.last_message_preview` 同样保留 `/Users/ecohnoch/Desktop/honeclaw/data/agent-sandboxes/...` 绝对路径，说明这不是单次渲染误差，而是最终出站文本本身带了本机路径。
  - `data/runtime/logs/acp-events.log` 同时显示该轮正常 `stopReason=end_turn` 收口，没有证据表明系统把这类路径识别为内部实现细节并在出站前剥离。

## 2026-05-13 复发证据

- 本轮巡检确认该缺陷在最近四小时真实 Feishu direct 会话中复发，状态从 `Fixed` 调回 `New`。
- `data/sessions/Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1.json`
  - `2026-05-12T23:48:33.273558+08:00` 用户要求：`帮我建PDD的公司画像`。
  - `2026-05-12T23:53:47.706234+08:00` assistant final 返回“PDD 公司画像已建好”，但同时把 `[profile.md](/Users/fengming2/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__.../company_profiles/pdd/profile.md)` 这类本机 Markdown 文件链接直接放进用户可见文本。
  - 路径中包含本机仓库根目录、`data/agent-sandboxes/feishu` 存储布局、direct actor sandbox 标识与公司画像内部目录结构。
- 结论：
  - 这是同一根因 / 同一影响范围的复发，不新建重复文档。
  - 主功能“画像已创建”看起来完成，但回复继续暴露本机路径和内部文件落点；因此仍按质量与信息边界缺陷定级为 `P3`。
  - 为何不定为 `P1/P2`：本轮证据没有显示画像创建失败、跨用户错投、消息投递失败或数据损坏；受损的是外部渠道输出边界和可读性，因此不影响主功能链路完成度。

## 2026-05-14 修复

- 本轮确认 Feishu final 出站实际依赖共享 `sanitize_user_visible_output(...)`，而该层此前只清理内部协议 / reasoning，不会直接脱敏本地 Markdown 文件链接或裸绝对路径；因此 2026-04-26 的修复意图没有覆盖复发样本。
- 已在 `crates/hone-channels/src/runtime.rs` 将本地路径脱敏并入共享可见文本净化：
  - `[profile.md](/Users/.../company_profiles/pdd/profile.md)` 与 `file:///Users/...` 形式的本地 Markdown 链接会保留无路径标签，如 `profile.md`；
  - 裸 `/Users/...` 与 `C:\Users\...` 绝对路径会统一改写为 `<absolute-path>/<basename>`；
  - `data/agent-sandboxes/...`、direct actor sandbox 标识、本机仓库根目录不再进入 Feishu / Telegram / scheduler 等共享出站文本。
- 状态从 `New` 更新为 `Fixed`。本轮未依赖当前机器 live 运行态复核；后续若部署后仍看到本地路径或 sandbox 标识进入外部渠道回复，应在本单追加复发证据。

## 2026-05-14 验证

- `cargo test -p hone-channels sanitize_user_visible_output_redacts --lib -- --nocapture`
- `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/runtime.rs`

## 端到端链路

1. Feishu 直聊用户要求“帮我建 ccld 公司画像”。
2. 系统完成画像建档，并在最终回复里同时汇报“已创建两份文件”。
3. 汇报内容直接包含本机绝对文件路径和 Markdown 文件链接，路径中暴露了本地仓库根目录、渠道 sandbox 结构和用户 open_id 作用域。
4. 用户虽然知道“画像已建好”，但拿到的是只对本机调试者有意义的内部路径，而不是面向 Feishu 用户的业务结果摘要。

## 2026-04-26 修复

- 在 `crates/hone-channels/src/runtime.rs` 的共享 `sanitize_user_visible_output(...)` 中加入本地绝对路径与本地 Markdown 文件链接脱敏：
  - `[...]( /Users/... )` 这类本地文件链接会改写成不含绝对路径的纯文本标签；
  - 裸 `/Users/...` / `C:\...` 路径会统一收口成 `<absolute-path>/<basename>`，不再泄露仓库根目录、sandbox 结构或 open_id 作用域。
- 该规则位于共享出站净化层，因此会覆盖公司画像建档回复以及其它外部渠道上的同类本地文件路径外泄。
- 新增 `crates/hone-channels/src/runtime.rs` 回归测试，覆盖本地 Markdown 文件链接与绝对路径掩码行为。

## 2026-04-26 验证

- `cargo test -p hone-channels sanitize_user_visible_output_ -- --nocapture`

## 期望效果

- 对外回复应只说明“画像已创建/已初始化事件”，并给出用户可理解的后续动作，例如继续分析、补交易计划、财报后更新。
- 本机绝对路径、sandbox 目录结构、open_id 作用域、Markdown 本地文件链接不应直接暴露给 Feishu 用户。
- 如果需要保留文件定位能力，也应只在桌面/app 内部使用，不应进入渠道用户可见文本。

## 当前实现效果

- 2026-05-14 修复后，共享出站净化层会剥离本地 Markdown 文件链接中的绝对路径，并掩码裸本机绝对路径。
- 画像建档主功能仍按原流程完成；对外回复只保留用户可理解的文件标签或 `<absolute-path>/<basename>` 占位，不再暴露本地工程目录、运行时存储布局或 direct actor sandbox 标识。
- 该问题属于输出质量与信息边界问题，不是“任务没做完”；因此按独立质量缺陷跟踪，当前状态为 `Fixed`。

## 用户影响

- 用户会看到对自己没有操作意义的本机绝对路径，回复可读性下降，也暴露了不该面向终端用户公开的实现细节。
- 当前证据里，画像文件实际已创建，后续继续分析的主功能链路仍可继续使用，没有出现错误投递、任务失败或数据损坏。
- 因此该问题不影响主功能完成度，按 `P3` 定级，而不是 `P1/P2` 的功能性故障。

## 根因判断

- 公司画像建档链路很可能直接复用了桌面/app 场景下的 Markdown 文件链接输出模板，没有区分“本机可点击路径”与“渠道用户可见文本”。
- 修复前，出站净化主要关注 `<think>`、原始报错、工具协议等污染文本，但没有把本地绝对路径和 markdown file-link 视为需要剥离的内部实现细节。
- 这与已有“空回复 fallback”或“过早收口”不是同一根因；本轮是建档成功后的结果组织与出站边界问题。

## 下一步建议

- 排查公司画像创建成功后的用户态总结模板，确认是否直接复用了桌面 markdown link 生成逻辑。
- 为 Feishu / Telegram / Discord 等外部渠道增加本地路径净化规则：命中 `/Users/...`、`data/agent-sandboxes/...`、Markdown 本地文件链接时改写为用户可理解的纯文本摘要。
- 在缺陷修复前，后续巡检继续关注“建画像/写报告/生成文件”类真实会话，确认是否还有同类本地路径外泄样本。
