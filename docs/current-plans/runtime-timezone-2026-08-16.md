# 运行时时区去硬编码

- title: 运行时时区去硬编码
- status: in_progress
- created_at: 2026-08-16
- updated_at: 2026-08-16
- owner: Codex
- related_files: `crates/hone-core/src/time.rs`, `crates/hone-core/src/config/mod.rs`, `memory/src/cron_job/`, runtime time call sites, PostgreSQL TEXT timestamp queries
- related_docs: `docs/current-plan.md`, `docs/invariants.md`, `docs/runbooks/backend-deployment.md`, `config.example.yaml`

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

- 新增非 `+08:00`（`America/New_York`）回归，覆盖 cron 到期、日期键和时间渲染。
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test -p hone-memory --all-targets -- --ignored`
- `bash tests/regression/run_ci.sh`（不依赖 `rg` shim）
- `bun run test:web`
- 执行任务指定的最终 `grep` 并逐项分类剩余命中。

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
