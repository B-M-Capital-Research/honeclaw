# 安全治理（开源前准备）交接

日期：2026-03-13

## 目标回顾

- 增加模型工具调用安全 guard
- 强化 KB 工具的会话隔离
- 加固控制台文件代理与研究 API key 管理
- 清理硬编码密钥与配置来源

## 已完成

- 工具执行前 guard：`hone-tools` 新增 `ToolExecutionGuard`，`ToolRegistry` 执行前检查。
- KB 工具隔离：`kb_search` 默认按 actor/session 过滤 stock_table 与 load_file。
- 控制台安全：非本地模式禁用文件代理；文件代理限制到 storage 根目录白名单；研究 API key 改为 `web.research_api_key` 配置。
- 配置与文档：新增 `security` 配置块、`.env.example` 与 README 更新；`scripts/diagnose_llm.sh` 改为读取 `llm.openrouter.api_key` 配置。
- **按你的要求**：`config.yaml` 继续作为内部种子配置保留。

## 关键改动文件

- `crates/hone-core/src/config.rs`
- `crates/hone-tools/src/guard.rs`
- `crates/hone-tools/src/registry.rs`
- `crates/hone-tools/src/kb_search.rs`
- `crates/hone-channels/src/core.rs`
- `memory/src/kb.rs`
- `bins/hone-console-page/src/main.rs`
- `config.yaml`
- `config.example.yaml`
- `.env.example`
- `README.md`
- `docs/invariants.md`

## 验证

- `cargo check --workspace --all-targets`
  - 失败：`src-tauri` 缺少 `binaries/hone-console-page-aarch64-apple-darwin`

## 注意事项 / 风险

- `security.tool_guard.apply_tools` 默认排除 `web_search` / `data_fetch` / `kb_search`，如需更严格可改配置。
- 控制台文件代理仅本地模式开放；远程部署需配置 `web.auth_token`。
- `config.yaml` 作为内部种子配置继续保留，公开复制时不要直接带入对外仓库。
