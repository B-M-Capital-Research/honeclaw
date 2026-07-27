# 飞书原生表格渲染修复

- title: 飞书原生表格渲染修复
- status: done
- created_at: 2026-07-27
- updated_at: 2026-07-27
- owner: Codex
- related_files:
  - `bins/hone-feishu/src/markdown.rs`
  - `bins/hone-feishu/src/outbound.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `docs/repo-map.md`
  - `docs/bugs/feishu_raw_table_component_code_leak.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/archive/plans/feishu-table-sanitization.md`
  - `docs/handoffs/2026-07-27-feishu-native-table-rendering.md`
  - `docs/archive/index.md`

## Goal

修复标准 Markdown 表格被改写成 `<table columns={...} data={...}/>` 后，在飞书卡片 Markdown 元素中以源码形式展示的问题；改为生成飞书卡片 JSON 2.0 原生 `table` 元素，并保证损坏或超限表格不会向用户泄漏组件代码。

## Scope

- 梳理 direct、scheduler、placeholder update 共用的飞书出站卡片构造入口。
- 把标准 Markdown 表格和可解析的历史 raw table 标签转换成卡片根节点原生表格元素。
- 为流式/旧路径提供无 raw tag 的可读文本降级，并限制单卡原生表格数量。
- 更新飞书输出提示、代码库地图和既有 bug 记录。

## Validation

- 通过：`cargo test -p hone-feishu markdown -- --nocapture`（18 个相关测试）
- 通过：`cargo test -p hone-feishu outbound -- --nocapture`（6 个相关测试）
- 通过：`cargo test -p hone-feishu -- --nocapture`（69 个测试）
- 通过：`cargo test -p hone-channels prompt --lib -- --nocapture`（55 个相关测试）
- 通过：`cargo check -p hone-feishu --tests`
- 通过：`cargo check -p hone-channels --tests`
- 通过：两条既有 shared sanitizer / scheduler raw-table 回归。
- 通过：本次 Rust 文件 scoped `rustfmt --check` 与相关文件 `git diff --check`。
- `bash scripts/ci/check_fmt_changed.sh` 因检查 `HEAD^...HEAD` 中与本任务无关的既有 Rust 格式差异失败；未改写用户的无关文件。
- 未执行真实 Feishu 实发、进程重启或部署。

## Documentation Sync

- 已更新 `docs/bugs/feishu_raw_table_component_code_leak.md`、`docs/bugs/README.md` 与 `docs/repo-map.md`。
- 已新增 handoff，把本计划移至 `docs/archive/plans/`，从活跃索引移除，并更新 `docs/archive/index.md`。
- 已把旧计划 `docs/archive/plans/feishu-table-sanitization.md` 标记为被本计划替代。
- 无需更新 `docs/decisions.md` / ADR：本次纠正既有飞书协议实现，不改变模块所有权或跨模块长期架构取舍。

## Risks / Open Questions

- 代码已按飞书单卡最多 5 个原生表格、最多 50 列的边界实现拆卡与列数限制，并用纯文本列降低客户端兼容风险。
- 仍需在新版本部署后用 direct 与 scheduler 各做一次真实客户端复核；本任务没有实发消息或重启运行时。
- 工作树仍包含大量用户变更及一个无关冲突文件；本任务未处理、覆盖或清理这些改动。
