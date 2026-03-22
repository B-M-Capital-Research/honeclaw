# 2026-03-14 Launch 与 Runtime Hardening 收口

## 归并范围

- `launch.sh` 多轮启动脚本修补
- Telegram `getUpdates` 冲突保护

## 已完成

- `launch.sh` 改为通过数组构造多组 `--bin` 参数，修复参数拼接错误。
- 构建流程改为单次 `cargo build` 覆盖启用的 bins，利用 Cargo 自身并行。
- 默认启动前先执行 stop，全量重启已有相关进程。
- stop / restart 路径增加等待退出与必要时强杀，降低重启重叠概率。
- 启动语义进一步收口为“默认仅后端，`--web` 控制前端”，并拆出 `scripts/build_desktop.sh` 承接桌面打包。
- Telegram 入口新增单机实例锁与更明确的 `TerminatedByOtherGetUpdates` 冲突提示。

## 验证

- `bash -n launch.sh`
- `bash -n scripts/build_desktop.sh`
- `cargo check -p hone-console-page`
- `cargo check -p hone-telegram`

## 备注

- 本文档替代原先多个 `launch.sh` 小修计划页与 Telegram 冲突保护计划页。
