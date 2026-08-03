#!/bin/bash
#=============================================================================
# 脚本名称: auth.sh
# 功能描述: 登录授权全流程 - 检查状态 + 执行登录 + 授权确认 + scope/MCC 校验 + 重新授权
# 调用位置: Step 3 登录授权
# 调用前置: 脚本通过 error_handler.sh 间接初始化 DEV_TOOL_NAME；必要时可用 DEV_TOOL_NAME 环境变量覆盖
# 用法:
#   auth init   --scope <scope> --sales-code <code> --mcc-code <code> [--product-name <name> --mcc-name <name>]
#   auth confirm [--scope <scope> --sales-code <code> --mcc-code <code> --product-name <name> --mcc-name <name>]
#   auth mismatch [--scope <scope> --sales-code <code> --mcc-code <code> --product-name <name> --mcc-name <name>]
# 返回值:
#   init:     AUTH_FLOW:SKIP | AUTH_FLOW:READY | AUTH_FLOW:SCOPE_MISMATCH | AUTH_FLOW:MCC_MISMATCH | AUTH_FLOW:FAILED
#   confirm:  AUTH_FLOW:AUTH_SUCCESS | AUTH_FLOW:PENDING | AUTH_FLOW:EXPIRED | AUTH_FLOW:SCOPE_MISMATCH | AUTH_FLOW:MCC_MISMATCH | AUTH_FLOW:FAILED
#   mismatch: 执行一次 logout 后调用 auth init 重新生成授权链接（不输出 FLOW 标记，由 auth init 输出 AUTH_FLOW:READY）
#=============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 一次性 source error_handler（含 unwrap_mcp）
if [ -f "${SCRIPT_DIR}/error_handler.sh" ]; then
  source "${SCRIPT_DIR}/error_handler.sh"
else
  echo "❌ 缺少错误处理脚本: ${SCRIPT_DIR}/error_handler.sh"
  exit 1
fi

require_command jq || exit 1
require_command alipay-cli || exit 1
require_command mktemp || exit 1

# ─── 授权状态文件（用于独立执行 confirm/mismatch 时兜底恢复非敏感参数）────
AUTH_STATE_OWNER="$(id -u 2>/dev/null || printf '%s' "${USER:-unknown}" | tr -c 'A-Za-z0-9_-' '_')"
AUTH_STATE_DIR="${TMPDIR:-/tmp}"
AUTH_STATE_DIR="${AUTH_STATE_DIR%/}"
AUTH_STATE_FILE="${ALIPAY_AUTH_STATE_FILE:-${AUTH_STATE_DIR}/.alipay_auth_state_${AUTH_STATE_OWNER}.json}"

cleanup_auth_state() {
  rm -f "$AUTH_STATE_FILE"
}

save_auth_state() {
  jq -n \
    --arg salesCode "$SALES_CODE" \
    --arg mccCode "$MCC_CODE" \
    --arg scope "$SCOPE" \
    --arg productName "${PRODUCT_NAME:-}" \
    --arg mccName "${MCC_NAME:-}" \
    --arg deviceCode "${DEVICE_CODE:-}" \
    '{
      salesCode: $salesCode,
      mccCode: $mccCode,
      scope: $scope,
      productName: $productName,
      mccName: $mccName,
      deviceCode: $deviceCode
    }' > "$AUTH_STATE_FILE"
  chmod 600 "$AUTH_STATE_FILE" 2>/dev/null || true
}

load_auth_state() {
  if [ ! -f "$AUTH_STATE_FILE" ]; then
    echo "❌ 缺少授权上下文（请传入 --sales-code/--mcc-code，或先执行 auth.sh init）"
    return 1
  fi
  if ! jq -e . "$AUTH_STATE_FILE" >/dev/null 2>&1; then
    echo "❌ 授权状态文件不是合法 JSON: $AUTH_STATE_FILE"
    echo "📋 请重新执行 auth.sh init 生成授权状态。"
    return 1
  fi

  [ -n "${SALES_CODE:-}" ] || SALES_CODE=$(jq -r '.salesCode // ""' "$AUTH_STATE_FILE")
  [ -n "${MCC_CODE:-}" ] || MCC_CODE=$(jq -r '.mccCode // ""' "$AUTH_STATE_FILE")
  [ -n "${SCOPE:-}" ] || SCOPE=$(jq -r '.scope // ""' "$AUTH_STATE_FILE")
  [ -n "${PRODUCT_NAME:-}" ] || PRODUCT_NAME=$(jq -r '.productName // ""' "$AUTH_STATE_FILE")
  [ -n "${MCC_NAME:-}" ] || MCC_NAME=$(jq -r '.mccName // ""' "$AUTH_STATE_FILE")
}

