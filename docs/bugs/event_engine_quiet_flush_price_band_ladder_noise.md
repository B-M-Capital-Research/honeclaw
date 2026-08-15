# Bug: quiet_flush 复活整串价格档阶梯，早报一半行数是同一波行情

- **发现时间**: 2026-08-15 CST（MU 评级事故排查时的全量推送审计发现）
- **Bug Type**: Business Error（推送噪声 / 信息冗余）
- **严重等级**: P2
- **状态**: Fixed
- **修复记录**:
  - `2026-08-15 CST` 状态更新为 `Fixed`，两项修复：
  - **quiet_flush 阶梯合流**（`digest/coalesce.rs` 新模块，接入
    `unified_digest/scheduler.rs::run_quiet_flush`）：同 (symbol, 交易日, 方向)
    的 band 组只留 |bps| 最大一条；同 (symbol, 交易日) 存在 `price_close` 收盘
    事件时 band 并入收盘行，标题追加「（盘中曾跨 +X% 档）」注记。冲高回落日
    up/down 两组都并入注记。被合并行走 omitted 审计（`status=omitted`）。
  - **开盘 band High 批内合流**（`router/config.rs` + `dispatch.rs` +
    `pipeline.rs`）：`process_events` 激活批模式后，同一批 poll 内同 actor 的
    盘中 band High ≥3 条合成一条「⚡ 盘中集体异动 · N 只」汇总消息，<3 条维持
    逐条即时；直接调 `dispatch` 的调用方（测试/嵌入）行为不变。每条成员事件
    仍单独落 `sink=sent` 审计。
  - 回归测试：SNDK 2026-08-13（5 band + close → 1 行带注记）、NBIS 2026-08-12
    （9 band → 1 行）、冲高回落双向注记、2026-07-30 开盘 7 只集体跳空 → 1 条
    合并消息、低于阈值逐条发送、未激活批模式即时发送。
  - 离线回放验收（`tests::replay_push_quality_audit`）：全量 events.jsonl 重放
    后 band 行 914 → 530（含 close），294 条收盘行带盘中注记，且断言合流后
    不存在任何同 (symbol, 日, 方向) 的重复 band 行。
  - 验证：`cargo test -p hone-event-engine` 601 通过。
- **证据来源**:
  - `data/events.sqlite3` `delivery_log`（2026-07-28 → 2026-08-15）：实际送达
    的 141 条 band 行中 70 条（50%）是同一波行情的重复阶梯；26 个 symbol-day
    同时收到 band 行和收盘行。
  - 最恶劣样本：2026-08-14 07:30（北京）早报 10 条里 6 条是 SNDK 阶梯
    （+8%/+10%/+12%/+14%/+16% 档 + 收盘 +13.67%）；2026-08-12 NBIS 一条 digest
    出现 9 行（+14% 至 +34% 档）。
  - 根因：用户勿扰时段（北京 23:00–07:30）覆盖美股主交易时段，全部盘中 band
    High 被 `quiet_held`；`run_quiet_flush` 从 store 逐条复活，绕过
    `DigestBuffer::price_digest_key` 的同 symbol 同日 latest-wins 去重（该去重
    只作用于 Medium/Low 入 buffer 路径），且 High 不受 curation per-symbol cap
    约束。
  - 开盘连环 DM：73 条即时 DM 中 54 条集中在美股开盘后一小时；2026-07-30
    13:33:57–13:34:05 UTC 十秒内连发 7 条（7 只自选股集体跳空各一条）。
