# Bug: Heartbeat 跨 job 预览去重把不同标的误判为重复，导致真实触发被压成 noop 漏发

- 发现时间：2026-05-04 23:10 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：Fixed

## 修复记录（2026-05-14 12:07 CST）

- 本轮修复同一 job / 同一实体的“实质性事实增量”被 heartbeat preview 去重误吞的复发形态。
- `crates/hone-channels/src/scheduler.rs` 在既有 ticker / entity anchor 与 token overlap 去重之外，新增修订敏感事实检查：
  - 当提醒涉及定价区间、发行价、募资、发行股数、估值、上调 / 下调等事实修订语境时，会抽取 `$115-$125`、`$150-$160`、百分比、金额、股数等关键数字事实。
  - 如果本轮提醒与历史 preview 的关键数字事实集合已经变化，即使语义 overlap 很高，也不再视为 duplicate，避免把 IPO 定价区间上调等实质性更新压成 `noop + skipped_noop`。
  - 既有跨 ticker / 跨实体保护和同事实改写抑制仍保留。
- 新增回归 `heartbeat_duplicate_preview_match_allows_cerebras_ipo_pricing_range_revision`，覆盖 `Cerebras IPO` 定价区间从 `$115-$125` 上调至 `$150-$160` 后不应被旧 preview 抑制。
- 验证：
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`
  - `cargo test -p hone-channels heartbeat_duplicate_preview_match --lib -- --nocapture`
- 关联 GitHub Issue：无。

## 最新进展（2026-05-13 15:04 CST）

- 本轮巡检把本单从 `Fixed` 回退为 `New`。这次复发发生在 `2026-05-13 10:22 CST` Feishu runtime 重启之后，因此不再按旧运行态处理；同窗 `mimo-v2.5-pro` heartbeat 已恢复正常 `JsonNoop` / `completed + sent`，也没有继续出现 `Param Incorrect`。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=19860`
  - `job_name=Cerebras IPO与业务进展心跳监控`
  - `executed_at=2026-05-13T14:30:55.221156+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - `detail_json.parse_kind=JsonTriggered`
  - `detail_json.duplicate_suppressed=true`
  - `suppressed_preview` 已生成实质性新提醒：Cerebras IPO 定价区间从 `$115-$125` 上调至 `$150-$160`，最终定价预计当晚或次日早间确定。
  - `matched_preview` 却指向 `2026-05-13 13:00` 的旧提醒：Cerebras IPO “临门”、定价区间仍为 `$115-$125`。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=19870`
  - `job_name=Cerebras IPO与业务进展心跳监控`
  - `executed_at=2026-05-13T15:01:14.576970+08:00`
  - 同样为 `JsonTriggered + duplicate_suppressed=true + noop + skipped_noop`
  - `suppressed_preview` 明确写出 `定价区间上调 30%，今日定价`，并列出发行股数、募资额和估值增量；`matched_preview` 仍是 13:00 的旧 `$115-$125` preview。
- `data/runtime/logs/sidecar.log`
  - `2026-05-13 15:01:14 CST` 记录同一 job 先生成 `deliver_preview`，随后进入 `duplicate_suppressed`，最终 `[Feishu] 心跳任务未命中，本轮不发送`。
- 结论：这仍属于 heartbeat preview 去重层把真实触发转成未发送的同一功能性缺陷，但复发形态从“跨 job / 跨 ticker”扩展到“同一 job 的重大事实增量”。用户会漏收本应送达的 IPO 定价区间上调提醒，因此维持 `P2`，无关联 GitHub Issue。

## 修复结论复核（2026-05-12 11:16 CST）

- 本轮按当前自动化约束复核：当前机器旧运行态 / 未重启进程的 live 数据不再作为重新打开本单的依据。
- 当前仓库代码已覆盖 `DRAM 心跳监控` 被上一窗 `Cerebras IPO` preview 误抑制的关键条件：
  - `heartbeat_entity_anchors_compatible(...)` 会在两边存在明确英文实体且无交集时拒绝进入宽松 token overlap 判重。
  - `heartbeat_duplicate_preview_match(...)` 对 `DRAM 盘中创历史新高（满足条件2）` 与 `Cerebras IPO 重大更新` 返回 `None`。
