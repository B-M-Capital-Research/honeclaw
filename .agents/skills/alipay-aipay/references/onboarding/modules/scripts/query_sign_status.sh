#!/bin/bash
#=============================================================================
# 脚本名称: query_sign_status.sh
# 功能描述: 查询签约状态并根据结果判断后续流程
# 调用位置: Step 3.1 状态与资源查询
# 调用前置: 脚本通过 error_handler.sh 间接初始化 DEV_TOOL_NAME；需传入 --sales-code，或兼容使用 SALES_CODE 环境变量
# 用法: bash query_sign_status.sh --sales-code <code> [--product-type aipay|webpay|apppay]
# 返回值: 输出签约状态判断结果和处理建议
#           SIGN_STATUS=NOT_SIGNED | SIGN_STATUS=SIGNED_EFFECTIVE | SIGN_STATUS=SIGN_SUBMITTED | SIGN_STATUS=OTHER_STATUS
#           按量付费:   FLOW:AI_PAY_NOT_SIGNED | FLOW:AI_PAY_SIGNED
#           网站支付:   FLOW:PC_WEB_NOT_SIGNED  | FLOW:PC_WEB_SIGNED
#           APP支付:    FLOW:APP_NOT_SIGNED     | FLOW:APP_SIGNED
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

export PRODUCT_TYPE="${PRODUCT_TYPE:-unknown}"

while [[ $# -gt 0 ]]; do
  case $1 in
    --sales-code)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      SALES_CODE="$2"; shift 2 ;;
    --product-type)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      PRODUCT_TYPE="$2"; shift 2 ;;
    *)
      echo "❌ 未知参数: $1"
      echo "用法: bash query_sign_status.sh --sales-code <code> [--product-type aipay|webpay|apppay]"
      exit 1 ;;
  esac
done

if [ -z "$SALES_CODE" ]; then
  echo "❌ 缺少 salesCode（请传入 --sales-code，或设置 SALES_CODE 环境变量）"
  exit 1
fi

case "$SALES_CODE" in
  "I1080300001000160457")
    EXPECTED_PRODUCT_TYPE="aipay"
    ;;
  "I1080300001000041203")
    EXPECTED_PRODUCT_TYPE="webpay"
    ;;
  "I1080300001000041313")
    EXPECTED_PRODUCT_TYPE="apppay"
    ;;
  *)
    echo "❌ 未知 SALES_CODE: ${SALES_CODE}"
    exit 1
    ;;
esac

if [ "$PRODUCT_TYPE" != "unknown" ] && [ "$PRODUCT_TYPE" != "$EXPECTED_PRODUCT_TYPE" ]; then
  echo "❌ PRODUCT_TYPE 与 SALES_CODE 不匹配：PRODUCT_TYPE=${PRODUCT_TYPE}, SALES_CODE=${SALES_CODE}"
  echo "📋 期望 PRODUCT_TYPE=${EXPECTED_PRODUCT_TYPE}"
  exit 1
fi

emit_signed_flow() {
  case "$SALES_CODE" in
    "I1080300001000041203")
      echo "FLOW:PC_WEB_SIGNED"
      # 网站支付已生效或已提交：Step 4 只处理应用决策，Step 5 跳过签约提交
      ;;
    "I1080300001000041313")
      echo "FLOW:APP_SIGNED"
      # APP支付已生效或已提交：Step 4 只处理应用决策，Step 5 跳过签约提交
      ;;
    "I1080300001000160457")
      echo "FLOW:AI_PAY_SIGNED"
      # 按量付费已生效或已提交：Step 4 处理服务/应用决策，Step 5 跳过签约提交
      ;;
  esac
}

# 查询签约状态
RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call ar-query.queryArInfosBySalesProd \
  -d "{\"request\":{\"salesProductCodes\":[\"${SALES_CODE}\"]},\"ctx\":{}}" \
  --json 2>/dev/null)

# 错误检测
if ! handle_error "$RESULT"; then
  exit 1
fi

# 解包 MCP 信封后解析业务字段
BUSINESS=$(unwrap_mcp "$RESULT")

# 只有明确返回数组才可判断签约状态；缺少字段必须阻断，避免误判为未签约
AR_LIST=$(echo "$BUSINESS" | jq -c '
  if (.resultObj | type) == "object" and (.resultObj.arInfoList | type) == "array" then
    .resultObj.arInfoList
  else
    null
  end
' 2>/dev/null)
if [ -z "$AR_LIST" ] || ! echo "$AR_LIST" | jq -e 'type == "array"' >/dev/null 2>&1; then
  echo "❌ 签约状态返回结构异常，无法解析 arInfoList"
  exit 1
fi
AR_COUNT=$(echo "$AR_LIST" | jq 'length')

if [ "$AR_COUNT" -eq 0 ]; then
  echo "📋 签约状态: 未签约 (NOT_SIGNED)；继续 Step 4 一次性资料与资源决策"
  echo "SIGN_STATUS=NOT_SIGNED"

  case "$SALES_CODE" in
    "I1080300001000041203")
      echo "FLOW:PC_WEB_NOT_SIGNED"
      # 网站支付未签约：需先采集 3 张网站截图，再进入 Step 5 签约
      ;;
    "I1080300001000041313")
      echo "FLOW:APP_NOT_SIGNED"
      # APP支付未签约：需先采集 APP 名称和 3 张APP界面截图，再进入 Step 5 签约
      ;;
    "I1080300001000160457")
      echo "FLOW:AI_PAY_NOT_SIGNED"
      # 按量付费未签约：Step 4 无需页面图片，但仍完成服务/应用决策
      ;;
  esac
else
  # AR_LIST 已在上方确认为数组
  HAS_EFFECTIVE=$(echo "$AR_LIST" | jq -r '[.[] | select(.arStatus == "02")] | length')
  HAS_SUBMITTED=$(echo "$AR_LIST" | jq -r '[.[] | select(.arStatus == "01")] | length')

  if [ "$HAS_EFFECTIVE" -gt 0 ]; then
    echo "✅ 签约已生效；无需签约材料，继续 Step 4 资源决策"
    echo "SIGN_STATUS=SIGNED_EFFECTIVE"
    emit_signed_flow
  elif [ "$HAS_SUBMITTED" -gt 0 ]; then
    echo "📋 已提交签约（待生效），无需重复提交签约，继续推进后续步骤"
    echo "SIGN_STATUS=SIGN_SUBMITTED"
    emit_signed_flow
  else
    echo "📋 其他签约状态；保留已取得的只读结果并将签约分支标记为待核验"
    echo "SIGN_STATUS=OTHER_STATUS"
    echo "FLOW:OTHER_STATUS"
  fi
fi
