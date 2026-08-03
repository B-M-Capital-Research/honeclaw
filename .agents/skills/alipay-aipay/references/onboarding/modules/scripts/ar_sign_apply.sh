#!/bin/bash
#=============================================================================
# 脚本名称: ar_sign_apply.sh
# 功能描述: 提交产品签约申请，支持按量付费、网站支付、APP支付三种模式
# 调用位置: Step 5.1 产品签约
# 调用前置: 脚本通过 error_handler.sh 间接初始化 DEV_TOOL_NAME；需传入 --sales-code 和 --mcc-code，或兼容使用同名环境变量
# 用法:
#   按量付费: bash ar_sign_apply.sh --product aipay --sales-code <code> --mcc-code <code>
#   网站支付: bash ar_sign_apply.sh --product webpay --sales-code <code> --mcc-code <code> --picurl1 <imageRef1> --picurl2 <imageRef2> --picurl3 <imageRef3>
#   APP支付: bash ar_sign_apply.sh --product apppay --sales-code <code> --mcc-code <code> --app-name <name> --picurl1 <imageRef1> --picurl2 <imageRef2> --picurl3 <imageRef3>
# 返回值:
#   成功: FLOW:AI_PAY_SIGN_CONTINUE | FLOW:PC_WEB_SIGN_CONTINUE | FLOW:APP_SIGN_CONTINUE
#   失败: exit 1（由 handle_error 输出详细错误信息）
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
if ! command -v python3 >/dev/null 2>&1 && ! command -v uuidgen >/dev/null 2>&1; then
  echo "❌ 缺少依赖命令: python3 或 uuidgen"
  exit 1
fi

# 生成 UUID 作为 bizRequestNo
BIZ_REQUEST_NO=$(python3 -c "import uuid; print(uuid.uuid4())" 2>/dev/null || uuidgen | tr '[:upper:]' '[:lower:]')

# 优先使用 --product 参数，未传时从环境变量继承
PRODUCT_TYPE="${PRODUCT_TYPE:-}"
PICURL1=""
PICURL2=""
PICURL3=""
APP_NAME=""

# 解析参数
while [[ $# -gt 0 ]]; do
  case $1 in
    --product)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      PRODUCT_TYPE="$2"; shift 2 ;;
    --sales-code)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      SALES_CODE="$2"; shift 2 ;;
    --mcc-code)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      MCC_CODE="$2"; shift 2 ;;
    --picurl1)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      PICURL1="$2"; shift 2 ;;
    --picurl2)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      PICURL2="$2"; shift 2 ;;
    --picurl3)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      PICURL3="$2"; shift 2 ;;
    --app-name)
      [ $# -ge 2 ] || { echo "❌ 缺少参数值: $1"; exit 1; }
      APP_NAME="$2"; shift 2 ;;
    *)
      echo "❌ 未知参数: $1"; exit 1 ;;
  esac
done

if [ -z "$SALES_CODE" ] || [ -z "$MCC_CODE" ]; then
  echo "❌ 缺少必需参数（请传入 --sales-code 和 --mcc-code，或设置 SALES_CODE/MCC_CODE 环境变量）"
  exit 1
fi

if [ -n "$PRODUCT_TYPE" ] && [ "$PRODUCT_TYPE" != "aipay" ] && [ "$PRODUCT_TYPE" != "webpay" ] && [ "$PRODUCT_TYPE" != "apppay" ]; then
  echo "❌ --product 参数值无效: ${PRODUCT_TYPE}（仅支持 aipay|webpay|apppay）"
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
    echo "❌ 未知 SALES_CODE: $SALES_CODE"
    exit 1
    ;;
esac

if [ -n "$PRODUCT_TYPE" ] && [ "$PRODUCT_TYPE" != "$EXPECTED_PRODUCT_TYPE" ]; then
  echo "❌ --product 与 SALES_CODE 不匹配：--product=${PRODUCT_TYPE}, SALES_CODE=${SALES_CODE}"
  echo "📋 期望 --product=${EXPECTED_PRODUCT_TYPE}"
  exit 1
fi

if [ -z "$BIZ_REQUEST_NO" ]; then
  echo "❌ 无法生成 bizRequestNo"
  exit 1
fi

# 场景判断（精确比较而非子串匹配）
if [ "$SALES_CODE" = "I1080300001000160457" ]; then
  # ============ 按量付费签约 JSON（无截图） ============
  REQUEST_JSON=$(jq -n \
    --arg bizRequestNo "$BIZ_REQUEST_NO" \
    --arg mccCode "$MCC_CODE" \
    --arg salesCode "$SALES_CODE" \
    '{
      request: {
        bizFeatures: {},
        bizRequestNo: $bizRequestNo,
        businessProperty: {
          mccCode: $mccCode
        },
        channelCode: "B_SK_SH_RPC",
        extension: {},
        orderType: "NEW_SIGN",
        salesProductCodes: [$salesCode]
      },
      ctx: {}
    }')

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call ar-sign.apply \
    -d "$REQUEST_JSON" --json 2>/dev/null)

