# Bug: Web 定时任务回复外露“技能未加载”内部降级措辞

- **发现时间**: 2026-06-02 07:04 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-02 06:30 CST
  - session_id: `Actor_web__direct__web-user-14f4cadb069f`
  - 触发输入摘要：Web 定时任务 `[定时任务触发] 任务名称：1亿美元AI科技组合每日跟踪`，要求按目标权重复盘 ORCL / DELL / CSCO / QCOM / IBM / AMD / MRVL / MU / ARM / CRCL。
  - ACP 事件显示该轮有 `session/prompt`、`session/update` 和最终 `response stopReason=end_turn`，说明回复链路已收口。
  - 最终回复开头包含用户可见句子：`定时任务技能在当前运行器里没有成功加载，我改用行情和新闻工具直接完成这次复盘`。

## 端到端链路

1. Web scheduler 触发组合每日跟踪任务。
2. runner 收到带权威触发配置的 prompt，并开始执行行情、新闻和计算。
3. 最终回复成功通过 ACP stream 输出，并以 `stopReason=end_turn` 收口。
4. 回复内容完成了组合市值、持仓贡献、权重漂移与新闻复盘，但同时把“定时任务技能没有成功加载”这类内部运行降级说明写给用户。

## 期望效果

- 定时任务最终回复应只呈现用户可理解的业务结果、必要的数据口径和风险提示。
- 若内部 skill / tool 选择发生降级，应由系统内部记录或转化为自然业务措辞，例如“本轮改用行情与新闻数据完成复盘”，不应暴露“当前运行器”“技能未加载”等实现细节。

## 当前实现效果

- 本轮最终回复虽然完成了主要业务内容，但开头直接说明“定时任务技能在当前运行器里没有成功加载”。
- 这会让用户看到内部运行器和技能加载状态，降低专业感，并可能误导用户认为定时任务系统本身异常。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 本轮 Web scheduler 已成功输出复盘，ACP `response` 以 `stopReason=end_turn` 收口，没有未回复、未投递、错投、空回复、格式断裂或数据链路中断证据。
- 因此它不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 直接证据只能证明最终回复把内部降级措辞透出给用户。
- 初步判断是 answer 阶段缺少对 skill/tool 降级语句的用户态改写，或 Web scheduler prompt 允许模型自行解释内部 tool/skill 可用性。
- 该问题不同于历史的 raw tool output / `<think>` / provider error 外泄：本轮没有原始 JSON、工具日志或 provider 报错外泄，而是自然语言层面的内部实现细节暴露。

## 下一步建议

- 在 Web scheduler / shared response finalizer 中增加用户态措辞 guard，过滤或改写“技能未加载”“当前运行器”“tool unavailable”等内部降级短语。
- 对 scheduler final 增加一条回归：当模型输出内部 skill 降级说明时，最终用户可见文本应保留业务复盘并移除实现细节。
- 后续巡检若只看到内部日志 `rawOutput`，但最终回复不含该类文本，不应把本缺陷扩大为用户可见外泄。

## 修复记录

- **修复时间**: 2026-06-02 12:06 CST
- **修复状态**: Fixed
- **修复摘要**:
  - 共享 `sanitize_user_visible_output(...)` 的内部前言识别新增 skill/tool 降级措辞，包括“技能未加载”“技能没有成功加载”“当前运行器”“tool unavailable”“skill unavailable”“failed to load skill”等。
  - scheduler delivery 出站净化继续调用共享用户态净化器；当最终回复开头暴露内部 skill/tool 降级说明且后续还有业务正文时，会剥离该前言并保留复盘内容。
  - 新增 `scheduler_delivery_text_strips_skill_load_degradation_prelude` 回归，覆盖本缺陷的“当前运行器 / 技能未加载”用户可见污染形态。
- **验证**:
  - `cargo test -p hone-channels scheduler_delivery_text_strips_skill_load_degradation_prelude --lib -- --nocapture`
  - `cargo test -p hone-channels sanitize_user_visible_output_strips_internal_workflow_prelude --lib -- --nocapture`
  - `cargo test -p hone-channels scheduler_delivery_text_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/runtime.rs crates/hone-channels/src/scheduler.rs`
- **文档同步**:
  - 已同步 `docs/bugs/README.md` 活跃计数、状态和已修复表。
  - 本修复不改变模块边界、入口、长期约束或运行工作流，不需要更新 `docs/repo-map.md`、`docs/current-plan.md` 或新增 handoff。
