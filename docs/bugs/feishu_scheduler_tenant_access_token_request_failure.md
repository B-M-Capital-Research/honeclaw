# Bug: Feishu scheduler 在最终投递前统一卡在 `tenant_access_token` 请求失败，已生成内容也无法送达

- **发现时间**: 2026-04-21 08:04 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#35](https://github.com/B-M-Capital-Research/honeclaw/issues/35)
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-04-21 11:22 用户侧最新反馈：
    - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
    - `2026-04-21T11:22:49.960870+08:00` 用户明确发送：`今天你的指令工作怎么没发？需要你继续执行。`
    - 同一会话对应的定时任务在 `08:34:13` 到 `08:49:12` 之间至少有 `run_id=3868`（`美股盘后AI及高景气产业链推演`）、`run_id=3871`（`美股AI产业链盘后报告`）、`run_id=3872`（`A股盘前高景气产业链推演`）全部落成 `execution_failed + send_failed + delivered=0`
    - 这些 run 的 `error_message` 都是 `集成错误: Feishu token request failed: error sending request for url (https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal)`
    - 用户随后只能手动要求“继续执行”，assistant 在 `2026-04-21T11:26:47.585590+08:00` 才补发一份当前时点报告，说明该发送故障已经转化为用户可感知的定时任务缺席。
  - 2026-04-21 08:04-09:04 最近一小时真实样本：
    - `run_id=3852`，`job_name=HoneClaw每日使用Tips`，`executed_at=2026-04-21T08:04:12.373341+08:00`
      - `execution_status=execution_failed`
      - `message_send_status=send_failed`
      - `delivered=0`
      - `response_preview=抱歉，处理超时了。请稍后再试。`
      - `error_message=集成错误: Feishu token request failed: error sending request for url (https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal)`
    - `run_id=3853/3854/3855`（`每日宏观与AI早报`、`每日持仓分析早报`、`每日美股收盘与持仓早报`）在 `08:08 -> 08:16` 继续统一落成 `execution_failed + send_failed + delivered=0`，错误体完全相同
    - `run_id=3866/3867/3868/3869/3870/3871/3872`（`创新药持仓每日动态推送`、`Hone_AI_Morning_Briefing`、`美股盘后AI及高景气产业链推演`、`闪迪(SNDK)每日行情与行业简报`、`每日有色化工标的新闻追踪`、`美股AI产业链盘后报告`、`A股盘前高景气产业链推演`）在 `08:34 -> 08:49` 继续统一落成 `send_failed`
    - `run_id=3883/3884/3885`（`特斯拉与火箭实验室新闻日报`、`早9点市场复盘(XME及加密ETF)`、`核心观察池早间简报`）在 `09:04:12 -> 09:04:13` 再次统一落成 `execution_failed + send_failed + delivered=0`
    - 以上失败样本覆盖多个不同 target：`+8617326027390`、`ou_e31244b1208749f16773dce0c822801a`、`+8618066271556`、`+8615967889916`、`+8618676788567`，说明这不是单个目标或单个 direct task 的局部坏态
  - 2026-04-21 10:01 最近一小时新增样本：
    - `run_id=3902` 前后同批 heartbeat 已恢复执行；其中 `run_id=3895`（`ASTS 重大异动心跳监控`，`executed_at=2026-04-21T09:31:36.112721+08:00`）在 `09:31` 还是 `noop`
    - 到 `10:01` 对应运行日志里，`ASTS 重大异动心跳监控` 已完成 `parse_kind=JsonTriggered` 与 `deliver_preview="【ASTS 重大异动触发提醒】..."`，但最终仍在 Feishu 发送前失败：
      - `data/runtime/logs/sidecar.log`
      - `2026-04-21 10:01:45.391` `deliver ... parse_kind=JsonTriggered`
      - `2026-04-21 10:01:45.396` `[Feishu] 定时任务投递失败: job=ASTS 重大异动心跳监控 ... Feishu token request failed: error sending request for url (https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal)`
    - 这说明问题不只影响普通日报类 direct scheduler，也已经影响 heartbeat 真正命中后的最终告警投递
  - `data/runtime/logs/web.log`
    - `2026-04-21 08:04:12.371` `job=HoneClaw每日使用Tips` 首次记录 `[Feishu] 定时任务投递失败 ... Feishu token request failed`
    - `2026-04-21 08:34:12.734 -> 08:49:12.843` 多个不同 job 连续记录相同错误
    - `2026-04-21 09:04:12.782 -> 09:04:13.070` 三个不同 target 再次连续记录相同错误
  - `data/runtime/logs/sidecar.log`
    - `2026-04-21 10:01:45.396` `ASTS 重大异动心跳监控` 在真正生成触发提醒后，仍然落成同一 `tenant_access_token/internal` 取票失败
  - 相关已有缺陷：
    - [`feishu_scheduler_send_failed_http_400_after_generation.md`](./feishu_scheduler_send_failed_http_400_after_generation.md) 关注的是发送接口返回 `HTTP 400 / open_id cross app`
    - 本单的最新错误发生在更早的 Feishu token 获取阶段，请求尚未走到最终发信 API，因此属于新的独立发送链路根因

