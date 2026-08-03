#!/usr/bin/env python3
"""执行本地服务的支付宝沙箱 Payment-Needed 支付流程。"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any
from urllib.parse import quote


PAY_ENDPOINT = "http://aicashier.dl.alipaydev.com/openclawpay/agent/v1/pay"
PAY_URL_PREFIX = "https://render.alipay.com/p/yuyan/180020010001290755/pay.html?schema="
A2M_SANDBOX_SERVICE_ID = "api_mock_service_id"
DEFAULT_PAYMENT_NEEDED_FILE = "/tmp/402_needed_file.txt"
DEFAULT_PAY_RETRIES = 3
DEFAULT_PAY_RETRY_DELAY_SECONDS = 2.0
RETRYABLE_PAY_ERROR_CODES = {"PAY_SUBMIT_FAILED"}
REDACTED = "<redacted>"
SENSITIVE_KEY_PATTERNS = (
    "signature",
    "payment_proof",
    "paymentproof",
    "proof_header",
    "proofheader",
)


def now_ms() -> str:
    return str(int(time.time() * 1000))


def ensure_curl() -> None:
    if not shutil.which("curl"):
        raise SystemExit("需要 curl，但当前 PATH 中未找到 curl")


def run_curl(args: list[str]) -> str:
    ensure_curl()
    result = subprocess.run(
        ["curl", *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"curl 执行失败，退出码 {result.returncode}：\n{result.stdout}")
    return result.stdout


def prompt_missing(value: str | None, label: str, *, secret: bool = False) -> str:
    if value:
        return value
    if secret:
        import getpass

        entered = getpass.getpass(f"{label}: ").strip()
    else:
        entered = input(f"{label}: ").strip()
    if not entered:
        raise SystemExit(f"缺少必填值：{label}")
    return entered


def load_body(args: argparse.Namespace, method: str, default_body: str | None = None) -> str | None:
    if args.body_file:
        return Path(args.body_file).read_text(encoding="utf-8")
    if args.body:
        if args.body.startswith("@"):
            return Path(args.body[1:]).read_text(encoding="utf-8")
        return args.body
    if default_body is not None:
        return default_body
    if method.upper() == "POST":
        return prompt_missing(None, "POST 请求体 body")
    return None


def parse_payment_needed(headers: str) -> str | None:
    for line in headers.splitlines():
        if line.lower().startswith("payment-needed:"):
            return line.split(":", 1)[1].strip().strip('"')
    compact = headers.strip()
    if compact and "\n" not in compact and ":" not in compact:
        return compact.strip('"')
    return None


def parse_header_value(headers: str, name: str) -> str | None:
    header_prefix = f"{name.lower()}:"
    for line in headers.splitlines():
        if line.lower().startswith(header_prefix):
            return line.split(":", 1)[1].strip().strip('"')
    return None


def fetch_payment_needed(
    url: str,
    method: str,
    body: str | None,
    content_type: str,
    output_file: str,
) -> str:
    method = method.upper()
    if method == "GET":
        headers = run_curl(["-s", "-I", url])
        payment_needed = parse_payment_needed(headers)
        if not payment_needed:
            headers = run_curl(["-s", "-D", "-", "-o", "/dev/null", "-X", "GET", url])
            payment_needed = parse_payment_needed(headers)
    else:
        curl_args = ["-s", "-D", "-", "-o", "/dev/null", "-X", method, url]
        if body is not None:
            curl_args.extend(["-H", f"Content-Type: {content_type}", "-d", body])
        headers = run_curl(curl_args)
        payment_needed = parse_payment_needed(headers)

    if not payment_needed:
        raise RuntimeError(f"未找到 Payment-Needed 响应头。原始响应头：\n{headers}")

    Path(output_file).write_text(payment_needed, encoding="utf-8")
    return payment_needed


def b64decode_json(value: str) -> dict[str, Any]:
    compact = value.strip().strip('"')
    padded = compact + ("=" * ((4 - len(compact) % 4) % 4))
    try:
        decoder = base64.urlsafe_b64decode if ("-" in compact or "_" in compact) else base64.b64decode
        decoded = decoder(padded).decode("utf-8")
        data = json.loads(decoded)
    except Exception as exc:  # noqa: BLE001 - 保留原始异常，便于定位账单解码问题。
        raise ValueError(f"无法将 Payment-Needed 按 base64 解码为 JSON：{exc}") from exc
    if not isinstance(data, dict):
        raise ValueError("解码后的 Payment-Needed JSON 必须是对象")
    return data


def snake_to_camel(key: str) -> str:
    if "_" not in key:
        return key
    first, *rest = key.split("_")
    return first + "".join(part[:1].upper() + part[1:] for part in rest if part)


def convert_keys(value: Any) -> Any:
    if isinstance(value, list):
        return [convert_keys(item) for item in value]
    if isinstance(value, dict):
        return {snake_to_camel(str(key)): convert_keys(item) for key, item in value.items()}
    return value


def timestamp_from_out_trade_no(out_trade_no: str) -> str:
    match = re.search(r"(\d{10,})$", out_trade_no or "")
    return match.group(1) if match else now_ms()


def build_cashier_payload(decoded_bill: dict[str, Any], buyer_id: str, buyer_signature: str) -> dict[str, Any]:
    converted = convert_keys(decoded_bill)
    method = converted.get("method") if isinstance(converted.get("method"), dict) else {}
    protocol = converted.get("protocol") if isinstance(converted.get("protocol"), dict) else {}
    method = dict(method)
    protocol = dict(protocol)

    out_trade_no = str(protocol.get("outTradeNo") or "")
    timestamp = timestamp_from_out_trade_no(out_trade_no)

    method["buyerUniqueIdKey"] = "buyerExternalId"
    protocol["buyerUniqueId"] = buyer_id

    return {
        "method": method,
        "protocol": protocol,
        "signature": {
            "buyerExternalId": buyer_id,
            "buyerSignature": buyer_signature,
            "timestamp": timestamp,
        },
    }


def compact_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def pretty_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2)


def redact_for_display(value: Any) -> Any:
    if isinstance(value, list):
        return [redact_for_display(item) for item in value]
    if isinstance(value, dict):
        redacted: dict[str, Any] = {}
        for key, item in value.items():
            normalized = str(key).replace("-", "_").lower()
            if any(pattern in normalized for pattern in SENSITIVE_KEY_PATTERNS):
                redacted[str(key)] = REDACTED
            else:
                redacted[str(key)] = redact_for_display(item)
        return redacted
    return value


def shell_quote(value: str | Path) -> str:
    return shlex.quote(str(value))


def post_json(url: str, payload: dict[str, Any]) -> tuple[int, str, dict[str, Any]]:
    with tempfile.TemporaryDirectory() as tmp_dir:
        headers_path = Path(tmp_dir) / "headers.txt"
        body_path = Path(tmp_dir) / "body.txt"
        status_text = run_curl(
            [
                "-s",
                "-D",
                str(headers_path),
                "-o",
                str(body_path),
                "-w",
                "%{http_code}",
                "--connect-timeout",
                "10",
                "--max-time",
                "60",
                "-X",
                "POST",
                url,
                "-H",
                "Content-Type: application/json",
                "-d",
                compact_json(payload),
            ],
        ).strip()
        try:
            status_code = int(status_text)
        except ValueError as exc:
            raise RuntimeError(f"无法解析收银接口 HTTP 状态码：{status_text}") from exc
        headers = headers_path.read_text(encoding="utf-8", errors="replace")
        raw = body_path.read_text(encoding="utf-8", errors="replace")
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"收银接口返回不是 JSON：\n{raw}") from exc
    if not isinstance(data, dict):
        raise RuntimeError(f"收银接口返回的 JSON 必须是对象：\n{raw}")
    return status_code, headers, data


def is_retryable_pay_response(pay_response: dict[str, Any]) -> bool:
    error_code = str(pay_response.get("errorCode") or "")
    error_message = str(pay_response.get("errorMessage") or "")
    return (
        error_code in RETRYABLE_PAY_ERROR_CODES
        or "系统繁忙" in error_message
        or "稍后重试" in error_message
    )


def request_cashier_with_retry(
    url: str,
    payload: dict[str, Any],
    attempts: int,
    delay_seconds: float,
) -> tuple[int, str, dict[str, Any]]:
    attempts = max(1, attempts)
    last_result: tuple[int, str, dict[str, Any]] | None = None
    for attempt in range(1, attempts + 1):
        try:
            result = post_json(url, payload)
        except RuntimeError as exc:
            if attempt < attempts:
                print(f"\n收银接口请求异常，第 {attempt}/{attempts} 次：{exc}，{delay_seconds} 秒后重试...")
                time.sleep(delay_seconds)
                continue
            raise

        status_code, headers, pay_response = result
        last_result = result
        if pay_response.get("payScheme"):
            return result
        if attempt < attempts and is_retryable_pay_response(pay_response):
            error_code = pay_response.get("errorCode")
            error_message = pay_response.get("errorMessage")
            print(
                f"\n收银接口返回临时错误，第 {attempt}/{attempts} 次："
                f"HTTP {status_code} {error_code} {error_message}，{delay_seconds} 秒后重试..."
            )
            time.sleep(delay_seconds)
            continue
        return result

    if last_result is None:
        raise RuntimeError("收银接口未返回有效响应")
    return last_result


def pay_url_from_response(pay_response: dict[str, Any]) -> str:
    pay_scheme = pay_response.get("payScheme")
    if not pay_scheme:
        raise RuntimeError(f"收银接口返回中缺少 payScheme：\n{pretty_json(pay_response)}")
    return PAY_URL_PREFIX + quote(str(pay_scheme), safe="")


def trade_no_from_response(pay_response: dict[str, Any]) -> str:
    protocol = pay_response.get("protocol") if isinstance(pay_response.get("protocol"), dict) else {}
    trade_no = protocol.get("tradeNo") or protocol.get("tradeCoreTradeNo")
    if not trade_no:
        raise RuntimeError(f"收银接口返回中缺少 tradeNo：\n{pretty_json(pay_response)}")
    return str(trade_no)


def b64_json(value: dict[str, Any]) -> str:
    return base64.b64encode(compact_json(value).encode("utf-8")).decode("ascii")


def build_payment_proof_header(
    buyer_id: str,
    trade_no: str,
    payment_proof: str | None,
    buyer_signature: str,
) -> tuple[dict[str, Any], str]:
    timestamp = now_ms()
    proof = payment_proof or hashlib.sha256(f"{trade_no}|{buyer_id}|{timestamp}".encode("utf-8")).hexdigest()
    client_session = b64_json(
        {
            "externalId": buyer_id,
            "signature": buyer_signature,
            "timestamp": timestamp,
        },
    )
    proof_body = {
        "protocol": {
            "payment_proof": proof,
            "trade_no": trade_no,
        },
        "method": {
            "client_session": client_session,
        },
    }
    return proof_body, b64_json(proof_body)


def retry_original_service(
    url: str,
    method: str,
    body: str | None,
    content_type: str,
    proof_header: str,
) -> tuple[int, str, str]:
    with tempfile.TemporaryDirectory() as tmp_dir:
        headers_path = Path(tmp_dir) / "headers.txt"
        body_path = Path(tmp_dir) / "body.txt"
        args = [
            "-s",
            "-D",
            str(headers_path),
            "-o",
            str(body_path),
            "-w",
            "%{http_code}",
            "--connect-timeout",
            "10",
            "--max-time",
            "60",
            "-X",
            method.upper(),
            url,
            "-H",
            f"Payment-Proof: {proof_header}",
        ]
        if body is not None:
            args.extend(["-H", f"Content-Type: {content_type}", "-d", body])
        status_text = run_curl(args).strip()
        try:
            status_code = int(status_text)
        except ValueError as exc:
            raise RuntimeError(f"无法解析最终服务 HTTP 状态码：{status_text}") from exc
        headers = headers_path.read_text(encoding="utf-8", errors="replace")
        response_body = body_path.read_text(encoding="utf-8", errors="replace")
        return status_code, headers, response_body


def write_json(path: Path, value: Any) -> None:
    path.write_text(pretty_json(value) + "\n", encoding="utf-8")


def create_artifact_dir(path: str | None) -> Path:
    artifact_dir = Path(path or f"/tmp/alipay_local_402_sandbox_pay_{now_ms()}")
    if artifact_dir.exists() and not artifact_dir.is_dir():
        raise RuntimeError(f"过程产物路径已存在但不是目录：{artifact_dir}")
    artifact_dir.mkdir(parents=True, exist_ok=True)
    return artifact_dir


def load_reuse_artifact(path: str | None) -> tuple[dict[str, Any], str | None]:
    if not path:
        return {}, None
    artifact_dir = Path(path)
    if not artifact_dir.is_dir():
        raise RuntimeError(f"--reuse-artifact 指定的产物目录不存在：{artifact_dir}")
    state_path = artifact_dir / "state.json"
    payment_needed_path = artifact_dir / "payment_needed.txt"
    if not state_path.exists():
        raise RuntimeError(f"--reuse-artifact 缺少 state.json：{state_path}")
    if not payment_needed_path.exists():
        raise RuntimeError(f"--reuse-artifact 缺少 payment_needed.txt：{payment_needed_path}")

    loaded = json.loads(state_path.read_text(encoding="utf-8"))
    if not isinstance(loaded, dict):
        raise RuntimeError(f"--reuse-artifact 的 state.json 必须是 JSON 对象：{state_path}")
    state = loaded
    required_keys = ["url", "method", "buyerId"]
    missing_keys = [key for key in required_keys if not state.get(key)]
    if missing_keys:
        raise RuntimeError(
            f"--reuse-artifact 的 state.json 缺少必要字段：{', '.join(missing_keys)}。"
            "请使用同一脚本成功执行过 run 后再复用产物。"
        )

    payment_needed = payment_needed_path.read_text(encoding="utf-8").strip()
    if not payment_needed:
        raise RuntimeError(f"--reuse-artifact 的 payment_needed.txt 为空：{payment_needed_path}")
    return state, payment_needed


def command_run(args: argparse.Namespace) -> int:
    reuse_state, reuse_payment_needed = load_reuse_artifact(args.reuse_artifact)
    url = prompt_missing(args.url or reuse_state.get("url"), "本地服务请求地址")
    method = (args.method or reuse_state.get("method") or "GET").upper()
    if method not in {"GET", "POST"}:
        raise SystemExit("当前技能只支持 GET 和 POST")
    buyer_id = prompt_missing(args.buyer_id or reuse_state.get("buyerId"), "沙箱买家 2088 账号")
    content_type = args.content_type or str(reuse_state.get("contentType") or "application/json")
    body = load_body(args, method, reuse_state.get("body"))
    artifact_dir = create_artifact_dir(args.artifact_dir)

    if args.payment_needed:
        payment_needed = args.payment_needed.strip()
        Path(args.payment_needed_file).write_text(payment_needed, encoding="utf-8")
    elif reuse_payment_needed:
        payment_needed = reuse_payment_needed
        Path(args.payment_needed_file).write_text(payment_needed, encoding="utf-8")
    else:
        payment_needed = fetch_payment_needed(
            url,
            method,
            body,
            content_type,
            args.payment_needed_file,
        )

    decoded_bill = b64decode_json(payment_needed)
    method_data = decoded_bill.get("method") if isinstance(decoded_bill.get("method"), dict) else {}
    if method_data.get("service_id") != A2M_SANDBOX_SERVICE_ID:
        raise RuntimeError(
            f"沙箱联调 method.service_id 必须为 {A2M_SANDBOX_SERVICE_ID}，"
            "请修正用户服务的沙箱运行配置后重试"
        )
    cashier_payload = build_cashier_payload(decoded_bill, buyer_id, args.buyer_signature)

    (artifact_dir / "payment_needed.txt").write_text(payment_needed, encoding="utf-8")
    write_json(artifact_dir / "decoded_bill.json", decoded_bill)
    write_json(artifact_dir / "cashier_payload.json", cashier_payload)

    state = {
        "url": url,
        "method": method,
        "body": body,
        "contentType": content_type,
        "buyerId": buyer_id,
        "buyerSignature": args.buyer_signature,
        "paymentNeededFile": args.payment_needed_file,
        "payEndpoint": args.pay_endpoint,
        "reusedArtifact": args.reuse_artifact,
    }

    print("Payment-Needed 已获取并保存到本地过程产物，不在终端输出原始值。")
    print("\n解码后的 Payment-Needed JSON（已脱敏）：")
    print(pretty_json(redact_for_display(decoded_bill)))
    print("\n收银接口请求体（已脱敏）：")
    print(pretty_json(redact_for_display(cashier_payload)))

    if args.dry_run:
        state["dryRun"] = True
        write_json(artifact_dir / "state.json", state)
        print(f"\ndry-run 完成。过程产物目录：{artifact_dir}")
        return 0

    pay_status, pay_headers, pay_response = request_cashier_with_retry(
        args.pay_endpoint,
        cashier_payload,
        args.pay_retries,
        args.pay_retry_delay,
    )
    state["payStatus"] = pay_status
    state["payResponse"] = pay_response
    (artifact_dir / "pay_headers.txt").write_text(pay_headers, encoding="utf-8")
    (artifact_dir / "pay_status.txt").write_text(str(pay_status), encoding="utf-8")
    write_json(artifact_dir / "pay_response.json", pay_response)
    print("\n收银接口 HTTP 状态：")
    print(pay_status)
    print("\n收银接口返回：")
    print(pretty_json(pay_response))

    if not pay_response.get("payScheme"):
        state["payError"] = {
            "errorCode": pay_response.get("errorCode"),
            "errorMessage": pay_response.get("errorMessage"),
        }
        write_json(artifact_dir / "state.json", state)
        retry_mode = " --auto-complete" if args.auto_complete else ""
        raise RuntimeError(
            "收银接口未返回 payScheme，无法生成付款链接。"
            f"过程产物已保存：{artifact_dir}。"
            "如果是 PAY_SUBMIT_FAILED/系统繁忙，可稍后复用同一个 Payment-Needed 重试：\n"
            f"python3 {shell_quote(Path(__file__).resolve())} run "
            f"--reuse-artifact {shell_quote(artifact_dir)}{retry_mode}"
        )

    pay_url = pay_url_from_response(pay_response)
    trade_no = trade_no_from_response(pay_response)
    state["payUrl"] = pay_url
    state["tradeNo"] = trade_no
    write_json(artifact_dir / "state.json", state)

    print(f"\n过程产物目录：{artifact_dir}")

    if args.auto_complete:
        print("\n继续执行 Payment-Proof 服务端联调...")
        complete_args = argparse.Namespace(
            artifact_dir=str(artifact_dir),
            payment_proof=args.payment_proof,
            buyer_signature=args.buyer_signature,
        )
        result = command_complete(complete_args)
        if result == 0:
            print("\n按量付费沙箱服务端联调通过。")
            print("\n沙箱付款体验链接（可选）：")
            print(pay_url)
        return result

    print("\n请在浏览器打开以下链接，并使用沙箱买家账号完成付款：")
    print(pay_url)
    print("付款成功后运行：")
    print(f"python3 {Path(__file__).resolve()} complete --artifact-dir {artifact_dir}")

    if args.wait:
        input("\n沙箱付款成功后按回车，继续重试原始服务...")
        complete_args = argparse.Namespace(
            artifact_dir=str(artifact_dir),
            payment_proof=args.payment_proof,
            buyer_signature=args.buyer_signature,
        )
        return command_complete(complete_args)

    return 0


def command_complete(args: argparse.Namespace) -> int:
    artifact_dir = Path(prompt_missing(args.artifact_dir, "过程产物目录"))
    state_path = artifact_dir / "state.json"
    if not state_path.exists():
        raise SystemExit(f"缺少状态文件：{state_path}")
    state = json.loads(state_path.read_text(encoding="utf-8"))
    if not isinstance(state, dict):
        raise SystemExit(f"状态文件必须是 JSON 对象：{state_path}")
    missing_keys = [key for key in ("url", "method", "buyerId", "tradeNo") if not state.get(key)]
    if missing_keys:
        raise SystemExit(f"状态文件缺少必要字段：{', '.join(missing_keys)}")

    buyer_signature = args.buyer_signature if args.buyer_signature is not None else state.get("buyerSignature", "-")
    proof_body, proof_header = build_payment_proof_header(
        buyer_id=str(state["buyerId"]),
        trade_no=str(state["tradeNo"]),
        payment_proof=args.payment_proof,
        buyer_signature=str(buyer_signature),
    )

    write_json(artifact_dir / "payment_proof_body.json", proof_body)
    (artifact_dir / "payment_proof_header.txt").write_text(proof_header, encoding="utf-8")

    print("Payment-Proof 请求体（已脱敏）：")
    print(pretty_json(redact_for_display(proof_body)))
    print("\nPayment-Proof 请求头值已生成并保存到本地过程产物，不在终端输出原始值。")

    status_code, response_headers, final_response = retry_original_service(
        url=str(state["url"]),
        method=str(state["method"]),
        body=state.get("body"),
        content_type=str(state.get("contentType") or "application/json"),
        proof_header=proof_header,
    )
    (artifact_dir / "final_response.txt").write_text(final_response, encoding="utf-8")
    (artifact_dir / "final_headers.txt").write_text(response_headers, encoding="utf-8")
    (artifact_dir / "final_status.txt").write_text(str(status_code), encoding="utf-8")

    print("\n最终服务 HTTP 状态：")
    print(status_code)
    payment_validation = parse_header_value(response_headers, "Payment-Validation")
    if payment_validation:
        print("\nPayment-Validation 响应头：")
        print(payment_validation)
    print("\n最终服务响应体：")
    print(final_response)
    print(f"\n过程产物目录：{artifact_dir}")
    if status_code < 200 or status_code >= 300:
        print(
            f"\nPayment-Proof 重试未通过：最终服务返回 HTTP {status_code}。"
            "请优先检查服务端订单映射、Payment-Proof 验证、资源防串和履约确认逻辑。"
        )
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command")

    run_parser = subparsers.add_parser("run", help="获取 Payment-Needed 并生成沙箱付款链接")
    run_parser.add_argument("--url", help="本地服务请求地址，GET 参数需要直接拼在 URL 中")
    run_parser.add_argument("--method", help="HTTP 方法：GET 或 POST")
    run_parser.add_argument("--body", help="POST 请求体，或用 @file 从文件读取请求体")
    run_parser.add_argument("--body-file", help="保存 POST 请求体的文件")
    run_parser.add_argument("--content-type", help="POST 请求的 Content-Type")
    run_parser.add_argument("--buyer-id", help="沙箱买家 2088 账号")
    run_parser.add_argument("--buyer-signature", default="-", help="买家签名占位值")
    run_parser.add_argument("--payment-needed", help="已有的 Payment-Needed 响应头值")
    run_parser.add_argument("--payment-needed-file", default=DEFAULT_PAYMENT_NEEDED_FILE)
    run_parser.add_argument("--reuse-artifact", help="复用上一轮 run 产物中的 url/method/body/content-type/payment_needed")
    run_parser.add_argument("--pay-endpoint", default=PAY_ENDPOINT)
    run_parser.add_argument("--artifact-dir", help="生成过程产物的目录")
    run_parser.add_argument(
        "--payment-proof",
        help="使用 --auto-complete 或 --wait 时可选的最终 payment_proof 值",
    )
    run_parser.add_argument("--pay-retries", type=int, default=DEFAULT_PAY_RETRIES, help="沙箱收银接口临时失败时的重试次数")
    run_parser.add_argument(
        "--pay-retry-delay",
        type=float,
        default=DEFAULT_PAY_RETRY_DELAY_SECONDS,
        help="沙箱收银接口重试间隔秒数",
    )
    run_parser.add_argument("--dry-run", action="store_true", help="调用收银接口前停止")
    run_parser.add_argument(
        "--auto-complete",
        action="store_true",
        help="生成付款链接后连续携带 Payment-Proof 重试原始服务",
    )
    run_parser.add_argument("--wait", action="store_true", help="等待付款后立刻重试原始服务")
    run_parser.set_defaults(func=command_run)

    complete_parser = subparsers.add_parser("complete", help="携带 Payment-Proof 重试原始服务")
    complete_parser.add_argument("--artifact-dir", required=True, help="run 命令生成的过程产物目录")
    complete_parser.add_argument("--payment-proof", help="可选 payment_proof；未传时由脚本按沙箱联调约定处理")
    complete_parser.add_argument("--buyer-signature", help="买家签名占位值")
    complete_parser.set_defaults(func=command_complete)

    return parser


def main(argv: list[str]) -> int:
    if argv and argv[0] not in {"run", "complete", "-h", "--help"}:
        argv = ["run", *argv]
    parser = build_parser()
    args = parser.parse_args(argv)
    if not hasattr(args, "func"):
        parser.print_help()
        return 2
    try:
        return args.func(args)
    except (RuntimeError, ValueError) as exc:
        print(f"执行失败：{exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
