# Runbook: API Liveness Monitoring

Last updated: 2026-09-20

## Purpose and boundaries

Use `scripts/diagnose_api_liveness.py` to distinguish a responding backend from
a process that is still running but no longer serves HTTP. It collects bounded,
read-only evidence on the managed backend host. It does not restart services,
kill application processes, send chat messages, authenticate as a user, read
process environments, or query business tables. There is no automatic recovery.

The sampler requires Linux, Python 3.9+, and root access for private logs and
`/proc` inspection. It has no pip dependencies. `systemctl` and `ss` enrich
failure snapshots; unavailable diagnostics are recorded rather than treated as
proof that the backend is healthy. Root access may still be insufficient to read
some kernel stack information under the host's security policy.

The default loop runs for **24 hours**, samples every **30 seconds**, and gives
each HTTP probe a **3-second** total time budget. A detailed snapshot has an
8-second budget and can be marked `partial`. The sampler also respects its
overall duration; reaching that duration is not reported as an API failure.

## What the three probes mean

| Probe | Expected result | What it tests and what it does not |
| --- | --- | --- |
| `control`: `http://127.0.0.1:8088/api/public/auth/dev-login/config` | 200 JSON object | A read-only configuration handler without a database operation or its own normal request log. It only reports whether the feature is enabled; it does not perform a login. This helps distinguish a blocked request/logging path from broader HTTP execution failure. |
| `public`: `http://127.0.0.1:8088/api/public/auth/me` | 401 JSON object, with no Cookie header | The unauthenticated public API path. It may emit an authentication warning; it does not exercise the authenticated database lookup or prove connection-pool health. |
| `meta`: `http://127.0.0.1:8077/api/meta` | 200 JSON object, healthy PostgreSQL/object storage/authoritative-storage flags, valid 40-character Git SHA | Deployment identity and the health checks exposed by the metadata route. Only the allowed boolean flags and SHA are retained; configuration, error messages, and the rest of the body are discarded. Metadata health may be cached and does not cover every application query. |

HTTP redirects are not followed. The sampler does not use a cookie jar or proxy
environment settings. URLs must not contain credentials, query parameters, or
fragments. The loopback defaults measure the origin, not browser-to-Cloudflare
latency. `--admin-url`, `--public-url`, and `--control-url` accept alternative
full URLs. `--service` identifies the backend service to inspect, while
`--listener-ports` selects local sockets; change the latter when local ports
change. Remote HTTP targets do not make local `/proc` evidence describe the
remote machine.

## Run a bounded observation window

From a reviewed repository checkout on the managed backend host:

```bash
python3 --version
sudo install -o root -g root -m 0755 scripts/diagnose_api_liveness.py \
  /usr/local/sbin/hone-api-liveness-monitor
sudo install -d -o root -g root -m 0700 /var/log/hone-api-liveness
```

For a foreground run:

```bash
sudo python3 /usr/local/sbin/hone-api-liveness-monitor \
  --service hone-web.service \
  --interval 30 --timeout 3 --duration 86400 \
  --output-dir /var/log/hone-api-liveness
```

For an independent server-side run, a transient systemd service can apply CPU,
memory, process-count, and wall-clock limits. This example limits only the
sampler and its diagnostic helpers; it does not change `hone-web.service`:

```bash
sudo systemd-run --unit=hone-api-liveness-monitor \
  --property=Type=exec \
  --property=User=root \
  --property=UMask=0077 \
  --property=CPUQuota=10% \
  --property=MemoryMax=128M \
  --property=TasksMax=16 \
  --property=Nice=10 \
  --property=RuntimeMaxSec=25h \
  --property=TimeoutStopSec=30s \
  /usr/bin/python3 /usr/local/sbin/hone-api-liveness-monitor \
  --service hone-web.service \
  --interval 30 --timeout 3 --duration 86400 \
  --output-dir /var/log/hone-api-liveness
```

The script's 24-hour deadline ends the observation window; the 25-hour systemd
limit is an independent outer bound. This transient service does not survive a
host reboot. It has no restart policy. Check that the sampler is running and
that a fresh sample appears after starting it. Resource-limit termination is a
monitoring failure requiring attention, not evidence of an application outage.
If a previous failed transient unit still occupies the name, inspect its result
before clearing it with `systemctl reset-failed hone-api-liveness-monitor.service`.
Do not start another sampler into the same log directory: a lock rejects
concurrent writers.

## Inspect and stop

```bash
sudo systemctl show hone-api-liveness-monitor.service \
  -p ActiveState -p SubState -p Result -p ExecMainStatus
sudo journalctl -u hone-api-liveness-monitor.service --since '-10 min' \
  --no-pager -n 30
sudo tail -n 1 /var/log/hone-api-liveness/metrics.jsonl | \
  python3 -c 'import json, sys; r=json.load(sys.stdin); print(json.dumps({k:r[k] for k in ("at", "kind", "reason", "probes", "consecutive_failures", "events") if k in r}, indent=2))'
```

