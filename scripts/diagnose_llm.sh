#!/usr/bin/env bash
# LLM 连接诊断脚本
# 逐步排查 OpenRouter 请求失败的根本原因

set -uo pipefail

CONFIG_PATH="${HONE_USER_CONFIG_PATH:-${HONE_CONFIG_PATH:-config.yaml}}"
if ! command -v python3 >/dev/null 2>&1; then
  echo "[FAIL] python3 is required to read Hone config" >&2
  exit 1
fi
if ! command -v curl >/dev/null 2>&1; then
  echo "[FAIL] curl is required to probe OpenRouter" >&2
  echo "Install curl or add it to PATH, then rerun: bash scripts/diagnose_llm.sh" >&2
  exit 1
fi
if [[ ! -f "$CONFIG_PATH" ]]; then
  echo "[FAIL] config file not found: $CONFIG_PATH" >&2
  exit 1
fi

API_KEY=$(python3 - "$CONFIG_PATH" <<'PY' 2>/dev/null
import sys
from pathlib import Path


def clean(value: str) -> str:
    value = value.strip().strip('"').strip("'")
    if not value or value == "[]":
        return ""
    return value


def first_value_at_path(text: str, target: tuple[str, ...]) -> str:
    stack: list[tuple[int, str]] = []
    for raw_line in text.splitlines():
        line = raw_line.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        indent = len(line) - len(line.lstrip(" "))
        content = line.strip()
        while stack and indent <= stack[-1][0]:
            stack.pop()
        if content.startswith("-"):
            if tuple(key for _, key in stack) == target:
                found = clean(content[1:])
                if found:
                    return found
            continue
        if ":" not in content:
            continue
        key, raw_value = content.split(":", 1)
        key = key.strip().strip('"').strip("'")
        path = tuple([*(key for _, key in stack), key])
        value = clean(raw_value)
        if path == target and value:
            return value
        if not raw_value.strip():
            stack.append((indent, key))
    return ""


config_path = Path(sys.argv[1]).expanduser()
text = config_path.read_text(encoding="utf-8")
for path in (
    ("llm", "providers", "openrouter", "api_key"),
    ("llm", "providers", "openrouter", "api_keys"),
    ("llm", "openrouter", "api_key"),
    ("llm", "openrouter", "api_keys"),
):
    value = first_value_at_path(text, path)
    if value:
        print(value)
        break
PY
)
MODEL="moonshotai/kimi-k2.5"
BASE_URL="https://openrouter.ai/api/v1"
TCP_TEST_LOG="$(mktemp "${TMPDIR:-/tmp}/hone_tcp_test.XXXXXX")"
trap 'rm -f "$TCP_TEST_LOG"' EXIT

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }
info() { echo -e "${YELLOW}[INFO]${NC} $1"; }
sep()  { echo "------------------------------------------------------------"; }

echo "============================================================"
echo "  Hone LLM 连接诊断"
echo "  目标: $BASE_URL"
echo "  模型: $MODEL"
echo "============================================================"
echo

if [ -z "$API_KEY" ]; then
  fail "未在 ${CONFIG_PATH} 中配置 llm.providers.openrouter.api_key/api_keys（legacy llm.openrouter.* 仍可读取）"
  exit 1
fi

# ── 1. DNS 解析 ──────────────────────────────────────────────
sep
info "步骤 1/5: DNS 解析 openrouter.ai"
if dns_result=$(dscacheutil -q host -a name openrouter.ai 2>/dev/null | grep "ip_address" | head -1); then
  if [ -n "$dns_result" ]; then
    pass "DNS 解析成功: $dns_result"
  else
    fail "DNS 解析返回空结果，可能存在 DNS 故障"
  fi
else
  # fallback: ping -c1 仅测试域名解析
  if ping -c1 -W2 openrouter.ai &>/dev/null; then
    pass "DNS 解析成功（ping 验证）"
  else
    fail "DNS 解析失败，无法解析 openrouter.ai"
  fi
fi

# ── 2. TCP 连通性（443 端口）────────────────────────────────
sep
info "步骤 2/5: TCP 连通性测试（openrouter.ai:443）"
if curl -s --connect-timeout 10 --max-time 10 \
    -o /dev/null -w "%{http_code}" \
    "https://openrouter.ai" > "$TCP_TEST_LOG" 2>&1; then
  pass "TCP 443 端口可达，HTTPS 握手成功"
else
  fail "无法连接 openrouter.ai:443，请检查网络/防火墙/代理"
  echo "      详细错误: $(cat "$TCP_TEST_LOG" 2>/dev/null)"
fi

# ── 3. 代理环境变量检测 ────────────────────────────────────
sep
info "步骤 3/5: 代理环境变量检测"
proxy_found=false
for var in http_proxy https_proxy HTTP_PROXY HTTPS_PROXY all_proxy ALL_PROXY; do
  proxy_value="${!var:-}"
  if [ -n "$proxy_value" ]; then
    info "  $var = $proxy_value"
    proxy_found=true
  fi
