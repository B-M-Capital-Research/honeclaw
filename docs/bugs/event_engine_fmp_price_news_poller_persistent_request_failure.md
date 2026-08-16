# Bug: Event-engine FMP price/news poller 持续请求失败导致行情与新闻增量退化

## 发现时间

- 2026-06-07 03:02 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- New

## 证据来源

- `data/logs/hone-console-page-source.log`
  - 2026-08-16 18:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-16 14:00-18:02 CST 同窗继续出现 FMP 请求发送失败；`data/runtime/task_runs.2026-08-16.jsonl` 从 UTC `2026-08-16T06:00:13Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news 增量链路。
  - 失败样本覆盖 `poller.fmp.news` 的 `stock_news` 请求；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=268`、`run_start=72`、`run_finish=72`、`deliver=34`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-16 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-16 10:01-14:02 CST 同窗继续出现 FMP 请求发送失败；`data/runtime/task_runs.2026-08-16.jsonl` 从 UTC `2026-08-16T02:01:13Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=16`、`poller.fmp.extended_hours ok=8`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news 增量链路。
  - 失败样本覆盖 `poller.fmp.news` 的 `stock_news` 请求；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=260`、`run_start=64`、`run_finish=67`、`deliver=42`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-16 06:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-16 02:01-06:02 CST 同窗继续出现 FMP 请求发送失败；`data/runtime/task_runs.2026-08-15.jsonl` 从 UTC `2026-08-15T18:01:41Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=16`、`poller.fmp.extended_hours ok=8`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news 增量链路。
  - 失败样本覆盖 `poller.fmp.news` 的 `stock_news` 请求；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=272`、`run_start=72`、`run_finish=72`、`deliver=35`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-15 22:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-15 18:02-22:01 CST 同窗继续出现 FMP 请求发送失败；`data/runtime/task_runs.2026-08-15.jsonl` 从 UTC `2026-08-15T10:02:09Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=16`、`poller.fmp.extended_hours ok=8`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news 增量链路。
  - 失败样本覆盖 `poller.fmp.news` 的 `stock_news` 请求；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=260`、`run_start=72`、`run_finish=72`、`deliver=31`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-15 18:04 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-15 14:02-18:03 CST 同窗继续出现 FMP 请求发送失败；`data/runtime/task_runs.2026-08-15.jsonl` 从 UTC `2026-08-15T06:01:08Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news 增量链路。
  - 失败样本覆盖 `poller.fmp.news` 的 `stock_news` 请求；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=229`、`run_start=64`、`run_finish=65`、`deliver=26`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-15 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-15 10:00-14:02 CST 同窗继续出现 FMP 请求发送失败；`data/runtime/task_runs.2026-08-15.jsonl` 从 UTC `2026-08-15T02:00:38Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news 增量链路。
  - 失败样本覆盖 `poller.fmp.news` 的 `stock_news` 请求；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=236`、`run_start=64`、`run_finish=68`、`deliver=26`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-15 10:03 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-15 06:01-10:03 CST 同窗继续出现 FMP 请求发送失败；source log 检出 `FMP 请求失败=12496`，集中在 extended-hours `prev_close` 批量请求以及 FMP 增量 poller 请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-14.jsonl` / `data/runtime/task_runs.2026-08-15.jsonl` 从 UTC `2026-08-14T22:01:37Z` 后记录 `poller.fmp.news failed=8`、`poller.fmp.earnings failed=2`、`poller.fmp.macro failed=2`，同时 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`、`poller.fmp.analyst_grade ok=2`、`poller.fmp.sec_filings ok=2`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news / earnings / macro 和 extended-hours 请求链路。
  - 同窗仍有 `HeartbeatDiag=219`、`run_start=64`、`run_finish=64`、`deliver=21`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻 / 财报 / 宏观事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-15 06:03 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-15 02:00-06:02 CST 同窗继续出现 FMP 请求发送失败；source log 检出 `FMP 请求失败=11045`，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.price` quote batch 和 `poller.fmp.news` 的 `stock_news` 请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-14.jsonl` 从 UTC `2026-08-14T18:00:06Z` 后记录 `poller.fmp.price failed=9`、`poller.fmp.news failed=8`，同时 `poller.fmp.price ok=8`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆，失败集中在 FMP price/news 增量链路。
  - 同窗仍有 `HeartbeatDiag=271`、`run_start=72`、`run_finish=72`、`deliver=38`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-15 02:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-14 22:00-2026-08-15 02:02 CST 同窗继续出现 FMP 请求发送失败；`data/runtime/task_runs.2026-08-14.jsonl` 从 UTC `2026-08-14T14:02:28Z` 后记录 `poller.fmp.price failed=16`、`poller.fmp.news failed=8`，同时 `poller.fmp.extended_hours ok=8`，说明 event-engine runtime 未整体停摆，失败集中在 FMP price/news 增量链路。
  - 失败样本覆盖 `poller.fmp.price` 的 quote batch 请求和 `poller.fmp.news` 的 `stock_news` 请求；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=225`、`run_start=65`、`run_finish=63`、`deliver=24`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-14 10:04 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-14 06:00-10:04 CST 同窗检出 12,496 条 `FMP 请求失败`，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.news` 的 `stock_news` 请求发送失败，以及 `poller.fmp.earnings` / `poller.fmp.macro` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-13.jsonl` / `data/runtime/task_runs.2026-08-14.jsonl` 从 UTC `2026-08-13T22:00:32Z` 后记录 `poller.fmp.news failed=8`、`poller.fmp.macro failed=2`、`poller.fmp.earnings failed=2`，同时 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`、`poller.fmp.sec_filings ok=2`、`poller.fmp.corp_action ok=2`、`poller.fmp.analyst_grade ok=2`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news / macro / earnings 和 extended-hours 请求链路。
  - 同窗仍有 `HeartbeatDiag=264`、`deliver=33`、`duplicate_suppressed=12`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻 / 财报 / 宏观事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-14 06:05 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-14 02:01-06:05 CST 同窗检出 8,861 条 `FMP 请求失败`，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.price` quote batch 和 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-13.jsonl` 从 UTC `2026-08-13T18:01:31Z` 后记录 `poller.fmp.price failed=9`、`poller.fmp.news failed=8`，同时 `poller.fmp.extended_hours ok=8`、`poller.fmp.price ok=7`，说明 event-engine runtime 未整体停摆，失败集中在 FMP price/news 增量链路。
  - 同窗仍有 `HeartbeatDiag=263`、`run_start=72`、`run_finish=72`、`deliver=39`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-13 18:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-13 14:01-18:02 CST 同窗检出 10,928 条 `FMP 请求失败`，集中在 extended-hours `prev_close` 批量请求以及 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-13.jsonl` 从 UTC `2026-08-13T06:01:58Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news / extended-hours 取数链路。
  - 同窗仍有 `HeartbeatDiag=230`、`run_start=64`、`run_finish=64`、`deliver=31`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-13 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-13 10:00-14:02 CST 同窗检出 8 条 `FMP 请求失败`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-13.jsonl` 从 UTC `2026-08-13T02:02:00Z` 后记录 `poller.fmp.news failed=8`，同时 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆，失败集中在 FMP news 增量请求链路。
  - 同窗仍有 `HeartbeatDiag=231`、`run_start=64`、`run_finish=64`、`deliver=33`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-13 10:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-13 06:01-10:02 CST 同窗检出 12,496 条 `FMP 请求失败`，集中在 extended-hours `prev_close` 批量请求以及部分 FMP 增量 poller；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-13.jsonl` 从 UTC `2026-08-12T22:01:27Z` 后记录 `poller.fmp.news failed=4`、`poller.fmp.earnings failed=2`、`poller.fmp.macro failed=2`，同时 `poller.fmp.price ok=9`、`poller.fmp.extended_hours ok=5`、`poller.fmp.analyst_grade ok=2`、`poller.fmp.sec_filings ok=2`，说明 event-engine runtime 未整体停摆，失败集中在 FMP 增量请求链路。
  - 同窗仍有 `HeartbeatDiag=235`、`run_start=64`、`run_finish=64`、`deliver=32`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻 / 财报 / 宏观事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-13 06:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-13 02:00-06:02 CST 同窗检出 11,045 条 `FMP 请求失败` 与 34 条 `poller.fmp` 信号，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.price` quote batch 与 `poller.fmp.news` / `stock_news` 请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-12.jsonl` 从 UTC `2026-08-12T18:00:26Z` 后记录 `poller.fmp.news failed=8`、`poller.fmp.price failed=17`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆，失败集中在 FMP price/news 增量链路。
  - 同窗仍有 `HeartbeatDiag=255`、`run_start=64`、`run_finish=71`、`deliver=43`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-13 02:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-12 22:00-2026-08-13 02:01 CST 同窗检出 216 条 `FMP 请求失败` 与 32 条 `poller.fmp` 信号，集中在 `poller.fmp.price` quote batch 与 `poller.fmp.news` / `stock_news` 请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - `data/runtime/task_runs.2026-08-12.jsonl` 同窗记录 `poller.fmp.news failed=8`、`poller.fmp.price failed=16`，同时 `poller.fmp.extended_hours ok=8`，说明 event-engine runtime 未整体停摆，失败集中在 FMP price/news 增量链路。
  - 同窗仍有 `HeartbeatDiag=238`、`run_start=64`、`run_finish=67`、`deliver=37`，说明 scheduler 其它链路仍在推进。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-12 22:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-12 18:01-22:01 CST 同窗检出 15,322 条 `FMP 请求失败` 与 32 条 `poller.fmp` 信号，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.news` / `stock_news` 等请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=258`、`run_start=64`、`run_finish=66`、`deliver=44`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP 行情和新闻增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-12 18:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-12 14:00-18:02 CST 同窗检出 10,928 条 `FMP 请求失败` 与 34 条 `poller.fmp` 信号，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.news` / `stock_news` 等请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=271`、`run_start=72`、`run_finish=72`、`deliver=42`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP 行情和新闻增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-12 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-12 10:01-14:01 CST 同窗检出 8 条 `FMP 请求失败` 与 32 条 `poller.fmp` 信号，集中在 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=238`、`run_start=64`、`run_finish=65`、`deliver=35`，且 `poller.fmp.price` / `poller.fmp.extended_hours` 有 ok 样本，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP news 增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-12 10:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-12 06:01-10:02 CST 同窗检出 12,496 条 `FMP 请求失败` 与 44 条 `poller.fmp` 信号，集中在 extended-hours prev_close / FMP news 等请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=239`、`run_start=64`、`run_finish=65`、`deliver=38`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP 行情和新闻增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-12 06:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-12 02:01-06:02 CST 同窗检出 11,045 条 `FMP 请求失败` 与 34 条 `poller.fmp` 信号，集中在 extended-hours prev_close / FMP news 等请求失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag=237`、`run_start=64`、`run_finish=64`、`deliver=43`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP 行情和新闻增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-12 02:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-11 22:00-2026-08-12 02:02 CST 同窗检出 238 条 `FMP 请求失败` / `poller.fmp` / `FMP quote batch failed` 失败信号，集中在 `poller.fmp.price` quote batch、`poller.fmp.news` / `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag run_start=108`、`run_finish=108`、`deliver=66`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP 行情和新闻增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-11 02:03 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-10 22:01-2026-08-11 02:02 CST 同窗 `FMP 请求失败` / `poll failed` 约 230 条，集中在 `poller.fmp.price`、`poller.fmp.news` / `stock_news`、extended-hours prev_close 等请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag run_start=110`、`run_finish=110`、`deliver=60`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP 行情、新闻和盘前盘后增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-10 22:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-10 18:00-22:02 CST 同窗有 11 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag run_start=97`、`run_finish=99`、`deliver=65`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP news 增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-10 18:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-10 14:02-18:02 CST 同窗有 8 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag` 相关行 359 条、`deliver=51`，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP news 增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-10 06:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-10 02:01-06:01 CST 同窗有 9 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag run_start=98`、`run_finish=105`、`deliver=55`，且 `poller.fmp.price` / `poller.fmp.extended_hours` 仍有 ok 样本，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP news 增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-09 22:03 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-09 18:03-22:03 CST 同窗有 9 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag run_start=96`、`run_finish=96`、`deliver=55`，且 22:02 CST `poller.fmp.price` / `poller.fmp.extended_hours` 仍有 ok 样本，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP news 增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-09 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-09 10:00-14:01 CST 同窗有 8 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag run_start=108`、`run_finish=108`、`deliver=56` 与 `poller.fmp.news` 以外的 heartbeat 运行信号，说明 event-engine / scheduler 未整体停摆；失败集中在 FMP news 增量抓取。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-09 10:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-09 06:00-10:02 CST 同窗继续出现 FMP 请求发送失败：`poller.fmp.news` / `stock_news` 约每 30 分钟失败；00:00 / 00:30 UTC 还扩展到 `poller.fmp.earnings`、`poller.fmp.macro`、split/dividend calendar 与大量 `fmp.sec_filings`。日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 `HeartbeatDiag run_start=109`、`run_finish=109`、`deliver=57` 与 `poller.fmp.price` / `poller.fmp.extended_hours` 的 ok 样本，说明 event-engine 与 scheduler 未整体停摆；失败集中在部分 FMP 事件源 / SEC filing 链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻、财报、宏观、公司行动和 SEC 文件增量的新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-09 06:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-09 02:01-06:01 CST 同窗有 8 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 heartbeat run / deliver 信号，说明 event-engine 未整体停摆；本轮未见 Web push channel closed / dryrun sink 复发。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-09 02:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-08 22:02-2026-08-09 02:02 CST 同窗有 8 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 heartbeat run / deliver 信号，说明 event-engine 未整体停摆；本轮未见 Web push channel closed / dryrun sink 复发。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-08 22:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-08 18:01-22:01 CST 同窗有 8 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 heartbeat run / deliver 信号，说明 event-engine 未整体停摆；20:30 CST 另有 Web push `channel closed` / `[dryrun sink]` fallback，归入独立投递台账缺陷。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-08 18:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-08 14:02-18:01 CST 同窗有 8 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 heartbeat run / deliver 信号，说明 event-engine 未整体停摆；本轮未见 Web push channel closed / dryrun sink 复发。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-08 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-08 10:01-14:02 CST 同窗有 8 条 `poll failed`，均为 `poller.fmp.news` 的 `stock_news` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 heartbeat run / deliver 信号，说明 event-engine 未整体停摆；本轮未见 Web push channel closed / dryrun sink 复发。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-08 10:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-08 06:01-10:01 CST 同窗有 10 条 `poll failed`，集中在 `poller.fmp.news`、`poller.fmp.macro`、`poller.fmp.earnings` 请求发送失败；日志中的 FMP URL 已由 runtime 脱敏为 `apikey=<redacted>`。
  - 同窗仍有 heartbeat run / deliver 与 event-engine sink fallback 信号，说明 event-engine 未整体停摆。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 宏观 / 财报事件增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-08 06:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-08 02:01-06:01 CST 同窗继续出现 FMP 请求发送失败：`poll failed` 18 条，覆盖 `poller.fmp.price`、`poller.fmp.news`，并有大量 `poller.fmp.extended_hours` prev_close fetch / skip 信号；同窗仍有 `poller ok` 与 heartbeat run / deliver 样本。
  - 说明 event-engine / scheduler runtime 未整体停摆；失败集中在 FMP price / news / extended-hours 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-08 02:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-07 22:01-2026-08-08 02:01 CST 同窗 `poll failed=24`，覆盖 `poller.fmp.price` 的 quote batch 请求与 `poller.fmp.news` 的 `stock_news` 请求；同窗未见 Web push channel closed 或 dryrun sink 复发。
  - 同窗仍有 heartbeat run / deliver / duplicate suppression 信号，说明 event-engine / scheduler runtime 未整体停摆；失败集中在部分 FMP event source 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-07 22:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-07 18:01-22:01 CST 同窗 `poll failed=11`，覆盖 `poller.fmp.news` 的 `stock_news` 请求、`poller.fmp.earnings` 的 `earning_calendar` 请求与 `poller.fmp.macro` 的 `economic_calendar` 请求。
  - 同窗仍有 heartbeat run / deliver / duplicate suppression 信号，说明 event-engine / scheduler runtime 未整体停摆；失败集中在部分 FMP event source 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 财报 / 宏观增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-07 18:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-07 14:02-18:01 CST 同窗 `poll failed=8`，覆盖 `poller.fmp.news` 的 `stock_news` 请求、`poller.fmp.earnings` 的 `earning_calendar` 请求与 `poller.fmp.macro` 的 `economic_calendar` 请求。
  - 同窗 `poller.fmp.price` 与 `poller.fmp.extended_hours` 有 26 条 `poller ok` 样本，说明 event-engine runtime 未整体停摆；失败集中在部分 FMP event source 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 财报 / 宏观增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-07 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-07 10:02-14:02 CST 同窗 `poll failed=8`，覆盖 `poller.fmp.news` 的 `stock_news` 请求、`poller.fmp.earnings` 的 `earning_calendar` 请求与 `poller.fmp.macro` 的 `economic_calendar` 请求。
  - 同窗 `poller.fmp.price` 与 `poller.fmp.extended_hours` 有 26 条 `poller ok` 样本，说明 event-engine runtime 未整体停摆；失败集中在部分 FMP event source 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 财报 / 宏观增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-07 10:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-07 06:02-10:02 CST 同窗 `poll failed=12`，覆盖 `poller.fmp.news` 的 `stock_news` 请求、`poller.fmp.earnings` 的 `earning_calendar` 请求与 `poller.fmp.macro` 的 `economic_calendar` 请求。
  - 同窗 `poller.fmp.price` 与 `poller.fmp.extended_hours` 有 32 条 `poller ok` 样本，说明 event-engine runtime 未整体停摆；失败集中在部分 FMP event source 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 财报 / 宏观增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-07 06:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-07 02:02-06:02 CST 同窗 `fmp_fail=126`、`fmp_ok=17`；`poller.fmp.news` 继续出现 `poll failed: FMP 请求失败 ... stock_news`，`poller.fmp.price` 多批 `FMP quote batch failed` 与 `poller.fmp.price poll failed` 覆盖多个 25 标的 batch。
  - 同窗仍有 `poller.fmp.price` / `poller.fmp.extended_hours` ok 样本，说明 event-engine runtime 未整体停摆；失败集中在 FMP news 与部分 price 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 行情增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-07 02:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-06 22:01-2026-08-07 02:01 CST 同窗 `fmp_fail=229`、`fmp_ok=9`；`poller.fmp.news` 继续出现 `poll failed: FMP 请求失败 ... stock_news`，`poller.fmp.price` 多批 `FMP quote batch failed` 与 `poller.fmp.price poll failed` 覆盖多个 25 标的 batch。
  - 同窗仍有 `poller.fmp.extended_hours` ok 样本，说明 event-engine runtime 未整体停摆；失败集中在 FMP news 与 price 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 行情增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-06 22:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-06 18:01-22:01 CST 同窗 `fmp_fail=24`、`fmp_ok=22`；前半窗 `poller.fmp.news` 仍约每 30 分钟 `poll failed: FMP 请求失败 ... stock_news`，后半窗 21:32 CST 起 `poller.fmp.price` 又出现多批 `FMP quote batch failed` 与 `poller.fmp.price poll failed`。
  - 同窗仍有 `poller.fmp.price` / `poller.fmp.extended_hours` ok 样本，说明 event-engine runtime 未整体停摆；失败集中在 FMP news 请求发送链路，并在 21:32 后扩展到 price batch 请求。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻 / 行情增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-06 18:03 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-06 14:00-18:03 CST 同窗 `poller.fmp.news` 仍出现 8 条 `poll failed: FMP 请求失败: error sending request ... stock_news`。
  - 同窗 `poller.fmp.price` 与 `poller.fmp.extended_hours` 合计 26 条 ok 样本，说明 event-engine runtime 未整体停摆；本轮失败集中在 FMP news 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-06 14:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-06 10:01-14:02 CST 同窗 `poller.fmp.news` 仍约每 30 分钟出现 `poll failed: FMP 请求失败: error sending request ... stock_news`。
  - 同窗 `poller.fmp.price` 有 17 条 ok 样本、`poller.fmp.extended_hours` 有 9 条 ok 样本，说明 event-engine runtime 未整体停摆；本轮失败集中在 FMP news 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在新闻增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-06 10:01 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-06 06:01-10:01 CST 同窗 FMP 相关请求失败继续存在，`poller.fmp.news` 仍出现 `poll failed: FMP 请求失败: error sending request ... stock_news`。
  - 同窗 `poller.fmp.price` 与 `poller.fmp.extended_hours` 仍有 ok 样本，说明 event-engine runtime 未整体停摆；失败集中在部分 FMP 请求发送链路与新闻增量。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-06 02:02 CST 运行态继续复发，状态维持 `New/P2`。
  - 2026-08-05 22:02-2026-08-06 02:02 CST 同窗 `poller.fmp.news` 仍出现 8 条 `poll failed: FMP 请求失败: error sending request ... stock_news`。
  - 同窗 `poller.fmp.price` 失败扩大为 204 条 batch / poll 失败信号，覆盖多个 25 标的 batch，并有 `poller.fmp.price` 的 `poll failed` 记录。
  - 同窗仍有 `poller.fmp.extended_hours` ok 样本，说明 event-engine runtime 未整体停摆；失败集中在 FMP news 与 price 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
  - 2026-08-05 18:03-22:02 CST 运行态复发，状态从 `Closed` 回退为 `New/P2`。
  - 同窗 `poller.fmp.news` 持续出现 `poll failed: FMP 请求失败: error sending request ... stock_news`，约每 30 分钟复发；近窗合计 `poll failed=11`。
  - 13:32-14:02 UTC / 21:32-22:02 CST 后 `poller.fmp.price` 也开始整批退化，近窗合计 36 条 `FMP quote batch failed`，覆盖多个 25 标的 batch，并有 `poller.fmp.price` 的 `poll failed` 记录。
  - 同窗仍有 `poller.fmp.extended_hours` ok 样本，说明 event-engine runtime 未整体停摆；失败集中在 FMP news 与后半窗 price 请求发送链路。
  - 尚未观察到用户可见 FMP 原始错误外泄；影响集中在行情 / 新闻增量、digest 候选和监控触发新鲜度，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。

