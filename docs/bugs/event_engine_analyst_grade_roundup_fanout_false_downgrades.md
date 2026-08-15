# Bug: FMP 多股汇总文扇出导致成组假评级下调 High 推送（MU 2026-08-14 事故）

- **发现时间**: 2026-08-15 13:20 CST（用户收到 Discord 推送后质疑）
- **Bug Type**: Data Quality / Business Error
- **严重等级**: P1（向用户推送成组的错误看空信号，可直接影响投资判断）
- **状态**: Fixed
- **修复记录**:
  - `2026-08-15 CST` 状态更新为 `Fixed`：`pollers/analyst_grade.rs` 新增
    `collapse_roundup_fanout` 源头防御——标题命中多股汇总模式（`top analyst
    calls` / `top N stock calls` / `Buy/Sell:` / `X upgraded, Y downgraded` 等）
    或同一 `newsURL` 扇出 ≥3 条评级记录的组，坍缩为**一条 Medium 汇总事件**
    （`grade_roundup:{ticker}:{published}`），原始 rows 保留在 payload 供核查，
    摘要明确标注「多股汇总文，评级归属存在跨股错配可能」。
  - 配套 `event::is_analyst_roundup_summary` 谓词：汇总事件不允许被
    `immediate_kinds` 提升为即时推送（`router/policy.rs`），也不进 digest floor
    prepend（`unified_digest/floor.rs`）。
  - 计数规则：`prev == new` 的脏 upgrade/downgrade 行计入「重申」而非上调/下调
    （如 Bernstein "upgrade" Outperform→Outperform）。
  - 回归测试：`mu_2026_08_14_roundup_incident_produces_no_high_events` 用事故
    当日 13 条真实 rows 固化——两篇汇总文 → 两条 Medium 汇总、0 条 High；
    `genuine_single_stock_actions_are_untouched` 保证单公司真实下调仍 High。
  - 离线回放验收（`tests::replay_push_quality_audit`，`HONE_REPLAY_EVENTS_JSONL`
    指向真实 `data/events.jsonl`）：评级 High 事件 38 → 7，其中汇总文来源残留
    0，真实单公司 High（如 GEV conviction list）全部保留。
  - 验证：`cargo test -p hone-event-engine` 601 通过。
- **证据来源**:
  - `data/events.jsonl` / FMP 线上 `v4/upgrades-downgrades?symbol=MU`（297 条
    历史）交叉核对：
    - 2026-08-14 全部 13 条 MU 看空记录仅来自两篇 TheFly 汇总文
      （`news_get.php?id=4411792` / `4411877`）；同批 rows 在 CSCO/AMD/NVDA
      名下为 0 条——FMP 把汇总文里**其他公司**的券商动作全部错配到标题
      第一只票（MU）。
    - 文章标题自证：《Micron upgraded, Cisco downgraded》里 Micron 是被上调
      的，却被解析出 3 条 MU 下调。
    - 同批数据内部矛盾：Wells Fargo 13:46 "upgrade" Overweight→Overweight，
      15:22 又 Overweight→Underweight；Jefferies 13:46 首评 Buy，15:22
      Buy→Underperform。
    - 同日单公司标题记录全部正面：New Street 上调 Buy、Cantor/Barclays 目标价
      $2,000、Raymond James $1,500 等。
  - `data/events.sqlite3` `delivery_log`：2026-08-14 23:30 UTC quiet_flush 向
    两个 Discord actor 各送达 6 条 MU 假 High 下调；审计窗口（07-28 → 08-15）
    内实际送达的 13 条独立 High 评级事件中 12 条来自这两类汇总文标题
    （错误率 92%）。同源历史事故：2026-07-31 GOOGL 6 条、DELL 9 条、AMD 8 条。
  - 既有防线为何全部失效：`is_noop_analyst_grade` 只拦 prev==new；dispatch 的
    同 newsURL fanout 冷却只看 sink 实发记录，quiet_hours hold →
    `run_quiet_flush` 复活路径完全绕过 dispatch 侧防线。
