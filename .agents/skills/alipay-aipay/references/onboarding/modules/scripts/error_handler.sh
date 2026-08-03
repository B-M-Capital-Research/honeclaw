#!/bin/bash
#=============================================================================
# 脚本名称: error_handler.sh
# 功能描述: 签约流程统一错误检测与处理，供所有 .sh 脚本 source 引用
# 源文件: error-handling.md（本文档为可执行版本）
# 用法: source scripts/error_handler.sh
#       if ! handle_error "$RESULT"; then return 1; fi
#=============================================================================

ERROR_HANDLER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMMON_SCRIPT="${ERROR_HANDLER_DIR}/../../../normal/scripts/common.sh"
if [ -f "$COMMON_SCRIPT" ]; then
  # shellcheck source=/dev/null
  source "$COMMON_SCRIPT"
  init_dev_tool_name
else
  echo "❌ 缺少公共脚本: $COMMON_SCRIPT"
  return 1 2>/dev/null || exit 1
fi

# ─── MCP 信封解包 ──────────────────────────────────────────────────────────
# alipay-cli mcp call 返回 MCP 协议信封：
#   { "content": [ { "text": "<业务JSON>", "type": "text" } ] }
# 业务 JSON（含 success / resultObj / errorCode）被包在 content[0].text 里。
# 本函数负责解包：若检测到信封结构，提取 content[0].text；否则原样返回。
# 当 stdout 混有日志时，只接受唯一可确定的 MCP 信封或 JSON；候选不唯一时
# 保留原始文本，让后续错误检测安全阻断，禁止猜测业务响应。
#
# 对于非 mcp call 的命令（如 alipay-cli login / whoami），返回不带信封，
# 保证这些命令的原样透传。
extract_unique_json() {
  local RAW="$1"
  printf '%s' "$RAW" | jq -Rsc '
    explode as $chars |
    reduce range(0; ($chars | length)) as $index (
      {
        depth: 0,
        start: null,
        in_string: false,
        escaped: false,
        raw_candidates: []
      };
      ($chars[$index]) as $char |
      if .depth == 0 then
        if $char == 123 or $char == 91 then
          .depth = 1 |
          .start = $index |
          .in_string = false |
          .escaped = false
        else
          .
        end
      elif .in_string then
        if .escaped then
          .escaped = false
        elif $char == 92 then
          .escaped = true
        elif $char == 34 then
          .in_string = false
        else
          .
        end
      elif $char == 34 then
        .in_string = true
      elif $char == 123 or $char == 91 then
        .depth += 1
      elif $char == 125 or $char == 93 then
        .depth -= 1 |
        if .depth == 0 then
          ($chars[.start:($index + 1)] | implode) as $candidate |
          .raw_candidates += [$candidate] |
          .start = null
        else
          .
        end
      else
        .
      end
    ) |
    [
      .raw_candidates[] |
      fromjson? |
      select(type == "object" or type == "array")
    ] |
    unique as $candidates |
    (
      $candidates |
      map(select(
        type == "object" and
        (.content | type) == "array" and
        any(.content[]?; (.text? | type) == "string")
      )) |
      unique
    ) as $envelopes |
    if ($envelopes | length) == 1 then
      {status: "ok", value: $envelopes[0]}
    elif ($envelopes | length) == 0 and ($candidates | length) == 1 then
      {status: "ok", value: $candidates[0]}
    elif ($candidates | length) == 0 then
      {status: "none"}
    else
      {status: "ambiguous"}
    end
  ' 2>/dev/null
}

unwrap_mcp() {
  local RAW="$1"
  if [ -z "$RAW" ]; then
    echo ""
    return
  fi

  local ANALYSIS STATUS JSON
  ANALYSIS=$(extract_unique_json "$RAW")
  STATUS=$(echo "$ANALYSIS" | jq -r '.status // "none"' 2>/dev/null)
  case "$STATUS" in
    ok)
      JSON=$(echo "$ANALYSIS" | jq -c '.value' 2>/dev/null)
      ;;
    ambiguous)
      echo "CLI 输出包含多个 JSON 候选，无法唯一解析"
      echo "$RAW"
      return
      ;;
    *)
      echo "$RAW"
      return
      ;;
  esac

  local TEXT
  TEXT=$(echo "$JSON" | jq -r 'if (.content | type == "array") and (.content[0].text != null) then .content[0].text else empty end' 2>/dev/null)
  if [ -n "$TEXT" ]; then
    echo "$TEXT"
  else
    echo "$JSON"
  fi
}

