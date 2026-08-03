#!/bin/bash
#=============================================================================
# 脚本名称: app.sh
# 功能描述: 应用发布全流程 - 查询/复用 + 创建 + 公钥设置 + 提交审核
# 调用位置: list 用于 Step 3.1 前置资源查询；其他命令用于 Step 5.3 应用发布
# 调用前置: 脚本通过 error_handler.sh 间接初始化 DEV_TOOL_NAME
#          list/create 可传 --application-type，未传时按 --product-type/--sales-code 或兼容环境变量推断
# 用法:
#   app list [--product-type <type> --sales-code <code> --application-type <type>]   - 查询已有应用并判断复用/新建
#   app create [--product-type <type> --sales-code <code> --application-type <type>] [--mobile-platform IOS|ANDROID|ALL ...] - 创建新应用
#   app key    <appId> <publicKey>       - 设置应用公钥
#   app verify-key <appId> [publicKey]   - 确认应用公钥已设置
#   app audit  <appId>                   - 提交应用审核
#   app reuse  <appId>                   - 复用已有应用（查询安全密钥）
# 返回值:
#   list:   FLOW:CREATE_NEW | FLOW:SELECT | FLOW:PENDING_APPLICATIONS | FLOW:ERROR
#   create: 成功输出 APP_ID=xxx；失败 exit 1，返回结构异常时同时输出 FLOW:ERROR
#   key:    输出 confirmPageUrl
#   audit:  输出审核结果
#   reuse:  FLOW:REUSE_SUCCESS | FLOW:REUSE_NO_KEY | FLOW:ERROR
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

parse_app_context_args() {
  while [[ $# -gt 0 ]]; do
    case $1 in
      --product-type)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        PRODUCT_TYPE="$2"; shift 2 ;;
      --sales-code)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        SALES_CODE="$2"; shift 2 ;;
      --application-type)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        APPLICATION_TYPE="$2"; shift 2 ;;
      --mobile-platform)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        MOBILE_PLATFORM=$(printf '%s' "$2" | tr '[:lower:]' '[:upper:]'); shift 2 ;;
      --bundle-id)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        BUNDLE_ID="$2"; shift 2 ;;
      --app-package)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        APP_PACKAGE="$2"; shift 2 ;;
      --app-sign)
        [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; return 1; }
        APP_SIGN="$2"; shift 2 ;;
      *)
        echo "❌ 未知参数: $1"
        return 1 ;;
    esac
  done
}

validate_mobile_platform_args() {
  local APP_APPLICATION_TYPE="$1"

  if [ -z "${MOBILE_PLATFORM:-}" ]; then
    if [ -n "${BUNDLE_ID:-}" ] || [ -n "${APP_PACKAGE:-}" ] || [ -n "${APP_SIGN:-}" ]; then
      echo "❌ bundleId/appPackage/appSign 必须与 --mobile-platform 一起传入" >&2
      return 1
    fi
    if [ "$APP_APPLICATION_TYPE" = "MOBILEAPP" ]; then
      echo "❌ 创建 MOBILEAPP 必须传 --mobile-platform IOS|ANDROID|ALL" >&2
      echo "📋 IOS 需同时传 --bundle-id；ANDROID 需同时传 --app-package 和 --app-sign；ALL 需三项都传" >&2
      return 1
    fi
    return 0
  fi

  if [ "$APP_APPLICATION_TYPE" != "MOBILEAPP" ]; then
    echo "❌ --mobile-platform 仅支持 MOBILEAPP 创建场景" >&2
    return 1
  fi

  case "$MOBILE_PLATFORM" in
    IOS)
      if [ -z "${BUNDLE_ID:-}" ]; then
        echo "❌ mobilePlatform=IOS 时必须传 --bundle-id" >&2
        return 1
      fi
      if [ -n "${APP_PACKAGE:-}" ] || [ -n "${APP_SIGN:-}" ]; then
        echo "❌ mobilePlatform=IOS 时不应传 --app-package 或 --app-sign；如需同时支持 iOS 和 Android，请使用 --mobile-platform ALL" >&2
        return 1
      fi
      ;;
    ANDROID)
      if [ -z "${APP_PACKAGE:-}" ] || [ -z "${APP_SIGN:-}" ]; then
        echo "❌ mobilePlatform=ANDROID 时必须传 --app-package 和 --app-sign" >&2
        return 1
      fi
      if [ -n "${BUNDLE_ID:-}" ]; then
        echo "❌ mobilePlatform=ANDROID 时不应传 --bundle-id；如需同时支持 iOS 和 Android，请使用 --mobile-platform ALL" >&2
        return 1
      fi
      ;;
    ALL)
      if [ -z "${BUNDLE_ID:-}" ] || [ -z "${APP_PACKAGE:-}" ] || [ -z "${APP_SIGN:-}" ]; then
        echo "❌ mobilePlatform=ALL 时必须传 --bundle-id、--app-package 和 --app-sign" >&2
        return 1
      fi
      ;;
    *)
      echo "❌ --mobile-platform 参数值无效: ${MOBILE_PLATFORM}（仅支持 IOS|ANDROID|ALL）" >&2
      return 1
      ;;
  esac
}