- `data/runtime/task_runs.2026-06-06.jsonl`
  - 2026-06-06 23:01-2026-06-07 03:01 CST 最近四小时内，`poller.fmp.price` 出现 48 次 `failed + items=0`，`poller.fmp.news` 出现 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 有 8 次 `ok + items=0`，说明 runtime 仍在调度 poller tick，失败集中在 FMP quote/news 请求链路。
  - 失败信息均为脱敏后的 `FMP 请求失败: error sending request for url (https://financialmodelingprep.com/...apikey=<redacted>)`，覆盖 `/api/v3/quote/...` 与 `/api/v3/stock_news?limit=50`。
- `data/runtime/task_runs.2026-06-06.jsonl`
  - 2026-06-07 03:01-07:01 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，`internal.unified_digest_scheduler` 与 `internal.daily_report` 仍按分钟 tick 记录 `skipped`，说明 runtime tick 未整体停摆。
  - 失败形态仍是 FMP quote/news 请求发送失败，没有观察到恢复样本。
- `data/runtime/task_runs.2026-06-06.jsonl` 与 `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 07:00-11:04 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.earnings` 与 `poller.fmp.macro` 各新增 2 次 `failed + items=0`，错误同为 FMP 请求发送失败；`poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，`poller.fmp.corp_action`、`poller.fmp.sec_filings`、`poller.fmp.analyst_grade` 各 2 次 `ok + items=0`。
  - `internal.unified_digest_scheduler` 同窗有 2 次 `ok + items=46`，说明 event-engine runtime 仍在运行，失败集中在 FMP 部分 API 请求链路。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 11:02-15:02 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，说明 runtime tick 未整体停摆，失败继续集中在 FMP price/news 请求发送链路。
  - `internal.unified_digest_scheduler` 与 `internal.daily_report` 同窗仅记录周期性 `skipped`，未观察到用户可见 FMP 原始错误外泄。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 15:03-19:03 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`；`internal.unified_digest_scheduler` 与 `internal.daily_report` 仅记录周期性 `skipped`。
  - 错误仍为脱敏后的 FMP quote/news 请求发送失败，尚未观察到恢复样本或用户可见原始 FMP 错误。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 19:02-23:02 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，`internal.daily_report` 有 1 次 `ok + items=1`，`internal.unified_digest_scheduler` 有 2 次 `ok`；说明 runtime 未整体停摆。
  - 失败仍集中在 FMP quote/news 请求发送链路，尚无本轮用户可见 FMP 原始错误外泄。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 23:02-2026-06-08 03:02 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`；`internal.unified_digest_scheduler` 与 `internal.daily_report` 仅记录周期性 `skipped`，说明 runtime tick 仍运行，但 FMP price/news 增量继续不可用。
  - 错误仍为脱敏后的 FMP quote/news 请求发送失败，尚未观察到恢复样本或用户可见 FMP 原始错误外泄。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-08 03:02-07:02 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`；`internal.daily_report` 与 `internal.unified_digest_scheduler` 仅记录周期性 `skipped`，说明 runtime tick 仍运行，但 FMP price/news 增量继续不可用。
  - 错误仍为脱敏后的 FMP quote/news 请求发送失败，尚未观察到恢复样本或用户可见原始 FMP 错误外泄。
