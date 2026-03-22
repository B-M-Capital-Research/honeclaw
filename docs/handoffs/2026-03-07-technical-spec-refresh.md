# Handoff: Technical Spec Refresh

日期：2026-03-07
状态：已完成

## 本次目标

- 按当前 Rust workspace 与多渠道/Web 实现，重写 `docs/technical-spec.md`
- 消除旧版 Python 架构描述带来的误导

## 已完成

- 将 `docs/technical-spec.md` 从历史 Python 文档重写为当前实现版技术规格说明
- 对齐了当前工作区结构、核心模块职责、主执行链、actor 隔离、配置来源与测试契约
- 明确标注了当前实现边界：
  - Telegram 渠道仍为占位实现
  - `hone-tools` 中部分工具已实现但未默认接入 `HoneBotCore::create_tool_registry`
- 更新 `docs/current-plan.md`，记录本轮任务目标、验证方式和文档同步步骤

## 影响范围

- `docs/technical-spec.md`
- `docs/current-plan.md`
- `docs/handoffs/2026-03-07-technical-spec-refresh.md`

## 验证

- 已完成源码对照校验：
  - `README.md`
  - `Cargo.toml` / `package.json`
  - `docs/repo-map.md`
  - `config.example.yaml`
  - `bins/`、`crates/`、`agents/`、`memory/`、`packages/app` 的关键入口
- 已完成 `git diff -- docs/current-plan.md docs/technical-spec.md` 自检
- 本次仅涉及文档改写，未运行 `cargo` / `bun` 编译与测试

## 剩余风险

- 后续如果继续扩展默认工具注册集合，需要同步更新本文档中的“Tool 层”说明
- 若 Telegram 渠道补齐真实实现，需同步修正文档中的成熟度说明