- 本轮新增回归 `heartbeat_duplicate_preview_match_allows_dram_record_high_after_cerebras_ipo`，锁住 2026-05-12 09:01-10:31 CST 复发形态。
- 验证：
  - `cargo test -p hone-channels heartbeat_duplicate_preview_match_allows_dram_record_high_after_cerebras_ipo --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_record_high_trigger_is_not_near_threshold_suppressed --lib -- --nocapture`
- 结论：本单维持 `Fixed`；后续只有在部署/重启到当前代码后，仍能用本地可复现测试或新代码路径证明不同实体 heartbeat 被 preview 判重时，才应重新打开。

## 修复记录（2026-05-10 23:11 CST）

- `crates/hone-channels/src/scheduler.rs` 收紧 heartbeat preview 去重：
  - 同 ticker 的宽松 `shared >= 5` 重写匹配现在必须额外满足“非 ticker 实体交集 >= 2”或“日期 / 金额 / 百分比等事实 token 交集”，避免只因 `TSLA`、通用监控词或财经模板词重合就把不同事件压成 duplicate。
  - 跨 ticker / 不同实体的硬门槛继续保留。
- 新增回归覆盖本轮复发：
  - `heartbeat_duplicate_preview_match_allows_tsla_distinct_same_ticker_events`
  - `heartbeat_duplicate_preview_match_allows_cerebras_after_portfolio_summary`
- 验证：
  - `cargo test -p hone-channels heartbeat_duplicate_preview_match --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs memory/src/session.rs`
- 关联 GitHub Issue：无。

## 证据来源

- `2026-05-12 19:03 CST` 本轮补充当前机器旧运行态证据：最近四小时内 live `duplicate_suppressed` 仍把不同标的或同 ticker 不同事件的真实触发压成 `noop + skipped_noop`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=19331`，`job_name=持仓重大事件心跳检测`，`executed_at=2026-05-12T16:01:31.340173+08:00`，`detail_json.parse_kind=JsonTriggered` 且 `duplicate_suppressed=true`；本轮 `suppressed_preview` 为 ASTS Q1 财报提醒，`matched_preview` 却指向 `15:00` 的 RKLB 多重异动。
    - `run_id=19339`，`executed_at=2026-05-12T16:31:11.094929+08:00`，同样把 ASTS / TEM 持仓重大事件提醒匹配到 RKLB preview 后落成未发送。
    - `run_id=19379` 与 `run_id=19391`，分别在 `18:01` 与 `18:31 CST` 生成 ASTS Q1 财报相关 `JsonTriggered` 正文，却被 `17:30` 的 `Cerebras IPO 重大增量` preview 抑制。
    - `run_id=19398`，`executed_at=2026-05-12T19:01:53.087315+08:00`，本轮 ASTS Q1 业绩提醒被 `17:30` 的 RKLB 异动 preview 抑制。
    - `run_id=19403`，`job_name=TSLA 正负触发条件心跳监控`，`executed_at=2026-05-12T19:00:52.242165+08:00`，`suppressed_preview` 为 `TSLA 负向触发` 中 Robotaxi 运营等待问题，`matched_preview` 却指向 `17:00` 的 `TSLA 正向触发`（Musk/高管随访华）。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-12 19:00:52.240-19:00:52.241 CST` 记录 `TSLA 正负触发条件心跳监控` 的 `parse_kind=JsonTriggered -> deliver_preview -> duplicate_suppressed`，其中正负触发主题明显不同。
    - `2026-05-12 19:01:53.083-19:01:53.085 CST` 记录 `持仓重大事件心跳检测` 的 ASTS Q1 业绩提醒先生成 `deliver_preview`，随后被 RKLB preview 抑制。
  - 同一窗口存在 `run_id=19354`（TSLA 正向触发）、`run_id=19361`（Cerebras IPO）、`run_id=19365`（DRAM 心跳监控）等成功 `completed + sent`，说明不是 Feishu 出站或 scheduler 全局不可用。
  - 结论：这些证据仍与本单同根因 / 同影响范围一致，不新建重复文档。当前仓库代码已经有 `heartbeat_duplicate_preview_match_allows_tsla_distinct_same_ticker_events`、`heartbeat_duplicate_preview_match_allows_dram_record_high_after_cerebras_ipo` 等回归覆盖同 ticker 不同事件与跨实体误判路径；本轮按当前机器旧运行态 / 未确认重启处理，不把状态从 `Fixed` 回退。若部署 / 重启到当前代码后仍复现，再重新打开。

