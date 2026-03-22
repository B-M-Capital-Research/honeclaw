# 配置收敛（移除 .env 依赖）交接

日期：2026-03-13

## 目标回顾

- 取消 .env 文件读取，配置统一从 `config.yaml` 读取
- Secrets/Token 不再依赖环境变量作为首选
- 启动流程不再复制/注入 .env

## 已完成

- 移除 `hone-channels` 中的 `.env` 加载器与对外导出。
- Feishu 渠道凭证改为读取 `feishu.app_id` / `feishu.app_secret`；Tauri 启动不再注入 Feishu 环境变量。
- `launch.sh` 改为仅根据 `config.yaml` 导出 Feishu 变量（无 env fallback）。
- `config.example.yaml` 与 `config.yaml` 移除 `feishu.app_id_env` / `feishu.app_secret_env` 字段与相关说明。
- 安全硬化 handoff 中涉及研究 API Key 与诊断脚本的描述同步更新为配置读取。

## 关键改动文件

- `crates/hone-channels/src/core.rs`
- `crates/hone-channels/src/lib.rs`
- `bins/hone-feishu/src/main.rs`
- `src-tauri/src/main.rs`
- `launch.sh`
- `config.example.yaml`
- `config.yaml`
- `docs/handoffs/2026-03-13-security-hardening.md`

## 验证

- `cargo check --workspace --all-targets`
  - 失败：`src-tauri` 缺少 `binaries/hone-console-page-aarch64-apple-darwin`

## 注意事项 / 风险

- `config.yaml` 作为内部种子配置继续保留，公开复制时不要直接带入对外仓库。
