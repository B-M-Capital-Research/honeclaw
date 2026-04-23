#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

FIXTURE="tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json"

python3 - <<'PY' "$FIXTURE"
from pathlib import Path
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request

fixture_path = Path(sys.argv[1])
fixture = json.loads(fixture_path.read_text())
items = fixture["items"]
llm_items = [
    item
    for item in items
    if item.get("expected_llm_after_engine") in ("yes", "no")
]

if os.environ.get("RUN_EVENT_ENGINE_LLM_BASELINE") != "1":
    print("[PASS] event-engine news classifier baseline fixture loaded")
    print(f"fixture={fixture_path}")
    print(f"items={len(items)}")
    print(f"llm_items={len(llm_items)}")
    print(
        "[INFO] set RUN_EVENT_ENGINE_LLM_BASELINE=1 to rerun the saved "
        "items against a real OpenRouter model"
    )
    sys.exit(0)


def openrouter_key() -> str:
    if os.environ.get("OPENROUTER_API_KEY"):
        return os.environ["OPENROUTER_API_KEY"]
    cfg = Path("config.yaml")
    if not cfg.exists():
        raise SystemExit("[FAIL] OPENROUTER_API_KEY unset and config.yaml missing")
    text = cfg.read_text()
    match = re.search(
        r"(?ms)^llm:\s+.*?^\s+openrouter:\s+.*?^\s+api_key:\s*[\"']?([^\"'\n#]+)",
        text,
    )
    if not match:
        raise SystemExit("[FAIL] unable to read llm.openrouter.api_key from config.yaml")
    return match.group(1).strip()


model = (
    os.environ.get("EVENT_ENGINE_NEWS_CLASSIFIER_MODEL")
    or fixture.get("recommended_model")
    or fixture["model"]
)
limit = int(os.environ.get("EVENT_ENGINE_LLM_BASELINE_LIMIT", "0") or "0")
if limit > 0:
    llm_items = llm_items[:limit]

headers = {
    "Authorization": f"Bearer {openrouter_key()}",
    "Content-Type": "application/json",
    "HTTP-Referer": "https://openrouter.ai",
    "X-Title": "Hone event-engine baseline regression",
}
system_prompt = fixture["system_prompt"]
importance_prompt = fixture["importance_prompt"]


def classify(item: dict) -> tuple[str, str, float, float]:
    user = (
        "请按以下重要性标准判断这条新闻是否重要:\n"
        f"【重要性标准】{importance_prompt}\n\n"
        "【新闻】\n"
        f"- 标题: {item['title']}\n"
        f"- 涉及股票: {item['symbol']}\n"
        f"- 来源: fmp.stock_news:{item['site']}\n"
        "- 摘要: \n\n"
        "请只输出一个英文单词: 'yes' 表示重要, 'no' 表示不重要。"
        "不要输出其它任何字符。"
    )
    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user},
        ],
        "temperature": 0,
        "max_tokens": 8,
    }
    request = urllib.request.Request(
        "https://openrouter.ai/api/v1/chat/completions",
        data=json.dumps(payload).encode("utf-8"),
        headers=headers,
        method="POST",
    )
    start = time.time()
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            raw = response.read()
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")[:500]
        raise RuntimeError(f"HTTP {exc.code}: {body}") from exc
    elapsed = time.time() - start
    data = json.loads(raw)
    choice = (data.get("choices") or [{}])[0]
    message = choice.get("message") or {}
    content = message.get("content") or ""
    head = content.strip().split()[0].lower() if content.strip() else ""
    if head.startswith("yes"):
        answer = "yes"
    elif head.startswith("no"):
        answer = "no"
    else:
        answer = f"unparseable:{head or choice.get('finish_reason')}"
    cost = float((data.get("usage") or {}).get("cost") or 0.0)
    return answer, str(choice.get("finish_reason")), elapsed, cost


drifts = []
total_cost = 0.0
latencies = []
print(f"[INFO] model={model} items={len(llm_items)} fixture={fixture_path}")
for item in llm_items:
    answer, finish, elapsed, cost = classify(item)
    expected = item.get("expected_llm_title_only_after_engine") or item[
        "expected_llm_after_engine"
    ]
    total_cost += cost
    latencies.append(elapsed)
    status = "OK" if answer == expected else "DRIFT"
    print(
        f"[{status}] {item['id']} {item['symbol']} expected={expected} "
        f"actual={answer} finish={finish} elapsed={elapsed:.2f}s title={item['title']}"
    )
    if answer != expected:
        drifts.append((item, expected, answer))

avg_latency = sum(latencies) / len(latencies) if latencies else 0.0
print(f"[INFO] reported_cost={total_cost:.6f} avg_latency={avg_latency:.2f}s")

if drifts and os.environ.get("ALLOW_EVENT_ENGINE_LLM_BASELINE_DRIFT") != "1":
    print(f"[FAIL] {len(drifts)} baseline decisions drifted", file=sys.stderr)
    print(
        "[INFO] set ALLOW_EVENT_ENGINE_LLM_BASELINE_DRIFT=1 to collect a "
        "non-blocking drift report",
        file=sys.stderr,
    )
    sys.exit(1)

if drifts:
    print(f"[WARN] {len(drifts)} baseline decisions drifted")
else:
    print("[PASS] event-engine news classifier live baseline matched")
PY
