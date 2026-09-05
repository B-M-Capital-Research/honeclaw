#!/usr/bin/env python3
"""Run community archive CLI operations against the reviewed production authority.

The ignored operator JSON contains project, instance, zone and
expected_pg_identity_sha256. It must not contain credentials. The service's live
PG/R2 credentials travel through IAP into this process's memory only. A temporary
loopback SSH tunnel and an isolated, non-secret CLI config prevent repository
dotenv settings from silently selecting another database.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import tempfile
import time

REPO_ROOT = Path(__file__).resolve().parents[1]
COMMUNITY_COMMANDS = {
    'community-inspect', 'community-append', 'community-assets', 'community-publish',
}
READ_LIVE_ENV = r'''import pathlib,json,subprocess
p=int(subprocess.check_output(['systemctl','show','hone-web.service','-p','MainPID','--value']))
e=dict(x.split('=',1) for x in pathlib.Path(f'/proc/{p}/environ').read_bytes().decode().split('\0') if '=' in x)
if e.get('DATABASE_URL'):
 raise SystemExit('Runtime DATABASE_URL authority requires an explicit audit')
keys=[k for k in e if k.startswith('HONE_POSTGRES_') or k.startswith('HONE_OSS_') or k in ['HONE_CLOUD_MODE','HONE_CLOUD_STRICT_NO_LOCAL_STORAGE']]
print(json.dumps({k:e[k] for k in keys}))
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--operator-config', type=Path,
                        default=REPO_ROOT / 'data/community-imports/production-operator.json',
                        help='Ignored JSON with reviewed host and PostgreSQL identity (no credentials)')
    parser.add_argument('--cli', type=Path, default=REPO_ROOT / 'target/debug/hone-cli',
                        help='Exact local hone-cli binary to run')
    parser.add_argument('command', nargs=argparse.REMAINDER,
                        help='cloud community-inspect|community-append|community-assets|community-publish [arguments]')
    args = parser.parse_args()
    command = args.command
    if len(command) < 2 or command[0] != 'cloud' or command[1] not in COMMUNITY_COMMANDS:
        parser.error('Select a supported cloud community operation')
    cli = args.cli.resolve(strict=True)
    if not cli.is_file() or not os.access(cli, os.X_OK):
        parser.error('--cli must be an executable file')
    operator = json.loads(args.operator_config.read_text())
    for key in ('project', 'instance', 'zone'):
        if not isinstance(operator.get(key), str) or not re.fullmatch(r'[a-z0-9][a-z0-9-]{0,127}', operator[key]):
            parser.error(f'Invalid operator {key}')
    expected_identity = operator.get('expected_pg_identity_sha256', '')
    if not re.fullmatch(r'[0-9a-f]{64}', expected_identity):
        parser.error('Operator config must pin a reviewed PostgreSQL identity SHA-256')
    if set(operator) != {'project', 'instance', 'zone', 'expected_pg_identity_sha256'}:
        parser.error('Operator config accepts host identity fields only; never store credentials there')
    base = ['gcloud', 'compute', 'ssh', operator['instance'], '--project', operator['project'],
            '--zone', operator['zone'], '--tunnel-through-iap']
    result = subprocess.run(base + ['--command', 'sudo python3 -'], input=READ_LIVE_ENV,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=90)
    if result.returncode:
        raise RuntimeError('Production environment read failed over IAP')
    live = json.loads(result.stdout)
    identity_fields = ['HONE_POSTGRES_HOST', 'HONE_POSTGRES_PORT',
                       'HONE_POSTGRES_USER', 'HONE_POSTGRES_DATABASE']
    if any(not live.get(key) for key in identity_fields + ['HONE_POSTGRES_PASSWORD']):
        raise RuntimeError('Production PostgreSQL environment is incomplete')
    identity = hashlib.sha256('|'.join(live[key] for key in identity_fields).encode()).hexdigest()
    if identity != expected_identity:
        raise RuntimeError('Production database identity changed; audit before updating the operator config')
    host = live['HONE_POSTGRES_HOST']
    if not re.fullmatch(r'[A-Za-z0-9._-]+', host):
        raise RuntimeError('Production PostgreSQL host needs a reviewed tunnel encoding')
    port = int(live['HONE_POSTGRES_PORT'])
    if not 0 < port <= 65535:
        raise RuntimeError('Production PostgreSQL port is invalid')
    with socket.socket() as reserve:
        reserve.bind(('127.0.0.1', 0))
        local_port = reserve.getsockname()[1]
    forward = f'127.0.0.1:{local_port}:{host}:{port}'
    tunnel = subprocess.Popen(base + ['--ssh-flag=-N', '--ssh-flag=-oExitOnForwardFailure=yes',
        '--ssh-flag=-oServerAliveInterval=30', '--ssh-flag=-L' + forward],
        stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        ready = False
        for _ in range(120):
            if tunnel.poll() is not None:
                raise RuntimeError('Production PG tunnel exited before ready')
            try:
                with socket.create_connection(('127.0.0.1', local_port), timeout=.25):
                    ready = True
                break
            except OSError:
                time.sleep(.5)
        if not ready:
            raise RuntimeError('Production PG tunnel did not become ready')
        # Remove inherited cloud settings, then install only the reviewed live
        # environment. The temporary cwd also excludes HONE_DOTENV_OVERRIDE.
        command_env = {key: value for key, value in os.environ.items()
                       if not key.startswith(('HONE_POSTGRES_', 'HONE_OSS_'))
                       and key not in {'DATABASE_URL', 'HONE_DATABASE_URL', 'HONE_CONFIG_PATH', 'HONE_USER_CONFIG_PATH'}}
        command_env.update(live)
        command_env.update({'HONE_POSTGRES_HOST': '127.0.0.1', 'HONE_POSTGRES_PORT': str(local_port),
            'HONE_POSTGRES_NO_PROXY': 'true', 'HONE_CLOUD_MODE': 'cloud'})
        for flag in ('--manifest',):
            if flag in command:
                index = command.index(flag) + 1
                if index >= len(command):
                    parser.error(f'{flag} needs a file path')
                command[index] = str(Path(command[index]).resolve(strict=True))
        with tempfile.TemporaryDirectory(prefix='hone-production-community-') as working:
            config = Path(working) / 'config.yaml'
            config.write_text('cloud:\n  mode: cloud\n')
            command_env['HONE_CONFIG_PATH'] = str(config)
            command_env['HONE_USER_CONFIG_PATH'] = str(config)
            print(json.dumps({'authority': 'managed-production', 'pg_identity_sha256': identity,
                              'tunnel': 'loopback-only', 'credentials': 'memory-only'}),
                  file=sys.stderr)
            return subprocess.run([str(cli), *command], env=command_env, cwd=working).returncode
    finally:
        tunnel.terminate()
        try:
            tunnel.wait(timeout=10)
        except subprocess.TimeoutExpired:
            tunnel.kill()
            tunnel.wait()


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        # Errors remain diagnostic without dumping subprocess streams or config.
        raise SystemExit(f'Production community command stopped: {error}')
