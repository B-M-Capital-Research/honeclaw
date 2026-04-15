- title: Desktop 日志接口与 multi-agent 运行态恢复
- status: done
- created_at: 2026-04-15
- updated_at: 2026-04-15
- owner: codex
- related_files:
  - crates/hone-web-api/src/routes/logs.rs
  - crates/hone-core/src/config.rs
  - docs/runbooks/desktop-release-app-runtime.md
  - docs/archive/index.md
- related_docs:
  - AGENTS.md
  - docs/archive/index.md
  - docs/runbooks/desktop-release-app-runtime.md

## Goal

修复 desktop 运行态里两类会反复复发的问题：

- `/api/logs` 在遇到多字节纯文本或异常日志文件内容时返回空响应，导致前端日志面板不可用
- runtime config overlay 没有真正进入 `HoneConfig::from_file()`，导致渠道或 backend 可能掉回非预期 runner

## Scope

- 为日志读取和解析补 UTF-8 / panic / 锁中毒容错
- 为配置加载补 runtime overlay 合并回归
- 更新 release runtime runbook，明确缓存 `CARGO_TARGET_DIR` 的构建约束

## Validation

- `cargo test -p hone-web-api logs`
- `cargo test -p hone-core from_file_applies_runtime_overlay`
- `curl http://127.0.0.1:8077/api/logs`
- `curl http://127.0.0.1:8077/api/channels`

## Documentation Sync

- 本次补充了 `docs/runbooks/desktop-release-app-runtime.md`
- 任务在当前会话内完成，直接归档到 `docs/archive/plans/`
- 已同步在 `docs/archive/index.md` 追加索引入口

## Risks / Open Questions

- `/api/logs` 已尽量避免因为单条异常日志把整个接口拖死，但如果后续日志量继续放大，仍应考虑把文件 tail 与解析进一步拆到更独立的容错层
- runtime overlay 现在会进入 `from_file()`，后续若有只想校验“基础 YAML”而非有效配置的流程，应继续使用 `from_merged_value()` / `read_yaml_value()` 区分语义