- `data/runtime/task_runs.2026-06-07.jsonl` 与 `data/runtime/task_runs.2026-06-08.jsonl`
  - 2026-06-08 07:01-11:02 CST 复核窗口内，前半段仍失败：`poller.fmp.price` 27 次、`poller.fmp.news` 9 次 `failed + items=0`，失败持续到 09:14 CST。
  - 2026-06-08 09:19 CST 起出现恢复样本：`poller.fmp.price` 后续 21 次、`poller.fmp.news` 后续 7 次为 `ok + items=0`；`poller.fmp.earnings` 与 `poller.fmp.macro` 也各有 1 次从失败转为 `ok`。
  - 这说明 FMP 请求发送链路在本窗后半段部分恢复，但恢复窗口不足两小时，且 `poller.fmp.sec_filings` 同窗仍有 1 次 `failed`；本轮只记录状态变化，不直接关闭缺陷。
- `data/runtime/task_runs.2026-06-08.jsonl`
  - 2026-06-08 11:01-15:01 CST 复核窗口内，`poller.fmp.price` 48 次、`poller.fmp.news` 16 次全部为 `ok + items=0`。
  - 同窗 `poller.fmp.extended_hours` 8 次为 `ok + items=0`，未见 FMP poller `failed` 记录；`internal.daily_report` 与 `internal.unified_digest_scheduler` 仅按周期记录 `skipped`。
  - 结合 2026-06-08 09:19 CST 起的恢复样本，price/news 请求发送链路已连续超过一个完整巡检窗口恢复；本轮将状态从 `New` 调整为 `Closed`，后续若再次出现连续失败再重新打开。
