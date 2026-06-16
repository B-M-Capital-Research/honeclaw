# Bug: Feishu direct actor 读取 Cron 与持仓作用域为空，导致任务和投资上下文丢失

- **发现时间**: 2026-06-03 23:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#49](https://github.com/B-M-Capital-Research/honeclaw/issues/49)

## 证据来源

- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-03 19:02-23:02 CST。
  - `session_messages` 共有 21 个 user turn 与 22 个 assistant 记录，Feishu direct 最新会话均有 assistant 收口；多出的 assistant 是 daily-limit final/text 双记录，不构成重复回复缺陷。
  - assistant final 污染扫描未命中空回复、`hone-mcp binary not found`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400`、`Param Incorrect`、`Resource temporarily unavailable`、`quota exhausted`、panic 或 `index out of bounds`。
  - `session_id=Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3` 在 2026-06-03 21:08 CST 收到用户追问盘前 Cron 为何未执行。
  - 2026-06-03 21:10 CST assistant final 明确反馈当前定时任务列表返回 `jobs=[]`，并指出原有 `20:00` / `07:00` 任务在当前调度器视角下都不在任务表里。
  - 2026-06-03 21:13 CST assistant final 声称已重建两条 workday 常规任务，但同轮又反馈当前持仓工具返回 `holdings=[]`、`watchlist=[]`，即同一 actor 的持仓和关注列表也读成空。
  - 2026-06-03 21:21 CST assistant final 显示用户被迫重新提交并落库 13 条持仓；关注列表仍为空。
  - 2026-06-03 21:28 CST assistant final 声称已补建一次性补跑任务 `j_a9e14511`，计划 21:29 执行；2026-06-03 21:31 CST assistant final 又反馈该 once 任务到点后仍没有生成成功运行记录，`last_run_at=null`。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-06-03 19:02-23:02 CST 没有新增 `cron_job_runs`。
  - 全库最近 `executed_at` 仍停在 `2026-06-01T00:26:00.908925+08:00`，最新三条普通 scheduler row 仍是 `AAOI / RKLB / TEM 每日动态监控` 的 `running + pending`。
- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-05 03:01-07:01 CST。
  - 本窗有 10 个 user turn 与 10 个 assistant final，Feishu direct 均成对收口；普通 scheduler 本窗没有新增 `cron_job_runs`。
  - `session_id=Actor_feishu__direct__ou_5f58ff884640e647a1792f618f45209251` 在 2026-06-05 04:50 CST 收到用户输入摘要：`我的每天提醒没了吗？`
  - 04:50 CST assistant final 明确回复：查到定时任务列表为空，现在没有任何 `daily`、`trading_day` 或 `heartbeat` 提醒任务。
  - 05:28 CST 用户追问 `？` 后，assistant final 再次确认：每日 20 点和 21:30 两个提醒都不在了。
- `data/cron_jobs/cron_jobs_feishu__direct__ou_5f58ff884640e647a1792f618f45209251.json`
  - 同一 actor 用户本地 Cron 权威文件仍存在，且包含 2 条 `enabled=true` 任务：
    - `j_a22da26c`：每日 20:00 美股大盘风控简报，`repeat=daily`，`last_run_at=2026-05-30T20:00:04.862655+08:00`。
    - `j_91c512c1`：美股开盘道氏理论点位简报，`repeat=trading_day`，`last_run_at=2026-05-29T21:30:04.388419+08:00`。
  - 这说明用户可见工具链把仍存在的 Cron 数据读成空，而不是该用户从未创建过提醒。
- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-06 03:02-07:02 CST。
  - 本窗有 9 个 user turn 与 9 个 assistant final，3 个 Feishu direct 会话均成对收口；assistant final 污染扫描未命中空回复、`company_profiles/...`、本机绝对路径、`data/agent-sandboxes`、`hone-mcp binary not found`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic 或 `index out of bounds`。
  - `session_id=Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c` 在 2026-06-06 06:24 CST 收到用户输入摘要：`最值得加仓的3支股票`。
  - 06:26 CST assistant final 明确写出：按最近研究池且账本当前显示“暂无持仓”，给出 MSFT / GEV / CRCL 加仓优先级。
- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-16 19:02-23:02 CST。
  - 同窗有 41 个 user turn 与 41 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 33 条为 `completed + sent + delivered=1`，另有 1 条普通 scheduler `running + pending + delivered=0`。
  - 20:00 CST Feishu scheduler `美股持仓开盘前晚报`（`session_id=Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3` / assistant `ordinal=173`）正常送达，但用户可见 final 写出 `data/portfolio 是空目录`、`本轮未读到可展开持仓明细`，随后改用最近已记录股数继续计算组合。
  - 20:00 CST Feishu scheduler `美股盘前要闻、持仓评级与机会观察`（`session_id=Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768` / assistant `ordinal=204`）正常送达，但用户可见 final 写出当前沙盒 `data/portfolio` 目录为空、没有可读 `holdings.json`，随后沿用会话中最后确认过的持仓版本。
  - 两条样本均不是空回复或投递失败；问题集中在 scheduler / runner 读取持久化 portfolio 数据根时把现存数据读成空，并把内部存储口径暴露给用户。
- `data/portfolio/`
  - `portfolio_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3.json` 真实存在，且包含 RKLB、DRAM、AMD、MU、GOOGL、AAOI、SNDK、QCOM 等非空持仓。
  - `portfolio_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768.json` 真实存在，且包含 LITE、COHR 等非空持仓及多条 tracking-only 标的。
  - 这说明本轮不是用户没有 portfolio 文件，也不是简单的单条文案问题，而是工具 / scheduler 运行态仍会把权威 portfolio 读取为空。
- `data/portfolio/portfolio_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c.json`
  - 同一 actor 用户本地 portfolio 权威文件仍存在，`updated_at=2026-04-21T04:03:09.557122+00:00`。
  - 文件内 `holdings` 仍包含 NVO 60 股、NFLX 25 股、UNH 30 股及成本 / 策略备注。
  - 这说明用户可见工具链再次把仍存在的持仓数据读成空，而不是该用户真实没有账本。
- `data/runtime/logs/acp-events.log`
  - 同窗可见 Feishu direct runner tool / chunk / `stopReason=end_turn`，说明直聊生成链路仍可收口。
  - 未见新的用户可见 `hone-mcp binary not found` 或 provider 原始失败；异常集中在 actor 业务数据读取与 scheduler due-run 链路。
- 最近四小时非文档代码提交：`f20ea8ea Fix MCP data dir path for feishu actor scope`，是本缺陷 03:04 CST 代码修复；但 06:26 CST live 真实会话仍复现 portfolio 读取为空，因此本轮按运行态仍活跃处理。

## 端到端链路

1. Feishu 用户已有常规定时任务和持仓/关注投资上下文。
2. 用户在 2026-06-03 21:08 CST 反馈盘前 Cron 又未执行。
3. direct assistant 查询后发现当前 actor 视角下 Cron 任务列表为空，而不是单条任务执行失败。
4. 用户同意重建任务后，assistant 又发现同一 actor 的持仓与关注列表也为空。
5. 用户被迫手动重建持仓；随后补建的 once 任务仍未进入 `cron_job_runs` 执行台账。

## 期望效果

- Feishu direct 与 scheduler 应使用同一稳定 actor 身份和持久化作用域读取 Cron、portfolio 与 watchlist。
- 已存在的用户定时任务和持仓上下文不应在同一用户会话中读成空。
- 当任务或持仓真实不存在时，应能区分“从未创建”和“当前作用域/后端读取异常”，并给出可审计错误，而不是让用户重建数据后仍无法补跑。

## 当前实现效果

- 直聊主回复可以正常生成和投递，但同一 actor 的任务表、持仓表、关注表在工具读取阶段表现为空。
- 用户的定时任务交付被阻断：20:00 常规任务未执行，21:29 once 补跑也没有生成 run 台账。
- 用户投资上下文被迫重建：原持仓/关注数据在当前工具链下不可见，后续定时简报若使用空上下文会失真。
- 2026-06-05 04:50 CST 新样本中，用户已有两个 enabled Cron 任务，但直聊工具仍反馈任务列表为空，并引导用户“恢复这两个”，说明修复后的运行态仍可能读错 Cron 数据根或作用域。
- 2026-06-06 06:26 CST 新样本中，用户 portfolio 文件仍有 NVO / NFLX / UNH 持仓，但直聊工具仍反馈“暂无持仓”，说明代码修复后 live 侧仍有可见回退或部署未生效风险。
- 2026-06-16 20:00 CST 两条 scheduler 样本中，相关 actor 的 portfolio 文件均存在且非空，但 final 仍说 `data/portfolio` / 当前沙盒 portfolio 为空，并继续用历史记忆重建持仓口径，说明 2026-06-08 cloud runtime env 加固后的 live 窗口仍有真实复发。

## 用户影响

- 这是功能性缺陷，不是输出质量问题。
- 用户已配置的 Cron 与投资上下文不可见，会导致定时投研任务漏执行、补跑失败、组合简报缺少真实持仓，且用户需要手动重建关键投资数据。
- 定级为 `P1`：影响持久化业务数据正确性和 scheduler 核心交付链路；但本窗直聊仍能正常收口，未见跨用户错投或数据破坏扩散证据，因此不是 `P0`。

## 根因判断

- 直接根因不是 Feishu actor key 计算错误，而是 `hone-mcp` 作为独立进程在 actor sandbox `cwd` 下启动时，没有稳定拿到绝对 `HONE_DATA_DIR`。
- 当父进程本身依赖 repo root `cwd` 读取 `config.yaml` 中的相对 `./data/portfolio` / `./data/cron_jobs` 路径，而 `hone-mcp` 子进程改在 sandbox `cwd` 下加载同一配置时，这些相对路径会落到 sandbox 内的新空数据树，于是 `portfolio view` 与 `cron_job list` 同时返回空。
- 该问题与 `feishu_scheduler_no_runs_after_midnight.md` 不同：旧 P1 是任务存在但 scheduler loop 不再产生 run；本轮是工具读错数据根，直接把任务表和持仓表看成空。
- 2026-06-05 04:50 CST 新样本只直接证明 Cron 读取为空，未同时证明 portfolio / watchlist 也读空；但它复用同一 actor scope / data dir 读取链路，且影响范围仍是持久化 Cron 数据正确性，因此不新建重复缺陷，按本单回退状态。
- 2026-06-06 06:26 CST 新样本直接证明 portfolio 读取为空：同一 actor 有本地 portfolio 文件与非空 `holdings`，assistant 却按“账本当前显示暂无持仓”生成建议。该证据与 Cron 读空同属 actor data dir / storage schema 读取链路，不新建重复缺陷。
- 2026-06-16 20:00 CST 新样本再次直接证明 portfolio 读取为空：两个 Feishu scheduler actor 都有本地 portfolio 文件与非空 `holdings`，assistant final 却把当前 `data/portfolio` / 沙盒 portfolio 视为空目录。该证据与原 P1 同属 actor data dir / storage schema 读取链路，不新建重复缺陷；状态从 `Fixed` 回退为 `New`。

## 修复记录

- `2026-06-08 12:08 CST` 追加 cloud runtime 加固：
  - `crates/hone-channels/src/mcp_bridge.rs` 在构造 `hone-mcp` MCP server env 时，除了继续处理 `HONE_DATA_DIR` 绝对化和 `runtime_dir` 回退，也会把父进程 cloud runtime 相关环境变量透传给子进程。
  - 透传范围包含默认 PG / OSS env 名称，以及 `config.yaml` 中 `cloud.postgres.*_env`、`cloud.oss.*_env` 配置出的自定义 env 名称，避免 cloud-backed 运行态下 `hone-mcp` 子进程缺少数据库或对象存储上下文后回退到空本地数据根。
  - 新增回归 `hone_mcp_servers_exports_configured_cloud_runtime_env`，锁住自定义 cloud env 名称和值必须进入 MCP 子进程环境。
  - 本轮仍不依赖当前机器 live 服务、生产日志或渠道健康状态来判定修复；状态维持代码级 `Fixed`，等待正常部署/重启后的真实运行态复测。

- `2026-06-07 03:03 CST` 再次修复：
  - `crates/hone-channels/src/mcp_bridge.rs` 不再盲目透传父进程里的 `HONE_DATA_DIR`；若父进程把它设成相对路径（如 `data`）或空串，`hone-mcp` 之前会优先继承这个脏值，从而绕过 `runtime_dir -> data dir` 推导，并在 actor sandbox `cwd` 下把持久化根重新解释到空目录。
  - 当前修复把 `HONE_DATA_DIR` 透传收敛为两条规则：非空时先在父进程侧绝对化；空串则直接忽略并回退到 `request.runtime_dir` 的父目录。
  - 这解释了为什么 2026-06-06 03:04 已补过 `runtime_dir` 绝对化，live 仍可能复现：只要长跑父进程自身持有相对 `HONE_DATA_DIR`，旧逻辑就会优先拿它，导致前次修复被绕过。
  - 新增回归 `hone_mcp_servers_absolutizes_relative_hone_data_dir_env` 与 `hone_mcp_servers_ignores_empty_hone_data_dir_env_and_uses_runtime_dir`，锁住“父进程环境脏值覆盖正确 data root”这一复发形态。
  - 当前未重启 live 服务或渠道进程，先记 `Fixed` 而不是 `Closed`；若后续真实 Feishu direct 仍出现 `jobs=[]` / `holdings=[]` 且权威文件非空，再继续沿启动链路核对父进程实际环境。

- `2026-06-06 03:04 CST` 再次修复：
  - `crates/hone-channels/src/execution.rs` 现在会把传给 runner 的 `runtime_dir` 也固定成绝对路径，不再把相对 `data/runtime` 原样传下去。
  - `crates/hone-channels/src/mcp_bridge.rs` 在父进程未显式设置 `HONE_DATA_DIR` 时，改为先把 `request.runtime_dir` 绝对化，再取其父目录作为 `HONE_DATA_DIR` 透传给 `hone-mcp`。
  - 这补上了前次修复遗漏的链路：即使父进程本身已正确加载配置，只要 `runtime_dir` 仍是相对路径，`hone-mcp` 在 actor sandbox `cwd` 下就可能把 `HONE_DATA_DIR=data` 重新解释到 sandbox 内空目录，继续把 Cron / portfolio 读成空。
  - 本轮新增回归 `prepare_absolutizes_relative_runtime_paths`（同时断言 `config_path` / `runtime_dir` 为绝对路径）与 `hone_mcp_servers_absolutizes_relative_runtime_dir_before_deriving_data_dir`，锁住“相对 `runtime_dir` => sandbox 空数据根”复发形态。
- `2026-06-06 03:04 CST` 状态更新为 `Fixed`：
  - 本轮是代码级闭环，定向单测与 `cargo check` 已通过。
  - 尚未重启当前 live 服务做运行态复核，因此先记 `Fixed` 而不是 `Closed`；若后续真实 Feishu direct 再次出现“Cron 文件仍存在但工具返回空列表”，应基于新样本重新评估是否还有其它作用域链路未覆盖。

- `2026-06-04` 已修复：
  - `crates/hone-channels/src/execution.rs` 现在会把 runner 下发给 ACP/MCP 的 `HONE_CONFIG_PATH` 固定成绝对路径，避免 `hone-mcp` 在 sandbox `cwd` 下误读相对 `config.yaml`。
  - `crates/hone-channels/src/mcp_bridge.rs` 现在即使父进程环境里没有显式 `HONE_DATA_DIR`，也会从 `runtime_dir` 反推出数据根并透传给 `hone-mcp`，确保 `portfolio` / `cron_job` 继续读取同一份 repo/runtime 数据。
  - 这会把 direct 会话、scheduler 工具和同 actor 的持仓/任务存储重新收敛到同一数据根，不再把 sandbox 内空目录误判成“用户没有数据”。
- `2026-06-05 07:02 CST` 运行态复现，状态从 `Fixed` 回退为 `New`：
  - `Actor_feishu__direct__ou_5f58ff884640e647a1792f618f45209251` 的本地 Cron 文件仍有 2 条 enabled 任务，但 Feishu direct assistant 两次向用户确认任务列表为空。
  - 这是同一 Cron 作用域读取链路的真实用户可见错误，影响定时任务是否存在的判断和恢复动作。
  - 已有 GitHub Issue [#49](https://github.com/B-M-Capital-Research/honeclaw/issues/49)，本轮不重复创建。
- `2026-06-06 07:02 CST` 运行态再次复现，状态从 `Fixed` 回退为 `New`：
  - `f20ea8ea` 已在 03:08 CST 合入本缺陷的代码级修复，但 06:26 CST 新 Feishu direct 回复仍把同一 actor 现存 portfolio 文件读成“暂无持仓”。
  - 本轮只补充既有 P1 证据，不新建重复缺陷；已有 GitHub Issue [#49](https://github.com/B-M-Capital-Research/honeclaw/issues/49)，不重复创建。
- `2026-06-16 23:02 CST` 运行态再次复现，状态从 `Fixed` 回退为 `New`：
  - 20:00 CST 两条 Feishu scheduler 均成功送达，但都在用户可见 final 中把现存 portfolio 数据根读成空，并退回历史记忆口径生成持仓 / 组合分析。
  - 对应 actor 的 `data/portfolio/portfolio_feishu__direct__*.json` 文件真实存在且非空，因此这是持久化投资上下文读取链路复发，而不是用户数据缺失。
  - 已有 GitHub Issue [#49](https://github.com/B-M-Capital-Research/honeclaw/issues/49)，本轮不重复创建。

## 验证

- `cargo test -p hone-channels prepare_absolutizes_relative_runtime_paths -- --nocapture`
- `cargo test -p hone-channels hone_mcp_servers_derives_data_dir_from_runtime_dir_when_env_missing -- --nocapture`
- `cargo test -p hone-channels hone_mcp_servers_absolutizes_relative_runtime_dir_before_deriving_data_dir -- --nocapture`
- `cargo test -p hone-channels hone_mcp_servers_ -- --nocapture`
- `cargo check -p hone-channels --tests`
- `cargo test -p hone-channels hone_mcp_servers_derives_data_dir_from_runtime_dir_when_env_missing -- --nocapture`
- `cargo check -p hone-channels -p hone-cli --tests`
- `cargo test -p hone-channels hone_mcp_servers_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 后续关注

- 本轮是代码级闭环，没有重启现有服务做 live 复核；如需运行态确认，可在不重启当前服务的前提下优先检查新进程是否持有绝对 `HONE_CONFIG_PATH` / `HONE_DATA_DIR`。
- 若后续仍出现 `jobs=[]` + `holdings=[]` 组合症状，应优先检查对应进程的 `HONE_DATA_DIR` 与 `cwd`，而不是先怀疑 actor identity 漂移。
- 当前还需进一步确认 2026-06-05 04:50 运行态进程实际持有的 `HONE_CONFIG_PATH` / `HONE_DATA_DIR`，以及 `hone-mcp` 读取 Cron 文件时是否落到了 sandbox 或 runtime 下的空数据树。