# ─── JSON 错误字段提取 ─────────────────────────────────────────────────────
# 部分 MCP 后端会把真实业务响应包在 data/response 或 errorContext.errorStack 下。
# 下面的提取函数优先定位同一个错误对象，避免 success=false 时丢失真实错误体。
extract_first_named_field() {
  local JSON="$1"
  local FIELD="$2"
  echo "$JSON" | jq -r --arg field "$FIELD" '
    [.. | objects | .[$field]? | select(. != null and (tostring != ""))][0] // ""
  ' 2>/dev/null
}

extract_error_object() {
  echo "$1" | jq -c '
    [
      .. | objects |
      select(
        ((.errorCode? | type) == "string" and .errorCode != "") or
        ((.errorCode? | type) == "object" and ((.errorCode.code? // "") != ""))
      )
    ][0] // empty
  ' 2>/dev/null
}

extract_error_code() {
  local ERROR_OBJECT
  ERROR_OBJECT=$(extract_error_object "$1")
  if [ -n "$ERROR_OBJECT" ]; then
    echo "$ERROR_OBJECT" | jq -r '
      if (.errorCode | type) == "string" then .errorCode
      elif (.errorCode | type) == "object" then (.errorCode.code // "")
      else ""
      end
    ' 2>/dev/null
    return
  fi

  echo ""
}

extract_error_message() {
  local JSON="$1"
  local ERROR_OBJECT
  ERROR_OBJECT=$(extract_error_object "$JSON")
  if [ -n "$ERROR_OBJECT" ]; then
    echo "$ERROR_OBJECT" | jq -r '
      .errorMessage //
      .errorMsg //
      .error.message //
      .message //
      (if (.errorCode | type) == "object" then (.errorCode.desc // .errorCode.message) else empty end) //
      "未知错误"
    ' 2>/dev/null
    return
  fi

  echo "$JSON" | jq -r '
    if .data.success? == false then
      [
        .data.subMsg?,
        .data.msg?,
        .subMsg?,
        .errorMessage?,
        .errorMsg?,
        .message?,
        (.error? | objects | .message?),
        .msg?,
        (
          .. | objects |
          (.errorMessage?, .errorMsg?, .message?, (.error? | objects | .message?))
        )
      ] |
      [
        .[] |
        select(
          . != null and
          (tostring != "") and
          ((tostring | ascii_downcase) != "success")
        )
      ][0] // "未知错误"
    else
      [
        .. | objects |
        (.errorMessage?, .errorMsg?, .message?, (.error? | objects | .message?)) |
        select(. != null and (tostring != ""))
      ][0] // "未知错误"
    end
  ' 2>/dev/null
}

extract_biz_tips() {
  local ERROR_OBJECT
  ERROR_OBJECT=$(extract_error_object "$1")
  if [ -n "$ERROR_OBJECT" ]; then
    echo "$ERROR_OBJECT" | jq -r '.bizTips // ""' 2>/dev/null
    return
  fi

  extract_first_named_field "$1" "bizTips"
}

extract_need_retry() {
  local JSON="$1"
  local ERROR_OBJECT
  ERROR_OBJECT=$(extract_error_object "$JSON")
  if [ -n "$ERROR_OBJECT" ]; then
    echo "$ERROR_OBJECT" | jq -r 'if .needRetry == true then "true" else "false" end' 2>/dev/null
    return
  fi

  echo "$JSON" | jq -r '
    if any(.. | objects; .needRetry? == true) then "true" else "false" end
  ' 2>/dev/null
}

extract_error_object_field() {
  local JSON="$1"
  local FIELD="$2"
  local ERROR_OBJECT
  ERROR_OBJECT=$(extract_error_object "$JSON")
  if [ -n "$ERROR_OBJECT" ]; then
    echo "$ERROR_OBJECT" | jq -r --arg field "$FIELD" '
      .[$field] //
      (if (.errorCode | type) == "object" then (.errorCode[$field] // "") else "" end)
    ' 2>/dev/null
    return
  fi

  extract_first_named_field "$JSON" "$FIELD"
}

extract_success_state() {
  local JSON="$1"
  echo "$JSON" | jq -r '
    if any(.. | objects; .success? == false) then "false"
    elif any(.. | objects; .success? == true) then "true"
    else "null"
    end
  ' 2>/dev/null
}

extract_checked_errors() {
  local JSON="$1"
  echo "$JSON" | jq -c '
    [
      .. | objects |
      (
        .checkedError?,
        (.resultObj? | objects | .checkedError?),
        (.data? | objects | .checkedError?)
      ) |
      select(. != null)
    ][0] // empty
  ' 2>/dev/null
}

# ─── 统一错误检测 ──────────────────────────────────────────────────────────
# 参数: $1 - CLI 执行结果文本（原始输出，含信封或不含）
# 返回: MCP_AUTH_ERROR | MCP_SERVICE_ERROR | AUTH_MISMATCH | SERVICE_UNSTABLE | ERROR:xxx | CLI_ERROR:xxx | SUCCESS
detect_error() {
  local CLI_RESULT="$1"

  # 空输入视为异常，不应被当作 SUCCESS
  if [ -z "$CLI_RESULT" ]; then
    echo "CLI_ERROR:CLI 返回为空"
    return
  fi

  # 先解包 MCP 信封，再进行错误检测
  local UNWRAPPED=$(unwrap_mcp "$CLI_RESULT")

  # 1. 优先检测 MCP 认证错误（HTTP 401）
  if echo "$UNWRAPPED" | grep -qiE "HTTP 401|Authorization is empty|非法的认证信息"; then
    echo "MCP_AUTH_ERROR"
    return
  fi

  # 2. MCP 服务不稳定（后端返回的服务异常）
  if echo "$UNWRAPPED" | grep -qiE "MCP.*服务.*不稳定|服务暂时不可用"; then
    echo "SERVICE_UNSTABLE"
    return
  fi

  # 3. MCP 调用失败（网络/连接错误）
  if echo "$UNWRAPPED" | grep -qiE "MCP 调用失败|connection refused|timeout|network error"; then
    echo "MCP_SERVICE_ERROR"
    return
  fi

  # 4. 授权信息不匹配（MCC/产品/scope 未授权）
  if echo "$UNWRAPPED" | grep -qiE "mccCode.*is not auth|salesProductCodes.*is not auth|scope.*is not auth"; then
    echo "AUTH_MISMATCH"
    return
  fi

  if ! echo "$UNWRAPPED" | jq -e . >/dev/null 2>&1; then
    local FIRST_LINE
    FIRST_LINE=$(printf "%s" "$UNWRAPPED" | sed -n '1p')
    echo "CLI_ERROR:${FIRST_LINE:-CLI 返回非 JSON 内容}"
    return
  fi

  # 5. 通用业务错误（从解包后的 JSON 中提取 errorCode）
  local ERROR_CODE=$(extract_error_code "$UNWRAPPED")
  if [ -n "$ERROR_CODE" ] && [ "$ERROR_CODE" != "null" ]; then
    echo "ERROR:$ERROR_CODE"
    return
  fi

  # 6. CLI 命令本身的错误（success: false，从解包后的 JSON 提取）
  # 注意: MCP 业务响应可能嵌套在 data/response 下，因此递归检测 success=false。
  local SUCCESS=$(extract_success_state "$UNWRAPPED")
  if [ "$SUCCESS" = "false" ]; then
    local ERROR_MSG=$(extract_error_message "$UNWRAPPED")
    echo "CLI_ERROR:$ERROR_MSG"
    return
  fi

  echo "SUCCESS"
}

# ─── MCP 认证错误处理 ──────────────────────────────────────────────────────
handle_mcp_auth_error() {
  local LOGOUT_RESULT LOGOUT_EXIT_CODE LOGOUT_ERROR_TYPE LOGOUT_ERROR_MSG

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🔐 未登录或授权已过期"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "❌ 当前登录状态已失效，无法继续操作"
  echo ""
  echo "📋 原因：MCP 调用返回认证错误（HTTP 401）"
  echo "📋 说明：Authorization is empty（认证信息为空）"
  echo ""
  echo "🔄 正在退出当前登录..."
  LOGOUT_RESULT=$(PLATFORM=${DEV_TOOL_NAME:-unknown} alipay-cli logout --json 2>&1)
  LOGOUT_EXIT_CODE=$?
  LOGOUT_ERROR_TYPE=$(detect_error "$LOGOUT_RESULT")

  if [ "$LOGOUT_EXIT_CODE" -ne 0 ] || [ "$LOGOUT_ERROR_TYPE" != "SUCCESS" ]; then
    echo ""
    echo "❌ 退出当前登录失败，已停止重新授权"
    if echo "$LOGOUT_RESULT" | jq -e . >/dev/null 2>&1; then
      LOGOUT_ERROR_MSG=$(extract_error_message "$LOGOUT_RESULT")
      echo "📋 错误信息：$LOGOUT_ERROR_MSG"
    elif [ -n "$LOGOUT_RESULT" ]; then
      echo "📋 错误信息：$LOGOUT_RESULT"
    fi
    echo "📋 请先重试退出登录；确认退出成功后再重新授权。"
    return 1
  fi

  echo ""
  echo "✅ 已退出登录状态"
  echo ""
  echo "📌 请重新执行登录授权流程："
  echo "   1. 我会生成授权链接"
  echo "   2. 您使用支付宝扫码授权"
  echo "   3. 授权成功后继续当前操作"
  echo ""
  echo "📌 产品、MCC 和授权范围未变化时，沿用已有确认立即重新生成授权链接；发生变化时先重新确认受影响内容。"
}

# ─── MCP 服务不可用处理 ────────────────────────────────────────────────────
handle_mcp_service_error() {
  local CLI_RESULT="$1"
  local UNWRAPPED=$(unwrap_mcp "$CLI_RESULT")

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "⚠️  MCP 服务暂时不可用"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  if [ "${DEV_TOOL_NAME:-unknown}" != "unknown" ]; then
    echo "❌ 当前命令需要访问支付宝服务，但当前 Agent 执行环境可能没有外网权限"
    echo ""
    echo "📋 处理方式："
    echo "  - Agent 应申请可联网权限后重试同一条命令"
    echo "  - 不要要求用户排查本机网络，也不要更换业务参数"
    echo ""
    echo "请授权 Agent 联网重试，或稍后重新执行当前命令。"
    return
  fi

  echo "❌ 无法连接到支付宝服务，请稍后重试"
  echo ""
  echo "📋 错误详情："
  echo "$UNWRAPPED" | jq -r '.error.message // .errorMessage // .message // "未知错误"' 2>/dev/null || echo "$UNWRAPPED"
  echo ""
  echo "您可以："
  echo "  1. 检查网络连接是否正常"
  echo "  2. 等待几分钟后重新执行"
  echo "  3. 联系技术支持获取帮助"
  echo ""
  echo "请回复「重试」重新尝试，或「退出」结束流程。"
}

# ─── 授权信息不匹配处理 ────────────────────────────────────────────────────
handle_auth_mismatch() {
  local CLI_RESULT="$1"
  local UNWRAPPED=$(unwrap_mcp "$CLI_RESULT")

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "❌ 授权信息不匹配，需要重新授权"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  if echo "$UNWRAPPED" | grep -qi "mccCode is not auth"; then
    echo "📋 原因：经营类目未授权"
  elif echo "$UNWRAPPED" | grep -qi "salesProductCodes is not auth"; then
    echo "📋 原因：产品未授权"
  elif echo "$UNWRAPPED" | grep -qi "scope is not auth"; then
    echo "📋 原因：授权 scope 不满足"
  fi

  echo ""
  echo ""
  echo "⛔ 已停止当前操作和后续调用"
  echo "📋 请调用 auth.sh mismatch；该命令会统一退出当前登录并使用正确范围重新授权"
}

# ─── 后端业务错误处理（完整透出错误信息） ──────────────────────────────────
handle_backend_error() {
  local CLI_RESULT="$1"
  local ERROR_CODE="$2"
  local UNWRAPPED=$(unwrap_mcp "$CLI_RESULT")

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "❌ 业务处理失败"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "📋 错误码：$ERROR_CODE"

  # 提取错误信息
  local ERROR_MSG=$(extract_error_message "$UNWRAPPED")
  local BIZ_TIPS=$(extract_biz_tips "$UNWRAPPED")
  local NEED_RETRY=$(extract_need_retry "$UNWRAPPED")
  local ERROR_SCENE=$(extract_error_object_field "$UNWRAPPED" "errorScene")
  local ERROR_SPECIFIC=$(extract_error_object_field "$UNWRAPPED" "errorSpecific")

  echo "📋 错误信息：$ERROR_MSG"

  if [ -n "$ERROR_SCENE" ] || [ -n "$ERROR_SPECIFIC" ]; then
    echo "📋 错误场景：${ERROR_SCENE:-未知} / ${ERROR_SPECIFIC:-未知}"
  fi

  # 透出 checkedError 中的详细错误描述（如 ar-sign.apply 返回的字段级校验错误）
  local CHECKED_ERRORS=$(extract_checked_errors "$UNWRAPPED")
  if [ -n "$CHECKED_ERRORS" ] && [ "$CHECKED_ERRORS" != "null" ] && [ "$CHECKED_ERRORS" != "[]" ]; then
    echo ""
    echo "📋 详细校验错误："
    echo "$CHECKED_ERRORS" | jq -r '.[]? | .errorDesc // .message // . // empty' 2>/dev/null | while IFS= read -r line; do
      [ -n "$line" ] && echo "  - $line"
    done
  fi

  if [ -n "$BIZ_TIPS" ] && [ "$BIZ_TIPS" != "null" ] && [ "$BIZ_TIPS" != "" ]; then
    echo ""
    echo "💡 提示：$BIZ_TIPS"
  fi

  if [ "$ERROR_CODE" = "APP_MAX_ERROR" ]; then
    echo ""
    echo "📌 处理建议：当前主体下应用数量已达到上限，无法继续新建应用。请优先复用已有上线应用；如必须新建，请前往支付宝开放平台处理应用配额或联系支付宝技术支持。"
  fi

  if [ "$NEED_RETRY" = "true" ]; then
    echo ""
    echo "🔄 此错误可重试，请回复「重试」重新执行"
  fi

  echo ""
  echo "如需帮助，请联系技术支持并提供以上错误信息。"
}

# ─── MCP 服务不稳定处理 ────────────────────────────────────────────────────
handle_service_unstable() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "⚠️  MCP 服务暂时不稳定"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "当前无法继续执行，请稍后重试。"
  echo ""
  echo "您可以："
  echo "  1. 等待几分钟后重新执行"
  echo "  2. 检查网络连接是否正常"
  echo "  3. 联系技术支持获取帮助"
  echo ""
  echo "请回复「重试」重新尝试，或「退出」结束流程。"
}

# ─── 统一错误处理入口 ──────────────────────────────────────────────────────
# 参数: $1 - CLI 执行结果文本（原始输出，含信封或不含）
# 返回: 0=成功, 1=失败（需要用户干预）
handle_error() {
  local CLI_RESULT="$1"
  local ERROR_TYPE=$(detect_error "$CLI_RESULT")
  # 解包一次供 CLI_ERROR 分支使用（其他分支由各自的 handler 自己解包）
  local _UNWRAPPED=$(unwrap_mcp "$CLI_RESULT")

  case "$ERROR_TYPE" in
    "MCP_AUTH_ERROR")
      handle_mcp_auth_error
      ;;
    "MCP_SERVICE_ERROR")
      handle_mcp_service_error "$CLI_RESULT"
      ;;
    "AUTH_MISMATCH")
      handle_auth_mismatch "$CLI_RESULT"
      ;;
    "SERVICE_UNSTABLE")
      handle_service_unstable
      ;;
    "ERROR:"*)
      local CODE="${ERROR_TYPE#ERROR:}"
      handle_backend_error "$CLI_RESULT" "$CODE"
      ;;
    "CLI_ERROR:"*)
      local MSG="${ERROR_TYPE#CLI_ERROR:}"
      echo "❌ 命令执行失败：$MSG"
      # CLI_ERROR 也透出 bizTips 和 checkedError（如果有的话）
      local CLI_BIZ_TIPS=$(extract_biz_tips "$_UNWRAPPED")
      if [ -n "$CLI_BIZ_TIPS" ] && [ "$CLI_BIZ_TIPS" != "null" ] && [ "$CLI_BIZ_TIPS" != "" ]; then
        echo "💡 $CLI_BIZ_TIPS"
      fi
      local CLI_CHECKED=$(extract_checked_errors "$_UNWRAPPED")
      if [ -n "$CLI_CHECKED" ] && [ "$CLI_CHECKED" != "null" ] && [ "$CLI_CHECKED" != "[]" ]; then
        echo "📋 详细校验错误："
        echo "$CLI_CHECKED" | jq -r '.[]? | .errorDesc // .message // . // empty' 2>/dev/null | while IFS= read -r line; do
          [ -n "$line" ] && echo "  - $line"
        done
      fi
      ;;
    "SUCCESS")
      return 0
      ;;
  esac

  return 1
}
