#!/usr/bin/env bash
# 把 origin 上的 HONE_MEDIA_EDGE_HMAC_SECRET 直接管道进 Worker 的 MEDIA_EDGE_HMAC_SECRET。
# 密钥只在管道里走，不落盘、不回显、不进任何人的上下文。
#
# 用法：CLOUDFLARE_API_TOKEN=<hone-media-edge-deploy 令牌> bash set-worker-secret.sh
set -euo pipefail
: "${CLOUDFLARE_API_TOKEN:?需要 CLOUDFLARE_API_TOKEN（runbook 里那个三权限的 hone-media-edge-deploy）}"
cd "$(dirname "$0")"
gcloud compute ssh instance-20260731-081043 --zone us-central1-c \
  --command "sudo grep '^HONE_MEDIA_EDGE_HMAC_SECRET=' /etc/hone/runtime.env | sed 's/^[^=]*=//'" \
| tr -d '\r\n' \
| npx wrangler secret put MEDIA_EDGE_HMAC_SECRET
echo "已写入 Worker。runbook 说 var 要 deploy 才生效，secret 是立即生效的；保险起见可再跑一次："
echo "  npx wrangler deploy"