- 当天更早记录显示：
  - 2026-06-06 08:04-09:24 CST `poller.fmp.price` 曾连续成功 18 次，`poller.fmp.news` 曾成功 5 次。
  - 2026-06-06 09:29 CST 起，price/news poller 开始持续失败；截至 2026-06-07 03:01 CST 最近四小时仍未恢复。
- `data/sessions.sqlite3`
  - 2026-06-06 23:01-2026-06-07 03:01 CST 有 3 个 Feishu user turn 与 3 个 assistant final，均成对收口；本轮没有直接用户可见投递失败或原始 FMP 错误外泄。
  - 2026-06-07 03:01-07:01 CST 没有新增可判定直聊质量的新消息；SQLite 最新消息仍停在 2026-06-07 00:41 CST。
  - 2026-06-07 07:00-11:04 CST 有 5 个 user turn 与 5 个 assistant final，Feishu direct 与 Discord scheduler 均有 assistant 记录收口；assistant final 污染扫描未命中空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误或 panic。
  - 2026-06-07 15:03-19:03 CST 有 8 个 Feishu user turn 与 8 个 assistant final，4 个 Feishu direct 会话最新均以 assistant 收口；`cron_job_runs` 同窗无新增记录，assistant final 污染扫描未命中 FMP 原始错误、空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或 stream disconnect。
  - 2026-06-07 19:02-23:02 CST 有 14 个 Feishu user turn 与 15 个 assistant 记录，7 个 Feishu direct 活跃会话最新均以 assistant 收口；多出的 1 条 assistant 是 daily-limit final/text 双记录，另立 P3 跟踪。`cron_job_runs` 同窗无新增记录，assistant final 污染扫描未命中 FMP 原始错误、空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或 stream disconnect。
  - 2026-06-07 23:02-2026-06-08 03:02 CST 本地 SQLite 只新增 1 个 Feishu user turn 与 1 个 assistant final，成对收口；`cron_job_runs` 同窗无新增记录，assistant final 污染扫描未命中 FMP 原始错误、空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或 stream disconnect。
  - 2026-06-08 03:02-07:02 CST SQLite 没有新增 Feishu / Discord 落库会话或 scheduler 台账记录；`data/runtime/logs/acp-events.log` 同窗有 7 个 Web direct prompt，均以 `stopReason=end_turn` 收口，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误。
  - 2026-06-08 11:01-15:01 CST `data/sessions.sqlite3` 有 9 个 user turn 与 9 个 assistant final，3 个 Feishu direct 会话最新均以 assistant 收口；`cron_job_runs` 同窗无新增记录，assistant final 污染扫描未命中 FMP 原始错误、空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或 stream disconnect。
  - 同窗 `acp-events.log` 有 9 个 Feishu prompt 与 3 个 Web prompt，均以 `stopReason=end_turn` 收口，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误。

