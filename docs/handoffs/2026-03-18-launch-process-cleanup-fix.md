# 2026-03-18 launch.sh 真实进程清理修复

## 背景

- `launch.sh` 原先用 `cargo run -p <bin> &` 启动 backend 和各渠道 listener，并把 `$!` 写入 `data/runtime/*.pid`。
- 该 PID 实际对应 `cargo` 包装进程，不是最终的 `target/debug/hone-*` 二进制；`./launch.sh stop` 或重启时杀掉的也是 `cargo`，导致真实 Hone 进程残留。

## 本次变更

- 为 `launch.sh` 增加 `TARGET_DIR` / `bin_path()`，统一解析实际可执行文件路径，兼容 `CARGO_TARGET_DIR`。
- 新增 `build_runtime_binaries()`：启动前先执行一次 `cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`。
- 新增 `start_hone_bin()`：直接启动 `target/debug/hone-*`，并把真实服务 PID 写入 `data/runtime/*.pid`。
- backend 和四个 channel listener 均改为通过 `start_hone_bin()` 启动，不再使用 `cargo run -p ... &`。

## 结果

- 后续通过 `./launch.sh stop` 或脚本内 `stop_all` 停止时，会命中真实 `hone-*` 进程，不会再只杀掉 `cargo` 包装层。
- 这次修复不会自动清理由旧版脚本留下的历史孤儿进程；旧残留需要手动清一次，之后新的 pid 文件将恢复正确。

## 验证

- `bash -n launch.sh`
- `cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`
- 最小运行回归：
  - 直接按新逻辑启动 `target/debug/hone-console-page`
  - `ps -p <pid> -o pid=,comm=,args=` 返回的进程本体为 `target/debug/hone-console-page`

## 后续关注

- 如果未来发现某个渠道二进制在收到终止信号后仍会遗留自身子进程，再考虑补“按进程组清理”的第二层兜底，而不是重新回到按进程名粗暴扫描。
