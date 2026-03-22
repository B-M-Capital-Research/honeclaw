# 2026-03-11 Desktop Runtime 与 Observability 收口

## 归并范围

- Tauri 桌面宿主与 backend mode 切换
- 桌面端渠道开关与 sidecar 生命周期
- 诊断路径与日志落盘
- 可观测性增强与 warning 清理

## 已完成

- 新增 `src-tauri/` 桌面宿主，支持 `bundled` / `remote` backend mode、配置持久化与 sidecar 生命周期管理。
- 前端补齐 backend runtime 抽象、设置页、能力协商与远程模式降级提示。
- 设置页可直接编辑本地 `config.yaml` 的渠道开关，并在 `bundled` 模式下自动重启内置 backend。
- 桌面端新增 `desktop.log` / `sidecar.log` 落盘，设置页可展示配置目录、数据目录与日志路径。
- `hone-console-page` 接入跨进程 UDP 日志聚合，消息日志可带 `message_id` 与 `state` 追踪生命周期；LLM audit 补充 token 统计与展示。
- 清理桌面链路暴露出的遗留 warning，确保桌面相关检查输出干净。

## 验证

- `bun --filter @hone-financial/app test`
- `bun --filter @hone-financial/app typecheck`
- `bun --filter @hone-financial/app build`
- `cargo test -p hone-console-page`
- `cargo check -p hone-console-page`
- `cargo check -p hone-desktop`
- `cargo check -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram -p hone-channels`
- `bash scripts/prepare_tauri_sidecar.sh debug`

## 备注

- 本文档替代原先按子任务拆开的桌面/Tauri/diagnostics/observability/warning handoff。
