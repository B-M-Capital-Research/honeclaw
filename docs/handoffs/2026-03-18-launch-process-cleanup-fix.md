# launch.sh 真实进程清理修复

- title: launch.sh 真实进程清理修复
- status: done
- created_at: 2026-03-18
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `launch.sh`
- related_docs:
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

修复 `launch.sh` 记录包装进程而不是真实服务进程的问题。

## What Changed

- `launch.sh` 改为先构建再直接启动 `target/debug/hone-*`。
- pid 文件现在记录真实服务进程，而不是 `cargo run` 包装进程。

## Verification

- `bash -n launch.sh`
- `cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`
- 直接启动 `target/debug/hone-console-page` 后核对 `ps -p <pid> -o pid=,comm=,args=`

## Risks / Follow-ups

- 后续如果 `launch.sh` 再次改成包装启动，需要重新验证 pid 与清理脚本是否仍指向真实服务进程。

## Next Entry Point

- `docs/archive/index.md`