parse_auth_context_args() {
  while [[ $# -gt 0 ]]; do
    case $1 in
      --scope)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        SCOPE="$2"; shift 2 ;;
      --sales-code)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        SALES_CODE="$2"; shift 2 ;;
      --mcc-code)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        MCC_CODE="$2"; shift 2 ;;
      --product-name)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        PRODUCT_NAME="$2"; shift 2 ;;
      --mcc-name)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        MCC_NAME="$2"; shift 2 ;;
      *) echo "❌ 未知参数: $1"; return 1 ;;
    esac
  done
}

restore_auth_context_or_state() {
  SCOPE="${SCOPE:-}"
  SALES_CODE="${SALES_CODE:-}"
  MCC_CODE="${MCC_CODE:-}"
  PRODUCT_NAME="${PRODUCT_NAME:-}"
  MCC_NAME="${MCC_NAME:-}"

  if [ "$#" -gt 0 ]; then
    parse_auth_context_args "$@" || return 1
  fi

  if [ -z "$SALES_CODE" ] || [ -z "$MCC_CODE" ]; then
    load_auth_state || return 1
  fi

  if [ -z "$SCOPE" ]; then
    SCOPE=$(get_scope "$SALES_CODE")
  fi
}

# ─── 共用：根据 salesCode 获取 scope ─────────────────────────────────────────
get_scope() {
  case "$1" in
    "I1080300001000041203") echo "app:all,fast_instant_trade_pay:write" ;;
    "I1080300001000041313") echo "app:all,auth_alipay_apppay:write" ;;
    "I1080300001000160457") echo "app:all,machine_pay:write,agmnt:write" ;;
    *) echo "" ;;
  esac
}

extract_json_payload() {
  local raw="$1"
  local line_count start candidate

  if printf '%s' "$raw" | jq -e . >/dev/null 2>&1; then
    printf '%s' "$raw"
    return 0
  fi

  line_count=$(printf '%s\n' "$raw" | wc -l | tr -d ' ')
  start=1
  while [ "$start" -le "$line_count" ]; do
    candidate=$(printf '%s\n' "$raw" | sed -n "${start},\$p")
    if printf '%s' "$candidate" | jq -e . >/dev/null 2>&1; then
      printf '%s' "$candidate"
      return 0
    fi
    start=$((start + 1))
  done

  return 1
}

run_json_command() {
  local label="$1"
  shift
  local stderr_file stdout stderr_output combined_output exit_code json_payload

  stderr_file="$(mktemp "${TMPDIR:-/tmp}/alipay_auth_stderr.XXXXXX")" || return 1
  stdout=$(PLATFORM="${DEV_TOOL_NAME}" "$@" 2>"$stderr_file")
  exit_code=$?
  stderr_output=$(cat "$stderr_file")

  if json_payload=$(extract_json_payload "$stdout"); then
    rm -f "$stderr_file"
    printf '%s' "$json_payload"
    return 0
  fi

  if json_payload=$(extract_json_payload "$stderr_output"); then
    rm -f "$stderr_file"
    printf '%s' "$json_payload"
    return 0
  fi

  combined_output=$(printf '%s\n%s' "$stdout" "$stderr_output")
  if json_payload=$(extract_json_payload "$combined_output"); then
    rm -f "$stderr_file"
    printf '%s' "$json_payload"
    return 0
  fi

  echo "❌ ${label} 未返回合法 JSON" >&2
  echo "📋 退出码: ${exit_code}" >&2
  if [ -n "$stderr_output" ]; then
    echo "📋 错误输出：" >&2
    printf '%s\n' "$stderr_output" >&2
  fi
  if [ -n "$stdout" ]; then
    echo "📋 标准输出：" >&2
    printf '%s\n' "$stdout" >&2
  fi
  rm -f "$stderr_file"
  return 1
}

