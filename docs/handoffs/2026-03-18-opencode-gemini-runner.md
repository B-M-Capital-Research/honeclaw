# Runner 切换到 Gemini 3.1 Pro

- title: Runner 切换到 Gemini 3.1 Pro
- status: done
- created_at: 2026-03-18
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
- related_prs:
  - N/A

## Summary

把默认 runner 切到 `gemini_acp`，并固定 `gemini-3.1-pro-preview` 作为运行模型。

## What Changed

- 默认 runner 切换到 `gemini_acp`。
- 默认模型固定为 `gemini-3.1-pro-preview`。
- 同步更新 runtime 配置与项目根种子配置。

## Verification

- `gemini --version`
- `bash tests/regression/manual/test_gemini_streaming.sh`
- `printf 'Reply with exactly: HONE_HONECLI_GEMINI_ACP_OK\nquit\n' | cargo run -q -p hone-cli`

## Risks / Follow-ups

- 后续若 runner 或默认模型再次切换，需要连同种子配置、手工回归脚本和 runtime 验证一起调整。

## Next Entry Point

- `docs/adr/0002-agent-runtime-acp-refactor.md`
