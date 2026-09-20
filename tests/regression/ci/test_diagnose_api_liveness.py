#!/usr/bin/env python3
"""Local, no-account regression coverage for the read-only monitor."""

import importlib.util
import json
import os
from pathlib import Path
import signal
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from unittest.mock import patch

MODULE_PATH = Path(__file__).resolve().parents[3] / "scripts" / "diagnose_api_liveness.py"
SPEC = importlib.util.spec_from_file_location("diagnose_api_liveness", MODULE_PATH)
monitor = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(monitor)


class Handler(BaseHTTPRequestHandler):
    observed_cookie = None

    def log_message(self, *args):
        pass

    def do_GET(self):
        Handler.observed_cookie = self.headers.get("Cookie")
        if self.path.startswith("/meta"):
            payload = {"cloud_postgres_health": {"ok": True},
                       "cloud_oss_health": {"ok": True},
                       "cloud_storage_authoritative": True}
            if self.path != "/meta/missing-sha":
                payload["build"] = {"git_sha": "private non-sha" if self.path == "/meta/invalid-sha" else "b" * 40}
            self.send_response(200)
            self.end_headers()
            self.wfile.write(json.dumps(payload).encode())
            return
        if self.path == "/slow":
            time.sleep(0.4)
        if self.path == "/redirect":
            self.send_response(302)
            self.send_header("Location", "/private")
            self.end_headers()
            return
        self.send_response(401)
        self.end_headers()
        try:
            self.wfile.write(b'{"error":"not authenticated","secret":"do not log"}')
        except BrokenPipeError:
            pass


class MonitorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        signal.signal(signal.SIGALRM, monitor.alarm_timeout)
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        cls.worker = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.worker.start()
        cls.url = f"http://127.0.0.1:{cls.server.server_port}"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.worker.join()

    def test_meta_whitelist_drops_secrets_and_messages(self):
        payload = {"build": {"git_sha": "a" * 40, "secret": "secret"},
                   "cloud_postgres_health": {"ok": True, "message": "postgres://secret"},
                   "cloud_oss_health": {"ok": True}, "cloud_storage_authoritative": True,
                   "messages": ["private conversation"]}
        result = monitor.filter_meta(payload)
        self.assertEqual(result, {"git_sha": "a" * 40, "postgres_ok": True,
                                  "object_store_ok": True, "cloud_authoritative": True})

    def test_public_401_no_cookie_or_response_content(self):
        result = monitor.probe(self.url, "public", 1, time.monotonic() + 2)
        self.assertTrue(result["ok"])
        self.assertIsNone(Handler.observed_cookie)
        self.assertNotIn("secret", json.dumps(result))

    def test_missing_revision_does_not_claim_healthy_or_recovered(self):
        state = monitor.State()
        state.observe({"public": {"ok": True}, "meta": {"ok": True, "health": {"git_sha": "a" * 40}}})
        for suffix in ("missing-sha", "invalid-sha"):
            result = monitor.probe(self.url + "/meta/" + suffix, "meta", 1, time.monotonic() + 2)
            self.assertFalse(result["ok"])
            self.assertEqual(result["error"], "missing_or_invalid_revision")
            self.assertIsNone(result["health"]["git_sha"])
            self.assertNotIn("private non-sha", json.dumps(result))
            events = state.observe({"public": {"ok": True}, "meta": result})
            self.assertNotIn("recovered", events)
        self.assertEqual(events, ["failure"])
        result = monitor.probe(self.url + "/meta/good", "meta", 1, time.monotonic() + 2)
        self.assertTrue(result["ok"])
        self.assertEqual(state.observe({"public": {"ok": True}, "meta": result}),
                         ["recovered", "revision_changed"])

    def test_redirect_is_not_followed(self):
        result = monitor.probe(self.url + "/redirect", "public", 1, time.monotonic() + 2)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], 302)

    def test_probe_total_deadline(self):
        started = time.monotonic()
        result = monitor.probe(self.url + "/slow", "public", 1, started + 0.08)
        self.assertEqual(result["error"], "timeout")
        self.assertLess(time.monotonic() - started, 0.3)

    def test_two_failures_then_recovery_and_revision_change(self):
        state = monitor.State()
        good = {"public": {"ok": True}, "meta": {"ok": True, "health": {"git_sha": "a" * 40}}}
        bad = {"public": {"ok": False}, "meta": {"ok": True}}
        self.assertEqual(state.observe(good), [])
        self.assertEqual(state.observe(bad), [])
        self.assertEqual(state.observe(bad), ["failure"])
        self.assertEqual(state.observe(bad), [])
        self.assertEqual(state.observe(good), ["recovered"])
        good["meta"]["health"]["git_sha"] = "b" * 40
        self.assertEqual(state.observe(good), ["revision_changed"])

    def test_recovery_requires_all_probes_to_recover(self):
        state = monitor.State()
        failed = {"public": {"ok": False}, "meta": {"ok": True}}
        state.observe(failed)
        self.assertEqual(state.observe(failed), ["failure"])
        partial = {"public": {"ok": True}, "meta": {"ok": False}}
        self.assertEqual(state.observe(partial), [])
        self.assertTrue(state.failed)

    def test_log_socket_filter_drops_unrelated_socket_paths(self):
        class Result:
            stdout = "u_str ESTAB 7 9 * 123 * 456 ino:123 sk:1\nESTAB 1 2 /private/user 999 * 998 ino:999 sk:2\n"
        processes = [{"stdio_targets": {"1": "socket:[123]", "2": "/dev/null"}}]
        with patch.object(monitor.subprocess, "run", return_value=Result()):
            result = monitor.log_socket_queues(processes)
        self.assertEqual(result["rows"], [{"inode": "123", "state": "ESTAB", "rx_queue": 7, "tx_queue": 9}])
        self.assertNotIn("private", json.dumps(result))

    def test_socket_ownership_finds_listener_child_not_supervisor(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "net").mkdir()
            (root / "net/tcp").write_text("header\n 0: 0100007F:1F8D 00000000:0000 0A 00000000:00000003 00:00000000 00000000 1000 0 777 1\n")
            (root / "net/tcp6").write_text("header\n")
            (root / "100/fd").mkdir(parents=True)
            (root / "101/fd").mkdir(parents=True)
            (root / "101/fd/8").symlink_to("socket:[777]")
            rows = monitor.socket_rows(root, {8077, 8088})
            self.assertEqual(rows[0]["rx_queue"], 3)
            self.assertEqual(monitor.listener_owners(root, rows), {"101": ["777"]})
            self.assertNotIn("0100007F", json.dumps(rows))

    def test_log_rotation_modes_and_symlink_rejection(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw) / "logs"
            writer = monitor.PrivateLog(root, 150, 2)
            try:
                for i in range(10):
                    writer.write({"i": i, "value": "x" * 80})
            finally:
                writer.close()
            self.assertEqual(len(list(root.glob("metrics.jsonl*"))), 2)
            self.assertEqual(root.stat().st_mode & 0o777, 0o700)
            self.assertTrue(all(p.stat().st_mode & 0o777 == 0o600 for p in root.iterdir()))
            (root / "metrics.jsonl").unlink()
            target = Path(raw) / "private"
            target.write_text("unchanged")
            (root / "metrics.jsonl").symlink_to(target)
            writer = monitor.PrivateLog(root, 150, 2)
            try:
                with self.assertRaises(OSError):
                    writer.write({"test": "blocked"})
            finally:
                writer.close()
            self.assertEqual(target.read_text(), "unchanged")

    def test_url_credentials_and_queries_rejected(self):
        for url in ("http://user:secret@localhost/api/meta", "http://localhost/api/meta?token=x"):
            with self.assertRaises(Exception):
                monitor.validate_url(url)


if __name__ == "__main__":
    unittest.main()
