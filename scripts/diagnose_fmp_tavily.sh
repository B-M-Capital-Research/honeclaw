#!/usr/bin/env bash
# Diagnose FMP and Tavily API key availability / quota status.

set -euo pipefail

python3 - "$@" <<'PY'
import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Optional

import yaml


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Diagnose FMP and Tavily API keys, including quota exhaustion."
    )
    parser.add_argument(
        "--config",
        help="Base config path. Defaults to HONE_CONFIG_PATH, then data/runtime/config_runtime.yaml, then config.yaml.",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=20.0,
        help="HTTP timeout in seconds for each probe. Default: 20",
    )
    parser.add_argument(
        "--fmp-symbol",
        default="AAPL",
        help="Ticker used for the FMP probe. Default: AAPL",
    )
    parser.add_argument(
        "--tavily-query",
        default="Apple stock latest news",
        help="Query used for the Tavily probe. Default: Apple stock latest news",
    )
    parser.add_argument(
        "--show-body",
        action="store_true",
        help="Print truncated response bodies for non-healthy results.",
    )
    return parser.parse_args()


def choose_config_path(cli_path: Optional[str]) -> Path:
    candidates = []
    if cli_path:
        candidates.append(Path(cli_path))
    env_path = os.environ.get("HONE_CONFIG_PATH")
    if env_path:
        candidates.append(Path(env_path))
    candidates.extend(
        [
            Path("data/runtime/config_runtime.yaml"),
            Path("config.yaml"),
        ]
    )

    seen: set[Path] = set()
    for candidate in candidates:
        resolved = candidate.expanduser()
        if resolved in seen:
            continue
        seen.add(resolved)
        if resolved.exists():
            return resolved
    raise FileNotFoundError(
        "No config file found. Tried HONE_CONFIG_PATH, data/runtime/config_runtime.yaml, and config.yaml."
    )


def overlay_path(config_path: Path) -> Path:
    if config_path.suffix:
        return config_path.with_name(
            f"{config_path.stem}.overrides{config_path.suffix}"
        )
    return config_path.with_name(f"{config_path.name}.overrides")


def read_yaml(path: Path):
    content = path.read_text(encoding="utf-8")
    if not content.strip():
        return {}
    data = yaml.safe_load(content)
    return data or {}


def merge_yaml(base, overlay):
    if isinstance(base, dict) and isinstance(overlay, dict):
        merged = dict(base)
        for key, value in overlay.items():
            if key in merged:
                merged[key] = merge_yaml(merged[key], value)
            else:
                merged[key] = value
        return merged
    return overlay


def load_effective_config(config_path: Path):
    base = read_yaml(config_path)
    overlay = overlay_path(config_path)
    if overlay.exists():
        return merge_yaml(base, read_yaml(overlay))
    return base


def normalize_keys(values):
    seen = set()
    keys = []
    for value in values:
        if not isinstance(value, str):
            continue
        key = value.strip()
        if not key or key in seen:
            continue
        seen.add(key)
        keys.append(key)
    return keys


def mask_key(key: str) -> str:
    if len(key) <= 8:
        return "*" * len(key)
    return f"{key[:4]}...{key[-4:]}"


def truncate_text(text: str, limit: int = 240) -> str:
    compact = " ".join(text.strip().split())
    if len(compact) <= limit:
        return compact
    return compact[: limit - 3] + "..."


def extract_json_text(payload) -> str:
    if isinstance(payload, str):
        return payload
    if isinstance(payload, list):
        return " ".join(extract_json_text(item) for item in payload)
    if isinstance(payload, dict):
        parts = []
        for value in payload.values():
            parts.append(extract_json_text(value))
        return " ".join(part for part in parts if part)
    return str(payload)


def http_json_request(method: str, url: str, timeout: float, body=None, headers=None):
    request_headers = dict(headers or {})
    request_headers.setdefault("User-Agent", "Hone-Financial diagnose_fmp_tavily.sh")
    data = None
    if body is not None:
        data = json.dumps(body).encode("utf-8")
        request_headers.setdefault("Content-Type", "application/json")

    req = urllib.request.Request(url, data=data, method=method, headers=request_headers)
    started = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read().decode("utf-8", errors="replace")
            status = resp.getcode()
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        status = exc.code
    latency_ms = int((time.perf_counter() - started) * 1000)

    try:
        payload = json.loads(raw) if raw.strip() else None
    except json.JSONDecodeError:
        payload = raw
    return status, payload, latency_ms


def probe_fmp(base_url: str, key: str, symbol: str, timeout: float):
    endpoint = f"{base_url.rstrip('/')}/v3/quote/{urllib.parse.quote(symbol)}"
    connector = "&" if "?" in endpoint else "?"
    url = f"{endpoint}{connector}apikey={urllib.parse.quote(key)}"

    try:
        status, payload, latency_ms = http_json_request("GET", url, timeout)
    except Exception as exc:  # noqa: BLE001
        return {
            "provider": "fmp",
            "status": "network_error",
            "message": f"request failed: {exc}",
            "latency_ms": None,
            "http_status": None,
            "body": None,
        }

    text = extract_json_text(payload).lower()
    message = ""
    result_status = "healthy"

    if status in (401, 403):
        result_status = "invalid"
        message = f"HTTP {status}"
    elif status == 429:
        result_status = "quota_exhausted"
        message = f"HTTP {status}"
    elif "error message" in text or "invalid api key" in text or "api key" in text:
        result_status = "invalid"
        message = extract_json_text(payload)
    if any(token in text for token in ("limit reach", "limit reached", "upgrade", "quota", "too many requests")):
        result_status = "quota_exhausted"
        message = extract_json_text(payload)
    elif status >= 400 and result_status == "healthy":
        result_status = "error"
        message = f"HTTP {status}"

    if result_status == "healthy":
        if isinstance(payload, list) and payload:
            message = f"ok ({len(payload)} records)"
        elif isinstance(payload, dict):
            message = "ok"
        else:
            result_status = "error"
            message = "unexpected response shape"

    return {
        "provider": "fmp",
        "status": result_status,
        "message": message,
        "latency_ms": latency_ms,
        "http_status": status,
        "body": payload,
    }


