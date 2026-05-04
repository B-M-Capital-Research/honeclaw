# Bug: Feishu 晨报在 `data_fetch` 连续失败后仍以成功态发送旧价格早报

- **发现时间**: 2026-05-04 21:25 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New

## 证据来源

- `2026-05-04 08:32 CST` 最近一小时真实调度窗口确认本单活跃：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=15500`
    - `job_id=j_a1772833`
    - `job_name=Hone_AI_Morning_Briefing`
    - `actor_channel=feishu`
    - `actor_user_id=ou_3f69c84593eccd71142ed767a885f595`
    - `executed_at=2026-05-04T08:32:07.993971+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 明确写出：`本轮重新拉取持仓实时行情时，底层行情数据链路暂时阻断。以下价格使用本会话此前已核验的美股5月1日收盘口径`
  - `data/sessions/Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595.json`
    - `updated_at=2026-05-04T08:32:03.786307+08:00`
    - 最新 assistant final 与台账一致，正文同时包含：
      - `说明：本轮重新拉取持仓实时行情时，底层行情数据链路暂时阻断。`
      - `以下价格使用本会话此前已核验的美股5月1日收盘口径；新闻、评级与产业动态使用本轮搜索核验。`
    - 同一条消息仍继续给出 `BE / GOOGL / VST / GEV / MU / COHR / CIEN / SNDK / RKLB / TEM / AVGO` 的逐项判断，说明系统并未把这轮视为失败，而是把“旧价格降级版早报”作为正式结果送达
  - `data/runtime/logs/sidecar.log`
    - `2026-05-04 08:30:57.860`、`08:30:58.080`、`08:30:58.303`、`08:30:58.522` 同一会话连续记录 `runner.stage=acp.tool_failed`
    - 紧邻的 `08:30:57.861`、`08:30:58.083`、`08:30:58.306` 继续记录 `tool=Tool: hone/data_fetch status=start`
    - `08:31:07.079` 之后链路改为 `Searching the Web`
    - `08:32:03.786` 最终仍以 `done ... success=true elapsed_ms=120608 iterations=1 tools=7(Tool: hone/skill_tool,Tool: hone/web_search) reply.chars=3635` 收口
    - 这说明本轮 `data_fetch` 没有成功返回，但 scheduler 没有转成失败态，而是改用 `web_search` 拼出一条“旧价格 + 新闻”的混合摘要
  - `data/runtime/logs/acp-events.log`
    - `2026-05-04T00:30:55.147155+00:00` 收到 `session/request_permission`，请求 `data_fetch quote QQQ`
    - `2026-05-04T00:30:55.147304+00:00` 紧接着自动回写 `approved-for-session`
    - 说明这轮并不是卡死在审批未通过；权限已自动批准后，`data_fetch` 仍在执行阶段退化成 `acp.tool_failed`

- 历史对照说明这不是任务定义允许的固定降级口径：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=12210`
    - `executed_at=2026-05-01T08:31:43.348071+08:00`
    - 同一 `job_name=Hone_AI_Morning_Briefing` 当时可正常完成，不含“数据链路阻断”自述
  - `run_id=10989`
    - `executed_at=2026-04-30T08:31:55.828874+08:00`
    - 同一 job 已出现相近坏态：`本轮报价接口触及限额，以下持仓价格采用同一会话04:30已校验的美股4月29日收盘口径`
  - 结论：
    - 这条链路不是长期设计成“允许用旧价格成功送达”；它在正常窗口可给出完整早报，但在行情抓取失败时会间歇性退化成“旧价格口径 + 成功态送达”

## 端到端链路

1. Feishu scheduler 在 `2026-05-04 08:30` 触发 `Hone_AI_Morning_Briefing`，任务要求同时覆盖宏观、AI 前沿与持仓标的实时动态。
2. 链路先进入 `market_analysis` / `data_fetch` 路径，尝试拉取行情与持仓标的数据。
3. `data_fetch` 在执行阶段连续多次失败，日志反复出现 `acp.tool_failed`。
4. 系统随后改用 `web_search` 继续拼装内容，并在正文中自行声明“底层行情数据链路暂时阻断，以下价格使用本会话此前已核验的收盘口径”。
5. 最终这条降级早报仍被记为 `completed + sent + delivered=1` 并送达用户。

## 期望效果

- 这类要求“实时动态、研报观点及机构调价”的晨报，在关键 `data_fetch` 失败时不应继续以成功态送出混合旧价格结果。
- 若只能拿到新闻而拿不到本轮行情，应明确收口为失败或部分完成态，而不是把“沿用旧价格”的版本当作正常日报。
- 同一条早报里的价格、新闻与“当前时间”口径应保持一致，不能在北京时间 `2026-05-04 08:30` 的任务里静默回退到更早收盘数据，却仍展示为一次正常完成的晨报。

## 当前实现效果

- `Hone_AI_Morning_Briefing` 的主链路没有中断，用户确实收到了消息。
- 但最新 `2026-05-04 08:32` 样本里，任务要求中的“持仓标的实时动态”并未完成；系统自己承认本轮行情链路阻断，并回退到旧收盘口径。
- 日志进一步表明这不是回答时单纯保守措辞，而是 `data_fetch` 实际连续失败后，链路仍被记成 `success=true`。
- `2026-04-30 08:31` 的历史样本说明，这类“旧价格 fallback 但仍成功送达”的坏态已至少第二次复现，不是单窗偶发噪声。

## 用户影响

- 这是质量类缺陷。消息仍被正确送达，没有出现错投、无回复、系统崩溃或数据破坏。
- 但用户订阅的是“每日新闻早报 + 持仓实时动态”，实际收到的是“旧价格 fallback + 新搜索新闻”的混合结果，任务完成度明显下降。
- 之所以定级为 `P3`，是因为主功能链路仍可用，用户仍拿到一条可读早报；当前问题主要在于时效性、数据口径一致性和成功态判定失真，而不是发送链路中断。

## 根因判断

- 直接诱因是 `data_fetch` 在执行阶段连续失败，`sidecar.log` 已明确记录多次 `acp.tool_failed`。
- 这轮并非卡在 MCP 审批未响应：`acp-events.log` 显示 `data_fetch quote QQQ` 的权限请求已经自动 `approved-for-session`。
- 当前 scheduler 成功态判定过于宽松，只要后续 `web_search` 还能拼出一条可读摘要，就会把缺少本轮实时行情的退化版本记成 `completed + sent`。
- 这与 [`feishu_direct_quote_tool_result_ignored.md`](./feishu_direct_quote_tool_result_ignored.md) 不同：那条缺陷是 `quote` 已成功返回但 Answer 仍谎报链路阻断；本单是 `data_fetch` 真失败后，scheduler 仍把旧价格 fallback 版本当成成功结果送达。

## 下一步建议

- 为这类要求“最新/实时/当日动态”的 scheduler 任务增加硬约束：关键 `data_fetch` 失败时，不得直接落成 `completed + sent`。
- 在调度台账中区分“完整成功”和“旧价格 fallback 的部分完成态”，避免巡检把这类样本误当作正常成功。
- 为 `Hone_AI_Morning_Briefing` 增加回归：当 `data_fetch` 连续失败且只剩 `web_search` 可用时，应输出明确失败/部分完成态，而不是正式成功晨报。