The last command prints the latest metrics without dumping any detailed
process snapshot. A journal without errors is insufficient: check the JSONL
timestamp as well. For the default cadence, a last sample older than **120
seconds**, a missing/unreadable output file, a `stopped` record, or an inactive
sampler means monitoring is no longer providing fresh evidence. Treat this as
a monitoring exception and inspect the cause. A planned 24-hour expiry needs an
explicit decision to end or renew observation; never interpret the old healthy
sample as continuing coverage.

To stop only the sampler:

```bash
sudo systemctl stop hone-api-liveness-monitor.service
```

A foreground run also handles Ctrl-C. SIGTERM/SIGINT request a graceful stop;
the script records `kind=stopped` with `reason=signal`. Normal expiry records
`reason=duration_elapsed`. An OOM kill or other abrupt termination may leave no
stop record, which is why timestamp and service-state checks are necessary.
These commands do not restart or stop the backend.

## Interpret failures and snapshots

Each probe has its own consecutive-failure counter. Two failed samples of the
same probe enter the failed state and add a `failure` event. A failure includes
a timeout, connection/response error, unexpected HTTP status, unhealthy storage
flag, or missing/invalid metadata SHA. A missing SHA is recorded as
`missing_or_invalid_revision`; it cannot produce a healthy or recovered sample.

Once failed, the monitor emits `recovered` only when all three probes pass. A
new valid SHA is compared with the last valid SHA and emits `revision_changed`
when different. These state changes trigger snapshots. A continuing identical
failure does not repeatedly collect detailed snapshots; it continues recording
probe results and counters. A healthy startup produces metrics without a
baseline process snapshot.

Snapshots collect:

- TCP listeners and queue lengths for the selected local ports, then their
  owning PIDs by matching socket inodes to `/proc/<pid>/fd` links. This identifies
  the actual HTTP-serving child, rather than assuming systemd `MainPID` is the
  listener. The service's `MainPID` remains supplemental context.
- Listener executable path, process status, CPU counters, file-descriptor count,
  and up to 64 threads by default. Each thread includes `comm`, `wchan`, `syscall`,
  and its available kernel stack. `syscall` contains register values and pointer
  addresses; the script never dereferences them or reads `/proc/<pid>/mem`.
  See the Linux [`proc_pid_syscall(5)` manual](https://man7.org/linux/man-pages/man5/proc_pid_syscall.5.html).
- The targets of file descriptors 1 and 2, without reading their contents.
  Where available, `ss` output is filtered to queue sizes for these Unix socket
  inodes; other socket paths and peers are not retained.
- Host load, memory, CPU counters, and pressure metrics. These are point-in-time
  evidence and require comparison or interpretation; low CPU alone does not
  rule out blocked threads.

Kernel waits can distinguish likely socket/pipe write blocking from futex waits,
but they do not identify the full Rust/Tokio user-space call stack or by
themselves establish a deadlock's cause. Check `partial`, truncation indicators,
and unavailable fields before interpreting absent evidence. Processes can exit
during capture; the before/after process stat records help identify such races.
The sampler does not run PostgreSQL commands. Any database aggregate collection
must be reviewed and executed separately without exposing application rows.

## Retention and privacy

The output directory must be owned by root and private (0700); created log files
are 0600. Unsafe symlink append targets and multiply-linked log files are
rejected. Defaults retain at most **four 8 MiB segments**:
`metrics.jsonl` and `.1` through `.3`; oldest segments are replaced. `monitor.lock`
is a separate small lock file. `--max-bytes` and `--max-files` adjust these bounds.
Individual records are also bounded; an oversized snapshot is omitted with an
explicit `snapshot_omitted` field.

Do not publish raw process snapshots: executable paths, thread names, kernel
addresses, and host details are operational evidence even though credentials,
HTTP bodies, and business records are not retained. Keep evidence on the host
or in an approved private location and put only sanitized conclusions in public
handoffs. Preserve needed evidence before starting a new window if rotation
would overwrite it.

## Notification and validation

The server-side sampler runs independently of Codex. It writes evidence; it does
not send alerts. An external monitor can inspect its service state, timestamps,
probe results, and state-change events. If a Codex task heartbeat is used for
follow-up notifications, its checks depend on the desktop app and host being
online. This is an additional observer, not a replacement for server-side
sampling or an always-on alerting service. Only notify on meaningful failures,
recovery, version changes, stopped/stale monitoring, or completion of an agreed
observation window.

Successful probes do **not** prove authenticated APIs, conversation isolation,
message delivery, database pool behavior under sustained load, or public network
performance. Those require separate targeted regressions and, where authorized,
real authenticated checks.

The CI-safe regression wrapper validates the sampler with temporary local HTTP
fixtures and files, without production credentials or root access:

```bash
bash tests/regression/ci/test_diagnose_api_liveness.sh
```

It is discovered by `bash tests/regression/run_ci.sh`. The tests cover response
privacy, deadlines, consecutive failures and recovery, missing SHA handling,
listener-child attribution, filtered logging sockets, private log rotation,
and unsafe log targets. Actual Linux permissions and kernel diagnostics still
need a read-only smoke check on the target host.
