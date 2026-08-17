# ACP 首事件终态工具调用修复交接

- title: ACP 首事件终态工具调用修复交接
- status: blocked
- created_at: 2026-08-17
- updated_at: 2026-08-17
- owner: shared
- related_files:
  - `crates/hone-channels/src/runners/acp_common/ingest.rs`
  - `crates/hone-channels/src/runners/acp_common/tests.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
- related_docs:
  - `docs/current-plan.md`
  - `docs/current-plans/acp-runtime-refactor.md`
- related_prs: []

## Summary

Discord/Codex ACP 的同步本地工具可能在首个 `tool_call` 中直接返回终态，且不再发送 `tool_call_update`。共享 ACP ingest 的最小修复与回归测试已经完成；只剩完整 PostgreSQL workspace 门禁和 commit。

## What Changed

- 首个 `tool_call` 明确带 `completed`、`failed` 或 `cancelled` 时立即调用现有 `capture_tool_finish` 收口。
- 无独立 result 的同步成功调用保存最小 `{"status":"completed"}` 结果，避免 turn 结束时被合成为缺失终态。
- `pending`、`in_progress` 和未知字符串仍保持 pending；`codex_acp.rs` 的缺失终态失败判断没有放宽。

## Verification

- 定向回归：`1 passed; 0 failed`。
- 变异验证：注释首事件终态收口后 `0 passed; 1 failed`；恢复后公共 ACP 组 `25 passed; 0 failed`。
- 显式 touched-file `rustfmt --check` 与 `git diff --check` 通过。
- 完整门禁已按要求 source `/Users/zhangxuanren/Workspace/honeclaw/.env`，但当前执行沙箱禁止 TCP 和 Unix socket；在 `hone-channels` 阶段因 PostgreSQL 连接失败中止为 `654 passed; 173 failed; 1 ignored`。这不是合格的门禁结果。

## Risks / Follow-ups

- 门禁未通过前不要把本修复标记完成或 push。
- 不要通过忽略 pending 工具绕过失败判断；本修复只接受明确协议终态。
- 当前安装的稳定 ACP schema 明确定义 `completed` / `failed`，新版 schema另有 `cancelled`；其它开放字符串不能推断为终态。

## Next Entry Point

1. 在具备本机 PostgreSQL 访问权限的普通终端运行：
   `set -a && . /Users/zhangxuanren/Workspace/honeclaw/.env && set +a && cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
2. 记录最终 passed/failed 数字；若通过，把本文件改为 `done`，更新 active plan 的验证条目。
3. 审查并只暂存本交接列出的代码/文档文件，创建中文 scoped commit，不 push。