## 端到端链路

1. Feishu scheduler 或 heartbeat 正常执行到生成/触发阶段，已经拿到待发送正文或 `deliver_preview`。
2. 发送链路准备调用 Feishu API 前，需要先请求 `tenant_access_token/internal`。
3. 这一取票请求在多个时间点、多个任务、多个目标上统一报 `error sending request for url (...)`。
4. 当前发送侧没有把这类取票失败吸收为自动重试或统一降级，因此整轮直接记为 `send_failed`。
5. 用户侧即使面对“已生成完成的日报”或“已命中的 heartbeat 告警”，最终也完全收不到消息。

## 状态更新（2026-05-05 10:13 CST）

- 本轮巡检确认该缺陷已从 `Later` 回退为活跃 `New`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=15667` `美股AI产业链盘后报告`
    - `run_id=15668` `Hone_AI_Morning_Briefing`
    - `run_id=15669` `每日有色化工标的新闻追踪`
    - `run_id=15670` `港股持仓与关注股早间行情研判`
    - `run_id=15671` `创新药持仓每日动态推送`
    - `run_id=15672` `闪迪(SNDK)每日行情与行业简报`
    - `run_id=15674` `早9点市场复盘(XME及加密ETF)`
    - `run_id=15675` `特斯拉与火箭实验室新闻日报`
  - 上述 8 条 run 全部位于 `2026-05-05 09:42:28` 到 `10:13:14` 的最近一小时窗口，并且统一满足：
    - `execution_status=execution_failed`
    - `message_send_status=target_resolution_failed`
    - `delivered=0`
    - `response_preview=抱歉，处理超时了。请稍后再试。`
    - `error_message=集成错误: Feishu resolve mobile api error 99991663: Invalid access token for authorization. Please make a request with token attached.`
  - 同窗另有 `run_id=15676`（`核心观察池早间简报`）继续落成 `target_resolution_failed`，但错误是 `batch_get_id` 联系人查询传输失败，已继续归入 [`feishu_scheduler_target_resolution_failed.md`](./feishu_scheduler_target_resolution_failed.md) 跟踪，不与本单混档。
- `data/runtime/logs/web.log.2026-05-05` 同时给出对应链路日志：
  - `2026-05-05 09:42:33.599` `每日有色化工标的新闻追踪` 首次记录 `[Feishu] 定时任务目标解析失败 ... Invalid access token for authorization`
  - `2026-05-05 09:42:34.261` `闪迪(SNDK)每日行情与行业简报` 同样报错
  - `2026-05-05 09:42:34.742` `Hone_AI_Morning_Briefing` 同样报错
  - `2026-05-05 09:42:35.492` `美股AI产业链盘后报告` 同样报错
  - `2026-05-05 10:13:14.478` `早9点市场复盘(XME及加密ETF)` 同样报错
  - `2026-05-05 10:13:14.978` `特斯拉与火箭实验室新闻日报` 同样报错
- 结论：
  - 2026-04-26 的短重试只吸收了传输错误、`429` 与 `5xx`，并没有覆盖当前这轮 `99991663 Invalid access token` 的认证失效。
  - 本轮异常已经同时覆盖多个不同 actor / target / 任务模板，且全部发生在 Feishu 发送前置共享链路，不是单个 direct task 的局部坏态。
  - 因此本单必须重新回到活跃 P1 队列。

## 期望效果

- Feishu scheduler 在最终发送前请求 `tenant_access_token/internal` 时，应至少具备基本重试与可恢复策略，而不是一次网络抖动就让整轮 `send_failed`。
- 即使 token 获取失败，系统也应留下清晰的链路级可观测信息，便于区分“上游认证接口不可达”和“业务发送接口返回 4xx/5xx”。
- 对用户而言，已生成完成的日报或已命中的 heartbeat 告警不应因为公共 token 获取失败而整批静默丢失。

## 当前实现效果

- `2026-04-21 08:04 -> 09:04` 的最新真实窗口里，至少 11 条 Feishu 定时任务跨多个不同 target 统一落成 `send_failed`，错误体完全一致，说明问题发生在共享发送前置链路，而不是单个任务 prompt、单个用户配置或单个 receive_id。
- 这些 run 的 `response_preview` 多数已经是稳定的用户态失败文案或完整正文开头，说明 scheduler 主体并未卡在 search / answer 阶段，真正失败点发生在最终 Feishu 发送准备阶段。
- `2026-04-21 10:01` 的 `ASTS 重大异动心跳监控` 更进一步证明：即使 heartbeat 已经完成事件判断并生成了明确的 `deliver_preview`，最终仍会因为 `tenant_access_token/internal` 请求失败而彻底丢失提醒。
- `2026-04-21 11:22` 的直聊反馈进一步证明，这不是只有台账可见的后台失败：用户已经明确感知到今天定时指令没有送达，并要求人工补跑。
- 这说明本故障不是“某类日报任务触发超时 fallback 后发不出去”的局部问题，而是 Feishu 发送公共前置依赖当前不稳定。

## 用户影响

- 这是功能性缺陷，不是回答质量问题。任务内容已经生成或 heartbeat 已经确认触发，但用户最终完全收不到消息。
- 最新用户反馈说明，自动任务缺席已经直接影响用户工作流；用户需要主动追问并触发人工补发，才能拿到原本应自动送达的内容。
- 之所以定级为 `P1`，是因为问题在最近一小时内同时影响了多个 Feishu 定时任务与至少一条 heartbeat 告警，已经是跨任务、跨目标的共享发送链路故障。
- 这不是 `P3`：损害不在于“内容写得一般”，而在于 Feishu 自动投递主能力直接中断。

## 根因判断

- 直接触发点是 Feishu 发送链路在请求 `https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal` 时发生 HTTP 发送失败。
- 由于同类错误在多个 target、多个 job、不同任务类型（普通日报与 heartbeat 告警）上统一出现，根因更接近 Feishu 公共取票/网络可达性缺口，而不是单个任务 payload 或 receive_id 构造问题。
- 该问题与 `open_id cross app` 的 `HTTP 400` 不同：后者说明已经拿到 token 并走到了发送接口；本次样本显示请求在更早的 token 获取阶段就已经失败。
- `2026-05-05 09:42-10:13` 的最新回归表明，这条公共取票链路现在还会进入更明确的认证失效分支：`resolve mobile api error 99991663: Invalid access token for authorization`。这说明 2026-04-26 的止血并没有覆盖 token 失效/刷新异常场景。

## 下一步建议

- 优先排查 Feishu `tenant_access_token/internal` 的网络可达性、DNS/代理、证书或上游接口可用性，并确认发送链路当前是否缺少取票重试。
- 后续台账应继续区分两类 Feishu scheduler 发送故障：
  - `tenant_access_token` 获取失败
  - 最终发送接口 `HTTP 400 / open_id cross app`
- 修复后优先复核两个场景：
  - 普通 Feishu 定时日报是否恢复送达
  - 已触发的 heartbeat 告警是否能在 `deliver_preview` 之后真正发出

## 修复进展（2026-04-26）

- 已在 `bins/hone-feishu/src/client.rs` 为 `tenant_access_token/internal` 请求补有限传输重试：
  - 最多 3 次；
  - 重试请求传输错误、`429` 与 `5xx`；
  - `4xx` 认证/配置错误仍立即返回，避免掩盖真实配置问题。
- 同一重试封装也用于 Feishu 发送、回复、更新消息请求，保证 token 获取恢复后后续出站阶段也具备基本吸震。
- 已验证：`cargo test -p hone-feishu`。
- 当时状态曾调整为 `Later`，因为代码止血已落地。
- `2026-05-05 10:13 CST` 最近窗口已确认认证失效场景再次整批复现，因此状态重新调回 `New`。
