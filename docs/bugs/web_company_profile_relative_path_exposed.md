# Bug: Web / Feishu 直聊公司画像沉淀后向用户暴露内部相对文件路径

- **发现时间**: 2026-06-02 11:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 时间窗：2026-06-02 22:59-23:01 CST
  - session_id: `Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c`
  - 用户输入摘要：`HPE现在可以建仓吗`
  - Feishu direct 最终 assistant final 已完成 HPE 建仓判断、估值区间、证伪条件与来源，并正常写入会话。
  - 最终用户可见正文末尾包含内部相对路径：`company_profiles/hpe/profile.md` 与 `company_profiles/hpe/events/2026-06-02-build-position-check.md`。
- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-02 10:58-11:00 CST
  - session_id: `Actor_web__direct__web-user-14f4cadb069f`
  - 用户输入摘要：`avgo财报如何看`
  - ACP 事件显示该轮有 `session/prompt`、公司画像文件写入 tool update，以及最终 `response stopReason=end_turn`，说明 Web direct 回复链路已收口。
  - 最终用户可见正文末尾包含：`我已把 AVGO 财报前框架沉淀到 company_profiles/AVGO.md，后续财报出来可以直接对照更新。`

## 端到端链路

1. Web / Feishu direct 用户询问个股财报、估值或建仓判断。
2. runner 校验财报、行情、新闻和估值数据，并写入 actor sandbox 下的公司画像或事件文件。
3. 最终回复完成业务分析并正常收口。
4. 回复末尾把内部长期画像相对文件路径直接展示给用户。

## 期望效果

- 对外回复可以说明“已为后续跟踪沉淀本轮公司画像 / 事件框架”。
- 不应把 `company_profiles/<ticker>.md` 这类内部文件组织路径作为用户可见结论的一部分。
- 若产品要暴露画像入口，应使用前端可点击的业务入口、附件或自然语言说明，而不是 runner sandbox 的内部目录名。

## 当前实现效果

- 主分析内容完整，用户可以基于正文理解 AVGO 财报前看点。
- 但最终回复把 `company_profiles/AVGO.md` 作为沉淀位置告诉用户；该相对路径不是 Web 用户可直接使用的稳定产品入口。
- 23:01 CST Feishu direct HPE 建仓回复也把 `company_profiles/hpe/profile.md` 与 `company_profiles/hpe/events/2026-06-02-build-position-check.md` 发给用户，说明问题不局限于 Web direct。
- 本轮没有看到 `/Users/...`、`data/agent-sandboxes/...`、`/var/folders/...` 等绝对路径进入最终正文；绝对路径只出现在 ACP tool update 诊断事件中。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 它暴露了公司画像的内部文件组织方式，降低回复的产品感，也可能让用户误以为自己能直接访问该路径。
- 本轮 AVGO / HPE 分析已完成、文件写入也成功、最终回复正常收口，没有未回复、错投、数据损坏或投递失败证据。
- 因此它不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 初步判断是公司画像沉淀流程把 runner 原生文件路径作为“沉淀完成”的证明写入最终用户回复。
- 既有 `feishu_company_profile_absolute_path_leak.md` 修复覆盖的是绝对路径、本地 Markdown 链接和 sandbox 标识脱敏；本轮新增证据是 Web direct 最终正文里的内部相对路径，属于相邻但独立的用户态文案边界。
- 该问题也不同于 `web_direct_tool_call_raw_output_leak`：本轮最终正文没有 raw JSON、工具协议或 provider 报错外泄。

## 下一步建议

- 在公司画像 / 长期跟踪最终回复模板或共享出站净化层中，将 `company_profiles/<ticker>.md`、`events/*.md` 等内部相对路径改写为自然语言。
- 对 Web / Feishu direct 增加一条回归：当 runner 成功写入公司画像文件时，最终用户可见文本只说明已沉淀，不包含内部文件路径。
- 后续巡检继续区分两类证据：绝对路径 / sandbox 标识泄漏应回看既有路径脱敏缺陷；仅相对内部路径进入自然语言回复时按本单跟踪。

## 修复记录

- 2026-06-02 23:06 CST 复核：本轮在 Feishu direct HPE 建仓回复中观察到同类相对路径外泄，但当前远端 main 已在 12:15 CST 合入共享净化修复并有回归；该样本按 live 未确认部署 / 旧运行态证据保留，不把状态从 `Fixed` 回退。
- **修复时间**: 2026-06-02 12:15 CST
- **修复状态**: Fixed
- **修复摘要**:
  - 共享 `sanitize_user_visible_output(...)` 的路径脱敏层新增 `company_profiles/...` 与 `events/*.md` 内部相对路径改写。
  - 最终用户可见文本会把这类 runner sandbox 文件组织路径替换为自然的“公司画像”表述，保留“已沉淀 / 后续可对照更新”的业务语义。
  - 新增 `sanitize_user_visible_output_redacts_internal_relative_company_profile_paths` 回归，覆盖 `company_profiles/AVGO.md` 进入 Web direct final 的复发形态。
- **验证**:
  - `cargo test -p hone-channels sanitize_user_visible_output_redacts_internal_relative_company_profile_paths --lib -- --nocapture`
  - `cargo test -p hone-channels sanitize_user_visible_output_redacts_bare_absolute_paths --lib -- --nocapture`
- **文档同步**:
  - 已同步 `docs/bugs/README.md` 活跃计数、状态和已修复表。
  - 本修复不改变模块边界、入口、长期约束或运行工作流，不需要更新 `docs/repo-map.md`、`docs/current-plan.md` 或新增 handoff。