# 校验当前登录会话是否覆盖目标产品 scope 和 MCC。
# 可复用调用方已经取得的 whoami JSON，避免重复查询。
validate_current_authorization() {
  local whoami_result="${1:-}"
  local logged_in is_expired granted_scope required_scope
  local scope_match mcc_verify_result mcc_verify_unwrapped mcc_match
  local mcc_error_type mcc_error_code mcc_success
  local granted_scope_item
  local -a granted_scope_items

  if [ -z "$SALES_CODE" ] || [ -z "$MCC_CODE" ]; then
    echo "❌ 缺少授权校验参数"
    echo "AUTH_FLOW:FAILED"
    return 1
  fi

  if [ -z "$whoami_result" ]; then
    whoami_result=$(run_json_command "alipay-cli whoami --json" alipay-cli whoami --json) || {
      echo "AUTH_FLOW:FAILED"
      return 1
    }
  fi

  if ! handle_error "$whoami_result"; then
    echo "AUTH_FLOW:FAILED"
    return 1
  fi

  logged_in=$(echo "$whoami_result" | jq -r '.data.logged_in // false')
  is_expired=$(echo "$whoami_result" | jq -r '.data.is_expired // false')
  if [ "$logged_in" != "true" ] || [ "$is_expired" = "true" ]; then
    echo "❌ 当前登录状态无效或已过期"
    echo "AUTH_FLOW:FAILED"
    return 1
  fi

  granted_scope=$(echo "$whoami_result" | jq -r '.data.scope // empty')
  case "$SALES_CODE" in
    "I1080300001000041203") required_scope="fast_instant_trade_pay:write" ;;
    "I1080300001000041313") required_scope="auth_alipay_apppay:write" ;;
    "I1080300001000160457") required_scope="machine_pay:write" ;;
    *)
      echo "❌ 未知产品码: $SALES_CODE"
      echo "AUTH_FLOW:FAILED"
      return 1
      ;;
  esac

  scope_match="false"
  IFS=',' read -r -a granted_scope_items <<< "$granted_scope"
  for granted_scope_item in "${granted_scope_items[@]}"; do
    granted_scope_item="${granted_scope_item#"${granted_scope_item%%[![:space:]]*}"}"
    granted_scope_item="${granted_scope_item%"${granted_scope_item##*[![:space:]]}"}"
    if [ "$granted_scope_item" = "$required_scope" ]; then
      scope_match="true"
      break
    fi
  done

  mcc_match="true"
  if [ "$scope_match" = "true" ]; then
    mcc_verify_result=$(run_json_command "alipay-cli mcp call ar-query.queryArInfosBySalesProd" \
      alipay-cli mcp call ar-query.queryArInfosBySalesProd \
      -d "{\"request\":{\"salesProductCodes\":[\"${SALES_CODE}\"]},\"ctx\":{}}" \
      --json) || {
      echo "AUTH_FLOW:FAILED"
      return 1
    }

    mcc_verify_unwrapped=$(unwrap_mcp "$mcc_verify_result")
    mcc_error_type=$(detect_error "$mcc_verify_result")
    if [ "$mcc_error_type" = "AUTH_MISMATCH" ]; then
      mcc_match="false"
    elif [ "$mcc_error_type" != "SUCCESS" ]; then
      echo "❌ MCC 校验失败"
      echo "AUTH_FLOW:FAILED"
      return 1
    fi

    # 保留对非标准错误响应的兼容，只在后端明确返回 MCC 未授权时判定不匹配。
    mcc_error_code=$(echo "$mcc_verify_unwrapped" | jq -r '.errorCode // .data.errorCode // ""' 2>/dev/null)
    mcc_success=$(echo "$mcc_verify_unwrapped" | jq -r '.success // "null"' 2>/dev/null)
    if [ "$mcc_success" = "false" ] && [ -n "$mcc_error_code" ] && \
       echo "$mcc_verify_unwrapped" | jq -r '.errorMessage // .data.errorMessage // .error.message // ""' 2>/dev/null | grep -qi "mccCode.*is not auth"; then
      mcc_match="false"
    elif echo "$mcc_verify_unwrapped" | grep -qi "mccCode.*is not auth"; then
      mcc_match="false"
    fi
  fi

  if [ "$scope_match" = "true" ] && [ "$mcc_match" = "true" ]; then
    echo "✅ 授权范围校验通过（scope + MCC）"
    return 0
  fi

  echo ""
  echo "⚠️ 授权范围不满足"
  if [ "$scope_match" != "true" ]; then
    echo "📋 原因：scope 权限不满足"
    echo "🤖 需要权限: $required_scope"
    echo "🤖 已授权 scope: $granted_scope"
  fi
  if [ "$mcc_match" != "true" ]; then
    echo "📋 原因：经营类目未授权"
    echo "🤖 当前类目: $MCC_CODE"
  fi

  echo ""
  echo "📋 必须通过 auth.sh mismatch 退出当前登录并重新授权"
  if [ "$scope_match" != "true" ]; then
    echo "AUTH_FLOW:SCOPE_MISMATCH"
  else
    echo "AUTH_FLOW:MCC_MISMATCH"
  fi
  return 1
}