## 端到端链路

1. event-engine runtime 定期执行 `poller.fmp.price` 与 `poller.fmp.news`。
2. poller 调用 Financial Modeling Prep quote/news API 获取观察池行情与新闻增量。
3. poller 结果进入 event-engine 的事件候选、digest、告警或后续投研上下文。
4. 最近窗口内 price/news poller 持续请求失败并返回 `items=0`，导致对应增量数据不可用。

## 期望效果

- FMP price/news poller 应在正常网络与有效 key 下持续产出可用行情/新闻增量。
- 单次请求失败应有重试、分批、降级或明确分类；持续失败应被标记为可运维的上游/网络/配置异常，而不是只在 task_runs 中反复记录同一失败。
- event-engine 下游若依赖这类数据，应能感知数据新鲜度不足并避免把缺失增量误当作“无事件”。

## 当前实现效果

- 最近四小时内 quote/news poller 全部失败且 `items=0`，没有观察到恢复样本。
- 后续 2026-06-07 03:01-07:01 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`，持续失败时长继续扩大。
- 后续 2026-06-07 07:00-11:04 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；earnings / macro 也出现同类请求发送失败，说明影响面从 price/news 扩展到更多 FMP API。
- 后续 2026-06-07 11:02-15:02 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours 仍按节奏 `ok`，说明当前证据仍指向 FMP price/news 请求发送链路持续退化。
- 后续 2026-06-07 15:03-19:03 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours 仍按节奏 `ok`，说明持续失败尚未恢复。
- 后续 2026-06-07 19:02-23:02 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours、daily report 与 unified digest scheduler 均有 `ok` 样本，说明失败继续集中在 FMP price/news 请求链路。
- 后续 2026-06-07 23:02-2026-06-08 03:02 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours 仍按节奏 `ok`，说明失败尚未恢复。
- 后续 2026-06-08 03:02-07:02 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours 仍按节奏 `ok`，说明失败尚未恢复。
- 后续 2026-06-08 07:01-11:02 CST 复核窗口内，quote/news poller 在 09:14 CST 前仍失败，09:19 CST 起 price/news 转为连续 `ok + items=0`；当前按“部分恢复待复核”处理，状态暂不关闭。
- 后续 2026-06-08 11:01-15:01 CST 复核窗口内，quote/news poller 曾连续恢复为 `ok + items=0`，当日因此关闭过该缺陷；但 2026-08-05 之后真实运行窗口持续复发，当前状态已回退为 `New/P2`。
- 2026-08-09 10:00-14:01 CST 复核窗口内，`poller.fmp.news` / `stock_news` 继续约每 30 分钟请求发送失败，共 8 条；同窗 heartbeat run / deliver 仍运行，说明 event-engine 未整体停摆。
- 同一 runtime 的 extended-hours poller 仍按节奏运行并返回 `ok`，说明不是调度器完全停止。
- 当前复发窗口未直接表现为用户可见错误、错投或格式污染；影响集中在事件引擎数据摄取链路退化。

## 用户影响

- 用户依赖的实时行情、新闻增量、digest 候选与监控触发可能变旧、变少或漏报。
- 若下游没有严格的新鲜度检查，系统可能把“FMP 数据抓取失败”误解释为“观察池没有新闻/价格变化”。
- 该问题影响功能链路的数据正确性与监控完整性，因此定级为 P2；它没有直接导致本轮用户请求失败、跨用户错投、数据破坏或大面积可见错误，因此不定为 P1。

## 根因判断

- 直接原因是 FMP quote/news HTTP 请求在 poller 层持续 `error sending request`。
- 当前证据不足以确认是本机网络、FMP 上游、key/plan 限制、请求 batch 形态或客户端超时配置导致；2026-06-08 09:19 CST 后链路曾自行恢复，但 2026-08-05 之后再次复发，说明仍缺少稳定修复闭环。
- 该问题不同于已关闭的 `event_engine_price_poller_transient_fetch_failure.md`：本轮不是单 tick 抖动，而是从 2026-06-06 09:29 CST 起持续到最近四小时的 price/news poller 全失败。
- 也不同于历史 `event_engine_price_poller_unbounded_quote_batch.md` 的已修复 batch 拆分问题：本轮错误信息是请求发送失败，尚未证明为 URL path 过长或单 batch 丢弃其它成功 batch。

## 下一步建议

- 先检查 event-engine FMP client 对 `error sending request` 的错误分类、重试和超时设置，确认是否需要按网络/上游/配置分别记录 `failure_kind`。
- 对 price/news poller 增加连续失败阈值告警，避免长时间只写 `task_runs` 而没有运行态告警。
- 后续巡检继续统计 FMP price/news poller；若连续一个完整巡检窗口恢复为 `ok` 且无下游新鲜度投诉，再考虑从 `New` 调整为 `Closed`。
- 检查失败期间是否仍有其它行情源或缓存被下游使用；若没有，应在 digest / alert 生成前显式注入数据新鲜度缺口。
- 对比 FMP quote/news 与 extended-hours 的请求域名、batch 大小、timeout、key 使用路径，定位为何 extended-hours 仍 ok 而 quote/news 持续失败。

## 最新运行态复核（2026-08-09 18:03 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-09 14:02-18:03 CST。
  - `poller.fmp.news` / `stock_news` 继续每约 30 分钟请求发送失败，共 8 条 `poll failed: FMP 请求失败: error sending request`。
  - 同窗 `poller.fmp.price` 与 `poller.fmp.extended_hours` 仍有 `poller ok` 样本，且 heartbeat / scheduler 继续运行，说明不是 event-engine 整体停摆。
- 本轮判断
  - 最新证据仍落在 FMP news 数据摄取链路持续退化的既有 P2 范围内，不是新的独立根因。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-16 10:02 CST）

- `data/runtime/task_runs.2026-08-16.jsonl`
  - 巡检窗口：2026-08-16 06:01-10:02 CST。
  - `poller.fmp.news failed=8`，错误仍为脱敏后的 `stock_news` 请求发送失败。
  - 同窗新增 `poller.fmp.earnings failed=2` 与 `poller.fmp.macro failed=2`，同时 `poller.fmp.price ok=16`、`poller.fmp.extended_hours ok=8`，说明 event-engine runtime 未整体停摆。
- `data/logs/hone-console-page-source.log`
  - 同窗可见 FMP 请求失败信号，包含 `poller.fmp.news`、`poller.fmp.earnings`、`poller.fmp.macro`、`fmp.sec_filings`、`fmp.analyst_grade` 等请求发送失败；日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
- 本轮判断
  - 最新证据仍落在 FMP news / earnings / macro 数据摄取链路持续退化的既有 P2 范围内，不是新的独立根因。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-11 06:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-11 02:02-06:02 CST。
  - 同窗仍检出 17 条 `poller.fmp` 失败信号，集中在 FMP price/news/extended-hours 相关批量请求；未见对应 event-engine 整体停摆。
  - 同窗 heartbeat 仍有 `run_start=97`、`run_finish=98`、`deliver=57`，说明 scheduler 与 event-engine 其它链路仍在推进。
- 本轮判断
  - FMP poller 仍存在持续请求失败，行情与新闻增量可能退化；但运行态不是全链路不可用。
  - 严重等级维持功能性 `P2 / New`，非 P1。

## 最新运行态复核（2026-08-10 02:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-09 22:03-2026-08-10 02:02 CST。
  - `poller.fmp.news` / `stock_news` 继续每约 30 分钟请求发送失败，共 8 条 `poll failed: FMP 请求失败: error sending request`。
  - 同窗 `poller.fmp.price` 与 `poller.fmp.extended_hours` 仍有 `poller ok` 样本，且 heartbeat / scheduler 继续运行，说明不是 event-engine 整体停摆。
- 本轮判断
  - 最新证据仍落在 FMP news 数据摄取链路持续退化的既有 P2 范围内，不是新的独立根因。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-11 10:00 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-11 06:00-10:00 CST。
  - 同窗检出 12,496 条 `FMP 请求失败: error sending request`，其中 3,120 条带 `poller="fmp.*"` 结构化字段，集中在 sec filings、extended-hours prev close 等 FMP 批量请求。
  - 同窗 heartbeat 仍有 `run_start=108`、`run_finish=108`、`deliver=70`，说明 scheduler / event-engine 其它链路仍在推进，不是整体停摆。
- 本轮判断
  - FMP poller 仍存在持续请求失败，行情、新闻或公司行动增量可能退化；但运行态不是全链路不可用。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-11 22:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-11 18:01-22:02 CST。
  - 同窗检出 15,322 条 `FMP 请求失败` / `poller.fmp` 失败信号，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.news` / `stock_news` 与若干 FMP 事件源请求失败。
  - 日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
  - 同窗 heartbeat 继续运行：`run_start=109`、`run_finish=109`、`deliver=66`，说明不是 event-engine / scheduler 整体停摆。
