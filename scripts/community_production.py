#!/usr/bin/env python3
"""Run community archive CLI operations against the reviewed production authority.

The ignored operator JSON contains project, instance, zone and
expected_pg_identity_sha256. It must not contain credentials. The service's live
PG/R2 credentials stay in memory. Local append/assets use an IAP PG tunnel;
--remote inspect/publish run the actual managed runtime CLI on GCP without
transferring credentials or R2 object bytes to this machine. Both modes use an
isolated, non-secret config to exclude repository dotenv settings.
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


REMOTE_COMMANDS = {'community-inspect', 'community-publish'}


def validate_remote_command(command):
    """Reject local paths/configuration overrides before any IAP connection."""
    if len(command) < 2 or command[0] != 'cloud' or command[1] not in REMOTE_COMMANDS:
        raise ValueError('--remote supports only cloud community-inspect or community-publish')
    parser = argparse.ArgumentParser(prog='community_production.py --remote',
                                     add_help=False, allow_abbrev=False)
    parser.add_argument('--source')
    parser.add_argument('--external-id')
    parser.add_argument('--help', '-h', action='store_true')
    if command[1] == 'community-inspect':
        parser.add_argument('--limit', type=int)
        parser.add_argument('--anchor-only', action='store_true')
    else:
        parser.add_argument('--page-size', type=int)
        parser.add_argument('--feed-prefix')
        parser.add_argument('--asset-prefix')
        parser.add_argument('--apply', action='store_true')
    parser.parse_args(command[2:])


REMOTE_RUNNER = r"""import hashlib,json,os,pathlib,re,stat,subprocess,sys,tempfile,urllib.parse
request=json.loads(REQUEST_JSON)
sensitive=[]
def is_credential(key,value):
 # Credential names end at the secret, not at flags such as TOKEN_BUDGET or
 # SECRET_ENABLED. NO_PROXY is a routing setting, never a proxy credential.
 if re.search(r'(?:^|_)(?:PASSWORD|SECRET|TOKEN|ACCESS_KEY(?:_ID|_SECRET)?|API_KEY|DATABASE_URL)$',key,re.IGNORECASE):
  return True
 if key.upper() in ('HTTP_PROXY','HTTPS_PROXY','ALL_PROXY'):
  try:
   proxy=urllib.parse.urlsplit(value if '://' in value else 'http://'+value)
   return proxy.username is not None or proxy.password is not None
  except ValueError:
   return False
 return False

def redact(data):
 for value in sensitive:
  for variant in (value,urllib.parse.quote(value,safe=''),json.dumps(value)[1:-1]):
   data=data.replace(variant.encode(),b'[REDACTED]')
 return data

def runtime_identity():
 pid=int(subprocess.check_output(['systemctl','show','hone-web.service','-p','MainPID','--value'],stderr=subprocess.DEVNULL))
 if pid <= 0:
  raise RuntimeError('Managed service has no active MainPID')
 proc=pathlib.Path('/proc')/str(pid)
 executable=(proc/'exe').resolve(strict=True)
 if not re.fullmatch(r'/opt/hone/releases/[0-9a-f]{40}-ghcr-runtime/bin/hone-cli',str(executable)):
  raise RuntimeError('Managed MainPID is not a reviewed immutable runtime hone-cli')
 info=executable.stat()
 if not stat.S_ISREG(info.st_mode) or info.st_uid != 0 or info.st_mode & 0o022 or not os.access(executable,os.X_OK):
  raise RuntimeError('Managed runtime executable ownership or permissions changed')
 # starttime distinguishes PID reuse without reading or printing process argv.
 started=(proc/'stat').read_text().rsplit(') ',1)[1].split()[19]
 return pid,str(executable),started,info.st_dev,info.st_ino