- `2026-05-12 15:03 CST` 本轮补充当前机器旧运行态证据：最近四小时内 live `duplicate_suppressed` 仍把不同标的 / 不同事件的真实触发压成 `noop + skipped_noop`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=19234`，`job_name=DRAM 心跳监控`，`executed_at=2026-05-12T11:30:41.862828+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`；`detail_json.parse_kind=JsonTriggered` 且 `duplicate_suppressed=true`，`suppressed_preview` 为 DRAM 创上市以来新高，`matched_preview` 却指向 `2026-05-12 10:00` 的 `持仓重大事件`。
    - `run_id=19262`，`job_name=Cerebras IPO与业务进展心跳监控`，`executed_at=2026-05-12T13:01:04.951490+08:00`，`suppressed_preview` 为 Cerebras IPO 定价与上市日期确认，`matched_preview` 却指向 `12:30` 的 DRAM ETF 创上市以来新高。
    - `run_id=19273`，`executed_at=2026-05-12T13:30:50.187416+08:00`，Cerebras IPO 重大变化被 `12:30` 的持仓重大事件 preview 抑制。
    - `run_id=19283`，`executed_at=2026-05-12T14:00:47.571485+08:00`，Cerebras IPO 定价区间 / 股份数变化被 `12:30` 的 RKLB 异动 preview 抑制。
  - 同一时间窗也存在 `run_id=19259/19255/19260/19309/19312` 等成功 `completed + sent` heartbeat，说明不是 Feishu 出站或 scheduler 全局不可用，而是当前 live preview 去重仍在误吞已触发正文。
  - 结论：这是同一根因/同一影响范围的旧运行态证据，不新建重复文档；由于当前仓库代码已在 `2026-05-12 11:16 CST` 复核修复相关实体 / ticker 锚点路径，本轮不把本单从 `Fixed` 回退。后续只有在部署 / 重启到当前代码后仍复现，才重新打开。

- `2026-05-12 11:02 CST` 本轮巡检把本单从 `Fixed` 回退为 `New`：最近四小时真实 heartbeat 窗口再次出现同根因，`DRAM 心跳监控` 连续把已生成的 `JsonTriggered` 正文误匹配到上一条 `Cerebras IPO` 预览后压成 `noop + skipped_noop`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=19171`，`job_name=DRAM 心跳监控`，`executed_at=2026-05-12T09:01:19.854287+08:00`，终态为 `noop + skipped_noop + delivered=0`；`detail_json.parse_kind=JsonTriggered` 且 `duplicate_suppressed=true`，`matched_preview` 指向 `Cerebras IPO 重大更新 | 2026-05-12 08:30 北京时间`。
    - `run_id=19187`，`executed_at=2026-05-12T09:30:58.652684+08:00`，同样 `JsonTriggered + duplicate_suppressed=true + delivered=0`，被同一 Cerebras IPO preview 抑制。
    - `run_id=19203`，`executed_at=2026-05-12T10:01:16.414754+08:00`，同样先生成 DRAM 创上市以来新高提醒，再被 Cerebras IPO preview 抑制。
    - `run_id=19211`，`executed_at=2026-05-12T10:31:24.338973+08:00`，同样被 Cerebras IPO preview 抑制。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-12 09:01:19 / 09:30:58 / 10:01:16 / 10:31:24 CST` 连续记录 `DRAM 心跳监控` 的 `parse_kind=JsonTriggered -> deliver_preview -> duplicate_suppressed`。
    - 四次 `matched_preview` 都是 `Cerebras IPO 重大更新`，而本轮被抑制内容是 `DRAM ETF 创上市以来新高`，标的、主题和触发条件明显不同。
  - 同一窗口 `run_id=19161` 的 Cerebras IPO 已正常 `completed + sent + delivered=1`，说明不是 Feishu 出站或 scheduler 全局不可用，而是 actor 级 preview 去重继续把已触发正文转成未发送。
  - 结论：这仍是同一根因/同一影响范围，不新建重复文档；由于证据来自 2026-05-12 09:00-10:31 CST 的真实运行窗口，且当前台账仍会漏发用户应收到的 heartbeat，本单恢复为功能性 `P2 / New`。

- `2026-05-11 03:02 CST` 本轮在本机 live 数据中仍看到修复前 duplicate suppression 漏发形态延续：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18331`，`job_name=持仓重大事件心跳检测`，`executed_at=2026-05-10T23:31:02.108624+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`；`detail_json.parse_kind=JsonTriggered` 且 `duplicate_suppressed=true`，`suppressed_preview` 为持仓重大事件中的 RKLB / TEM / ASTS 等聚合触发，`matched_preview` 指向上一窗 Cerebras IPO preview。
    - `run_id=18360`，`job_name=持仓重大事件心跳检测`，`executed_at=2026-05-11T00:30:44.816270+08:00`，同样 `parse_kind=JsonTriggered + duplicate_suppressed=true + delivered=0`；`suppressed_preview` 为 TEM 可转债等重大资本事件，`matched_preview` 指向 ASTS 23:30 preview。
    - `run_id=18399` 与 `run_id=18422`，`job_name=Cerebras IPO与业务进展心跳监控`，分别在 `02:00` 与 `03:00` 生成 Cerebras IPO 定价区间上修 / 定价时间线更新后，被 `01:30` Cerebras preview 抑制为 `noop + skipped_noop`。
    - `run_id=18410`，`job_name=持仓重大事件心跳检测`，`executed_at=2026-05-11T02:31:51.988661+08:00`，`suppressed_preview` 为 TEM Q1 / 可转债等事件，`matched_preview` 却指向 RKLB 01:30 preview。
  - 同窗仍有 `RKLB`、`ASTS`、`Cerebras`、`TSLA` 等任务成功送达，说明不是 Feishu 出站或 scheduler 全局不可用，而是去重策略继续把已触发正文转成未发送。
  - 结论：该样本来自当前本机旧运行态 / 未确认重启后的 live 窗口；由于仓库代码已在 `2026-05-10 23:11 CST` 修复同 ticker / 跨 job 去重边界，本轮不把状态从 `Fixed` 回退为 `New`。后续若部署新代码后仍复现，再重新打开。