- 本轮判断
  - FMP poller 仍存在持续请求失败，行情、新闻或公司行动增量可能退化；但运行态不是全链路不可用。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-11 14:01 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-11 10:00-14:01 CST。
  - `poller.fmp.news` / `stock_news` 继续每约 30 分钟请求发送失败，共 8 条 `poll failed: FMP 请求失败: error sending request`。
  - 同窗 heartbeat 仍有 `run_start=96`、`run_finish=106`、`deliver=65`，说明 scheduler / event-engine 其它链路仍在推进，不是整体停摆。
- 本轮判断
  - 最新证据仍落在 FMP news 数据摄取链路持续退化的既有 P2 范围内，不是新的独立根因。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-11 18:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-11 14:00-18:02 CST。
  - 同窗检出 10,928 条 `FMP 请求失败: error sending request` 与 34 条 `poller.fmp` 信号，集中在 `poller.fmp.news` / `stock_news` 以及 extended-hours `prev_close` 批量请求失败。
  - 同窗仍有 `poller.fmp.price` / `poller.fmp.extended_hours` 的部分 `poller ok` 样本，且 heartbeat 继续运行：`run_start=97`、`run_finish=102`、`deliver=67`，说明不是 event-engine 整体停摆。
- 本轮判断
  - FMP poller 仍存在持续请求失败，行情、新闻或公司行动增量可能退化；但运行态不是全链路不可用。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-13 22:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-13 18:01-22:02 CST。
  - 同窗检出 15,322 条 `FMP 请求失败`，集中在 extended-hours `prev_close` 批量请求、`poller.fmp.news` / `stock_news` 与 FMP price 批量请求失败。
  - 日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