try:
 before=runtime_identity()
 pid,cli=before[:2]
 service_env=dict(item.split('=',1) for item in (pathlib.Path('/proc')/str(pid)/'environ').read_bytes().decode().split('\0') if '=' in item)
 sensitive=sorted({value for key,value in service_env.items() if value and is_credential(key,value)},key=len,reverse=True)
 if service_env.get('DATABASE_URL'):
  raise RuntimeError('Runtime DATABASE_URL authority requires an explicit audit')
 fields=['HONE_POSTGRES_HOST','HONE_POSTGRES_PORT','HONE_POSTGRES_USER','HONE_POSTGRES_DATABASE']
 if any(not service_env.get(key) for key in fields+['HONE_POSTGRES_PASSWORD']):
  raise RuntimeError('Production PostgreSQL environment is incomplete')
 identity=hashlib.sha256('|'.join(service_env[key] for key in fields).encode()).hexdigest()
 if identity != request['expected_pg_identity_sha256']:
  raise RuntimeError('Production database identity changed; audit before reuse')
 command=request['command']
 if len(command)<2 or command[0]!='cloud' or command[1] not in ('community-inspect','community-publish'):
  raise RuntimeError('Unsupported remote community operation')
 # Start from a minimal environment, never the SSH caller's cloud settings.
 env={'PATH':'/usr/local/bin:/usr/bin:/bin','LANG':'C.UTF-8','TZ':'Asia/Shanghai'}
 env.update({key:value for key,value in service_env.items() if key.startswith(('HONE_POSTGRES_','HONE_OSS_')) or key in ('HONE_CLOUD_MODE','HONE_CLOUD_STRICT_NO_LOCAL_STORAGE')})
 env['HONE_CLOUD_MODE']='cloud'
 with tempfile.TemporaryDirectory(prefix='hone-remote-community-') as working:
  config=pathlib.Path(working)/'config.yaml'
  config.write_text('timezone: Asia/Shanghai\ncloud:\n  mode: cloud\n')
  env['HONE_CONFIG_PATH']=str(config)
  env['HONE_USER_CONFIG_PATH']=str(config)
  if runtime_identity()!=before:
   raise RuntimeError('Managed runtime changed during preparation; retry after cutover')
  print(json.dumps({'authority':'managed-production','pg_identity_sha256':identity,'execution':'remote-managed-runtime','runtime_cli':cli,'credentials':'remote-memory-only'}),file=sys.stderr,flush=True)
  result=subprocess.run([cli,*command],env=env,cwd=working,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  sys.stdout.buffer.write(redact(result.stdout))
  sys.stderr.buffer.write(redact(result.stderr))
  status=result.returncode
except RuntimeError as error:
 sys.stderr.buffer.write(redact(('Remote production community operation stopped: '+str(error)+'\n').encode()))
 status=1
except (OSError,ValueError,KeyError,IndexError,subprocess.SubprocessError):
 # No traceback or subprocess environment/output may expose service credentials.
 sys.stderr.write('Remote production community operation stopped: runtime inspection or execution failed\n')
 status=1
raise SystemExit(status)
"""


def build_remote_script(command, expected_identity):
    validate_remote_command(command)
    payload = json.dumps({'command': command,
                          'expected_pg_identity_sha256': expected_identity})
    # The payload is Python string data sent on stdin, never shell source.
    return 'REQUEST_JSON = ' + repr(payload) + '\n' + REMOTE_RUNNER


def run_remote(base, command, expected_identity):
    script = build_remote_script(command, expected_identity)
    # Publisher apply may take time for full object hashing. Do not impose an
    # artificial short timeout or download the objects back through IAP.
    return subprocess.run(base + ['--command', 'sudo python3 -'],
                          input=script, text=True).returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--operator-config', type=Path,
                        default=REPO_ROOT / 'data/community-imports/production-operator.json',
                        help='Ignored JSON with reviewed host and PostgreSQL identity (no credentials)')
    parser.add_argument('--cli', type=Path,
                        help='Exact local hone-cli binary; defaults to target/debug/hone-cli')
    parser.add_argument('--remote', action='store_true',
                        help='Run inspect/publish on the active GCP runtime; no --cli or local manifest')
    parser.add_argument('command', nargs=argparse.REMAINDER,
                        help='cloud community-inspect|community-append|community-assets|community-publish [arguments]')
    args = parser.parse_args()
    command = args.command
    if len(command) < 2 or command[0] != 'cloud' or command[1] not in COMMUNITY_COMMANDS:
        parser.error('Select a supported cloud community operation')
    if args.remote:
        if args.cli is not None:
            parser.error('--remote cannot be combined with --cli')
        validate_remote_command(command)
        cli = None
    else:
        cli = (args.cli or REPO_ROOT / 'target/debug/hone-cli').resolve(strict=True)
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
    if args.remote:
        return run_remote(base, command, expected_identity)
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
            # Knowledge Planet's captured minute timestamps are in Shanghai;
            # importing from a machine set to UTC must not shift the timeline.
            config.write_text('timezone: Asia/Shanghai\ncloud:\n  mode: cloud\n')
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
