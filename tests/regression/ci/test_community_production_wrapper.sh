#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

python3 - <<'PY'
"""Exercise the real wrapper/remote runner with no cloud account or network."""
import builtins
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import types
import unittest
import urllib.parse

spec = importlib.util.spec_from_file_location('community_production', 'scripts/community_production.py')
wrapper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(wrapper)
RUNTIME = '/opt/hone/releases/' + 'a' * 40 + '-ghcr-runtime/bin/hone-cli'


class Output:
    def __init__(self):
        self.buffer = io.BytesIO()

    def write(self, text):
        return self.buffer.write(text.encode())

    def flush(self):
        pass


class WrapperTests(unittest.TestCase):
    def run_remote_fixture(self, *, wrong_identity=False, runtime_changed=False):
        service = {
            'HONE_POSTGRES_HOST': 'db.internal', 'HONE_POSTGRES_PORT': '5432',
            'HONE_POSTGRES_USER': 'community', 'HONE_POSTGRES_DATABASE': 'hone',
            'HONE_POSTGRES_PASSWORD': 'synthetic-password/? value',
            'HONE_OSS_ACCESS_KEY_ID': 'synthetic-access-id',
            'HONE_OSS_ACCESS_KEY_SECRET': 'synthetic-oss-secret',
            'HONE_POSTGRES_NO_PROXY': 'true', 'NO_PROXY': 'localhost',
            'TOKEN_BUDGET': '12345', 'SERVICE_TOKEN_ENABLED': 'true',
            'HTTP_PROXY': 'http://plain.proxy:8080',
            'HTTPS_PROXY': 'https://proxy-user:synthetic-proxy-password@proxy.invalid',
            'UNRELATED_API_KEY': 'synthetic-unrelated-key',
        }
        fields = ['HONE_POSTGRES_HOST', 'HONE_POSTGRES_PORT',
                  'HONE_POSTGRES_USER', 'HONE_POSTGRES_DATABASE']
        identity = hashlib.sha256('|'.join(service[key] for key in fields).encode()).hexdigest()
        if wrong_identity:
            identity = '0' * 64
        command = ['cloud', 'community-publish', '--feed-prefix', 'reviewed/feed', '--apply']
        calls = []
        main_pid_checks = []

        class RuntimePath:
            def __init__(self, path):
                self.path = str(path)

            def __truediv__(self, child):
                return RuntimePath(self.path + '/' + str(child))

            def __str__(self):
                return self.path

            def resolve(self, strict=False):
                return RuntimePath(RUNTIME)

            def stat(self):
                return types.SimpleNamespace(st_mode=0o100755, st_uid=0, st_dev=1, st_ino=2)

            def read_bytes(self):
                self_outer.assertTrue(self.path.endswith('/environ'))
                return '\0'.join(key + '=' + value for key, value in service.items()).encode()

            def read_text(self):
                self_outer.assertTrue(self.path.endswith('/stat'))
                return '100 (hone-cli) S ' + '0 ' * 18 + '12345'

        self_outer = self

        def fake_path(path):
            if str(path).startswith(('/proc', '/opt/hone/releases/')):
                return RuntimePath(path)
            return Path(path)

        def main_pid(arguments, **kwargs):
            self.assertEqual(arguments, ['systemctl', 'show', 'hone-web.service', '-p', 'MainPID', '--value'])
            main_pid_checks.append(arguments)
            return b'101' if runtime_changed and len(main_pid_checks) > 1 else b'100'

        def execute(arguments, **kwargs):
            calls.append(arguments)
            self.assertEqual(arguments, [RUNTIME, *command])
            child = kwargs['env']
            self.assertEqual(child['HONE_POSTGRES_HOST'], service['HONE_POSTGRES_HOST'])
            self.assertEqual(child['HONE_POSTGRES_PASSWORD'], service['HONE_POSTGRES_PASSWORD'])
            self.assertEqual(child['TZ'], 'Asia/Shanghai')
            self.assertEqual(child['HONE_POSTGRES_NO_PROXY'], 'true')
            self.assertEqual(child['HONE_CLOUD_MODE'], 'cloud')
            self.assertNotIn('DATABASE_URL', child)
            self.assertNotIn('UNRELATED_API_KEY', child)
            self.assertNotIn('HTTPS_PROXY', child)
            config = Path(child['HONE_CONFIG_PATH'])
            self.assertEqual(config.parent, Path(kwargs['cwd']))
            self.assertEqual(config.read_text(), 'timezone: Asia/Shanghai\ncloud:\n  mode: cloud\n')
            response = {'ok': True, 'no_op': True, 'would_write': 0, 'conflicts': [],
                        'token_budget': 12345, 'plain_proxy': service['HTTP_PROXY'],
                        'credential_echo': [service['HONE_POSTGRES_PASSWORD'],
                            urllib.parse.quote(service['HONE_POSTGRES_PASSWORD'], safe=''),
                            service['HONE_OSS_ACCESS_KEY_ID'], service['HONE_OSS_ACCESS_KEY_SECRET'],
                            service['HTTPS_PROXY'], service['UNRELATED_API_KEY']]}
            return subprocess.CompletedProcess(arguments, 0, json.dumps(response).encode(), b'')

        stdout, stderr = Output(), Output()
        fake_modules = {
            'pathlib': types.SimpleNamespace(Path=fake_path),
            'os': types.SimpleNamespace(X_OK=os.X_OK, access=lambda *_: True),
            'sys': types.SimpleNamespace(stdout=stdout, stderr=stderr),
            'subprocess': types.SimpleNamespace(check_output=main_pid, run=execute,
                DEVNULL=subprocess.DEVNULL, PIPE=subprocess.PIPE,
                SubprocessError=subprocess.SubprocessError),
        }
        original_import = builtins.__import__

        def fake_import(name, *args, **kwargs):
            return fake_modules[name] if name in fake_modules else original_import(name, *args, **kwargs)

        namespace = {'__builtins__': dict(vars(builtins), __import__=fake_import)}
        with self.assertRaises(SystemExit) as stopped:
            exec(compile(wrapper.build_remote_script(command, identity), '<remote-runner>', 'exec'), namespace)
        return stopped.exception.code, stdout.buffer.getvalue(), stderr.buffer.getvalue(), calls, namespace

    def test_remote_argument_boundary(self):
        for command in [
            ['cloud', 'community-inspect', '--anchor-only'],
            ['cloud', 'community-inspect', '--limit', '10'],
            ['cloud', 'community-publish', '--feed-prefix', 'feed', '--asset-prefix', 'assets', '--apply'],
        ]:
            compile(wrapper.build_remote_script(command, 'a' * 64), '<remote-runner>', 'exec')
        for command in [
            ['cloud', 'community-append', '--manifest', '/local/file'],
            ['cloud', 'community-assets', '--manifest', '/local/file'],
            ['cloud', 'community-publish', '--manifest=/local/file'],
            ['cloud', 'community-publish', '--config=/local/config'],
            ['cloud', 'community-inspect', '--apply'],
            ['cloud', 'community-inspect', '--cli=/local/cli'],
        ]:
            with self.subTest(command=command), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises((ValueError, SystemExit)):
                    wrapper.build_remote_script(command, 'a' * 64)
        result = subprocess.run([sys.executable, 'scripts/community_production.py',
            '--operator-config', '/missing-operator-config', '--remote', '--cli', '/missing-cli',
            'cloud', 'community-inspect'], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('--remote cannot be combined with --cli', result.stderr)

    def test_managed_runtime_isolated_environment_and_redaction(self):
        status, stdout, stderr, calls, namespace = self.run_remote_fixture()
        self.assertEqual(status, 0)
        self.assertEqual(len(calls), 1)
        response = json.loads(stdout)
        self.assertIs(response['ok'], True)
        self.assertIs(response['no_op'], True)
        self.assertEqual(response['token_budget'], 12345)
        self.assertEqual(response['plain_proxy'], 'http://plain.proxy:8080')
        self.assertEqual(response['credential_echo'], ['[REDACTED]'] * 6)
        self.assertEqual(json.loads(stderr)['runtime_cli'], RUNTIME)
        for key in ('NO_PROXY', 'HONE_POSTGRES_NO_PROXY', 'TOKEN_BUDGET', 'SECRET_ENABLED'):
            self.assertFalse(namespace['is_credential'](key, 'true'), key)
        self.assertFalse(namespace['is_credential']('ALL_PROXY', 'socks5://plain.proxy:1080'))
        self.assertTrue(namespace['is_credential']('all_proxy', 'socks5://user:password@proxy:1080'))
        # Real short credentials must still be hidden; do not fix false positives
        # by introducing a minimum secret length.
        self.assertTrue(namespace['is_credential']('HONE_POSTGRES_PASSWORD', 'x'))
        namespace['sensitive'] = ['x']
        self.assertEqual(namespace['redact'](b'x'), b'[REDACTED]')

    def test_wrong_production_identity_never_executes_cli(self):
        status, stdout, stderr, calls, _ = self.run_remote_fixture(wrong_identity=True)
        self.assertEqual(status, 1)
        self.assertEqual(stdout, b'')
        self.assertEqual(calls, [])
        self.assertIn(b'Production database identity changed', stderr)

    def test_runtime_cutover_during_preparation_never_executes_cli(self):
        status, stdout, stderr, calls, _ = self.run_remote_fixture(runtime_changed=True)
        self.assertEqual(status, 1)
        self.assertEqual(stdout, b'')
        self.assertEqual(calls, [])
        self.assertIn(b'Managed runtime changed', stderr)


unittest.main(argv=['community-production-wrapper'], verbosity=2)
PY