- `data/runtime/task_runs.2026-08-13.jsonl`
  - 同窗 `poller.fmp.news failed=8`、`poller.fmp.price failed=2`、`poller.fmp.price ok=14`、`poller.fmp.extended_hours ok=8`。
  - 同窗 heartbeat 继续运行：`run_start=72`、`run_finish=72`、`deliver=37`，说明不是 event-engine / scheduler 整体停摆。
- 本轮判断
  - FMP poller 仍存在持续请求失败，行情、新闻或公司行动增量可能退化；但运行态不是全链路不可用。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-14 02:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-13 22:00-2026-08-14 02:02 CST。
  - 同窗检出 229 条 `FMP 请求失败` / `poller.fmp` 失败信号，集中在 `poller.fmp.price` quote batch 与 `poller.fmp.news` / `stock_news` 请求发送失败。
  - 日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
- `data/runtime/task_runs.2026-08-13.jsonl`
  - 同窗 `poller.fmp.price failed=17`、`poller.fmp.news failed=8`、`poller.fmp.extended_hours ok=9`。
  - 同窗 heartbeat 继续运行：`run_start=64`、`run_finish=72`、`deliver=44`，说明不是 event-engine / scheduler 整体停摆。
- 本轮判断
  - FMP poller 仍存在持续请求失败，行情、新闻或公司行动增量可能退化；但运行态不是全链路不可用。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-14 14:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-14 10:02-14:02 CST。
  - 同窗 `poll failed: FMP 请求失败` 共 8 条，集中在 `poller.fmp.news` / `stock_news` 请求发送失败。
  - 日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