build_create_application_request() {
  local APP_APPLICATION_TYPE="$1"

  if [ -z "${MOBILE_PLATFORM:-}" ]; then
    jq -n --arg applicationType "$APP_APPLICATION_TYPE" \
      '{request:{applicationType:$applicationType,createScene:"cli"}}'
    return 0
  fi

  jq -n \
    --arg applicationType "$APP_APPLICATION_TYPE" \
    --arg mobilePlatform "$MOBILE_PLATFORM" \
    --arg bundleId "${BUNDLE_ID:-}" \
    --arg appPackage "${APP_PACKAGE:-}" \
    --arg appSign "${APP_SIGN:-}" \
    '{
      request: (
        {
          applicationType:$applicationType,
          createScene:"cli",
          mobilePlatform:$mobilePlatform
        }
        + (if $bundleId != "" then {bundleId:$bundleId} else {} end)
        + (if $appPackage != "" then {appPackage:$appPackage} else {} end)
        + (if $appSign != "" then {appSign:$appSign} else {} end)
      )
    }'
}

validate_app_context() {
  local EXPECTED_PRODUCT_TYPE=""
  local EXPECTED_APPLICATION_TYPE=""

  if [ -n "${PRODUCT_TYPE:-}" ] && [ "$PRODUCT_TYPE" != "aipay" ] && [ "$PRODUCT_TYPE" != "webpay" ] && [ "$PRODUCT_TYPE" != "apppay" ]; then
    echo "❌ --product-type 参数值无效: ${PRODUCT_TYPE}（仅支持 aipay|webpay|apppay）" >&2
    return 1
  fi

  case "${SALES_CODE:-}" in
    "") ;;
    "I1080300001000160457") EXPECTED_PRODUCT_TYPE="aipay" ;;
    "I1080300001000041203") EXPECTED_PRODUCT_TYPE="webpay" ;;
    "I1080300001000041313") EXPECTED_PRODUCT_TYPE="apppay" ;;
    *) echo "❌ 未知 salesCode: ${SALES_CODE}" >&2; return 1 ;;
  esac

  if [ -n "$EXPECTED_PRODUCT_TYPE" ] && [ -n "${PRODUCT_TYPE:-}" ] && [ "$PRODUCT_TYPE" != "$EXPECTED_PRODUCT_TYPE" ]; then
    echo "❌ --product-type 与 --sales-code 不匹配：--product-type=${PRODUCT_TYPE}, --sales-code=${SALES_CODE}" >&2
    echo "📋 期望 --product-type=${EXPECTED_PRODUCT_TYPE}" >&2
    return 1
  fi

  case "${PRODUCT_TYPE:-$EXPECTED_PRODUCT_TYPE}" in
    apppay) EXPECTED_APPLICATION_TYPE="MOBILEAPP" ;;
    aipay|webpay) EXPECTED_APPLICATION_TYPE="WEBAPP" ;;
  esac

  if [ -n "$EXPECTED_APPLICATION_TYPE" ] && [ -n "${APPLICATION_TYPE:-}" ] && [ "$APPLICATION_TYPE" != "$EXPECTED_APPLICATION_TYPE" ]; then
    echo "❌ --application-type 与产品上下文不匹配：--application-type=${APPLICATION_TYPE}" >&2
    echo "📋 期望 --application-type=${EXPECTED_APPLICATION_TYPE}" >&2
    return 1
  fi
}