elif [ "$SALES_CODE" = "I1080300001000041203" ]; then
  # ============ 网站支付签约 JSON（有截图） ============
  if [ -z "$PICURL1" ] || [ -z "$PICURL2" ] || [ -z "$PICURL3" ]; then
    echo "❌ 网站支付需要 3 个上传后的图片引用值"
    exit 1
  fi

  REQUEST_JSON=$(jq -n \
    --arg bizRequestNo "$BIZ_REQUEST_NO" \
    --arg mccCode "$MCC_CODE" \
    --arg salesCode "$SALES_CODE" \
    --arg picUrl1 "$PICURL1" \
    --arg picUrl2 "$PICURL2" \
    --arg picUrl3 "$PICURL3" \
    '{
      request: {
        bizFeatures: {},
        bizRequestNo: $bizRequestNo,
        businessProperty: {
          mccCode: $mccCode,
          webAppDTO: {
            placeType: "ONLINE_WEBAPP",
            appType: "PC_WEB",
            appStatus: "OFFLINE",
            screenshot: [$picUrl1, $picUrl2, $picUrl3]
          }
        },
        channelCode: "B_SK_SH_RPC",
        extension: {},
        orderType: "NEW_SIGN",
        salesProductCodes: [$salesCode]
      },
      ctx: {}
    }')

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call ar-sign.apply \
    -d "$REQUEST_JSON" --json 2>/dev/null)

elif [ "$SALES_CODE" = "I1080300001000041313" ]; then
  # ============ APP支付签约 JSON（有 APP 名称和截图） ============
  if [ -z "$PICURL1" ] || [ -z "$PICURL2" ] || [ -z "$PICURL3" ]; then
    echo "❌ APP支付需要 3 个上传后的图片引用值"
    exit 1
  fi
  if [ -z "$APP_NAME" ]; then
    echo "❌ APP支付需要 --app-name"
    exit 1
  fi

  REQUEST_JSON=$(jq -n \
    --arg bizRequestNo "$BIZ_REQUEST_NO" \
    --arg mccCode "$MCC_CODE" \
    --arg salesCode "$SALES_CODE" \
    --arg appName "$APP_NAME" \
    --arg picUrl1 "$PICURL1" \
    --arg picUrl2 "$PICURL2" \
    --arg picUrl3 "$PICURL3" \
    '{
      request: {
        bizFeatures: {},
        bizRequestNo: $bizRequestNo,
        businessProperty: {
          mccCode: $mccCode,
          nativeAppDTO: {
            placeType: "ONLINE_NATIVEYAPP",
            name: $appName,
            appStatus: "OFFLINE",
            screenshot: [$picUrl1, $picUrl2, $picUrl3]
          }
        },
        channelCode: "B_SK_SH_RPC",
        extension: {},
        orderType: "NEW_SIGN",
        salesProductCodes: [$salesCode]
      },
      ctx: {}
    }')

  RESULT=$(export PLATFORM=${DEV_TOOL_NAME} && alipay-cli mcp call ar-sign.apply \
    -d "$REQUEST_JSON" --json 2>/dev/null)

else
  echo "❌ 未知 SALES_CODE: $SALES_CODE"
  exit 1
fi

# 错误检测（先于业务解析，handle_error 内部会自动解包信封）
if ! handle_error "$RESULT"; then
  exit 1
fi

# 解包 MCP 信封后解析业务字段
BUSINESS=$(unwrap_mcp "$RESULT")
SUCCESS=$(echo "$BUSINESS" | jq -r '.success // false')

if [ "$SUCCESS" = "true" ]; then
  echo "✅ 签约提交成功"
  echo "$BUSINESS" | jq '.resultObj // .data // {}'
  # 签约提交成功后无需等待审核，立即推进后续步骤
  # 按量付费: 继续 5.2 服务注册 → 5.3 应用发布
  # 网站支付/APP支付: 继续 5.3 应用发布
  if [ "$SALES_CODE" = "I1080300001000160457" ]; then
    echo "FLOW:AI_PAY_SIGN_CONTINUE"
  elif [ "$SALES_CODE" = "I1080300001000041313" ]; then
    echo "FLOW:APP_SIGN_CONTINUE"
  else
    echo "FLOW:PC_WEB_SIGN_CONTINUE"
  fi
else
  # handle_error 已处理 ERROR/CLI_ERROR 路径并透出详细错误信息
  # 此处为防御性兜底：handle_error 返回成功但 success=false 的极端场景
  ERROR_MSG=$(echo "$BUSINESS" | jq -r '.error.message // .errorMessage // "未知错误"')
  echo "❌ 签约提交失败: $ERROR_MSG"
  exit 1
fi