- `data/runtime/task_runs.2026-08-14.jsonl`
  - 同窗 `poller.fmp.news failed=8`、`poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`。
  - 同窗 heartbeat 继续运行：`run_start=64`、`run_finish=64`、`deliver=31`，说明不是 event-engine / scheduler 整体停摆。
- 本轮判断
  - FMP news poller 仍存在持续请求失败，新闻增量可能退化；但行情 price / extended-hours poller 仍有成功样本，运行态不是全链路不可用。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-14 18:02 CST）

- `data/runtime/task_runs.2026-08-14.jsonl`
  - 巡检窗口：2026-08-14 14:02-18:02 CST。
  - `poller.fmp.news failed=8`，失败时间从 UTC `2026-08-14T06:04:05Z` 到 `2026-08-14T09:34:05Z`，错误仍为脱敏后的 `stock_news` 请求发送失败。
  - 同窗 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`，`internal.daily_report` / `internal.unified_digest_scheduler` 仅周期性 `skipped`，说明 event-engine runtime 未整体停摆。
- `data/logs/hone-console-page-source.log`
  - 同窗检出 `FMP 请求失败=10929`，集中在 `poller.fmp.news` 与 extended-hours prev_close 批量请求；日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
- 本轮判断
  - 最新证据仍落在 FMP news / extended-hours 数据摄取链路持续退化的既有 P2 范围内，不是新的独立根因。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-14 22:02 CST）

- `data/runtime/task_runs.2026-08-14.jsonl`
  - 巡检窗口：2026-08-14 18:02-22:02 CST。
  - `poller.fmp.news failed=8`，错误仍为脱敏后的 `stock_news` 请求发送失败。
  - 同窗新增 `poller.fmp.price failed=2`，集中在 quote batch 请求发送失败；同时 `poller.fmp.price ok=14`、`poller.fmp.extended_hours ok=8`，说明 event-engine runtime 未整体停摆。
- `data/logs/hone-console-page-source.log`
  - 同窗检出 `FMP 请求失败=15322`，集中在 extended-hours prev_close 批量请求、`poller.fmp.news` 与 FMP price 批量请求；日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
- 本轮判断
  - 最新证据仍落在 FMP news / price / extended-hours 数据摄取链路持续退化的既有 P2 范围内，不是新的独立根因。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。

## 最新运行态复核（2026-08-16 02:02 CST）

- `data/runtime/task_runs.2026-08-15.jsonl`
  - 巡检窗口：2026-08-15 22:00-2026-08-16 02:02 CST。
  - `poller.fmp.news failed=8`，错误仍为脱敏后的 `stock_news` 请求发送失败。
  - 同窗 `poller.fmp.price ok=17`、`poller.fmp.extended_hours ok=9`，说明 event-engine runtime 未整体停摆。
- `data/logs/hone-console-page-source.log`
  - 同窗可见 8 条 `poll failed: FMP 请求失败`，集中在 `poller.fmp.news` / `stock_news`；日志中的 FMP URL 仍写成 `apikey=<redacted>`，未见用户可见原始 key 外泄。
- 本轮判断
  - 最新证据仍落在 FMP news 数据摄取链路持续退化的既有 P2 范围内，不是新的独立根因。
  - 本窗未见用户可见 FMP 原始错误、错投或全渠道不可用；状态维持 `New`、严重等级维持 `P2`，非 P1。
