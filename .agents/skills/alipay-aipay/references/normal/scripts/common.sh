#!/bin/bash
#=============================================================================
# 脚本名称: common.sh
# 功能描述: 集成流程和签约流程共用的 shell 基础函数
# 调用前置: 无
#=============================================================================

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "❌ 缺少依赖命令: $1"
    return 1
  fi
}

init_dev_tool_name() {
  if [ -n "${DEV_TOOL_NAME:-}" ] && [ "$DEV_TOOL_NAME" != "unknown" ]; then
    export DEV_TOOL_NAME
    return
  fi

  local COMMON_DIR
  COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local DETECT_SCRIPT="${COMMON_DIR}/detect_dev_tool.sh"

  if [ -f "$DETECT_SCRIPT" ]; then
    # shellcheck source=/dev/null
    source "$DETECT_SCRIPT"
    DEV_TOOL_NAME="$(detect_dev_tool 2>/dev/null || echo unknown)"
  fi

  export DEV_TOOL_NAME="${DEV_TOOL_NAME:-unknown}"
}