- `2026-05-10 23:10 CST` 本轮继续确认同一 duplicate suppression 漏发链路活跃：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18231`，`job_name=ASTS 重大异动心跳监控`，`executed_at=2026-05-10T19:30:31.921457+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`；`detail_json.parse_kind=JsonTriggered` 且 `duplicate_suppressed=true`，`suppressed_preview` 为 ASTS 单日涨幅触发，`matched_preview` 指向上一窗持仓重大事件。
    - `run_id=18259`，`job_name=ASTS 重大异动心跳监控`，`executed_at=2026-05-10T20:31:07.474435+08:00`，同样 `parse_kind=JsonTriggered + duplicate_suppressed=true + delivered=0`；本轮 ASTS 异动被 Cerebras / 持仓类旧 preview 抑制。
    - `run_id=18296` 与 `run_id=18308`，`job_name=Cerebras IPO与业务进展心跳监控`，分别在 `22:01` 与 `22:31` 生成 Cerebras IPO 定价区间 / 时间线更新后，被 `21:00` 的持仓重大事件 preview 抑制为 `noop + skipped_noop`。
  - 同窗也能看到正常送达的 `run_id=18314` Cerebras 和正常 `JsonNoop`，说明不是 Feishu 出站或 scheduler 全局不可用，而是去重策略仍会把不同主题的已触发正文吞掉。
  - 结论：本轮证据仍属于同一根因/同一影响范围，不新建重复文档，维持功能性 `P2 / New`。

- `2026-05-10 19:02 CST` 本轮巡检确认同一 duplicate suppression 漏发链路继续活跃，状态从 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18196`，`job_name=ASTS 重大异动心跳监控`，`executed_at=2026-05-10T18:30:28.285273+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`；`detail_json.parse_kind=JsonTriggered` 且 `duplicate_suppressed=true`，`matched_preview` 指向 `Cerebras IPO 价格区间上调`，`suppressed_preview` 为 ASTS 涨幅、Rakuten 减持完成与财报预期。
    - `run_id=18207`，`job_name=Cerebras IPO与业务进展心跳监控`，`executed_at=2026-05-10T19:00:42.244389+08:00`，同样被 `duplicate_suppressed=true` 压成 `noop`；`matched_preview` 指向 `持仓重大事件`，`suppressed_preview` 为 Cerebras IPO 定价区间和上市时间线更新。
    - `run_id=18214`，`job_name=ASTS 重大异动心跳监控`，`executed_at=2026-05-10T19:00:46.581912+08:00`，`matched_preview` 指向 `持仓重大事件` 中的 RKLB/TEM 聚合摘要，`suppressed_preview` 为 ASTS 单独异动提醒。
    - `run_id=18163` 与 `run_id=18212`，`job_name=TSLA 正负触发条件心跳监控`，分别在 `17:01` 和 `19:02` 把召回 / FSD 诉讼等不同负向触发压成 `noop + skipped_noop`；`matched_preview` 是 `15:00` 的 Tesla Semi 订单与 SEC/Musk 和解主题。
  - 同一窗口内还有多条正常 `completed + sent` 和 `noop + skipped_noop`，说明不是 Feishu 出站失败，而是 heartbeat 去重层把已生成的新触发正文转成未发送。
  - 结论：这仍是同一根因/同一影响范围，不新建重复文档。与 2026-05-09 的跨 ticker 复发相比，本轮还覆盖同 ticker 不同事件主题被旧 preview 误抑制的形态；用户会漏收本应送达的真实 heartbeat，维持功能性 `P2 / New`。

