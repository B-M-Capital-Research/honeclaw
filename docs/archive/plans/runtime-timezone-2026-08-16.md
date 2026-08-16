# 运行时时区去硬编码

- title: 运行时时区去硬编码
- status: done
- created_at: 2026-08-16
- updated_at: 2026-08-16
- owner: Codex
- related_files: `crates/hone-core/src/time.rs`, `crates/hone-core/src/config/mod.rs`, `memory/src/cron_job/`, runtime time call sites, PostgreSQL TEXT timestamp queries
- related_docs: `docs/current-plan.md`, `docs/invariants.md`, `docs/runbooks/backend-deployment.md`, `config.example.yaml`
- related_prs: none; local commits `7ca42198`, `23208d2c`, `87ec8536`; not pushed

## Goal

建立唯一运行时时区来源，按显式配置、`HONE_TIMEZONE`、机器 IANA/本地偏移、UTC
的顺序解析；消除运行时北京时间硬编码，使 cron、日期键、时间渲染和跨偏移
PostgreSQL 时间比较保持正确。

## Scope

- 在 `hone-core` 建立可配置且支持 IANA/DST 的运行时时区原语，并迁移旧
  `beijing_*` 调用。
- 全仓分类并处理 `+08:00`、`Asia/Shanghai`、`FixedOffset::east*`、`28800`
  等固定时区命中；测试固定输入可以保留。
- 让 cron 到期窗口、heartbeat 半点槽、日期键和用户可见时区文案使用运行时时区。
- 审计 TEXT 时间列比较，所有跨偏移比较按 `timestamptz` 时刻语义执行，并保持
  PostgreSQL 参数先钉为 `text`。
- 不迁移、不改写任何历史数据库行。

## Validation

- [x] 新增非 `+08:00`（`America/New_York`）回归，覆盖 cron 到期、日期键和时间渲染。
- [x] `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`：
  退出码 0；2,586 passed，113 ignored，0 failed。
- [x] `cargo test -p hone-memory --all-targets -- --ignored`：退出码 0；93 passed，
  0 failed。
- [x] `bash tests/regression/run_ci.sh`：退出码 0；未添加 `rg` shim，finance contracts
  为 49 success / 0 review / 0 fail。
- [x] `bun run test:web`：退出码 0；486 passed，0 failed。
- [x] 执行任务指定的最终 `grep`：153 行均为内联测试固定样例、历史偏移注释、
  旧字段 serde 兼容别名，或核心动态固定偏移构造器；无运行时默认北京时区。

## Documentation Sync

- 更新 `config.example.yaml` 与 `docs/runbooks/backend-deployment.md`，明确容器/GCE
  必须显式配置时区，机器探测在容器中通常得到 UTC。
- 更新 `docs/invariants.md` 或 `docs/decisions.md`，记录运行时时区真相源和禁止固定
  北京时区回退的长期约束。
- 完成后从 `docs/current-plan.md` 移除并把本计划移到 `docs/archive/plans/`，同时更新
  `docs/archive/index.md`；遵照用户要求不创建或修改 `docs/handoffs/`。

## Risks / Open Questions

- IANA 时区有 DST，不能把运行时时区缓存成全年固定偏移；每个时刻必须重新计算偏移。
- 配置加载与测试并行可能共享进程级时区状态；测试覆盖需使用显式时区参数或受控覆盖，
  避免环境变量/全局状态串扰。
- 历史 TEXT 时间戳与新行偏移不同，字符串排序和比较不再等价于时刻比较。
- 当前 worktree 基于 detached `1ac64605`，已新建 `codex/runtime-timezone` 分支；不 push。

## Completion

- 顶层 `timezone`、`HONE_TIMEZONE`、机器 IANA、机器当前偏移、UTC 构成唯一解析链；
  IANA 时区按目标时刻计算 DST 偏移。
- cron、heartbeat、运行时日期键、报告和用户文案均使用同一运行时时区；actor 时区与
  交易所日历时区作为明确的业务域覆盖保留，不再充当进程默认值。
- PostgreSQL TEXT 时间列的筛选和排序按 `::timestamptz` 比较；字符串参数采用
  `$N::text::timestamptz`，未改写任何历史行。
- 按任务约定不创建或修改 `docs/handoffs/`；本计划完成后归档并写入历史索引。