# ─── auth init: 检查登录状态 + 执行登录 + 构建授权链接 + 输出授权信息 ────────
auth_init() {
  SCOPE=""
  SALES_CODE=""
  MCC_CODE=""
  PRODUCT_NAME=""
  MCC_NAME=""

  while [[ $# -gt 0 ]]; do
    case $1 in
      --scope)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
        SCOPE="$2"; shift 2 ;;
      --sales-code)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
        SALES_CODE="$2"; shift 2 ;;
      --mcc-code)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
        MCC_CODE="$2"; shift 2 ;;
      --product-name)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
        PRODUCT_NAME="$2"; shift 2 ;;
      --mcc-name)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
        MCC_NAME="$2"; shift 2 ;;
      *) echo "❌ 未知参数: $1"; exit 1 ;;
    esac
  done

  if [ -z "$SCOPE" ] || [ -z "$SALES_CODE" ] || [ -z "$MCC_CODE" ]; then
    echo "❌ 缺少必填参数"
    echo "用法: bash auth.sh init --scope <scope> --sales-code <code> --mcc-code <code> [--product-name <name> --mcc-name <name>]"
    exit 1
  fi

  # Step 1: 检查登录状态
  CHECK_RESULT=$(run_json_command "alipay-cli whoami --json" alipay-cli whoami --json) || {
    echo "AUTH_FLOW:FAILED"
    return 1
  }
  if ! handle_error "$CHECK_RESULT"; then
    echo "AUTH_FLOW:FAILED"
    return 1
  fi
  LOGGED_IN=$(echo "$CHECK_RESULT" | jq -r '.data.logged_in // false')
  IS_EXPIRED=$(echo "$CHECK_RESULT" | jq -r '.data.is_expired // false')

  if [ "$LOGGED_IN" = "true" ] && [ "$IS_EXPIRED" = "false" ]; then
    validate_current_authorization "$CHECK_RESULT" || return 1
    DEVICE_CODE=""
    save_auth_state
    echo "✅ 当前登录及授权范围有效，可继续后续流程"
    echo "AUTH_FLOW:SKIP"
    return 0
  fi

  # Step 2: 执行登录
  LOGIN_RESULT=$(run_json_command "alipay-cli login --non-interactive --scope" alipay-cli login --non-interactive --scope "$SCOPE" --json) || {
    echo "AUTH_FLOW:FAILED"
    return 1
  }
  if ! handle_error "$LOGIN_RESULT"; then
    echo "AUTH_FLOW:FAILED"
    return 1
  fi
  LOGIN_STATUS=$(echo "$LOGIN_RESULT" | jq -r '.data.status // ""')

  if [ "$LOGIN_STATUS" = "already_logged_in" ]; then
    validate_current_authorization || return 1
    DEVICE_CODE=""
    save_auth_state
    echo "✅ 当前登录及授权范围有效，可继续后续流程"
    echo "AUTH_FLOW:SKIP"
    return 0
  fi

  # Step 3: 解析返回结果
  # alipay-cli login 返回结构为 .data.device_code（不是 .data.data.device_code）
  DEVICE_CODE=$(echo "$LOGIN_RESULT" | jq -r '.data.device_code // empty')
  VERIFICATION_CODE=$(echo "$LOGIN_RESULT" | jq -r '.data.verification_code // empty')
  EXPIRES_IN=$(echo "$LOGIN_RESULT" | jq -r '.data.expires_in // 600')

  # Step 4: 参数校验
  if [ -z "$DEVICE_CODE" ] || [ -z "$SALES_CODE" ] || [ -z "$MCC_CODE" ]; then
    echo "❌ 授权链接参数不完整，无法生成授权链接"
    echo "📋 deviceCode: ${DEVICE_CODE:-空}"
    echo "📋 productCode: ${SALES_CODE:-空}"
    echo "📋 mccCode: ${MCC_CODE:-空}"
    exit 1
  fi

  # Step 5: 构建并校验授权链接（禁止使用 CLI 返回的 verification_url）
  BROWSER_URL="https://aipay.alipay.com/cli-auth?deviceCode=${DEVICE_CODE}&productCode=${SALES_CODE}&mccCode=${MCC_CODE}"
  if [ -n "${DEV_TOOL_NAME:-}" ] && [ "$DEV_TOOL_NAME" != "unknown" ]; then
    AUTH_PLATFORM=$(jq -nr --arg value "$DEV_TOOL_NAME" '$value|@uri')
    BROWSER_URL="${BROWSER_URL}&platform=${AUTH_PLATFORM}"
  fi
  if [[ "$BROWSER_URL" != https://aipay.alipay.com/cli-auth\?* ]] ||
     [[ "$BROWSER_URL" != *"deviceCode=${DEVICE_CODE}"* ]] ||
     [[ "$BROWSER_URL" != *"productCode=${SALES_CODE}"* ]] ||
     [[ "$BROWSER_URL" != *"mccCode=${MCC_CODE}"* ]] ||
     [[ "$BROWSER_URL" == *"opengw.alipay.com/oauth/device"* ]]; then
    echo "❌ 授权链接构建失败，缺少必要参数或误用了 CLI verification_url，已停止。"
    echo "📋 正确格式: https://aipay.alipay.com/cli-auth?deviceCode=...&productCode=...&mccCode=..."
    exit 1
  fi

  # Step 6: 保存参数到状态文件（供 confirm/mismatch 使用）
  save_auth_state

  # Step 7: 有效期转换（对非数字做防护）
  if [[ "$EXPIRES_IN" =~ ^[0-9]+$ ]] && [ "$EXPIRES_IN" -ge 60 ]; then
    EXPIRES_MIN=$((EXPIRES_IN / 60))
    EXPIRES_DISPLAY="${EXPIRES_MIN} 分钟"
  elif [[ "$EXPIRES_IN" =~ ^[0-9]+$ ]]; then
    EXPIRES_DISPLAY="${EXPIRES_IN} 秒"
  else
    EXPIRES_DISPLAY="10 分钟"
  fi

  # Step 8: 输出授权信息
  echo ""
  echo "🔐 支付宝授权登录"
  echo ""
  echo "📋 授权信息"
  echo ""
  echo "| 项目 | 信息 |"
  echo "|------|------|"
  echo "| 产品类型 | ${PRODUCT_NAME:-按量付费} |"
  echo "| 经营类目 | ${MCC_NAME:-${MCC_CODE}} (${MCC_CODE}) |"
  if [ -n "$VERIFICATION_CODE" ]; then
    echo "| 确认码 | ${VERIFICATION_CODE} |"
  else
    echo "| 确认码 | （请查看支付宝授权页面） |"
  fi
  echo "| 授权链接有效期 | ${EXPIRES_DISPLAY} |"
  echo ""
  echo "⚠️ 安全提示：请核对授权页面显示的确认码是否与上方一致，如不一致，请勿授权，立即停止操作！"
  echo ""
  echo "提示：无法跳链时，请复制下方链接到网页浏览器打开。"
  echo "🌐 授权链接：[点击跳转进行授权](${BROWSER_URL})"
  echo ""
  echo "请在完成授权后告诉我\"好了\"继续后续流程。"
  echo ""
  echo "AUTH_FLOW:READY"
}

# ─── auth confirm: 授权确认 + scope 校验 + MCC 校验 ──────────────────────────
auth_confirm() {
  # 优先使用显式传入的非敏感上下文；未传时再从状态文件兜底恢复。
  restore_auth_context_or_state "$@" || exit 1

  if [ -z "$SALES_CODE" ]; then
    echo "❌ 缺少 SALES_CODE（请传入 --sales-code，或先执行 auth.sh init）"
    exit 1
  fi
  if [ -z "$MCC_CODE" ]; then
    echo "❌ 缺少 MCC_CODE（请传入 --mcc-code，或先执行 auth.sh init）"
    exit 1
  fi

  # Step 1: 执行授权确认
  LOGIN_COMPLETE_RESULT=$(run_json_command "alipay-cli login --complete --json" alipay-cli login --complete --json) || {
    echo "AUTH_FLOW:FAILED"
    return 1
  }

  # Step 2: 判断授权结果
  # alipay-cli login --complete 返回：外层 .success + .data.status
  OUTER_SUCCESS=$(echo "$LOGIN_COMPLETE_RESULT" | jq -r '.success // false')
  DATA_STATUS=$(echo "$LOGIN_COMPLETE_RESULT" | jq -r '.data.status // ""')
  ERROR_CODE=$(echo "$LOGIN_COMPLETE_RESULT" | jq -r '.data.error.code // ""')

  if [ "$OUTER_SUCCESS" = "true" ] || [ "$DATA_STATUS" = "already_logged_in" ] || [ "$DATA_STATUS" = "completed" ]; then
    echo "✅ 授权确认成功"
  elif [ "$ERROR_CODE" = "authorization_pending" ]; then
    echo "⏳ 尚未完成授权，请在手机上确认授权后再次告诉我"
    echo "AUTH_FLOW:PENDING"
    return 0
  elif [ "$ERROR_CODE" = "auth_expired" ]; then
    echo "⏰ 授权链接已过期（有效期为10分钟），需要重新生成"
    cleanup_auth_state
    echo "AUTH_FLOW:EXPIRED"
    return 0
  else
    if handle_error "$LOGIN_COMPLETE_RESULT"; then
      echo "❌ 授权确认失败：返回结果未包含可识别的完成状态"
    fi
    cleanup_auth_state
    echo "AUTH_FLOW:FAILED"
    return 1
  fi

  # Step 3: 复用 init 的 scope + MCC 校验
  WHOAMI_RESULT=$(run_json_command "alipay-cli whoami --json" alipay-cli whoami --json) || {
    echo "AUTH_FLOW:FAILED"
    return 1
  }
  validate_current_authorization "$WHOAMI_RESULT" || return 1
  cleanup_auth_state
  echo "AUTH_FLOW:AUTH_SUCCESS"
  return 0
}

# ─── auth mismatch: 授权范围不满足 → logout → 重新授权 ──────────────────────
auth_mismatch() {
  local logout_result logout_error_type

  # 优先使用显式传入的非敏感上下文；未传时再从状态文件兜底恢复。
  restore_auth_context_or_state "$@" || exit 1

  if [ -z "$SALES_CODE" ]; then
    echo "❌ 缺少 SALES_CODE（请传入 --sales-code，或先执行 auth.sh init）"
    return 1
  fi

  # Step 1: 根据 salesCode 构造正确的 scope；目标无效时不退出当前登录。
  SCOPE=$(get_scope "$SALES_CODE")
  if [ -z "$SCOPE" ]; then
    echo "❌ 未知产品码: $SALES_CODE"
    return 1
  fi

  # Step 2: mismatch 是授权不匹配恢复路径中唯一执行 logout 的入口。
  echo "⚠️ 授权范围不满足，正在退出当前登录并准备重新授权..."
  logout_result=$(run_json_command "alipay-cli logout --json" alipay-cli logout --json) || {
    echo "❌ 退出当前登录失败，已停止重新授权"
    echo "AUTH_FLOW:FAILED"
    return 1
  }
  logout_error_type=$(detect_error "$logout_result")
  if [ "$logout_error_type" != "SUCCESS" ]; then
    echo "❌ 退出当前登录失败，已停止重新授权"
    echo "AUTH_FLOW:FAILED"
    return 1
  fi
  echo "✅ 已退出当前登录"

  # Step 3: 调用 auth init 执行登录 + 构建授权链接 + 输出授权信息
  # DEV_TOOL_NAME 已 export，子进程可继承
  echo "📋 请重新扫码授权，新的授权将包含正确的权限范围"
  echo ""

  auth_init \
    --scope "$SCOPE" \
    --sales-code "$SALES_CODE" \
    --mcc-code "$MCC_CODE" \
    --product-name "${PRODUCT_NAME:-}" \
    --mcc-name "${MCC_NAME:-}"
}

# ─── 入口分发 ────────────────────────────────────────────────────────────────
case "${1:-}" in
  init)
    shift
    auth_init "$@"
    ;;
  confirm)
    shift
    auth_confirm "$@"
    ;;
  mismatch)
    shift
    auth_mismatch "$@"
    ;;
  *)
    echo "用法: bash auth.sh <init|confirm|mismatch> [参数...]"
    echo ""
    echo "  auth init    --scope <scope> --sales-code <code> --mcc-code <code> [--product-name <name> --mcc-name <name>]"
    echo "  auth confirm [--scope <scope> --sales-code <code> --mcc-code <code> --product-name <name> --mcc-name <name>]"
    echo "  auth mismatch [--scope <scope> --sales-code <code> --mcc-code <code> --product-name <name> --mcc-name <name>]"
    exit 1
    ;;
esac