- `2026-05-09 19:12 CST` 本轮重新修复并关闭复发：
  - `crates/hone-channels/src/scheduler.rs` 在既有实体锚点兼容检查前新增 ticker 级硬门槛：若本轮 message 与历史 preview 都能抽取到明确 ticker，且 ticker 集合没有交集，直接禁止进入宽松 token overlap 去重。
  - 同时把 `Q1/Q2/Q3/Q4`、`CEO`、`SEC`、`FDA` 等通用英文片段排除出实体锚点，避免不同公司因季度、监管或职位词产生假交集。
  - 新增回归覆盖本次复发三类样本：`RKLB` 历史 preview 后的 `ASTS`、`TEM`、聚合持仓 `ASTS` 触发均不得被 duplicate suppression 吞掉；既有同一事件改写样本仍保持抑制。
  - 验证通过：
    - `cargo test -p hone-channels heartbeat_duplicate_preview_match --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
    - `rustfmt --edition 2024 --check crates/hone-channels/src/scheduler.rs`
    - `cargo check -p hone-channels --tests`
  - 无关联 GitHub Issue。本单状态更新为 `Fixed`；当前机器不是生产机器，本轮不以旧 live runtime 是否已重启作为闭环门槛。

- `2026-05-09 19:05 CST` 本轮巡检把本单从 `Fixed` 回退为 `New`：最近四小时真实 heartbeat 窗口再次出现同根因，且这次不是单个 job，而是同一目标下多个不同标的 / 不同主题被上一小时 `RKLB` preview 误抑制。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=17548`，`job_id=j_1241aad0`，`job_name=RKLB异动监控`，`executed_at=2026-05-09T18:00:31.741748+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`，`response_preview` 为 `【RKLB 单日暴涨34% · 2026-05-09 18:00 北京时间】...`。
  - `run_id=17557`，`job_id=j_db12f27f`，`job_name=持仓重大事件心跳检测`，`executed_at=2026-05-09T18:31:00.776206+08:00`，终态为 `noop + skipped_noop + delivered=0`；同一日志窗口先生成了 `ASTS 单日+14.8% / Rakuten退出完成 / Q1财报临近` 的 `JsonTriggered` 正文，随后被 `duplicate_suppressed` 命中上一条 `RKLB 单日暴涨34%` preview。
  - `run_id=17575`，`job_id=j_fc7749ca`，`job_name=ASTS 重大异动心跳监控`，`executed_at=2026-05-09T19:00:49.804086+08:00`，终态为 `noop + skipped_noop + delivered=0`；日志先记录 `parse_kind=JsonTriggered` 与 `deliver_preview="【ASTS 单日涨跌幅超阈值】..."`，随后 `duplicate_suppressed` 的 `matched_preview` 指向 `RKLB 单日暴涨34%`。
  - `run_id=17568`，`job_id=j_818f0150`，`job_name=TEM大事件心跳监控`，`executed_at=2026-05-09T19:00:59.933459+08:00`，终态为 `noop + skipped_noop + delivered=0`；日志先记录 `parse_kind=JsonTriggered` 与 `deliver_preview="【TEM Q1财报超预期 + 可转债发行 + 新合作】..."`，随后同样被 `RKLB 单日暴涨34%` preview 抑制。
  - `run_id=17576`，`job_id=j_db12f27f`，`job_name=持仓重大事件心跳检测`，`executed_at=2026-05-09T19:01:05.555661+08:00`，终态为 `noop + skipped_noop + delivered=0`；日志先记录 `parse_kind=JsonTriggered` 与 `deliver_preview="【ASTS 单日暴涨近15%】..."`，随后同样被上一小时 `RKLB` preview 抑制。