def probe_tavily(key: str, query: str, timeout: float):
    body = {
        "api_key": key,
        "query": query,
        "search_depth": "basic",
        "max_results": 1,
        "include_answer": False,
        "include_raw_content": False,
    }

    try:
        status, payload, latency_ms = http_json_request(
            "POST",
            "https://api.tavily.com/search",
            timeout,
            body=body,
        )
    except Exception as exc:  # noqa: BLE001
        return {
            "provider": "tavily",
            "status": "network_error",
            "message": f"request failed: {exc}",
            "latency_ms": None,
            "http_status": None,
            "body": None,
        }

    text = extract_json_text(payload).lower()
    message = ""
    result_status = "healthy"

    if status in (401, 403):
        result_status = "invalid"
        message = f"HTTP {status}"
    elif status == 429:
        result_status = "quota_exhausted"
        message = f"HTTP {status}"
    elif status == 432:
        result_status = "quota_exhausted"
        message = extract_json_text(payload)
    elif any(
        token in text
        for token in (
            "exceeded",
            "quota",
            "rate limit",
            "credits",
            "limit reached",
            "usage limit",
            "upgrade your plan",
        )
    ):
        result_status = "quota_exhausted"
        message = extract_json_text(payload)
    elif any(token in text for token in ("invalid api key", "unauthorized", "forbidden", "api key")):
        result_status = "invalid"
        message = extract_json_text(payload)
    elif status >= 400:
        result_status = "error"
        message = f"HTTP {status}"

    if result_status == "healthy":
        if isinstance(payload, dict) and ("results" in payload or "answer" in payload):
            result_count = len(payload.get("results", [])) if isinstance(payload.get("results"), list) else 0
            message = f"ok ({result_count} results)"
        else:
            result_status = "error"
            message = "unexpected response shape"

    return {
        "provider": "tavily",
        "status": result_status,
        "message": message,
        "latency_ms": latency_ms,
        "http_status": status,
        "body": payload,
    }


def print_provider_summary(provider_name: str, keys, results, show_body: bool):
    print(f"\n[{provider_name}] configured keys: {len(keys)}")
    if not keys:
        print("  - FAIL no keys configured")
        return

    healthy = 0
    for index, (key, result) in enumerate(zip(keys, results), start=1):
        status = result["status"]
        if status == "healthy":
            healthy += 1
        tag = {
            "healthy": "OK",
            "quota_exhausted": "WARN",
            "invalid": "FAIL",
            "network_error": "FAIL",
            "error": "FAIL",
        }.get(status, "FAIL")
        latency = (
            f", latency={result['latency_ms']}ms"
            if result["latency_ms"] is not None
            else ""
        )
        print(
            f"  - [{tag}] key#{index} {mask_key(key)} status={status}{latency} message={truncate_text(result['message'])}"
        )
        if show_body and status != "healthy" and result.get("body") is not None:
            body_text = truncate_text(extract_json_text(result["body"]))
            if body_text:
                print(f"      body={body_text}")

    print(f"  summary: healthy {healthy}/{len(keys)}")


def main() -> int:
    args = parse_args()
    try:
        config_path = choose_config_path(args.config)
    except FileNotFoundError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    config = load_effective_config(config_path)

    fmp_config = config.get("fmp") or {}
    search_config = config.get("search") or {}
    fmp_keys = normalize_keys(
        [fmp_config.get("api_key", "")]
        + list(fmp_config.get("api_keys") or [])
    )
    tavily_keys = normalize_keys(list(search_config.get("api_keys") or []))
    fmp_base_url = (fmp_config.get("base_url") or "https://financialmodelingprep.com/api").strip()

    print("Hone API quota diagnosis")
    print(f"config: {config_path}")
    overlay = overlay_path(config_path)
    print(f"overlay: {overlay if overlay.exists() else '(none)'}")
    print(f"FMP base_url: {fmp_base_url}")
    print("This script performs live API probes and does not modify business data.")

    fmp_results = [probe_fmp(fmp_base_url, key, args.fmp_symbol, args.timeout) for key in fmp_keys]
    tavily_results = [probe_tavily(key, args.tavily_query, args.timeout) for key in tavily_keys]

    print_provider_summary("FMP", fmp_keys, fmp_results, args.show_body)
    print_provider_summary("Tavily", tavily_keys, tavily_results, args.show_body)

    fmp_healthy = any(result["status"] == "healthy" for result in fmp_results)
    tavily_healthy = any(result["status"] == "healthy" for result in tavily_results)

    print("\nOverall")
    if fmp_healthy and tavily_healthy:
        print("  healthy: each provider has at least one working key")
        unhealthy_keys = [
            result
            for result in fmp_results + tavily_results
            if result["status"] != "healthy"
        ]
        if unhealthy_keys:
            print("  warning: some fallback keys are exhausted or invalid; consider rotation/replacement")
        return 0

    if not fmp_healthy:
        print("  FAIL: FMP currently has no healthy key")
    if not tavily_healthy:
        print("  FAIL: Tavily currently has no healthy key")
    return 1


if __name__ == "__main__":
    sys.exit(main())
PY
