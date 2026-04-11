# Discord 重复“正在思考中”排查

- title: Discord 重复“正在思考中”排查
- status: done
- created_at: 2026-03-18
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- related_prs:
  - N/A

## Summary

排查 Discord 重复 thinking 的根因，确认问题更像入口重复消费而不是 runner 本身重复发消息。

## What Changed

- 确认 Discord 单次 `opencode_acp` run 不会自行双发 thinking。
- 观察到 direct session 在 705ms 内落下两条完全相同的 user message。
- 结论更偏向入口被两个独立 consumer / 进程重复消费。

## Verification

- `sed -n '1,220p' data/sessions/Actor_discord__direct__483641214445551626.json`
- `pgrep -lf hone-discord`
- `bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- 直接驱动 `opencode acp` 统计 `tool_call_count=1`

## Risks / Follow-ups

- 后续若再次出现重复 thinking，应优先排查多 consumer / 多进程消费，而不是直接怀疑 runner 双发。

## Next Entry Point

- `docs/archive/index.md`