- `data/runtime/logs/sidecar.log`
  - `2026-05-09 19:00:49.801-19:01:05.555 CST` 连续记录 `ASTS`、`TEM`、`持仓重大事件` 三条不同 heartbeat 的 `parse_kind=JsonTriggered -> deliver_preview -> duplicate_suppressed`。
  - 三次 `matched_preview` 都是 `【RKLB 单日暴涨34% · 2026-05-09 18:00 北京时间】...`，而本轮被抑制内容分别是 ASTS、TEM 与聚合持仓中的 ASTS，实体 / ticker 明显不同。
  - 同窗 `run_id=17567`（`CAI破位预警`）成功 `completed + sent + delivered=1`，说明不是 Feishu 出站或整轮 scheduler 不可用，而是去重层把已触发内容转成未发送。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=15576`
  - `job_id=j_9ee85d42`
  - `job_name=Cerebras IPO与业务进展心跳监控`
  - `executed_at=2026-05-04T22:31:43.364323+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 为 `【Cerebras IPO重大进展 | 检查时间: 2026-05-04 22:30 北京时间】...`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=15588`
  - `job_id=j_db12f27f`
  - `job_name=持仓重大事件心跳检测`
  - `executed_at=2026-05-04T23:01:27.260262+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - `delivered=0`
  - `detail_json.parse_kind=JsonTriggered`
  - `detail_json.duplicate_suppressed=true`
  - `detail_json.matched_preview` 却指向上一窗 `Cerebras IPO重大进展`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=15591`
  - `job_id=j_39a96b7a`
  - `job_name=ORCL 大事件监控`
  - `executed_at=2026-05-04T23:01:37.212021+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - `delivered=0`
  - `detail_json.parse_kind=JsonTriggered`
  - `detail_json.duplicate_suppressed=true`
  - `detail_json.matched_preview` 同样错误指向上一窗 `Cerebras IPO重大进展`
