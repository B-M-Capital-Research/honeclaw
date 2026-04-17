# Bug: Feishu 直聊自动 compact 后仍无法完成新话题回答，旧会话会反复卡在“仍无法继续”

- **发现时间**: 2026-04-18 00:20 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fba037d8699a7194dfe01a1fda5ced052`
    - `2026-04-17T19:22:29.098516+08:00` 用户提问：`请预测联合健康财报会怎样？`
    - `2026-04-17T19:23:32.488338+08:00` assistant 返回：`当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。`
    - `2026-04-17T22:13:12.458203+08:00` 用户再次追问：`请预测联合健康这一季的财报会怎样？`
    - `2026-04-17T22:14:34.973152+08:00` assistant 再次返回同样的“仍无法继续”文案
    - 最近一小时最新样本：`2026-04-17T23:54:40.706923+08:00` 用户明确切换新话题：`开启新的话题：请预测联合健康的财报`
    - `2026-04-17T23:55:10.242164+08:00` / `23:55:10.242188+08:00` 会话写入 `Conversation compacted` 与 `【Compact Summary】...`，且 compact summary 已正确以 `role=system` 落库
    - `2026-04-17T23:55:32.986749+08:00` assistant 仍第三次返回同一条“当前会话上下文过长...仍无法继续”文案，用户始终没有拿到 `UNH` 财报预测结果
  - `data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-17T15:54:44.989342Z` 同轮搜索阶段先执行 `local_search_files query="UnitedHealth UNH" path="company_profiles"`，立即报 `文件不存在: company_profiles`
    - `2026-04-17T15:54:59.747075Z` 记录 `context overflow detected, compacting and retrying`
    - `2026-04-17T15:55:10.246979Z` 记录 `context overflow recovery compacted=true`，本轮已完成自动 compact
    - `2026-04-17T15:55:32.938584Z` compact 后重试的 search 仍落成 `stage=search.done success=false iterations=2 tool_calls=3`
    - `2026-04-17T15:55:32.975618Z` 最终整轮以产品化失败文案收口，而不是输出用户请求的 `UNH` 财报预测
  - 已修复旧缺陷对照：
    - `docs/bugs/context_overflow_recovery_gap.md` 已说明“如果后续出现 compact 成功率不足，应单独登记新缺陷”

## 端到端链路

1. 用户在同一条 Feishu 直聊里连续追问联合健康（`UNH`）财报预测，并在最新一次显式说明“开启新的话题”。
2. 搜索阶段先尝试读取画像与行情信息，其中画像读取仍命中 `company_profiles` 路径错误。
3. runner 检测到上下文溢出后触发 `context_overflow_recovery`，确实执行了一次自动 compact 和重试。
4. 但 compact 后的重试仍没有完成 search/answer，最终只向用户返回“当前会话上下文过长...仍无法继续”的统一失败提示。
5. 用户侧连续三次得到相同 fallback，却始终拿不到当前问题答案，只能被迫切到新会话。

## 期望效果

- 当会话进入新的独立话题时，自动 compact 应足以把旧上下文压缩到可继续回答的范围，而不是持续卡在 fallback 文案。
- 即使第一次自动 compact 后仍不足，也应尽量避免让同一 session 长期进入“每次都失败”的粘滞状态。
- 用户明确说明“开启新的话题”后，系统应更积极地收缩旧上下文，优先完成当前问题，而不是反复要求用户再开新会话。

## 当前实现效果

- 旧的“底层报错外泄”问题已经修复，最新会话里用户看到的是产品化提示，而不是 provider 原始错误。
- 但这轮真实样本证明，自动 compact 只是把失败文案变得可接受，并没有恢复主功能链路。
- 同一个 `UNH` 话题在 `19:22`、`22:13`、`23:54` 三次尝试中都没有产出答案，说明这不是单次 provider 抖动，而是会话一旦进入某种高负载状态后，会持续卡在“compact 后仍无法继续”的粘滞失败。
- 最新一轮 `23:55` 的 compact summary 已经是 `role=system`，表明这也不是旧的 compact summary 污染回灌问题原样回归。

## 用户影响

- 这是功能性缺陷，不是单纯文案或质量波动。用户明确提出了 `UNH` 财报预测请求，但连续三次都没有得到答案。
- 用户虽然收到了友好提示，不再暴露内部错误细节，但主任务仍未完成，只能被迫开启新会话或手动压缩上下文。
- 之所以定级为 `P2` 而不是 `P1`，是因为当前证据集中在单会话粘滞失败，仍有“开新会话”这一绕行路径，也没有发现误投递、数据损坏或跨用户影响。

## 根因判断

- 旧缺陷修复后，`AgentSession` 已能识别超窗并自动 compact；当前问题更像是 compact 粒度和保留窗口仍不足以让新话题顺利脱离旧上下文负担。
- 最新日志里 compact 后仍保留 6 条 recent items，且重试 search 继续携带画像读取与多次工具调用，说明 prompt 体积或上下文噪声在 retry 后仍可能超出可用预算。
- `company_profiles` 路径错误同时出现在这轮重试前，说明无效工具尝试也在放大 search 阶段的上下文和耗时，但它更像放大器，不足以单独解释“连续三次都无法完成回答”。
- 因此当前更可能是“会话瘦身策略不足 + 多代理搜索重试后上下文再膨胀”的组合问题，而不是单一 provider 临时抖动。

## 下一步建议

- 优先审视 `context_overflow_recovery` 在 direct session 中的保留窗口、summary 长度与重试策略，确认“开启新话题”时是否应更激进地丢弃旧活跃窗口。
- 为“compact 成功但 retry 仍失败”的路径补独立可观测标记，区分是真正再次超窗、search 早停，还是 answer 阶段被短路。
- 给直聊场景补一条回归验证：同一 session 在长历史后切到新话题时，自动 compact 后仍应能完成至少一条新问题答复，而不是长期卡在统一 fallback。
