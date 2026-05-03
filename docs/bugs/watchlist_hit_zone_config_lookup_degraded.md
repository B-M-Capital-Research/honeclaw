# Bug: 核心观察股池晚间快报退化为“击球区待确认”，固定观察池区间未被正确带入

- **发现时间**: 2026-04-29 23:06 CST
- **Bug Type**: System Error
- **严重等级**: P3
- **状态**: New
- **修复结论复核**:
  - `2026-05-03 21:37 CST` 同链路缺陷在最近一小时真实窗口再次复现：
    - `data/sessions/Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773.json`
      - `2026-05-03T21:35:02.888866+08:00` 最新 `[定时任务触发] 任务名称：科技核心股池 · 晚间击球区快报` 再次明确要求 25 支观察池“列出每个标的的当前价格、击球区区间值、下一次财报时间”，且仅额外给出 `LITE` 的击球区配置
      - `2026-05-03T21:37:11.184468+08:00` assistant final 仍把 `MSFT / NVDA / GOOGL / AAPL / AVGO / AMZN / META` 及其余 17 支拓展股统一写成 `击球区：待确认`，只保留 `LITE` 的固定区间，并在末尾再次声明 `除 LITE 外，其余 24 支击球区区间未完成校验`
    - `data/runtime/prompt-audit/feishu/latest-Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773.json`
      - `runtime_input` 仍原样要求“每次简要列出每个标的的当前价格、击球区区间值、下一次财报时间”，说明任务模板本身没有降级要求
      - 同一 prompt 仅把 `LITE` 的击球区显式写入本轮输入，最终答案继续把其余 24 支统一降成 `待确认`
    - 结论：
      - 到 `2026-05-03 21:37` 为止，这条缺陷在最新晚间窗口仍未恢复；当前坏态继续表现为“任务成功送达，但固定击球区静默降级为待确认”，且最新证据直接来自用户可见最终答复
  - `2026-05-03 09:01 CST` 同链路缺陷在最近一小时真实窗口再次复现：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=14430`
      - `job_name=核心观察池早间简报`
      - `executed_at=2026-05-03T09:01:07.463168+08:00`
      - `execution_status=completed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview` 开头继续写出：`本次已用 data_fetch 校验 25 支观察池最新美股价格与下一次财报时间`，随后再次明确 `除 LITE 外，其余 24 支击球区区间未完成校验，不能计算真实击球区距离`，并把 `MSFT / NVDA / GOOGL / AAPL / AVGO / AMZN / META` 等核心股继续统一标成 `击球区：待确认`
    - 结论：
      - 到 `2026-05-03 09:01` 为止，这条缺陷已从前一晚的晚间快报延续到次日盘前；当前坏态继续表现为“任务成功送达，但固定击球区静默降级为待确认”。
  - `2026-05-02 23:01 CST` 同链路缺陷在最近一小时真实窗口再次复现：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=13974`
      - `job_name=核心观察股池晚间快报`
      - `executed_at=2026-05-02T23:01:05.345132+08:00`
      - `execution_status=completed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview` 开头继续写出：`以下为 data_fetch 返回的最新美股市场口径，价格与财报时间已校验`，随后将 `MSFT / NVDA / GOOGL` 等核心股继续统一标成 `击球区：待确认`
    - `data/runtime/logs/sidecar.log`
      - `2026-05-02 23:00:17-23:01:05` 同窗继续高频执行 `Tool: hone/data_fetch`
      - 终态仍成功送达，没有新的 `tool_failed` 或本地文件检索失败
    - 结论：
      - 到 `2026-05-02 23:01` 为止，这条缺陷在最新晚间窗口仍未恢复；当前坏态继续表现为“任务成功送达，但固定击球区静默降级为待确认”。
  - `2026-05-02 21:36 CST` 同链路缺陷在最近一小时真实窗口再次复现：
    - `data/sessions/Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773.json`
      - `updated_at=2026-05-02T21:36:27.864248+08:00`
      - 最新 `[定时任务触发] 任务名称：科技核心股池 · 晚间击球区快报` 继续要求“每个标的列出当前价格、击球区区间值、下一次财报时间”
      - 实际 assistant final 再次写出 `除 LITE 外，其余 24 支击球区区间未完成校验`，并把核心股与拓展股中的其余标的统一降成 `击球区：待确认`
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=13907`
      - `job_name=科技核心股池 · 晚间击球区快报`
      - `executed_at=2026-05-02T21:36:30.424136+08:00`
      - `execution_status=completed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview` 开头继续写出：`以下为 data_fetch 返回的最新美股市场口径，价格与财报时间已校验`，随后将除 `LITE` 外的 24 支标的统一标成 `击球区：待确认`
    - `data/runtime/logs/sidecar.log`
      - `2026-05-02 21:35:52-21:36:27` 同会话继续高频执行 `Tool: hone/data_fetch`
      - `2026-05-02 21:36:27.865-21:36:27.866` 整轮仍以 `success=true elapsed_ms=85373 tools=25(Tool: hone/data_fetch) reply.chars=1441` 收口
      - 同窗没有新的 `tool_failed` 或本地文件检索失败，说明当前坏态继续表现为“区间配置没有进入最终答案”，而不是主链路执行失败
  - `2026-05-02 09:02 CST` 同链路缺陷在最近一小时真实窗口再次复现：
    - `data/sessions/Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773.json`
      - `updated_at=2026-05-02T09:01:29.928698+08:00`
      - 最新 `[定时任务触发] 任务名称：核心观察池早间简报` 继续明确要求四段结构，并要求“列出每个标的的当前价格、击球区区间值、下一次财报时间”
      - 实际 assistant final 再次写出 `除 LITE 外，其余 24 支标的击球区区间未在本轮数据链路中完成校验，全部标注为“待确认”`
      - 同条回复的“击球区距离表”也只计算 `LITE`，其余 `MSFT / NVDA / GOOGL / AAPL / AVGO / AMZN / META / BABA / AAOI / MU / SNDK / STX / WDC / COHR / GEV / TSLA / ORCL / TSM / GLW / CRDO / RKLB / INTC / BE / AMD` 统一降成 `击球区数据待确认，本轮不计算距离`
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=13349`
      - `job_name=核心观察池早间简报`
      - `executed_at=2026-05-02T09:01:32.694401+08:00`
      - `execution_status=completed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview` 开头继续写出：`但除 LITE 外，其余 24 支击球区区间未在本轮数据链路中完成校验，全部标注为“待确认”`
    - `data/runtime/logs/sidecar.log`
      - `2026-05-02 09:00:02-09:01:29` 同会话继续跑完 `tools=25(Tool: hone/data_fetch)`，并以 `done success=true elapsed_ms=87651 reply.chars=2334` 收口
      - 同窗没有新的 `tool_failed`，说明当前坏态依旧是“区间配置/记忆没有进入最终答案”，而不是主链路执行失败
  - `2026-05-01 23:01 CST` 同链路缺陷在最近一小时真实窗口再次复现：
    - `data/sessions/Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773.json`
      - `updated_at=2026-05-01T23:01:14.020643+08:00`
      - 最新 `[定时任务触发] 任务名称：核心观察股池晚间快报` 仍要求“每个标的列出当前价格、击球区区间值、下一次财报时间”
      - 实际 assistant final 再次写出 `除 LITE 外，其余击球区区间仍未完成备案，统一标注待确认`，并把 25 支观察池中的 24 支继续降成 `击球区待确认`
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=12887`
      - `job_name=核心观察股池晚间快报`
      - `executed_at=2026-05-01T23:01:16+08:00`
      - `execution_status=completed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview` 继续原样写出：`除 LITE 外，其余击球区区间仍未完成备案，统一标注待确认`
    - `data/runtime/logs/sidecar.log`
      - `2026-05-01 23:00:31-23:01:14` 同会话再次连续执行 20+ 次 `Tool: hone/data_fetch`，并夹带 `portfolio`、`web_search`
      - `2026-05-01 23:01:14.021` 整轮仍以 `success=true elapsed_ms=71991 tools=26(Tool: hone/data_fetch,Tool: hone/skill_tool) reply.chars=1368` 收口
      - 最新窗口同样没有出现 `local_search_files` / `local_list_files` / `tool_failed`，说明坏态继续指向“击球区配置/记忆注入缺失”，而不是旧的本地文件搜索失败
  - `2026-05-01 21:36 CST` 同链路缺陷在最近一小时真实窗口再次复现，先前 `Fixed` 结论失效：
    - `data/sessions/Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773.json`
      - `updated_at=2026-05-01T21:36:10.365924+08:00`
      - 最新 `[定时任务触发] 任务名称：科技核心股池 · 晚间击球区快报` 仍要求“每个标的列出当前价格、击球区区间值、下一次财报时间”
      - 实际 assistant final 继续写出 `除 LITE 外，其余击球区区间当前仍未完成备案，统一标注待确认`，并把 25 支观察池中的 24 支继续降成 `击球区待确认`
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=12822`
      - `job_name=科技核心股池 · 晚间击球区快报`
      - `executed_at=2026-05-01T21:36:15+08:00`
      - `execution_status=completed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview` 继续原样写出：`除 LITE 外，其余击球区区间当前仍未完成备案，统一标注待确认`
    - `data/runtime/logs/sidecar.log`
      - `2026-05-01 21:35:30-21:35:45` 同会话连续执行 20+ 次 `Tool: hone/data_fetch`
      - `2026-05-01 21:36:10.366` 整轮仍以 `success=true elapsed_ms=68369 tools=26(Tool: hone/data_fetch,Tool: hone/skill_tool) reply.chars=1371` 收口
      - 最新窗口里没有再出现 `local_search_files` / `local_list_files` / `tool_failed`，说明当前坏态已经不再等同于 `2026-04-29` 记录里的“本地文件搜索被单个坏文件打断”