- 同一 actor 最近窗口对照：
  - `2026-05-04 22:00-22:02`：`ORCL` 为 `noop`、`持仓重大事件` 为 `execution_failed`
  - `2026-05-04 22:30-22:31`：`Cerebras IPO` 成功送达，`ORCL` 为 `execution_failed`、`持仓重大事件` 为 `noop`
  - `2026-05-04 23:00-23:01`：`Cerebras IPO` 自己回落为 `noop`，但 `ORCL` 与 `持仓重大事件` 都在本窗首次生成 `JsonTriggered` 后被跨 job 去重层压成 `noop`
- 相关已知缺陷对照：
  - [`scheduler_heartbeat_retrigger_duplicate_alerts.md`](./scheduler_heartbeat_retrigger_duplicate_alerts.md) 关注的是“旧事件重复推送”
  - 本单相反：去重层把不同标的、不同主题的真实新触发误判成重复，导致漏发，因此不是同一坏态

## 端到端链路

1. 同一 Feishu 目标 `+8613867793336` 在 `2026-05-04 22:30` 窗口先收到 `Cerebras IPO与业务进展心跳监控` 的正式送达。
2. 约 30 分钟后，`2026-05-04 23:00` 窗口里：
   - `持仓重大事件心跳检测` 生成了 `JsonTriggered`
   - `ORCL 大事件监控` 也生成了 `JsonTriggered`
3. 这两条本应独立判定的 heartbeat 没有按各自内容走送达，而是在落库前被“最近已送达 preview 去重”拦下。
4. 去重命中的 `matched_preview` 不是同一 job、同一 ticker、同一主题，而是上一窗完全不同主题的 `Cerebras IPO重大进展`。
5. 最终数据库把两条真实触发都记成 `noop + skipped_noop`，用户没有收到原本该送达的 ORCL / 持仓提醒。

## 期望效果

- 跨 job 去重只能拦截“同一事实被改写重发”的样本，不能把不同 ticker、不同事件类型的 heartbeat 互相吞掉。
- 若本轮 `parse_kind=JsonTriggered` 的正文主题与最近已送达 preview 明显不同，应继续送达，而不是压成 `duplicate_suppressed`。
- 去重命中时，至少应保留可审计的“本轮原始触发摘要”，避免台账只剩一个空的 `noop` 终态。

## 当前实现效果

- `2026-05-09 18:30-19:01` 最新真实窗口里，ASTS、TEM、持仓重大事件都先通过模型侧触发，日志已经记录 `parse_kind=JsonTriggered` 与可见 `deliver_preview`。
- 但最终台账把这些任务落成 `noop + skipped_noop`，`matched_preview` 指向前一小时同 actor 的 `RKLB 单日暴涨34%`。
- 这说明 2026-05-06 的实体 / ticker 锚点兼容检查仍存在漏网路径：不同标的的新触发仍可能被 actor 级 preview 去重误吞。
- `2026-05-13 14:30 / 15:00` 最新复发显示，即使是同一 job / 同一实体，也会因为旧 preview 与新提醒共享足够多 IPO 语义而误抑制重大事实增量；当前去重缺少“定价区间、发行股数、募资额、估值、日期”等结构化事实变化判断。
- `2026-05-04 23:00` 最新真实窗口里，`ORCL 大事件监控` 与 `持仓重大事件心跳检测` 都先通过模型侧触发，`detail_json.parse_kind` 明确为 `JsonTriggered`。
- 但最终台账没有记录原始触发正文，只剩 `duplicate_suppressed=true` 和一个错误的 `matched_preview=Cerebras IPO重大进展`。
- 这说明当前去重逻辑在 actor 级跨 job 复用时过于宽松，已经从“抑制旧事件重复提醒”退化成“误吞不同主题的新提醒”。

