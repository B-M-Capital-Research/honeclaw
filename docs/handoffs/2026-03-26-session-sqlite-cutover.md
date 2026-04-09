# Session SQLite 影子写入与运行时切换

- title: Session SQLite 影子写入与运行时切换
- status: done
- created_at: 2026-03-26
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/session-sqlite-migration-plan.md`
- related_prs:
  - N/A

## Summary

把 SessionStorage 收口到可切换的 `json | sqlite` 后端，并完成 SQLite shadow write 与 runtime 主读接入。

## What Changed

- SessionStorage 支持运行时后端切换。
- SQLite shadow write 与 runtime 主读都已接入。
- `/api/users` 改为统一走 SessionStorage，不再直扫 `data/sessions`。
- 本机 runtime 已切到 SQLite，JSON 继续双写作回退镜像。

## Verification

- `cargo test -p hone-memory`
- `cargo test -p hone-channels --no-run`
- `cargo test -p hone-web-api --no-run`
- `bash tests/regression/ci/test_session_sqlite_migration.sh`
- 重启服务后验证 `/api/meta` `/api/users` `/api/history`

## Risks / Follow-ups

- 后续若关闭 JSON 双写或改迁移策略，应保留新的切换与回退说明。

## Next Entry Point

- `docs/session-sqlite-migration-plan.md`