resolve_application_type() {
  validate_app_context || return 1

  if [ -n "${APPLICATION_TYPE:-}" ]; then
    case "$APPLICATION_TYPE" in
      WEBAPP|MOBILEAPP) echo "$APPLICATION_TYPE" ;;
      *) echo "❌ APPLICATION_TYPE 参数值无效: ${APPLICATION_TYPE}（仅支持 WEBAPP|MOBILEAPP）" >&2; return 1 ;;
    esac
    return 0
  fi

  if [ "${PRODUCT_TYPE:-}" = "apppay" ] || [ "${SALES_CODE:-}" = "I1080300001000041313" ]; then
    echo "MOBILEAPP"
  else
    echo "WEBAPP"
  fi
}

query_security_key() {
  local APP_ID="$1"
  local PUBLIC_KEY="${2:-}"

  if [ -n "$PUBLIC_KEY" ]; then
    REQUEST_JSON=$(jq -n \
      --arg appId "$APP_ID" \
      --arg publicKey "$PUBLIC_KEY" \
      '{request:{appId:$appId,publicKey:$publicKey}}')
  else
    REQUEST_JSON=$(jq -n --arg appId "$APP_ID" '{request:{appId:$appId}}')
  fi

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationSecurityKey \
    -d "$REQUEST_JSON" \
    --json 2>/dev/null)

  if ! handle_error "$RESULT"; then
    return 1
  fi

  SECURITY_BUSINESS=$(unwrap_mcp "$RESULT")
  SUCCESS=$(echo "$SECURITY_BUSINESS" | jq -r '.success // false')

  if [ "$SUCCESS" != "true" ]; then
    ERROR_MSG=$(echo "$SECURITY_BUSINESS" | jq -r '.error.message // .errorMessage // "未知错误"')
    echo "❌ 查询安全密钥失败: $ERROR_MSG"
    return 1
  fi
}

query_application_info() {
  local APP_ID="$1"

  REQUEST_JSON=$(jq -n --arg appId "$APP_ID" '{request:{appId:$appId}}')
  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationInfo \
    -d "$REQUEST_JSON" \
    --json 2>/dev/null)

  if ! handle_error "$RESULT"; then
    return 1
  fi

  APPLICATION_INFO_BUSINESS=$(unwrap_mcp "$RESULT")
  SUCCESS=$(echo "$APPLICATION_INFO_BUSINESS" | jq -r '.success // false')

  if [ "$SUCCESS" != "true" ]; then
    ERROR_MSG=$(echo "$APPLICATION_INFO_BUSINESS" | jq -r '.error.message // .errorMessage // "未知错误"')
    echo "❌ 查询应用信息失败: $ERROR_MSG"
    return 1
  fi
}

extract_application_status() {
  echo "$APPLICATION_INFO_BUSINESS" | jq -r '
    .resultObj.status //
    empty
  '
}