- `2026-05-01 23:01 CST` 的 `核心观察股池晚间快报` 与 `2026-05-01 21:36 CST` 的 `科技核心股池 · 晚间击球区快报` 说明，这条缺陷已同时影响两套观察池日报模板，而不再只限于单个晚间快报名字。
- `2026-05-02 09:02 CST` 的 `核心观察池早间简报` 说明，这条缺陷仍持续影响同一观察池的早间链路，而且已从前一晚延续到次日盘前，不是只在单个晚间模板里偶发。
- `2026-05-02 21:36 CST` 的 `科技核心股池 · 晚间击球区快报` 说明，这条缺陷在同一天晚间窗口仍未恢复，且当前坏态即便不再出现显式本地检索报错，也会把绝大多数固定击球区静默降级为 `待确认`。
- `2026-04-30 21:36 CST` 同症状其实已在前一日晚间快报继续活跃：
    - `cron_job_runs.run_id=11653`
    - `job_name=科技核心股池 · 晚间击球区快报`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `response_preview` 同样写出 `除 LITE 外，其余击球区未完成校验，统一标注待确认`
    - 说明 `2026-05-01` 的复现不是偶发回潮，而是至少连续两晚的稳定回退
- **证据来源**:
  - `data/sessions/Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773.json`
    - `updated_at=2026-05-01T09:01:25.107409+08:00`
    - 最近一小时真实会话里，同链路的 `核心观察池早间简报` 再次在正文开头明确写出：`除 LITE 外，其余击球区未在当前资料中完成备案，统一标注“待确认”`
    - 同条回复把 `MSFT / NVDA / GOOGL / AAPL / AVGO / AMZN / META` 以及拓展池大部分标的的击球区继续统一降成 `待确认`，只保留 `LITE` 的固定区间；末尾还写出 `除 LITE 外，其余 24 支击球区区间未在当前资料中校验到`
    - 这说明问题没有停留在 `2026-04-29 23:00` 的晚间快报单点，而是到 `2026-05-01 09:01` 已扩散到同一观察池的早间简报任务
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=10496`
    - `job_name=核心观察股池晚间快报`
    - `executed_at=2026-04-29T23:01:20+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 开头直接写出：`未找到本地完整击球区配置，除 LITE 外其余击球区标注为待确认`
    - 同条消息把 `MSFT / NVDA / GOOGL / AAPL / AVGO / AMZN / META` 以及拓展池标的的击球区统一降成 `待确认`
  - 对照同一任务上一日真实窗口：
    - `run_id=9269`
    - `job_name=核心观察股池晚间快报`
    - `executed_at=2026-04-28T23:00:00+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `response_preview` 仍能稳定给出固定击球区，例如 `MSFT $335-350`、`NVDA $150-165`、`GOOGL $255-275`、`AAPL $205-225`
    - 这说明问题不是任务定义本身没有击球区，而是 2026-04-29 23:00 这轮在检索/拼装阶段发生了新的退化
  - `data/runtime/logs/sidecar.log`
    - `2026-04-29 23:00:23.526` 同一会话开始调用 `Tool: hone/local_list_files`
    - `2026-04-29 23:00:23.529` 紧接着开始调用 `Tool: hone/local_search_files`
    - `2026-04-29 23:00:23.666` 同会话记录 `runner.stage=acp.tool_failed`
    - 随后 `23:00:44.492-23:00:44.493` 只看到两次 `local_read_file` 成功，`23:01:16.381` 整轮仍以 `success=true reply.chars=1485` 收口
    - 这说明链路在本地配置检索阶段发生了工具退化，但最终仍把缺失配置后的降级正文作为成功结果送达

## 端到端链路

1. Feishu scheduler 在 `2026-04-29 23:00` 触发 `核心观察股池晚间快报`。
2. 搜索/整理阶段先尝试通过本地文件工具恢复观察池击球区配置。
3. 本地检索阶段出现 `acp.tool_failed`，随后只读到了部分本地文件。
4. 最终回复仍以 `completed + sent` 送达，但正文把除 `LITE` 外几乎所有标的的击球区统一降成“待确认”。
5. 用户虽然收到了完整日报，但失去了这条任务最关键的固定参考字段之一。

## 期望效果

- `核心观察股池晚间快报` 应稳定带出观察池的固定击球区区间，而不是在已有历史配置的情况下把大部分标的退化成“待确认”。
- 若本地配置检索失败，链路应优先复用上一轮已知固定区间，或者显式标记本轮任务失败，而不是把缺字段正文当作成功播报送达。
- 对同一观察池任务，前后相邻窗口不应出现“昨天能给完整击球区，今天几乎全丢失”的无提示回退。

## 当前实现效果

- 当前任务主链路没有中断，`cron_job_runs` 与日志都显示这轮任务成功送达。
- `2026-05-03 09:01` 的 `核心观察池早间简报` 说明，问题到本轮巡检时仍持续活跃：任务继续成功送达，但核心股里的多支固定击球区仍被统一替换成 `待确认`，而且已经从前一晚延续到次日盘前。
- `2026-05-02 23:01` 的最新晚间窗口说明，问题到本轮巡检时仍持续活跃：任务继续成功送达，但核心股里的多支固定击球区仍被统一替换成 `待确认`。
- `2026-05-02 21:36` 的最新晚间窗口说明，问题到当晚仍持续活跃：任务继续成功送达，但绝大多数标的的固定击球区仍被统一替换成 `待确认`。
- `2026-05-01 09:01` 的 `核心观察池早间简报` 说明，这条缺陷并非只影响晚间快报：同一观察池链路在最近一小时仍继续把除 `LITE` 外几乎所有标的的击球区统一降成“待确认”。
- `2026-04-30 21:35`、`2026-05-01 21:35` 与 `2026-05-01 23:00` 三个连续窗口说明，这条缺陷在此前标记 `Fixed` 后仍持续影响相同观察池链路，而且已经覆盖不同日报模板，而不是只剩历史残留。
- 但真正送达给用户的正文已经不再提供多数标的的固定击球区，只剩“待确认”占位。
- 从上一日同任务对照看，配置并非天然缺失，因此这不是用户要求变更，而是当前任务在本地配置检索或上下文拼装阶段发生了退化。
- 这是质量类缺陷。之所以定级为 `P3`，是因为任务仍成功生成并送达，价格与财报字段也仍可读；受损的是分析完整性和参考价值，而不是主功能链路可用性。

## 用户影响

- 用户无法再直接把当日价格与既定击球区做对照，晚间快报的核心决策价值明显下降。
- 同一任务前后两天输出能力不一致，会削弱用户对“长期观察池记忆”与固定模板稳定性的信任。
- 当击球区被统一降级成“待确认”时，用户若继续依赖这条快报做观察池排序，会被迫回到手工核对或重新追问。

## 根因判断

- `2026-04-29` 的首个坏样本里，本地击球区配置检索链路退化是明确放大器：日志同轮出现 `local_list_files` / `local_search_files` 后立即 `acp.tool_failed`，与正文“未找到本地完整击球区配置”的自述一致。
- `2026-05-02 21:36` 的最新样本再次证明，当前活跃坏态已经不依赖显式 `tool_failed` 才会复现：同窗只有高频 `data_fetch`，没有本地文件检索失败，但最终答案仍把 24 支标的统一降成 `待确认`。
- 但 `2026-04-30 21:35` 与 `2026-05-01 21:35` 的连续复现说明，当前活跃坏态已经不再依赖相同的工具失败形态：最新窗口只看到高频 `data_fetch` 与 `skill_tool`，没有再出现 `local_search_files` / `tool_failed`，最终仍把绝大多数击球区降成“待确认”。
- 因此当前更可能是“观察池固定击球区记忆/配置注入在模板或答案拼装阶段继续缺失”，而不只是本地目录检索被单个坏文件打断。
- 同一症状已经连续影响 `核心观察池早间简报` 与 `科技核心股池 · 晚间击球区快报`，说明问题不局限于单个 scheduler 模板，而是观察池区间恢复链路仍未稳定。

## 修复情况（2026-05-01，已不足以覆盖当前坏态）

- `crates/hone-tools/src/local_files.rs` 已加固 `local_search_files` 的目录递归搜索：遇到单个二进制、非 UTF-8 或不可读文件时跳过该文件并继续搜索其它文本文件，不再让整次本地配置检索直接失败。
- 搜索结果新增 `skipped_binary_files`、`skipped_non_utf8_files`、`skipped_unreadable_files` 计数，保留可观测性，便于后续判断是否仍有坏文件污染 actor sandbox。
- 单文件读取 / 单文件搜索仍保持严格错误边界，避免把非文本文件内容误当成有效配置。
- 回归验证：
  - `cargo test -p hone-tools directory_search_skips_non_text_files_without_aborting --lib -- --nocapture`
  - `cargo test -p hone-tools local_files --lib -- --nocapture`
  - `cargo check -p hone-tools --tests`
- 当前回看 `2026-04-30 21:35` 与 `2026-05-01 21:35` 两个真实晚间窗口，这组修复并没有恢复固定击球区输出；它至多解释了 `2026-04-29` 的单一放大器，但没有覆盖当前仍在生产中出现的退化形态。

## 下一步建议

- 优先检查 `科技核心股池 · 晚间击球区快报` 与 `核心观察池早间简报` 当前从何处读取固定击球区；重点确认最近两晚是否已经不再走 `local_search_files`，而是直接丢失了观察池区间注入步骤。
- 对 `2026-04-30 21:35` 与 `2026-05-01 21:35` 两个样本回放 prompt / tool transcript，确认 `skill_tool` 是否仍能读到观察池配置，还是 answer 阶段忽略了已存在的区间数据。
- 若 `skipped_non_utf8_files` 长期非零，清理或隔离 actor sandbox 内的坏编码文件，降低检索噪声。