## 用户影响

- 用户会漏收原本应该送达的真实 heartbeat，影响提醒链路的完整性，不只是噪音问题。
- 这不是 `P3` 质量波动：损害点是实际漏发，用户无法感知本窗已触发的 ORCL / 持仓提醒。
- 之所以定级为 `P2` 而不是 `P1`，是因为当前证据集中在单个 actor 的两条 heartbeat 被误抑制，尚未证明跨大面积用户扩散或造成数据安全事故。

## 根因判断

- 当前 actor 级 heartbeat 去重很可能只做了宽松的 preview token overlap，没有建立足够强的 ticker / 事件主体一致性校验。
- 因此当上一窗 `Cerebras IPO` preview 足够长、包含大量通用财经词和事件模板词时，下一窗 `ORCL` / `持仓事件` 的触发正文可能被错误判成“高度相似”。
- 现有台账在 `duplicate_suppressed` 路径下也没有保留本轮原始触发摘要，导致巡检时只能从 `parse_kind=JsonTriggered` 与 `matched_preview` 的矛盾间接反推漏发。

## 修复记录（2026-05-06）

- 状态更新为 `Fixed`。
- `crates/hone-channels/src/scheduler.rs` 的 heartbeat preview 去重在进入宽松 overlap 判断前新增英文实体 / ticker 锚点兼容检查：
  - 两边都能抽取到明确实体锚点且没有交集时，直接视为不同主题，不再用通用中文 n-gram overlap 抑制；
  - `OpenAI`、`IPO`、`AWS`、`price`、`event` 等常见上下文词不作为实体锚点，避免跨公司叙事误连；
  - 既有同事实改写样本（`RKLB` 合同、`TEM` 旧催化、`Blue Origin / Rocket Lab`）仍会被抑制。
- `duplicate_suppressed` metadata 新增 `suppressed_preview`，保留本轮原始触发摘要，后续排查不再只能看到 `matched_preview`。
- 新增回归样本覆盖：
  - 上一窗 `Cerebras IPO` 已送达时，下一窗 `ORCL` 触发不得被压成 duplicate；
  - 上一窗 `Cerebras IPO` 已送达时，下一窗 `持仓重大事件` 中的 `TEM / ORCL` 触发不得被压成 duplicate。

## 当前验证（2026-05-06）

- 已通过：
  - `rustfmt --edition 2024 --check crates/hone-channels/src/scheduler.rs`
  - `cargo test -p hone-channels heartbeat_duplicate_preview_match -- --nocapture`
  - `cargo test -p hone-channels scheduler::tests -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `git diff --check`

## 下一步建议

- 优先复核 `heartbeat_duplicate_preview_match(...)` 在 `matched_preview=RKLB`、`suppressed_preview=ASTS/TEM/持仓ASTS` 时为什么仍返回 true，重点检查实体锚点抽取是否被通用 token、中文标题或聚合持仓文本稀释。
- 增加同一 job 重大事实变化回归：`Cerebras IPO $115-$125 临门` 之后，`Cerebras IPO $150-$160 定价区间上调 30%` 不得被 duplicate suppression 吞掉；可优先抽取金额、百分比、发行股数、估值、日期等事实 token 做变化门槛。
- 在回归里补入本轮 `RKLB -> ASTS`、`RKLB -> TEM`、`RKLB -> 持仓ASTS` 三类样本，并断言不同 ticker / 不同事件主题不得进入 duplicate suppression。
- 部署后观察下一轮 heartbeat：若仍出现 `duplicate_suppressed=true` 且 `matched_preview` 与 `suppressed_preview` 属于不同 ticker / 主题，应继续扩展实体锚点或把去重键提升到更结构化的事件签名。
