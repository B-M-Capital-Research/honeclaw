#!/bin/bash
#=============================================================================
# 脚本名称: upload_screenshots.sh
# 功能描述: 并行上传截图文件并解析可用于签约的图片引用值
# 调用位置: Step 4 一次性资料与资源决策
# 调用前置: 脚本通过 error_handler.sh 间接初始化 DEV_TOOL_NAME；需传入3个文件路径
# 用法: bash upload_screenshots.sh "$HOME_IMG" "$SHOP_IMG" "$PAY_IMG"
# 返回值: 输出上传结果和图片引用值
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

HOME_IMG="$1"
SHOP_IMG="$2"
PAY_IMG="$3"

# 校验参数
if [ -z "$HOME_IMG" ] || [ -z "$SHOP_IMG" ] || [ -z "$PAY_IMG" ]; then
  echo "❌ 请提供 3 个截图文件路径"
  echo "用法: bash upload_screenshots.sh <首页截图> <商品页截图> <支付页截图>"
  echo "  网站支付：传入网站首页、商品页、支付页截图"
  echo "  APP支付：传入APP首页、商品页、支付页截图"
  exit 1
fi

# 校验文件存在性
if [ ! -f "$HOME_IMG" ]; then
  echo "❌ 首页截图文件不存在: $HOME_IMG"
  exit 1
fi
if [ ! -f "$SHOP_IMG" ]; then
  echo "❌ 商品页截图文件不存在: $SHOP_IMG"
  exit 1
fi
if [ ! -f "$PAY_IMG" ]; then
  echo "❌ 支付页截图文件不存在: $PAY_IMG"
  exit 1
fi

# 使用 mktemp 创建并发安全的临时文件。模板需以 XXXXXX 结尾以兼容 macOS/BSD mktemp。
TMP_BASE="${TMPDIR:-/tmp}"
TMP_BASE="${TMP_BASE%/}"
TMP_HOME=$(mktemp "${TMP_BASE}/upload_home.XXXXXX") || { echo "❌ 创建临时文件失败"; exit 1; }
TMP_SHOP=$(mktemp "${TMP_BASE}/upload_shop.XXXXXX") || { echo "❌ 创建临时文件失败"; exit 1; }
TMP_PAY=$(mktemp "${TMP_BASE}/upload_pay.XXXXXX") || { echo "❌ 创建临时文件失败"; exit 1; }

# 清理临时文件
cleanup_upload_tmp() {
  rm -f "$TMP_HOME" "$TMP_SHOP" "$TMP_PAY"
}
trap cleanup_upload_tmp EXIT

# 并行上传 3 张截图
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli file upload "$HOME_IMG" -s payMerchantcodeSkill --json 2>/dev/null > "$TMP_HOME" &
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli file upload "$SHOP_IMG" -s payMerchantcodeSkill --json 2>/dev/null > "$TMP_SHOP" &
export PLATFORM=${DEV_TOOL_NAME} && alipay-cli file upload "$PAY_IMG" -s payMerchantcodeSkill --json 2>/dev/null > "$TMP_PAY" &
wait

# 解析图片引用值（兼容 file upload 返回 fileKey 或 URL 字段）
parse_image_ref() {
  local RAW=$(cat "$1")
  if ! handle_error "$RAW" >/dev/null; then
    return 1
  fi
  # alipay-cli file upload 可能返回 MCP 信封，先解包
  local UNWRAPPED=$(unwrap_mcp "$RAW")
  echo "$UNWRAPPED" | jq -r '
    .data.fileKey //
    .data.picUrl //
    .data.url //
    .data.fileUrl //
    .data.downloadUrl //
    .data.data.fileKey //
    .data.data.picUrl //
    .data.data.url //
    .data.data.fileUrl //
    .data.data.downloadUrl //
    .fileKey //
    .picUrl //
    .url //
    .fileUrl //
    .downloadUrl //
    .result.fileKey //
    .result.picUrl //
    .result.url //
    .result.fileUrl //
    .result.downloadUrl //
    empty
  ' 2>/dev/null
}

HOME_REF=$(parse_image_ref "$TMP_HOME")
SHOP_REF=$(parse_image_ref "$TMP_SHOP")
PAY_REF=$(parse_image_ref "$TMP_PAY")

# 校验上传结果（含认证错误检测）
check_upload_error() {
  local RAW=$(cat "$1")
  if ! handle_error "$RAW"; then
    return 1
  fi
  return 0
}

ERRORS=""
if [ -z "$HOME_REF" ]; then
  if ! check_upload_error "$TMP_HOME"; then
    ERRORS="$ERRORS  - 首页截图上传失败（认证或服务异常）"$'\n'
  else
    ERRORS="$ERRORS  - 首页截图上传后未返回可用于签约的图片引用值"$'\n'
  fi
fi
if [ -z "$SHOP_REF" ]; then
  if ! check_upload_error "$TMP_SHOP"; then
    ERRORS="$ERRORS  - 商品页截图上传失败（认证或服务异常）"$'\n'
  else
    ERRORS="$ERRORS  - 商品页截图上传后未返回可用于签约的图片引用值"$'\n'
  fi
fi
if [ -z "$PAY_REF" ]; then
  if ! check_upload_error "$TMP_PAY"; then
    ERRORS="$ERRORS  - 支付页截图上传失败（认证或服务异常）"$'\n'
  else
    ERRORS="$ERRORS  - 支付页截图上传后未返回可用于签约的图片引用值"$'\n'
  fi
fi

if [ -n "$ERRORS" ]; then
  echo "❌ 截图上传失败："
  printf "%s" "$ERRORS"
  exit 1
fi

echo "✅ 截图上传成功"
echo "  - 首页截图: $HOME_REF"
echo "  - 商品页截图: $SHOP_REF"
echo "  - 支付页截图: $PAY_REF"
echo ""
echo "# 更新对话上下文状态："
echo "# collect_information = {\"screenshot\":[\"$HOME_REF\",\"$SHOP_REF\",\"$PAY_REF\"]}"
