#!/usr/bin/env python3
"""Bounded, read-only Linux API monitor; run as root for /proc evidence.

Never restarts services, sends credentials/messages, reads process environments,
or queries business data. Only health flags and a validated build SHA survive
the HTTP response filter. Detailed snapshots identify socket owners, not the
service supervisor. A snapshot is diagnostic evidence, not a user-space stack.

Example (one hour, 30-second interval, at most four 8 MiB log segments):
  sudo python3 scripts/diagnose_api_liveness.py --duration 3600

Requires Python 3.9+ and Linux /proc. systemctl is optional. PostgreSQL aggregate
inspection is intentionally left to an independently reviewed operator command.
"""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import json
import os
from pathlib import Path
import re
import signal
import socket
import stat
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone


MAX_BODY = 64 * 1024
MAX_RECORD = 512 * 1024
PROC = Path("/proc")


def utc_now():
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds")


def alarm_timeout(_signum, _frame):
    raise TimeoutError("diagnostic time budget exhausted")


@contextlib.contextmanager
def budget(seconds, deadline):
    remaining = min(seconds, deadline - time.monotonic())
    if remaining <= 0:
        raise TimeoutError("diagnostic time budget exhausted")
    signal.setitimer(signal.ITIMER_REAL, remaining)
    try:
        yield
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def validate_url(value):
    parsed = urllib.parse.urlsplit(value)
    if (parsed.scheme not in ("http", "https") or not parsed.hostname
            or parsed.username or parsed.password or parsed.query or parsed.fragment):
        raise argparse.ArgumentTypeError("use an HTTP(S) URL without credentials/query/fragment")
    try:
        parsed.port
    except ValueError as exc:
        raise argparse.ArgumentTypeError("invalid URL port") from exc
    return value


def filter_meta(payload):
    """Whitelist scalar health data; never retain free-form errors or config."""
    if not isinstance(payload, dict):
        raise ValueError("invalid meta response")
    build = payload.get("build")
    sha = build.get("git_sha") if isinstance(build, dict) else None
    if not isinstance(sha, str) or not re.fullmatch(r"[0-9a-fA-F]{40}", sha):
        sha = None
    result = {"git_sha": sha}
    for source, target in (("cloud_postgres_health", "postgres_ok"),
                           ("cloud_oss_health", "object_store_ok")):
        value = payload.get(source)
        flag = value.get("ok") if isinstance(value, dict) else None
        result[target] = flag if type(flag) is bool else None
    flag = payload.get("cloud_storage_authoritative")
    result["cloud_authoritative"] = flag if type(flag) is bool else None
    return result


def probe(url, kind, timeout, deadline):
    started = time.monotonic()
    result = {"ok": False, "status": None, "error": None}
    try:
        with budget(timeout, deadline):
            # Ignore proxy environment settings and do not install a cookie jar.
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
            request = urllib.request.Request(url, headers={"Accept": "application/json",
                                                         "User-Agent": "hone-liveness/1"})
            try:
                response = opener.open(request, timeout=timeout)
            except urllib.error.HTTPError as exc:
                response = exc
            with response:
                result["status"] = response.code
                body = response.read(MAX_BODY + 1)
            if len(body) > MAX_BODY:
                raise ValueError("oversized response")
            payload = json.loads(body)
            if not isinstance(payload, dict):
                raise ValueError("invalid response")
            if kind == "meta" and result["status"] == 200:
                result["health"] = filter_meta(payload)
                result["ok"] = all(result["health"][key] is True for key in
                                   ("postgres_ok", "object_store_ok", "cloud_authoritative"))
                if result["health"]["git_sha"] is None:
                    result["ok"] = False
                    result["error"] = "missing_or_invalid_revision"
            elif kind == "public":
                result["ok"] = result["status"] == 401
            elif kind == "control":
                result["ok"] = result["status"] == 200
    except (TimeoutError, socket.timeout):
        result["error"] = "timeout"
    except urllib.error.URLError as exc:
        result["error"] = "timeout" if isinstance(exc.reason, TimeoutError) else "connection_error"
    except (OSError, ValueError, UnicodeError):
        result["error"] = "invalid_response_or_io_error"
    result["elapsed_ms"] = round((time.monotonic() - started) * 1000, 3)
    return result


