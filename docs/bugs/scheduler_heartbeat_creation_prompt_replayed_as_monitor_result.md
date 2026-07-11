# Bug: Heartbeat 已创建监控任务仍反复输出“无法创建定时任务”

## 发现时间

- 2026-07-11 19:01 CST

## Bug Type

- Business Error

## 严重等级

- P2

## 状态

- New

## GitHub Issue

- 无，当前不是 P1。

## 证据来源

- `data/runtime/logs/web.log.2026-07-11`
  - 15:00-19:00 CST `美股黄金坑信号心跳检测` 每 30 分钟继续被 scheduler 触发，说明系统侧已经存在并运行该 heartbeat job。
  - 15:30 CST raw preview 仍把任务理解成“用户想让我每 30 分钟创建市场监控”，随后按 `JsonNoop` 跳过。
  - 16:30 CST 同 job 输出自然语言市场判断后落成 `JsonMalformed + execution_failed`，本轮不发送。
  - 18:00 CST 同 job deliver preview 给出市场判断，但 duplicate suppression 匹配到旧的“无法创建自动化心跳监控”文本，最终未发送。
  - 19:00 CST 同 job deliver preview 直接写出“这是你第三次提出建立每30分钟自动化心跳监控的请求，当前无法创建此类定时任务”，而不是执行已创建监控的市场条件判断。

## 端到端链路

1. 用户曾要求创建“美股黄金坑信号”类 30 分钟心跳监控。
2. 系统已经产生并周期触发 `美股黄金坑信号心跳检测` heartbeat job。
3. heartbeat runner 把 job prompt 送入 function-calling LLM。
4. LLM 多次把 prompt 当成“创建定时任务请求”而不是“执行已存在监控任务”。
5. 出站层在自由文本、malformed JSON、duplicate suppression 和 skipped noop 之间漂移，用户无法稳定收到该 job 的有效监控结果。

## 期望效果

- 已创建的 heartbeat job 每次触发时只执行监控判断。
- 如果当前条件未触发，应返回稳定结构化 `noop`，并且不要给用户发送“无法创建定时任务”。
- 如果条件触发，应发送与监控条件相关的提醒正文。
- job prompt 应保存为可执行监控说明，而不是保留用户最初的“帮我创建/设置”请求语义。

## 当前实现效果

- 同一个已存在的 heartbeat job 在真实运行中仍反复解释为“创建自动化监控请求”。
- 部分窗口输出“无法创建自动化心跳监控 / 当前无法创建此类定时任务”，与 job 已被周期触发这一事实矛盾。
- 该输出还会进入 duplicate suppression 基线，导致后续真实市场判断文本被旧“无法创建”基线压成未发送。

## 用户影响

- 用户以为已经创建的 30 分钟监控不会稳定提供监控结果。
- 该问题影响单个 heartbeat 任务的核心用途：周期检查市场回撤/买点条件。
- 这是功能性监控链路缺陷，定级 P2；当前证据集中在一个 job，未见全渠道停摆、错对象投递、数据安全泄露或 P1 级全局任务丢失，因此不升级为 P1。

## 根因判断

- 初步判断是 job 创建 / prompt 持久化边界没有把“创建请求”规范化为“执行请求”，导致 runner 后续周期执行时仍收到用户原始意图。
- duplicate suppression 只基于近似文本匹配，可能把“无法创建”这类错误基线当成同 job 的历史结果，进一步压制后续有效检查文本。
- 该根因不同于通用 heartbeat JSON 结构化退化：即使解析层完全稳定，job prompt 仍可能执行错误任务。

## 下一步建议

- 在 cron / heartbeat job 创建时，将用户“请创建/设置/每 30 分钟检查”的请求改写为稳定的执行型监控 prompt。
- 对已有 job 增加迁移或修复工具：识别 prompt 中的“无法创建 / 不能设置 / 你第三次提出建立”一类创建期残留，改写为实际监控条件。
- duplicate suppression 应避免把“无法创建定时任务”“请你手动发起检查”等非监控结果作为 heartbeat 去重基线。
- 修复后用 `美股黄金坑信号心跳检测` 构造回归，验证触发时不会再输出创建失败话术，未触发时返回结构化 noop。