done
if $proxy_found; then
  info "检测到代理配置（如果代理异常可能导致 http error）"
else
  info "未检测到代理环境变量（如需代理，请设置 HTTPS_PROXY）"
fi

# ── 4. API Key 有效性（/auth/key 接口）────────────────────
sep
info "步骤 4/5: 验证 API Key 有效性"
auth_resp=$(curl -s --connect-timeout 15 --max-time 20 \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  "https://openrouter.ai/api/v1/auth/key" 2>&1)
auth_exit=$?

if [ $auth_exit -ne 0 ]; then
  fail "curl 请求失败 (exit=$auth_exit)，网络层问题"
  echo "      错误详情: $auth_resp"
else
  echo "      响应: $auth_resp"
  if echo "$auth_resp" | grep -q '"error"'; then
    fail "API Key 无效或已过期"
    echo "      建议: 登录 https://openrouter.ai/keys 确认 Key 状态"
  elif echo "$auth_resp" | grep -q '"data"'; then
    pass "API Key 有效"
    # 显示余额
    balance=$(echo "$auth_resp" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('limit_remaining','N/A'))" 2>/dev/null || echo "N/A")
    info "  剩余额度: $balance"
  else
    info "响应格式未知，原始内容: $auth_resp"
  fi
fi

# ── 5. 实际对话请求（最小化 payload）──────────────────────
sep
info "步骤 5/5: 发送最小化 chat 请求（模型: $MODEL）"
chat_resp=$(curl -s --connect-timeout 20 --max-time 60 \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -H "HTTP-Referer: https://openrouter.ai" \
  -H "X-Title: Hone-Financial" \
  -d "{
    \"model\": \"$MODEL\",
    \"messages\": [{\"role\": \"user\", \"content\": \"hi\"}],
    \"max_tokens\": 10
  }" \
  "$BASE_URL/chat/completions" 2>&1)
chat_exit=$?

if [ $chat_exit -ne 0 ]; then
  fail "curl 请求失败 (exit=$chat_exit)"
  echo "      错误详情: $chat_resp"
  echo
  info "常见原因:"
  echo "      - 网络不可达（需要科学上网）"
  echo "      - 系统代理未正确配置（Rust reqwest 不读 macOS 系统代理）"
  echo "      - 防火墙拦截了出站 HTTPS 请求"
else
  echo "      HTTP 响应: $chat_resp" | head -c 500
  echo
  if echo "$chat_resp" | grep -q '"choices"'; then
    pass "chat/completions 请求成功！模型可用"
    reply=$(echo "$chat_resp" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print(d['choices'][0]['message']['content'])
" 2>/dev/null || echo "(解析失败)")
    info "  模型回复: $reply"
  elif echo "$chat_resp" | grep -q '"error"'; then
    fail "API 返回错误"
    err_msg=$(echo "$chat_resp" | python3 -c "
import sys,json
d=json.load(sys.stdin)
e=d.get('error',{})
print(e.get('message', str(e)))
" 2>/dev/null || echo "$chat_resp")
    echo "      错误: $err_msg"
    if echo "$err_msg" | grep -qi "model\|not found\|unavailable"; then
      info "  建议: 模型 '$MODEL' 可能暂时不可用，尝试换用 'openai/gpt-4o-mini' 测试"
    fi
    if echo "$err_msg" | grep -qi "credit\|balance\|limit\|quota"; then
      info "  建议: 余额不足，请充值 https://openrouter.ai/credits"
    fi
    if echo "$err_msg" | grep -qi "auth\|key\|unauthorized"; then
      info "  建议: API Key 认证失败，请重新生成"
    fi
  else
    info "响应格式未知: $chat_resp"
  fi
fi

# ── 备用模型测试（如主模型失败）───────────────────────────
if ! echo "$chat_resp" | grep -q '"choices"' 2>/dev/null; then
  sep
  info "补充测试: 用 openai/gpt-4o-mini 验证 Key 和网络是否正常"
  fallback_resp=$(curl -s --connect-timeout 20 --max-time 60 \
    -H "Authorization: Bearer $API_KEY" \
    -H "Content-Type: application/json" \
    -d "{
      \"model\": \"openai/gpt-4o-mini\",
      \"messages\": [{\"role\": \"user\", \"content\": \"hi\"}],
      \"max_tokens\": 10
    }" \
    "$BASE_URL/chat/completions" 2>&1)
  if echo "$fallback_resp" | grep -q '"choices"'; then
    pass "备用模型 gpt-4o-mini 请求成功 → 问题出在原模型 '$MODEL' 本身（不可用/无权限）"
    info "  建议: 在 config.yaml 中将 model 改为 'openai/gpt-4o-mini' 或其他可用模型"
  else
    fail "备用模型也失败 → 问题不在模型，而在网络/Key 层面"
    echo "      响应: $fallback_resp" | head -c 300
    echo
  fi
fi

sep
echo "诊断完成"