def read_limited(path, limit=8192):
    try:
        with path.open("r", errors="replace") as handle:
            content = handle.read(limit + 1)
        return content[:limit] + ("\n[truncated]" if len(content) > limit else "")
    except OSError as exc:
        return {"unavailable_errno": exc.errno}


def socket_rows(proc_root, ports):
    """Keep only target server ports and queue sizes; omit peer addresses."""
    result = []
    for family in ("tcp", "tcp6"):
        content = read_limited(proc_root / "net" / family, 1024 * 1024)
        if not isinstance(content, str):
            continue
        for line in content.splitlines()[1:]:
            parts = line.split()
            if len(parts) < 10:
                continue
            try:
                port = int(parts[1].rsplit(":", 1)[1], 16)
                if port not in ports:
                    continue
                tx, rx = (int(value, 16) for value in parts[4].split(":"))
                result.append({"family": family, "port": port, "state": parts[3],
                               "tx_queue": tx, "rx_queue": rx, "inode": parts[9]})
            except (ValueError, IndexError):
                continue
    return sorted(result, key=lambda row: row["state"] != "0A")[:512]


def listener_owners(proc_root, rows):
    inodes = {row["inode"] for row in rows if row["state"] == "0A"}
    owners = {}
    if not inodes:
        return owners
    # Unlike systemd MainPID, socket ownership finds children that serve HTTP.
    for process in proc_root.iterdir():
        if not process.name.isdigit():
            continue
        try:
            with os.scandir(process / "fd") as entries:
                for entry in entries:
                    try:
                        match = re.fullmatch(r"socket:\[(\d+)\]", os.readlink(entry.path))
                        if match and match.group(1) in inodes:
                            owners.setdefault(int(process.name), set()).add(match.group(1))
                    except OSError:
                        continue
        except OSError:
            continue
    return {str(pid): sorted(inodes) for pid, inodes in sorted(owners.items())}


def process_snapshot(proc_root, pid, max_threads):
    path = proc_root / str(pid)
    result = {"pid": int(pid), "stat": read_limited(path / "stat")}
    try:
        result["exe"] = os.readlink(path / "exe")
        result["stdio_targets"] = {}
        for descriptor in ("1", "2"):
            try:
                result["stdio_targets"][descriptor] = os.readlink(path / "fd" / descriptor)
            except OSError as exc:
                result["stdio_targets"][descriptor] = {"unavailable_errno": exc.errno}
        with os.scandir(path / "fd") as entries:
            result["fd_count"] = sum(1 for _ in entries)
        tids = sorted((p for p in (path / "task").iterdir() if p.name.isdigit()),
                      key=lambda p: int(p.name))
    except OSError as exc:
        result["unavailable_errno"] = exc.errno
        return result
    result["status"] = read_limited(path / "status")
    result["thread_count"] = len(tids)
    result["threads_truncated"] = len(tids) > max_threads
    # /proc syscall exposes register values (including pointer addresses), not
    # the pointed-to buffers. Never read /proc/PID/mem or dereference those values.
    result["threads"] = [{"tid": int(thread.name), **{
        key: read_limited(thread / key, 2048) for key in ("comm", "wchan", "syscall", "stack")
    }} for thread in tids[:max_threads]]
    # Retain both stats so a raced exit/PID reuse can be detected via starttime.
    result["stat_after"] = read_limited(path / "stat")
    return result


