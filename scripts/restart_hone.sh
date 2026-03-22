#!/usr/bin/env bash
# scripts/restart_hone.sh — Hone 后台重启辅助脚本
#
# 由 RestartHoneTool (Rust) 在后台以 nohup 调用，不要手动运行。
# 使用方式：nohup bash scripts/restart_hone.sh <project_root> <old_pid> >> data/logs/restart.log 2>&1 &
#
# 流程：
#   1. 等待 3 秒（让当前对话回复发出去）
#   2. 向旧 launch.sh 发送 SIGTERM（触发其 cleanup 钩子）
#   3. 等待旧进程退出（最多 10 秒）
#   4. 在项目根目录重新启动 launch.sh（后台持久运行）
#   5. launch.sh 自身在就绪后会写入 data/runtime/current.pid

set -uo pipefail

PROJECT_ROOT="${1:-}"
OLD_PID="${2:-}"

if [[ -z "$PROJECT_ROOT" ]]; then
    exit 1
fi

LOG_DIR="$PROJECT_ROOT/data/logs"
mkdir -p "$LOG_DIR"

# 将脚本自身的所有输出追加到 restart.log（由于 Rust 侧重定向到 null，需在此处自行建立日志）
exec >> "$LOG_DIR/restart.log" 2>&1

echo "[restart_hone] $(date '+%Y-%m-%d %H:%M:%S') 重启任务开始，OLD_PID=${OLD_PID:-未知}"

# 等待 3 秒，让当前对话回复有时间发出
sleep 3

# 向旧 launch.sh 发送 SIGTERM，触发其 cleanup 钩子（优雅关闭子进程）
if [[ -n "$OLD_PID" ]] && kill -0 "$OLD_PID" 2>/dev/null; then
    echo "[restart_hone] $(date '+%Y-%m-%d %H:%M:%S') 发送 SIGTERM → PID ${OLD_PID}"
    kill -TERM "$OLD_PID" 2>/dev/null || true

    # 等待旧进程退出（最多 10 秒）
    waited=0
    while kill -0 "$OLD_PID" 2>/dev/null && [[ $waited -lt 10 ]]; do
        sleep 1
        waited=$((waited + 1))
    done

    if kill -0 "$OLD_PID" 2>/dev/null; then
        echo "[restart_hone] $(date '+%Y-%m-%d %H:%M:%S') 超时，强制 SIGKILL → PID ${OLD_PID}"
        kill -KILL "$OLD_PID" 2>/dev/null || true
        sleep 1
    else
        echo "[restart_hone] $(date '+%Y-%m-%d %H:%M:%S') 旧进程已退出"
    fi
else
    echo "[restart_hone] $(date '+%Y-%m-%d %H:%M:%S') 旧进程 PID=${OLD_PID:-未知} 已不存在，直接重启"
fi

# 再等 1 秒，让文件系统状态稳定
sleep 1

# 切换到项目根目录并启动新的 launch.sh
cd "$PROJECT_ROOT" || {
    echo "[restart_hone] 错误：无法 cd 到 ${PROJECT_ROOT}" >&2
    exit 1
}

echo "[restart_hone] $(date '+%Y-%m-%d %H:%M:%S') 启动新的 launch.sh..."

# 启动新 launch.sh，输出写入 launch.log
# current.pid 将由 launch.sh 自身在进程就绪后写入
nohup bash launch.sh >> "$LOG_DIR/launch.log" 2>&1 &
NEW_LAUNCH_PID=$!

echo "[restart_hone] $(date '+%Y-%m-%d %H:%M:%S') 新 launch.sh 已在后台启动 (PID=${NEW_LAUNCH_PID})"
echo "[restart_hone] 日志：${LOG_DIR}/restart.log"