extract_rsa2_application_key_info() {
  echo "$SECURITY_BUSINESS" | jq -c '
    (
      (.resultObj.securityConfigList // []) +
      (.resultObj.securityKeys // []) +
      (.data.securityConfigList // []) +
      (.data.securityKeys // []) +
      (.resultObj.alipayKeyList // []) +
      (.data.alipayKeyList // [])
    ) |
    map(select(
      .signType == "RSA2" and
      (
        ((.partnerPublicKey // "") != "") or
        ((.certInfoDTO | type) == "object" and ((.certInfoDTO.publicKey // "") != ""))
      )
    )) |
    .[0] // {}
  '
}

extract_rsa2_alipay_key_info() {
  echo "$SECURITY_BUSINESS" | jq -c '
    (
      (.resultObj.alipayKeyList // []) +
      (.data.alipayKeyList // []) +
      (.resultObj.securityKeys // []) +
      (.data.securityKeys // [])
    ) |
    map(select(.signType == "RSA2" and ((.alipayPublicKey // "") != ""))) |
    .[0] // {}
  '
}

extract_alipay_public_key() {
  local RSA2_KEY_INFO="$1"
  echo "$RSA2_KEY_INFO" | jq -r '.alipayPublicKey // empty'
}

is_rsa2_application_key_ready() {
  local RSA2_KEY_INFO="$1"

  echo "$RSA2_KEY_INFO" | jq -e '
    ((.partnerPublicKey // "") != "") or
    (
      (.certInfoDTO | type) == "object" and
      ((.certInfoDTO.publicKey // "") != "")
    )
  ' >/dev/null 2>&1
}

write_alipay_public_key() {
  local APP_ID="$1"
  local ALIPAY_PUBLIC_KEY="$2"

  local KEY_FILE="${HOME}/.config/${APP_ID}-alipayPublicKey.keytext"
  local ATTEMPT=1
  local MAX_ATTEMPTS="${ALIPAY_PUBLIC_KEY_WRITE_RETRIES:-3}"

  if ! [[ "$MAX_ATTEMPTS" =~ ^[1-9][0-9]*$ ]]; then
    MAX_ATTEMPTS=3
  fi

  while [ "$ATTEMPT" -le "$MAX_ATTEMPTS" ]; do
    if { mkdir -p "${HOME}/.config" && printf '%s\n' "$ALIPAY_PUBLIC_KEY" > "$KEY_FILE"; } 2>/dev/null; then
      return 0
    fi

    if [ "$ATTEMPT" -lt "$MAX_ATTEMPTS" ]; then
      echo "⚠️ 支付宝公钥文件写入失败，正在重试（${ATTEMPT}/${MAX_ATTEMPTS}）..."
      sleep 1
    fi
    ATTEMPT=$((ATTEMPT + 1))
  done

  echo "⚠️ 已获取到 RSA2 支付宝公钥，但本地文件写入失败: $KEY_FILE"
  echo "📋 这不是公钥查询失败；当前流程会继续，但后续集成时需要手动配置支付宝公钥。"
  echo "📋 可检查本地目录权限或设置 HOME 后重新执行导出。"
  return 1
}

ensure_rsa2_application_key_ready() {
  local APP_ID="$1"
  local PUBLIC_KEY="${2:-}"
  local ATTEMPT=1
  local MAX_ATTEMPTS="${APP_KEY_VERIFY_RETRIES:-6}"
  local INTERVAL_SECONDS="${APP_KEY_VERIFY_INTERVAL_SECONDS:-5}"

  if ! [[ "$MAX_ATTEMPTS" =~ ^[1-9][0-9]*$ ]]; then
    MAX_ATTEMPTS=6
  fi

  if ! [[ "$INTERVAL_SECONDS" =~ ^[0-9]+$ ]]; then
    INTERVAL_SECONDS=5
  fi

  while [ "$ATTEMPT" -le "$MAX_ATTEMPTS" ]; do
    query_security_key "$APP_ID" "$PUBLIC_KEY" || return 1

    RSA2_KEY_INFO=$(extract_rsa2_application_key_info)

    if is_rsa2_application_key_ready "$RSA2_KEY_INFO"; then
      return 0
    fi

    if [ "$ATTEMPT" -lt "$MAX_ATTEMPTS" ]; then
      echo "⏳ 支付宝侧尚未确认到应用公钥生效，正在重试（${ATTEMPT}/${MAX_ATTEMPTS}）..."
      sleep "$INTERVAL_SECONDS"
    fi

    ATTEMPT=$((ATTEMPT + 1))
  done

  echo "❌ 应用公钥尚未确认，请先完成公钥设置确认"
  return 1
}

verify_key_and_export_alipay_public_key() {
  local APP_ID="$1"

  query_security_key "$APP_ID" || return 1

  RSA2_KEY_INFO=$(extract_rsa2_application_key_info)
  RSA2_ALIPAY_KEY_INFO=$(extract_rsa2_alipay_key_info)
  ALIPAY_PUBLIC_KEY=$(extract_alipay_public_key "$RSA2_ALIPAY_KEY_INFO")

  if ! is_rsa2_application_key_ready "$RSA2_KEY_INFO"; then
    echo "❌ 应用公钥尚未确认，禁止提交应用审核"
    return 1
  fi

  if [ -z "$ALIPAY_PUBLIC_KEY" ]; then
    echo "❌ 已确认应用公钥生效，但未获取到 RSA2 支付宝公钥，禁止提交应用审核"
    return 1
  fi

  if write_alipay_public_key "$APP_ID" "$ALIPAY_PUBLIC_KEY"; then
    echo "✅ 已确认应用公钥并导出支付宝公钥"
    echo "支付宝公钥保存路径: ~/.config/${APP_ID}-alipayPublicKey.keytext"
    echo "ALIPAY_PUBLIC_KEY_EXPORT_STATUS=EXPORTED"
    echo "ALIPAY_PUBLIC_KEY_FILE=~/.config/${APP_ID}-alipayPublicKey.keytext"
  else
    echo "ALIPAY_PUBLIC_KEY_EXPORT_STATUS=MANUAL_CONFIGURATION_REQUIRED"
  fi

  return 0
}

# ─── app list: 查询已有应用 ──────────────────────────────────────────────────
app_list() {
  parse_app_context_args "$@" || {
    echo "用法: bash app.sh list [--product-type aipay|webpay|apppay] [--sales-code <code>] [--application-type WEBAPP|MOBILEAPP]"
    exit 1
  }
  APP_APPLICATION_TYPE=$(resolve_application_type) || exit 1
  REQUEST_JSON=$(jq -n --arg applicationType "$APP_APPLICATION_TYPE" '{request:{appTypes:[$applicationType]}}')

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.queryApplicationList \
    -d "$REQUEST_JSON" \
    --json 2>/dev/null)

  # 错误检测
  if ! handle_error "$RESULT"; then
    exit 1
  fi

  # 解包 MCP 信封后解析业务字段
  BUSINESS=$(unwrap_mcp "$RESULT")
  SUCCESS=$(echo "$BUSINESS" | jq -r '.success // false')

  if [ "$SUCCESS" != "true" ]; then
    ERROR_MSG=$(echo "$BUSINESS" | jq -r '.error.message // .errorMessage // "未知错误"')
    echo "❌ 查询应用列表失败: $ERROR_MSG"
    echo "FLOW:ERROR"
    exit 1
  fi

  APP_LIST=$(echo "$BUSINESS" | jq -c '
    if (.resultObj | type) == "object" and (.resultObj.applicationList | type) == "array" then
      .resultObj.applicationList
    elif (.resultObj | type) == "array" then
      .resultObj
    else
      null
    end
  ' 2>/dev/null)
  if [ -z "$APP_LIST" ] || ! echo "$APP_LIST" | jq -e 'type == "array"' >/dev/null 2>&1; then
    echo "❌ 应用列表返回结构异常，无法解析应用列表"
    echo "FLOW:ERROR"
    exit 1
  fi
  APP_COUNT=$(echo "$APP_LIST" | jq 'length')

  if [ "$APP_COUNT" -eq 0 ]; then
    echo "📋 暂无 ${APP_APPLICATION_TYPE} 应用；记录新建候选分支，等待 Step 4 统一决策"
    echo "FLOW:CREATE_NEW"
  else
    # 实际返回的状态字段值是 "ON_LINE"（带下划线）
    ONLINE_APPS=$(echo "$APP_LIST" | jq -r '[.[] | select(.status == "ON_LINE")]')
    NON_ONLINE_APPS=$(echo "$APP_LIST" | jq -r '[.[] | select(.status != "ON_LINE")]')
    ONLINE_COUNT=$(echo "$ONLINE_APPS" | jq 'length')
    NON_ONLINE_COUNT=$(echo "$NON_ONLINE_APPS" | jq 'length')
    ONLINE_COUNT=${ONLINE_COUNT:-0}
    NON_ONLINE_COUNT=${NON_ONLINE_COUNT:-0}

    if [ "$ONLINE_COUNT" -eq 0 ]; then
      echo "📋 当前已有以下尚未上线的 ${APP_APPLICATION_TYPE} 应用："
      echo ""
      echo "| 应用ID | 应用名称 | 实际状态 |"
      echo "|--------|----------|----------|"
      for i in $(seq 0 $((NON_ONLINE_COUNT-1))); do
        APP=$(echo "$NON_ONLINE_APPS" | jq ".[$i]")
        APP_ID=$(echo "$APP" | jq -r '.appId // "未知"')
        APP_NAME=$(echo "$APP" | jq -r '.appName // "未知"')
        APP_STATUS=$(echo "$APP" | jq -r '.status // "未知"')
        echo "| $APP_ID | $APP_NAME | $APP_STATUS |"
      done
      echo ""
      echo "以上应用当前不可复用。请等待应用上线，或在了解现有应用状态后明确选择新建。"
      echo "FLOW:PENDING_APPLICATIONS"
    else
      echo "📋 发现您已有以下上线 ${APP_APPLICATION_TYPE} 应用："
      echo ""
      echo "| 序号 | 应用ID | 应用名称 | 状态 |"
      echo "|------|--------|----------|------|"

      for i in $(seq 0 $((ONLINE_COUNT-1))); do
        APP=$(echo "$ONLINE_APPS" | jq ".[$i]")
        APP_ID=$(echo "$APP" | jq -r '.appId // "未知"')
        APP_NAME=$(echo "$APP" | jq -r '.appName // "未知"')
        echo "| $((i+1)) | $APP_ID | $APP_NAME | 已上线 |"
      done

      if [ "$NON_ONLINE_COUNT" -gt 0 ]; then
        echo ""
        echo "另有以下尚未上线的同类型应用，当前不可复用："
        echo ""
        echo "| 应用ID | 应用名称 | 实际状态 |"
        echo "|--------|----------|----------|"
        for i in $(seq 0 $((NON_ONLINE_COUNT-1))); do
          APP=$(echo "$NON_ONLINE_APPS" | jq ".[$i]")
          APP_ID=$(echo "$APP" | jq -r '.appId // "未知"')
          APP_NAME=$(echo "$APP" | jq -r '.appName // "未知"')
          APP_STATUS=$(echo "$APP" | jq -r '.status // "未知"')
          echo "| $APP_ID | $APP_NAME | $APP_STATUS |"
        done
        echo ""
        echo "新建前请结合以上状态避免重复创建。"
      fi

      echo ""
      echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
      echo "  请选择操作："
      echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
      echo ""
      echo "  【复用】输入序号 (1-$ONLINE_COUNT) → 复用对应应用"
      echo "  【新建】输入 \"新建\" → 创建新应用"
      echo ""
      echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
      echo "FLOW:SELECT"
    fi
  fi
}

# ─── app create: 创建新应用 ──────────────────────────────────────────────────
app_create() {
  parse_app_context_args "$@" || {
    echo "用法: bash app.sh create [--product-type aipay|webpay|apppay] [--sales-code <code>] [--application-type WEBAPP|MOBILEAPP] [--mobile-platform IOS|ANDROID|ALL --bundle-id <id> --app-package <pkg> --app-sign <sign>]"
    exit 1
  }
  APP_APPLICATION_TYPE=$(resolve_application_type) || exit 1
  validate_mobile_platform_args "$APP_APPLICATION_TYPE" || exit 1
  REQUEST_JSON=$(build_create_application_request "$APP_APPLICATION_TYPE") || {
    echo "❌ 创建应用请求参数组装失败"
    exit 1
  }

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.createApplication \
    -d "$REQUEST_JSON" \
    --json 2>/dev/null)

  # 错误检测
  if ! handle_error "$RESULT"; then
    exit 1
  fi

  # 解包 MCP 信封
  BUSINESS=$(unwrap_mcp "$RESULT")
  SUCCESS=$(echo "$BUSINESS" | jq -r '.success // false')

  if [ "$SUCCESS" = "true" ]; then
    APP_ID=$(echo "$BUSINESS" | jq -r '.resultObj.appId // .data.appId // empty')
    if [ -z "$APP_ID" ]; then
      echo "❌ 应用创建返回成功，但未解析到 appId，禁止进入公钥设置流程"
      echo "FLOW:ERROR"
      exit 1
    fi
    echo "✅ 应用创建成功"
    echo "APPLICATION_TYPE=$APP_APPLICATION_TYPE"
    echo "APP_ID=$APP_ID"
  else
    ERROR_MSG=$(echo "$BUSINESS" | jq -r '.error.message // .errorMessage // "未知错误"')
    echo "❌ 应用创建失败: $ERROR_MSG"
    exit 1
  fi
}

# ─── app key: 设置应用公钥 ───────────────────────────────────────────────────
app_key() {
  if [ "$#" -ne 2 ] || [[ "${1:-}" == --* ]]; then
    echo "❌ 参数格式错误：key 使用位置参数 <appId> <publicKey>，不支持 --app-id 等未定义选项"
    echo "用法: bash app.sh key <appId> <publicKey>"
    exit 1
  fi

  APP_ID="$1"
  PUBLIC_KEY="$2"

  if [ -z "$APP_ID" ] || [ -z "$PUBLIC_KEY" ]; then
    echo "❌ 请提供 appId 和 publicKey"
    echo "用法: bash app.sh key <appId> <publicKey>"
    exit 1
  fi

  REQUEST_JSON=$(jq -n \
    --arg appId "$APP_ID" \
    --arg publicKey "$PUBLIC_KEY" \
    '{request:{appId:$appId,signType:"RSA2",publicKey:$publicKey}}')

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.createKeyConfirmPage \
    -d "$REQUEST_JSON" \
    --json 2>/dev/null)

  # 错误检测
  if ! handle_error "$RESULT"; then
    exit 1
  fi

  # 解包 MCP 信封
  BUSINESS=$(unwrap_mcp "$RESULT")
  SUCCESS=$(echo "$BUSINESS" | jq -r '.success // false')

  if [ "$SUCCESS" = "true" ]; then
    CONFIRM_PAGE_URL=$(echo "$BUSINESS" | jq -r '.resultObj.confirmPageUrl // empty')

    if [ -n "$CONFIRM_PAGE_URL" ]; then
      echo "📋 应用公钥设置中..."
      echo ""
      echo "请点击以下链接确认公钥设置："
      echo ""
      echo "提示：无法跳链时，请复制下方链接到网页浏览器打开。"
      echo "[点击确认公钥设置]($CONFIRM_PAGE_URL)"
      echo ""
      echo "链接有效期为10分钟。"
      echo ""
      echo "确认完成后，请告诉我\"好了\"继续后续流程。"
    else
      echo "❌ 未获取到确认页面链接，请重试"
      exit 1
    fi
  else
    ERROR_MSG=$(echo "$BUSINESS" | jq -r '.error.message // .errorMessage // "未知错误"')
    echo "❌ 公钥设置失败: $ERROR_MSG"
    exit 1
  fi
}

# ─── app audit: 提交应用审核 ─────────────────────────────────────────────────
app_audit() {
  if [ "$#" -ne 1 ] || [[ "${1:-}" == --* ]]; then
    echo "❌ 参数格式错误：audit 使用位置参数 <appId>，不支持 --app-id 等未定义选项"
    echo "用法: bash app.sh audit <appId>"
    exit 1
  fi

  APP_ID="$1"

  if [ -z "$APP_ID" ]; then
    echo "❌ 请提供 appId"
    echo "用法: bash app.sh audit <appId>"
    exit 1
  fi

  verify_key_and_export_alipay_public_key "$APP_ID" || exit 1

  REQUEST_JSON=$(jq -n --arg appId "$APP_ID" '{request:{appId:$appId}}')

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call apprelease.submitApplicationAudit \
    -d "$REQUEST_JSON" \
    --json 2>/dev/null)

  # 错误检测
  if ! handle_error "$RESULT"; then
    exit 1
  fi

  # 解包 MCP 信封
  BUSINESS=$(unwrap_mcp "$RESULT")
  SUCCESS=$(echo "$BUSINESS" | jq -r '.success // false')

  if [ "$SUCCESS" = "true" ]; then
    echo "✅ 应用审核提交成功"
    echo "APP_ID=$APP_ID"
    echo "FLOW:AUDIT_SUBMITTED"
  else
    ERROR_MSG=$(echo "$BUSINESS" | jq -r '.error.message // .errorMessage // "未知错误"')
    echo "❌ 提交审核失败: $ERROR_MSG"
    exit 1
  fi
}

# ─── app verify-key: 确认应用公钥已设置 ─────────────────────────────────────
app_verify_key() {
  if [ "$#" -lt 1 ] || [ "$#" -gt 2 ] || [[ "${1:-}" == --* ]]; then
    echo "❌ 参数格式错误：verify-key 使用位置参数 <appId> [publicKey]，不支持 --app-id 等未定义选项"
    echo "用法: bash app.sh verify-key <appId> [publicKey]"
    exit 1
  fi

  APP_ID="$1"
  PUBLIC_KEY="${2:-}"

  if [ -z "$APP_ID" ]; then
    echo "❌ 请提供 appId"
    echo "用法: bash app.sh verify-key <appId> [publicKey]"
    exit 1
  fi

  if ensure_rsa2_application_key_ready "$APP_ID" "$PUBLIC_KEY"; then
    echo "✅ 应用公钥已确认"
    echo "APP_ID=$APP_ID"
    echo "FLOW:KEY_CONFIRMED"
  else
    echo "FLOW:KEY_NOT_CONFIRMED"
    exit 1
  fi
}

# ─── app reuse: 复用已有应用（查询安全密钥） ──────────────────────────────────
app_reuse() {
  if [ "$#" -ne 1 ] || [[ "${1:-}" == --* ]]; then
    echo "❌ 参数格式错误：reuse 使用位置参数 <appId>，不支持 --app-id 等未定义选项"
    echo "用法: bash app.sh reuse <appId>"
    exit 1
  fi

  APP_ID="$1"

  if [ -z "$APP_ID" ]; then
    echo "❌ 请提供 appId"
    echo "用法: bash app.sh reuse <appId>"
    exit 1
  fi

  if query_application_info "$APP_ID"; then
    APP_STATUS=$(extract_application_status)
    if [ "$APP_STATUS" != "ON_LINE" ]; then
      echo "❌ 仅允许复用 ON_LINE（已上线）应用，当前应用状态: ${APP_STATUS:-未知}"
      echo "FLOW:ERROR"
      exit 1
    fi
  else
    echo "FLOW:ERROR"
    exit 1
  fi

  if query_security_key "$APP_ID"; then
    RSA2_KEY_INFO=$(extract_rsa2_application_key_info)
    RSA2_ALIPAY_KEY_INFO=$(extract_rsa2_alipay_key_info)
    ALIPAY_PUBLIC_KEY=$(extract_alipay_public_key "$RSA2_ALIPAY_KEY_INFO")

    if ! is_rsa2_application_key_ready "$RSA2_KEY_INFO"; then
      echo "⚠️ 未确认到已生效的 RSA2 应用公钥"
      echo "FLOW:REUSE_NO_KEY"
      exit 1
    elif [ -n "$ALIPAY_PUBLIC_KEY" ]; then
      if write_alipay_public_key "$APP_ID" "$ALIPAY_PUBLIC_KEY"; then
        echo "✅ 已获取支付宝公钥"
        echo "支付宝公钥保存路径: ~/.config/${APP_ID}-alipayPublicKey.keytext"
        echo "ALIPAY_PUBLIC_KEY_EXPORT_STATUS=EXPORTED"
        echo "ALIPAY_PUBLIC_KEY_FILE=~/.config/${APP_ID}-alipayPublicKey.keytext"
      else
        echo "ALIPAY_PUBLIC_KEY_EXPORT_STATUS=MANUAL_CONFIGURATION_REQUIRED"
      fi
      echo "APP_ID=$APP_ID"
      echo "FLOW:REUSE_SUCCESS"
    else
      echo "❌ 已确认应用公钥生效，但未获取到 RSA2 支付宝公钥"
      echo "FLOW:ERROR"
      exit 1
    fi
  else
    echo "FLOW:ERROR"
    exit 1
  fi
}

# ─── 入口分发 ────────────────────────────────────────────────────────────────
APP_COMMAND="${1:-}"
case "$APP_COMMAND" in
  list|create|key|verify-key|audit|reuse)
    require_command alipay-cli || exit 1
    ;;
esac

case "$APP_COMMAND" in
  list)
    shift
    app_list "$@"
    ;;
  create)
    shift
    app_create "$@"
    ;;
  key)
    shift
    app_key "$@"
    ;;
  verify-key)
    shift
    app_verify_key "$@"
    ;;
  audit)
    shift
    app_audit "$@"
    ;;
  reuse)
    shift
    app_reuse "$@"
    ;;
  *)
    echo "用法: bash app.sh <list|create|key|verify-key|audit|reuse> [参数...]"
    echo ""
    echo "  app list [--product-type <type> --sales-code <code> --application-type <type>]   - 查询已有应用"
    echo "  app create [--product-type <type> --sales-code <code> --application-type <type>] [--mobile-platform IOS|ANDROID|ALL ...] - 创建新应用"
    echo "  app key    <appId> <pubKey> - 设置公钥"
    echo "  app verify-key <appId> [pubKey] - 确认公钥设置结果"
    echo "  app audit  <appId>          - 提交审核"
    echo "  app reuse  <appId>          - 复用应用"
    exit 1
    ;;
esac
