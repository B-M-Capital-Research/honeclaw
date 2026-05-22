# Bug: Heartbeat 使用 `mimo-v2.5-pro` 时批量触发 `HTTP 429 quota exhausted` 并漏发

- **发现时间**: 2026-05-20 19:04 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)

## 证据来源

- GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44) 与 bug 台账记录：最近四小时 heartbeat 任务批量命中 `mimo-v2.5-pro` 上游 `HTTP 429` / `quota exhausted`，多条监控检查落成 `execution_failed + skipped_error + delivered=0`。
- 受影响范围覆盖价格破位、持仓财报、重大新闻、板块关键事件、观察池等多个 heartbeat job；同窗直聊会话仍能正常收口，故障集中在 heartbeat provider quota / rate-limit 路径。
- 本轮修复不依赖当前机器生产日志、线上健康检查或真实投递状态；判断与验证基于 issue 摘要、现有 heartbeat 代码、配置解析和本地回归。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-22 23:01 CST` 复核，最近四小时窗口 `2026-05-22T19:01:33+08:00` 到 `2026-05-22T23:01:39+08:00` 内继续新增 `105` 条 heartbeat `execution_failed + skipped_error + delivered=0` 的 quota 失败，其中 Feishu `84` 条、Web `21` 条。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`：Feishu heartbeat 多为 `LLM 错误: limitation: quota exhausted (code: 429)`，Web heartbeat 多为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`。
  - 受影响范围：Feishu `12` 个 heartbeat job、Web `3` 个 heartbeat job；Feishu 失败集中在 19:13-22:14 CST，Web 失败集中在 19:30-22:30 CST。
  - 同窗还有 `84` 条 Feishu heartbeat `running + pending` started 残留、`31` 条普通 Feishu scheduler started 残留，以及 `32` 条 Feishu、`5` 条 Web 普通 scheduler `completed + sent + delivered=1` 终态。
  - 22:39 / 22:52 CST Feishu scheduler 启动回收 `5134` 条历史 `running/pending` row 为 `execution_failed + send_failed`，`detail_json.phase=recovered_stale_pending`、`detail_json.recovered_by=feishu_scheduler_startup`；这是既有 stale-pending 回收逻辑在生效，不作为本单的新失败根因。
  - 23:00 CST 以后又出现 `MiniMax-M2.7-highspeed` 输出 `<think>` / plain text 导致的非 quota 结构化失败，已回写 `scheduler_heartbeat_unknown_status_silent_skip.md` 并将该 P2 从 `Fixed` 重新打开。
  - 当前 HEAD 已有 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类修复，本轮继续按旧/未确认部署运行态证据处理，不把状态从 `Fixed` 回退为 `New`，也不重复创建 Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)。
  - `2026-05-22 19:02 CST` 复核，最近四小时窗口 `2026-05-22T15:02:00+08:00` 到 `2026-05-22T19:02:00+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`，其中 Feishu `96` 条、Web `24` 条。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`：Feishu heartbeat 多为 `LLM 错误: limitation: quota exhausted (code: 429)`，Web heartbeat 多为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`。
  - sqlite `detail_json.failure_kind` 中 Feishu 侧 `96` 条仍为空，Web 侧 `24` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现；同窗未再出现 `Param Incorrect` 或 `reasoning_content` 兼容错误。
  - 同窗还有 `96` 条 Feishu heartbeat `running + pending` started 残留；本窗无普通 scheduler 运行记录。
  - 会话侧按消息时间统计 `9` 个 user turn 与 `9` 个 assistant final；Feishu / Web 直聊均有收口，assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable`、`panic`、`index out of bounds`、`Searching the Web`、`本地命令`、`内容可能不完整` 或 provider 原始 `quota exhausted`。
  - 当前 HEAD 已有 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类修复，本轮继续按旧/未确认部署运行态证据处理，不把状态从 `Fixed` 回退为 `New`，也不重复创建 Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)。
  - `2026-05-22 15:02 CST` 复核，最近四小时窗口 `2026-05-22T11:00:00+08:00` 到 `2026-05-22T15:02:15+08:00` 内继续新增 `123` 条 heartbeat `execution_failed + skipped_error + delivered=0`，其中 Feishu `96` 条、Web `27` 条。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`：Feishu heartbeat 多为 `LLM 错误: limitation: quota exhausted (code: 429)`，Web heartbeat 多为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`。
  - sqlite `detail_json.failure_kind` 中 Feishu 侧 `96` 条仍为空，Web 侧 `27` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现。
  - 同窗还有 `108` 条 Feishu heartbeat `running + pending` started 残留；普通 scheduler 同窗有 `1` 条 Feishu、`1` 条 Web `completed + sent + delivered=1`。
  - 会话侧按 `datetime(...)` 归一化统计 `26` 个 user turn 与 `26` 个 assistant final；Feishu / Web 直聊均有收口，assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable`、`panic`、`index out of bounds`、`Searching the Web`、`本地命令`、`内容可能不完整` 或 provider 原始 `quota exhausted`。
  - 当前 HEAD 已有 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类修复，本轮继续按旧/未确认部署运行态证据处理，不把状态从 `Fixed` 回退为 `New`，也不重复创建 Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)。
  - `2026-05-22 11:01 CST` 复核，最近四小时窗口 `2026-05-22T07:01:02+08:00` 到 `2026-05-22T11:01:16+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`，其中 Feishu `96` 条、Web `24` 条。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`：Feishu heartbeat 多为 `LLM 错误: limitation: quota exhausted (code: 429)`，Web heartbeat 多为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`。
  - sqlite `detail_json.failure_kind` 中 Feishu 侧 `96` 条仍为空，Web 侧 `24` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现。
  - 同窗还有 `96` 条 Feishu heartbeat `running + pending` started 残留；普通 scheduler 同窗有 `16` 条 Feishu、`2` 条 Web、`1` 条 Discord `completed + sent + delivered=1`。
  - 会话侧按消息时间统计 `43` 个 user turn 与 `44` 个 assistant final；Feishu / Web / Discord 直聊和普通 scheduler 均有收口，assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable`、`panic`、`index out of bounds` 或 provider 原始 `quota exhausted`。
  - 当前 `main` 已有 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类修复，本轮继续按旧/未确认部署运行态证据处理，不把状态从 `Fixed` 回退为 `New`，也不重复创建 Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)。
  - `2026-05-22 07:05 CST` 复核，最近四小时窗口 `2026-05-22T03:01:31+08:00` 到 `2026-05-22T07:03:03+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`，其中 Feishu `96` 条、Web `24` 条。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`：Feishu heartbeat 多为 `LLM 错误: limitation: quota exhausted (code: 429)`，Web heartbeat 多为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`。
  - sqlite `detail_json.failure_kind` 中 Feishu 侧 `96` 条仍为空，Web 侧 `24` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现。
  - 同窗还有 `96` 条 Feishu heartbeat `running + pending` started 残留；普通 scheduler 同窗有 `5` 条 Feishu `completed + sent + delivered=1`，且 07:00 刚触发的普通 scheduler 已在 07:03 CST 成功送达。
  - 会话侧按消息时间统计 `20` 个 user turn 与 `19` 个 assistant final；Feishu / Web 直聊均有 assistant final 收口，assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable`、`panic`、`index out of bounds` 或 provider 原始 `quota exhausted`。
  - 当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮继续按旧/未确认部署运行态证据处理，不把状态从 `Fixed` 回退为 `New`，也不重复创建 Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)。
  - `2026-05-22 03:03 CST` 复核，最近四小时窗口 `2026-05-21T23:03:00+08:00` 到 `2026-05-22T03:03:00+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`，其中 Feishu `96` 条、Web `24` 条。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`：Feishu heartbeat 多为 `LLM 错误: limitation: quota exhausted (code: 429)`，Web heartbeat 多为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`。
  - 同窗还有 `96` 条 Feishu heartbeat `running + pending` started 残留；普通 scheduler 同窗有 `4` 条 Feishu `completed + sent + delivered=1`，且本轮无新的普通 scheduler target resolution 失败。
  - 会话侧按消息时间统计 `39` 个 user turn 与 `39` 个 assistant final；assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable` 或 provider 原始 `quota exhausted`。
  - 当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮继续按旧/未确认部署运行态证据处理，不把状态从 `Fixed` 回退为 `New`，也不重复创建 Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)。
  - `2026-05-21 23:03 CST` 复核，最近四小时窗口 `2026-05-21T19:03:00+08:00` 到 `2026-05-21T23:03:00+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`，其中 Feishu `96` 条、Web `24` 条。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`：Feishu heartbeat 多为 `LLM 错误: limitation: quota exhausted (code: 429)`，Web heartbeat 多为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`。
  - 同窗还有 `96` 条 Feishu heartbeat `running + pending` started 残留；普通 scheduler 同窗有 `31` 条 Feishu 和 `3` 条 Web `completed + sent + delivered=1`，另有 1 条 Feishu 普通 scheduler target resolution 失败已归入 `feishu_scheduler_target_resolution_failed.md`。
  - 当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮继续按旧/未确认部署运行态证据处理，不把状态从 `Fixed` 回退为 `New`，也不重复创建 Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)。
  - `2026-05-21 19:03 CST` 复核，最近四小时窗口 `2026-05-21T15:14:06+08:00` 到 `2026-05-21T19:00:04+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`；当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮将该证据作为当前机器旧/未确认部署运行态线索，不把状态从 `Fixed` 回退为 `New`。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`，覆盖 `15` 条 heartbeat job；其中 `96` 条为 `LLM 错误: limitation: quota exhausted (code: 429)`，`24` 条为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`，本轮 sqlite `detail_json` 中的 `failure_kind` 仍有 `96` 条为空、`24` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现。
  - 同窗还有 `96` 条 heartbeat `running + pending` started 残留，没有普通 scheduler 运行记录；说明本轮故障仍集中在 heartbeat provider quota 链路。
  - 会话侧按消息时间统计 `7` 个 user turn 与 `8` 个 assistant final；Feishu / Web 直聊均有 assistant final 收口。assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable` 或 provider 原始 `quota exhausted`。
  - 本轮是同一根因 / 同一影响范围的运行态复核，不新建重复缺陷；已有 GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)，不重复创建。
  - `2026-05-21 15:02 CST` 复核，最近四小时窗口 `2026-05-21T11:14:00+08:00` 到 `2026-05-21T15:00:04+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`；当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮将该证据作为当前机器旧/未确认部署运行态线索，不把状态从 `Fixed` 回退为 `New`。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`，覆盖 `15` 条 heartbeat job；其中 `96` 条为 `LLM 错误: limitation: quota exhausted (code: 429)`，`24` 条为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`，本轮 sqlite `detail_json` 中的 `failure_kind` 仍有 `96` 条为空、`24` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现。
  - 同窗还有 `96` 条 heartbeat `running + pending` started 残留、`1` 条普通 scheduler `running + pending` started 残留，以及 `1` 条普通 scheduler `completed + sent + delivered=1` 终态；说明直聊 / 普通 scheduler 主链路没有被同一问题整体阻断。
  - 会话侧按消息时间统计 `20` 个 user turn 与 `20` 个 assistant final；Feishu / Web 直聊和普通 scheduler 均有 assistant final 收口。assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable` 或 provider 原始 `quota exhausted`。
  - 本轮是同一根因 / 同一影响范围的运行态复核，不新建重复缺陷；已有 GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)，不重复创建。
  - `2026-05-21 11:02 CST` 复核，最近四小时窗口 `2026-05-21T07:02:28+08:00` 到 `2026-05-21T11:02:00+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`；当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮将该证据作为当前机器旧/未确认部署运行态线索，不把状态从 `Fixed` 回退为 `New`。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`，覆盖 `15` 条 heartbeat job；其中 `96` 条为 `LLM 错误: limitation: quota exhausted (code: 429)`，`24` 条为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`，本轮 sqlite `detail_json` 中的 `failure_kind` 仍有 `96` 条为空、`24` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现。
  - 同窗还有 `96` 条 heartbeat `running + pending` started 残留、`15` 条普通 scheduler `running + pending` started 残留，以及 `18` 条普通 scheduler `completed + sent + delivered=1` 终态；说明直聊 / 普通 scheduler 主链路没有被同一问题整体阻断。
  - 会话侧按消息时间统计 `50` 个 user turn 与 `50` 个 assistant final；Feishu / Web / Discord 直聊和普通 scheduler 均有 assistant final 收口。assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、飞书标签、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable` 或 provider 原始 `quota exhausted`。
  - 本轮是同一根因 / 同一影响范围的运行态复核，不新建重复缺陷；已有 GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)，不重复创建。
  - `2026-05-21 07:03 CST` 复核，最近四小时窗口 `2026-05-21T03:03:00+08:00` 到 `2026-05-21T07:03:00+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`；当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮将该证据作为当前机器旧/未确认部署运行态线索，不把状态从 `Fixed` 回退为 `New`。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`，覆盖 `15` 条 heartbeat job；其中 `96` 条为 `LLM 错误: limitation: quota exhausted (code: 429)`，`24` 条为 `LLM 错误: upstream HTTP 429: quota exhausted (code: 429)`，本轮 sqlite `detail_json` 中的 `failure_kind` 仍为空，符合未重启到当前 HEAD 分类修复的表现。
  - 同窗还有 `96` 条 heartbeat `running + pending` started 残留、`6` 条普通 scheduler `running + pending` started 残留，以及 `7` 条普通 scheduler `completed + sent + delivered=1` 终态；说明直聊 / 普通 scheduler 主链路没有被同一问题整体阻断。
  - 会话侧按消息时间统计 `9` 个 user turn 与 `9` 个 assistant final；Feishu / Web 直聊和普通 scheduler 均有 assistant final 收口。assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、飞书标签、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable` 或 provider 原始 `quota exhausted`。
  - 本轮是同一根因 / 同一影响范围的运行态复核，不新建重复缺陷；已有 GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)，不重复创建。
  - `2026-05-21 03:02 CST` 复核，最近四小时窗口 `2026-05-20T23:02:00+08:00` 到 `2026-05-21T03:02:00+08:00` 内继续新增 `120` 条 heartbeat `execution_failed + skipped_error + delivered=0`；当前 `main` 已在 `d4d45e2` 修复 OpenAI-compatible 多 key fallback 与 heartbeat 429 分类，本轮将该证据作为当前机器旧/未确认部署运行态线索，不把状态从 `Fixed` 回退为 `New`。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`，覆盖同一批 `15` 条 heartbeat job；其中 `failure_kind` 仍有 `96` 条为空、`24` 条为 `provider_http_error`，符合旧运行态或未重启到当前 HEAD 的表现。
  - 同窗还有 `96` 条 heartbeat `running + pending` started 残留、`4` 条普通 scheduler `running + pending` started 残留、`3` 条普通 scheduler `noop + skipped_noop` 终态与 `1` 条普通 scheduler `completed + sent + delivered=1` 终态；说明直聊 / 普通 scheduler 主链路没有被同一问题整体阻断。
  - 会话侧按消息时间统计 `13` 个 user turn 与 `9` 个 assistant final；唯一最新停在 user 的 Feishu direct 会话对应 3 条每日动态 scheduler 触发，终态已分别落为 `noop + skipped_noop`，不是直聊未回复。
  - assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、飞书标签、compact marker、`reasoning_content`、`Param Incorrect`、`Resource temporarily unavailable` 或 provider 原始 `quota exhausted`。
  - 本轮是同一根因 / 同一影响范围的运行态复核，不新建重复缺陷；已有 GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)，不重复创建。
  - `2026-05-20 23:02 CST` 复核，最近四小时窗口 `2026-05-20T19:00:00+08:00` 到 `2026-05-20T23:02:00+08:00` 内继续新增 `123` 条 heartbeat `execution_failed + skipped_error + delivered=0`；远端最新 `main` 已在 20:06 CST 修复代码路径，本轮将该证据作为当前机器运行态 / 部署复核线索，不把状态从 `Fixed` 回退为 `New`。
  - 错误仍集中为 `HTTP 429` / `quota exhausted`，覆盖同一批 `15` 条 heartbeat job；其中 `光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒` 各新增 `9` 条 `upstream HTTP 429: quota exhausted`，其余 `12` 条 job 各新增 `8` 条 `limitation: quota exhausted`。
  - 同窗还有 `108` 条 heartbeat `running + pending` started 残留，以及 `32` 条普通 scheduler `running + pending` started 残留；普通 scheduler 同窗另有 `34` 条 `completed + sent + delivered=1` 终态，说明直聊 / 普通定时投递主链路没有被同一问题整体阻断。
  - 会话侧按消息时间统计 `52` 个 user turn 与 `52` 个 assistant final，未发现孤立 user turn；assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、飞书标签、compact marker、`reasoning_content`、`Param Incorrect` 或 provider 原始 `quota exhausted`。
  - 本轮是同一根因 / 同一影响范围的运行态复核，不新建重复缺陷；已有 GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)，不重复创建。
  - 最近四小时窗口 `2026-05-20T15:02:00+08:00` 到 `2026-05-20T19:04:00+08:00` 内，heartbeat 任务新增 `100` 条 `execution_failed + skipped_error + delivered=0`。
  - 错误统一为 `mimo-v2.5-pro` 上游 `HTTP 429` / `quota exhausted`，其中 `21` 条已有 `detail_json.failure_kind=provider_http_error`，另有 `79` 条旧形态未写入 failure_kind。
  - 受影响 job 覆盖 `15` 条 heartbeat：`光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`全天原油价格3小时播报`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`heartbeat_绿田机械基本面跟踪`。
  - 同窗还有 `93` 条 heartbeat `running + pending` started 残留，另有 `5` 条 heartbeat 正常 `noop + skipped_noop`。
- `data/runtime/logs/web.log.2026-05-20`
  - `19:00:33-19:03:57 CST` 连续出现 `Rate limited: Too many requests` 与 `Rate limited: quota exhausted`。
  - 同窗有 `mimo-v2.5-pro` transport retry 记录，随后 heartbeat 台账继续落成 `skipped_error`。
- `data/runtime/logs/hone-feishu.runtime-recovery.log`
  - `2026-05-20T11:00:16Z` 起密集记录上游 rate limit / quota exhausted。
- 会话质量对照：
  - 最近四小时按消息时间统计 `49` 个 user turn 与 `49` 个 assistant final，未发现孤立 user turn。
  - assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、飞书标签、compact marker、`reasoning_content`、`Param Incorrect` 或 `Resource temporarily unavailable`。
  - 说明本轮新故障集中在 heartbeat provider quota 链路，而不是直聊回复结构污染或全局会话收口失败。
- 去重检查：
  - `scheduler_heartbeat_openrouter_402_credit_exhaustion_skips_alerts.md` 覆盖的是 OpenRouter `HTTP 402` / token budget / credits 不足，当前状态为 `Fixed`。
  - `scheduler_heartbeat_mimo_param_incorrect_batch_failures.md` 覆盖的是同一 `mimo-v2.5-pro` 的 `HTTP 400 Param Incorrect` / `reasoning_content` transcript 兼容问题，当前状态为 `Fixed`。
  - 本单是 `mimo-v2.5-pro` 在当前真实窗口里触发 `HTTP 429 quota exhausted`，状态码、直接原因和最新证据均不同，因此新建独立缺陷。

## 端到端链路

1. Heartbeat scheduler 在半点 / 整点窗口批量触发多条监控 job。
2. 公共 heartbeat runner 调用 auxiliary profile，对应 OpenAI-compatible provider `mimo-v2.5-pro`。
3. 上游返回 `HTTP 429` / `quota exhausted`。
4. `llm.providers.<name>.api_keys` 虽然支持配置多个 key，但非 OpenRouter provider 解析时只取第一把 key。
5. 第一把 key 被上游拒绝后，本地没有尝试同 provider 的后续 key，整轮 heartbeat 直接失败；失败分类也未把 `HTTP 429` / `rate limit exceeded` 稳定归入 quota exhaustion。

## 期望效果

- 非 OpenRouter 的 OpenAI-compatible provider 应和 OpenRouter 一样支持多 key 顺序 fallback。
- 单 key 的 429 / quota / rate-limit 失败不应在已配置备用 key 时压垮整轮 heartbeat。
- 如果所有 key 都不可用，heartbeat 仍应失败并保留可观测的 `provider_quota_exhausted` 分类，而不是伪装为 noop 或泛化 HTTP 错误。

## 用户影响

- 这是功能性 bug，不是质量性 bug。
- 用户可能错过价格破位、重大事件、持仓财报、板块关键事件和观察池等自动监控提醒。
- 定级为 `P1`：批量 heartbeat 执行失败直接影响自动告警送达链路；虽然直聊未受影响，但自动监控主功能在该 provider 路径上出现批量失效。

## 根因判断

- 这是可控代码缺陷，不是要为某次外部 429 写特判。
- 现有配置结构已经有 `llm.providers.<name>.api_key/api_keys`，但 `LlmResolver::provider_for_profile(...)` 对非 OpenRouter provider 只调用 `.first()`，后续 key 被忽略。
- `OpenAiCompatibleProvider` 自身只保存单个 client/key；非 streaming `chat` / `chat_with_tools` 无法跨 key fallback。
- 既有 `scheduler_heartbeat_openrouter_402_credit_exhaustion_skips_alerts.md` 关注 OpenRouter `HTTP 402` / token budget / credits 不足；本单是 mimo/OpenAI-compatible profile 的 `HTTP 429` key-pool fallback 缺口。
- 既有 `scheduler_heartbeat_mimo_param_incorrect_batch_failures.md` 关注同一模型的 `HTTP 400 Param Incorrect` / `reasoning_content` transcript 兼容问题；本单状态码、直接原因和修复边界不同。

## 修复情况

- 2026-05-20 修复：`OpenAiCompatibleProvider` 新增 key-pool 构造，非 streaming `chat` / `chat_with_tools` 会按配置 key 顺序尝试；单 key 传输错误仍保留一次短重试，HTTP 429 这类 provider 拒绝会直接尝试下一把 key。
- 2026-05-20 修复：`LlmResolver` 对非 OpenRouter profile 使用完整 `api_key/api_keys` pool，不再只取第一把 key。
- 2026-05-20 修复：heartbeat runner failure 分类补齐 `HTTP 429`、`code: 429`、`rate limit exceeded`、`too many requests`、`resource exhausted`，统一归入 `provider_quota_exhausted`。

## 验证

- `cargo test -p hone-llm chat_with_tools_falls_back_to_next_key_after_http_429 -- --nocapture`
- `cargo test -p hone-channels heartbeat_provider_429_quota_error_is_classified --lib -- --nocapture`
- `cargo test -p hone-llm openai_compatible -- --nocapture`
- `cargo test -p hone-llm resolver -- --nocapture`
- `cargo test -p hone-channels heartbeat_provider_ --lib -- --nocapture`
- `cargo check -p hone-llm -p hone-channels --tests`
- `rustfmt --edition 2024 --check crates/hone-llm/src/openai_compatible.rs crates/hone-llm/src/resolver.rs crates/hone-channels/src/scheduler.rs`
- `git diff --check`

## 未验证项 / 后续建议

- 如果配置只有一把已耗尽 key，本地仍会正确失败；这是外部额度不可用，不应继续降低模型预算或写一次性特殊兼容。
- 建议部署后确认 auxiliary profile 的 OpenAI-compatible provider 如需抗 429，应在 `llm.providers.<name>.api_keys` 配置多个有效 key。
