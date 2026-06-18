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

## 修复记录

- 2026-06-18 15:03 CST 运行态复发，状态从代码级 `Fixed` 回退为 `New`：
  - 11:03-15:03 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages` 与 `cron_job_runs` 在本窗均为 0；本轮证据来自 `data/runtime/logs/acp-events.log` 重构出的用户可见 final。
  - 15:00 CST Feishu direct session `Actor_feishu__direct__ou_5f8d3431a2b9ca4af0044ff8970fa36a52` 对三只 A 股“现在入手”问题完成行情、财务、排序、动作和证伪条件分析，并以 `stopReason=end_turn` 收口。
  - 但 final 前段继续外露内部流程和本地上下文动作，包括 `本地只有画像目录`、`已加载 stock_research 技能`、以及“正式答复前把长期判断沉淀到公司画像”等表达。
  - 该样本晚于 2026-06-18 03:04 CST 共享净化层修复记录；业务回答主体可用、投递 / 收口正常、没有空回复、错投或链路级数据破坏证据，因此仍为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-18 03:04 CST 代码级修复：
  - 共享 `sanitize_user_visible_output(...)` 新增净化本机命令切换、内部研究流程和画像沉淀前言，覆盖 `本机没有 python 命令，我改用 python3`、`已加载股票研究流程`、`Hone 的实时检索工具`、`把数据补进...画像` 等真实复发表达。
  - 新增回归 `sanitize_user_visible_output_strips_internal_runtime_progress_copy`，锁住“去内部执行过程、保留业务结论”的行为。
  - 验证通过：`cargo test -p hone-channels sanitize_user_visible_output_strips_internal_runtime_progress_copy --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests`。
  - 本轮未重启 live 服务，也不把当前机器运行态当作恢复证据；状态更新为代码级 `Fixed`，后续若部署后仍有新的自然语言内部流程前言进入 final，再基于新样本重新打开。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-18 11:03-15:03 CST。
  - session_id: `Actor_feishu__direct__ou_5f8d3431a2b9ca4af0044ff8970fa36a52`。
  - ACP 事件在 2026-06-18T07:04:27Z 前后以 `stopReason=end_turn` 收口；同窗未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误进入用户可见 final。
  - 用户请求是比较生益电子、沪电股份、兆易创新当前是否适合入手；assistant final 完成三家公司行情、估值、财务、排序、风险和证伪条件。
  - 但 final 前段写出“本地只有画像目录，暂时没看到这三只的现成画像文件”“已加载 stock_research 技能”“正式答复前我会把这轮可复用的长期判断沉淀到公司画像”等内部状态 / 执行过程。
  - 该回复没有本机绝对路径、token、原始工具 JSON、思维痕迹或 provider 报错外露；问题仍限定在用户可见文案边界。
- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-17 23:01-2026-06-18 03:01 CST。
  - 本窗从 ACP 流式日志重构出 24 条用户可见 assistant final，全部以 `stopReason=end_turn` 收口；未见未回复、response error、stream disconnect、quota、panic 或 provider 原始错误。
  - 其中多条 Feishu direct final 继续把执行进度和内部沉淀动作写进用户可见正文：
    - `2026-06-17 23:59 CST` session_id `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`，VST 问答开头连续写出“我先核一下 VST 当前股价口径、业务/财务和最新利率背景”“接下来我再看一下你本地有没有 VST 画像”等执行过程。
    - `2026-06-18 00:11 CST` session_id `Actor_feishu__direct__ou_5f9f2cd3505aab8fed0a6ffd582df285b1`，FCEL 问答写出“我会把这次 FCEL 的长期跟踪重点沉淀成公司画像”。
    - `2026-06-18 00:24 CST` 同 session 的 FCEL / FLNC 对比写出“我会新增 FLNC 的长期画像和本轮 FCEL/FLNC 对比事件”。
    - `2026-06-18 00:46 CST` session_id `Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768`，EOSE 深度研究写出“这个问题属于单股深度研究，也会检查本地是否已有公司画像”“我会把本轮形成的长期投资主线、估值纪律和证伪条件沉淀成 EOSE 画像”。
  - 上述样本的业务回答主体仍完整，且无本机绝对路径、token、原始工具 JSON 或 provider 报错外露；问题继续限定在用户可见文案边界。
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
- 2026-06-18 03:02 CST 复核显示，外露形态从“本机命令 / 工具名”扩展到更自然语言化的执行进度和画像沉淀动作，例如先核验、检查本地画像、新增画像、沉淀长期跟踪框架等。
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
- 同步覆盖自然语言化的执行进度句式，例如“我先核验 / 我会检查本地画像 / 我会新增画像 / 沉淀成公司画像 / 记录到长期跟踪框架”，最终回复应直接呈现已完成后的业务结论。
- 对 Feishu direct 投研问答增加回归：当 runner 需要切换命令或内部工具时，最终回复只保留数据来源和口径边界，不出现命令、工具名或画像写入过程。
- 后续巡检若只在 tool update / rawOutput 中看到这些词，但 final 没有外露，不应补充为本缺陷复发。