def log_socket_queues(processes):
    """Only output queues of stdout/stderr socket inodes; omit paths/peers."""
    inodes = set()
    for process in processes:
        for target in process.get("stdio_targets", {}).values():
            match = re.fullmatch(r"socket:\[(\d+)\]", target) if isinstance(target, str) else None
            if match:
                inodes.add(match.group(1))
    if not inodes:
        return []
    try:
        command = subprocess.run(["ss", "-xenH"], stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                 text=True, timeout=1, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return {"unavailable": True}
    rows = []
    for line in command.stdout[:262144].splitlines():
        match = re.search(r"\bino:(\d+)\b", line)
        parts = line.split()
        if parts and parts[0].startswith("u_"):
            parts = parts[1:]  # Some ss versions include the Unix socket type.
        if match and match.group(1) in inodes and len(parts) >= 3:
            try:
                rows.append({"inode": match.group(1), "state": parts[0],
                             "rx_queue": int(parts[1]), "tx_queue": int(parts[2])})
            except ValueError:
                continue
    return {"rows": rows, "truncated": len(command.stdout) > 262144}


def snapshot(service, ports, max_threads, deadline, proc_root=PROC):
    result = {"captured_at": utc_now(), "partial": False}
    try:
        with budget(8, deadline):
            result["host"] = {name: read_limited(proc_root / name) for name in
                              ("loadavg", "meminfo", "stat", "pressure/cpu", "pressure/memory", "pressure/io")}
            result["sockets"] = socket_rows(proc_root, ports)
            result["listener_owners"] = listener_owners(proc_root, result["sockets"])
            result["processes"] = []
            for pid in list(result["listener_owners"])[:8]:
                result["processes"].append(process_snapshot(proc_root, pid, max_threads))
            result["log_socket_queues"] = log_socket_queues(result["processes"])
            try:
                command = subprocess.run(["systemctl", "show", service,
                                          "--property=ActiveState,SubState,MainPID,NRestarts,Result"],
                                         stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                         stderr=subprocess.DEVNULL, text=True, timeout=2, check=False)
                result["service"] = command.stdout[:2048]
            except (OSError, subprocess.TimeoutExpired):
                result["service"] = {"unavailable": True}
    except TimeoutError:
        result["partial"] = True
        result["error"] = "snapshot_budget_exhausted"
    return result


class PrivateLog:
    """Fixed filenames, no symlink append, bounded segments, one writer."""

    def __init__(self, directory, max_bytes, max_files):
        directory.mkdir(mode=0o700, exist_ok=True)
        self.directory = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        info = os.fstat(self.directory)
        if info.st_uid != os.geteuid() or stat.S_IMODE(info.st_mode) & 0o077:
            os.close(self.directory)
            raise ValueError("output directory must belong to this user and have mode 0700")
        self.max_bytes, self.max_files = max_bytes, max_files
        self.lock = self.open_private("monitor.lock")
        fcntl.flock(self.lock, fcntl.LOCK_EX | fcntl.LOCK_NB)

    def open_private(self, name):
        fd = os.open(name, os.O_CREAT | os.O_WRONLY | os.O_APPEND | os.O_NOFOLLOW,
                     0o600, dir_fd=self.directory)
        info = os.fstat(fd)
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.geteuid()
                or stat.S_IMODE(info.st_mode) & 0o077 or info.st_nlink != 1):
            os.close(fd)
            raise ValueError("output file must be private, regular and have one link")
        return fd

    def write(self, record):
        data = (json.dumps(record, separators=(",", ":")) + "\n").encode()
        if len(data) > MAX_RECORD:
            record.pop("snapshot", None)
            record["snapshot_omitted"] = "record_size_limit"
            data = (json.dumps(record, separators=(",", ":")) + "\n").encode()
        fd = self.open_private("metrics.jsonl")
        try:
            if os.fstat(fd).st_size + len(data) > self.max_bytes:
                os.close(fd)
                fd = None
                for index in range(self.max_files - 1, 0, -1):
                    source = "metrics.jsonl" if index == 1 else f"metrics.jsonl.{index - 1}"
                    try:
                        os.replace(source, f"metrics.jsonl.{index}",
                                   src_dir_fd=self.directory, dst_dir_fd=self.directory)
                    except FileNotFoundError:
                        pass
                if self.max_files == 1:
                    os.unlink("metrics.jsonl", dir_fd=self.directory)
                fd = self.open_private("metrics.jsonl")
            with os.fdopen(fd, "ab", closefd=False) as handle:
                handle.write(data)
                handle.flush()
        finally:
            if fd is not None:
                os.close(fd)

    def close(self):
        os.close(self.lock)
        os.close(self.directory)


class State:
    def __init__(self):
        self.failures = {}
        self.failed = False
        self.sha = None

    def observe(self, probes):
        for name, result in probes.items():
            self.failures[name] = 0 if result["ok"] else self.failures.get(name, 0) + 1
        failed = any(count >= 2 for count in self.failures.values())
        if self.failed and not all(result["ok"] for result in probes.values()):
            failed = True
        reasons = []
        if failed != self.failed:
            reasons.append("failure" if failed else "recovered")
        sha = probes["meta"].get("health", {}).get("git_sha")
        if sha and self.sha and sha != self.sha:
            reasons.append("revision_changed")
        if sha:
            self.sha = sha
        self.failed = failed
        return reasons


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--admin-url", type=validate_url, default="http://127.0.0.1:8077/api/meta")
    parser.add_argument("--public-url", type=validate_url, default="http://127.0.0.1:8088/api/public/auth/me")
    parser.add_argument("--control-url", type=validate_url,
                        default="http://127.0.0.1:8088/api/public/auth/dev-login/config",
                        help="read-only 200 JSON route without database access or normal request logging")
    parser.add_argument("--service", default="hone-web.service")
    parser.add_argument("--listener-ports", type=int, nargs="+", default=[8077, 8088])
    parser.add_argument("--interval", type=float, default=30)
    parser.add_argument("--duration", type=float, default=86400)
    parser.add_argument("--timeout", type=float, default=3)
    parser.add_argument("--output-dir", type=Path, default=Path("/var/log/hone-api-liveness"))
    parser.add_argument("--max-bytes", type=int, default=8 * 1024 * 1024)
    parser.add_argument("--max-files", type=int, default=4)
    parser.add_argument("--max-threads", type=int, default=64)
    args = parser.parse_args()
    if not (5 <= args.interval <= 3600 and 1 <= args.duration <= 604800 and 0.2 <= args.timeout <= 10):
        parser.error("interval must be 5..3600s, duration 1s..7d, timeout 0.2..10s")
    if not (1024 * 1024 <= args.max_bytes <= 64 * 1024 * 1024 and 1 <= args.max_files <= 8
            and 1 <= args.max_threads <= 128 and all(1 <= port <= 65535 for port in args.listener_ports)):
        parser.error("invalid log size/files/thread/port limit")
    if not re.fullmatch(r"[a-zA-Z0-9_][a-zA-Z0-9_.@-]*\.service", args.service):
        parser.error("invalid service name")
    return args


def main():
    args = parse_args()
    if os.geteuid() != 0 or not PROC.is_dir() or not Path("/proc/self/stat").exists():
        raise SystemExit("run as root on Linux; output and /proc evidence must remain private")
    os.umask(0o077)
    signal.signal(signal.SIGALRM, alarm_timeout)
    stop = False

    def stop_gracefully(_signum, _frame):
        nonlocal stop
        stop = True

    signal.signal(signal.SIGTERM, stop_gracefully)
    signal.signal(signal.SIGINT, stop_gracefully)
    writer = PrivateLog(args.output_dir, args.max_bytes, args.max_files)
    deadline = time.monotonic() + args.duration
    state = State()
    try:
        while not stop and time.monotonic() < deadline:
            started = time.monotonic()
            probes = {name: probe(url, name, args.timeout, deadline) for name, url in
                      (("control", args.control_url), ("public", args.public_url), ("meta", args.admin_url))}
            if time.monotonic() >= deadline:
                # The monitor's own expiry is not evidence of an API failure.
                break
            reasons = state.observe(probes)
            record = {"at": utc_now(), "kind": "sample", "probes": probes,
                      "consecutive_failures": dict(state.failures), "events": reasons}
            if reasons:
                record["snapshot"] = snapshot(args.service, set(args.listener_ports), args.max_threads, deadline)
            writer.write(record)
            until = min(started + args.interval, deadline)
            while not stop and time.monotonic() < until:
                time.sleep(min(0.5, until - time.monotonic()))
        writer.write({"at": utc_now(), "kind": "stopped", "reason": "signal" if stop else "duration_elapsed"})
    finally:
        writer.close()


if __name__ == "__main__":
    main()
