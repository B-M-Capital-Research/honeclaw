---
name: event-engine-push-review
description: "Use in the honeclaw repository when Codex needs to review event-engine push quality for one or more actors: what each user/channel received, whether push timing and digest grouping were reasonable, whether messages were duplicated or useless, and whether useful events were filtered, omitted, capped, cooled down, or left undelivered."
---

# Event Engine Push Review

Review real delivery evidence from `data/events.sqlite3`. Treat this as an audit task, not a rewrite task. Prefer read-only exports and SQL first; only change code or fixtures after the user agrees on a durable failure pattern.

## Start Checklist

Write a short todo before running commands:

1. Goal: actor(s), local date/window, and review question.
2. Files/data: `data/events.sqlite3`, `data/notif_prefs/`, `data/portfolio/`, and ignored exports under `data/exports/event-engine-calibration/`.
3. Verification: read-only export/query commands; if code or baselines change, also use `event-engine-baseline-testing`.
4. Documentation: update active event-engine plan only when the review leads to a durable behavior change; one-off audit exports do not need plan updates.

Check the worktree first:

```bash
git status --short --branch
```

Do not commit private runtime DBs, ignored calibration exports, full article bodies, or user portfolio files unless the user explicitly asks and the repo policy allows it.

## Identify Actors And Window

Default timezone is `Asia/Shanghai`. For "last 24h", compute the exact UTC bounds and state them in the answer.

List active delivery actors in a local date window:

```bash
sqlite3 -header -column data/events.sqlite3 "
WITH bounds AS (
  SELECT
    CAST(strftime('%s','YYYY-MM-DD 00:00:00','-8 hours') AS INTEGER) AS start_ts,
    CAST(strftime('%s','YYYY-MM-DD 00:00:00','+1 day','-8 hours') AS INTEGER) AS end_ts
)
SELECT actor, channel, status, severity, COUNT(*) AS n
FROM delivery_log, bounds
WHERE sent_at_ts >= start_ts AND sent_at_ts < end_ts
GROUP BY actor, channel, status, severity
ORDER BY actor, channel, status, severity;
"
```

If the user says "everyone", review each actor that has `sink`, `digest`, `digest_item`, `global_digest_item`, `prefs`, or `router` rows in the window.

## Export The Evidence

Use the repository exporter for one actor/day:

```bash
python3 scripts/diagnose_event_engine_daily_pushes.py \
  --date YYYY-MM-DD \
  --actor telegram::::8039067465 \
  --format both
```

Use `--include-body` only when the body text is needed to judge render quality or duplicate digest copy. The output directory is ignored:

```text
data/exports/event-engine-calibration/
```

For quick terminal review, query sent rows directly:

```bash
sqlite3 -header -column data/events.sqlite3 "
WITH bounds AS (
  SELECT CAST(strftime('%s','now','-24 hours') AS INTEGER) AS start_ts
)
SELECT
  datetime(d.sent_at_ts,'unixepoch','localtime') AS sent_local,
  d.actor, d.channel, d.status, d.severity,
  e.title,
  substr(replace(coalesce(d.body,''), char(10), ' '), 1, 180) AS body_preview
FROM delivery_log d
LEFT JOIN events e ON e.id = d.event_id, bounds
WHERE d.sent_at_ts >= bounds.start_ts
  AND d.status IN ('sent','dryrun','quiet_held','quiet_dropped')
ORDER BY d.actor, d.sent_at_ts, d.id;
"
```

## Review Dimensions

Classify each suspect row with one of these labels:

- `useful`: timely, relevant, readable, and appropriately routed.
- `noise`: sent but low value, stale, generic, legal-ad/listicle/opinion-blog, or unrelated to the actor.
- `duplicate`: repeated same event, same headline, same social text, same earnings countdown, or same fact across immediate + digest without a reason.
- `bad_timing`: immediate should have been digest, digest should have been immediate, quiet-hours behavior was wrong, or digest slot timing was surprising.
- `should_filter`: sent but should have been filtered by prefs/source/kind/portfolio-only/cooldown.
- `should_immediate`: queued/digest/filtered but should have reached the sink immediately.
- `should_digest`: filtered/no_actor/omitted but should have appeared in digest.
- `baseline_candidate`: stable reusable classifier/router case worth adding to `tests/fixtures/event_engine/`.

### Timing Checks

Read actor prefs before judging timing:

```bash
cat data/notif_prefs/<actor-file>.json
```

Check:

- `digest_slots` and actor timezone: digest sends should cluster near configured slots.
- `quiet_hours`: high events during quiet time should be held, bundled, or dropped only according to freshness rules.
- `immediate_kinds`: configured immediate kinds should not resurrect Low-quality news.
- price alerts: respect actor thresholds, cooldowns, band gaps, and shelf life.
- macro: High macro should be immediate only inside due window; otherwise digest.
- earnings/corp-action/macro prefetch: stale calendar rows should not dominate a slot.

### Content Checks

Useful event-engine messages should answer: what happened, who it touches, why now, and where to inspect it.

Flag content problems when:

- macro rows lack actual/expected/previous values and also lack a clear publish time.
- earnings surprise rows lack metric names such as EPS.
- news rows lack concrete links when the event has a URL.
- render differs badly by channel: Telegram HTML, Discord embed, Feishu card, and iMessage/plain should each be readable in their native format.
- digest body hides high-value items behind too many low-value items.
- social/raw text is truncated into ambiguous fragments.
- source labels are misleading, missing, or too generic for audit.

### Duplicate Checks

Look for repeated event ids:

```bash
sqlite3 -header -column data/events.sqlite3 "
WITH bounds AS (
  SELECT CAST(strftime('%s','now','-24 hours') AS INTEGER) AS start_ts
)
SELECT d.actor, d.event_id, d.channel, d.status, COUNT(*) AS n,
       MIN(datetime(d.sent_at_ts,'unixepoch','localtime')) AS first_local,
       MAX(datetime(d.sent_at_ts,'unixepoch','localtime')) AS last_local,
       e.title
FROM delivery_log d
LEFT JOIN events e ON e.id = d.event_id, bounds
WHERE d.sent_at_ts >= bounds.start_ts
  AND d.status IN ('sent','dryrun')
GROUP BY d.actor, d.event_id, d.channel, d.status
HAVING COUNT(*) > 1
ORDER BY n DESC, last_local DESC;
"
```

Look for repeated titles/facts:

```bash
sqlite3 -header -column data/events.sqlite3 "
WITH bounds AS (
  SELECT CAST(strftime('%s','now','-24 hours') AS INTEGER) AS start_ts
)
SELECT d.actor, e.title, COUNT(*) AS n,
       GROUP_CONCAT(DISTINCT d.channel) AS channels,
       GROUP_CONCAT(DISTINCT d.status) AS statuses
FROM delivery_log d
JOIN events e ON e.id = d.event_id, bounds
WHERE d.sent_at_ts >= bounds.start_ts
  AND d.status IN ('sent','dryrun')
GROUP BY d.actor, lower(e.title)
HAVING COUNT(*) > 1
ORDER BY n DESC;
"
```

Manual duplicate judgment matters: immediate + later digest can be acceptable if the digest is a rollup and not a second alert; two identical sink pushes usually are not.

### Useless Push Checks

Inspect sent rows with low value signals:

```bash
sqlite3 -header -column data/events.sqlite3 "
WITH bounds AS (
  SELECT CAST(strftime('%s','now','-24 hours') AS INTEGER) AS start_ts
)
SELECT datetime(d.sent_at_ts,'unixepoch','localtime') AS sent_local,
       d.actor, d.channel, d.severity, e.source, e.title, e.summary
FROM delivery_log d
JOIN events e ON e.id = d.event_id, bounds
WHERE d.sent_at_ts >= bounds.start_ts
  AND d.status IN ('sent','dryrun')
  AND (
    e.source LIKE '%pr_wire%'
    OR e.source LIKE '%opinion%'
    OR lower(e.title) LIKE '%shareholder alert%'
    OR lower(e.title) LIKE '%class action%'
    OR lower(e.title) LIKE '%deadline%'
  )
ORDER BY d.sent_at_ts DESC;
"
```

Also scan for:

- generic analyst "hold/maintain" with no target change.
- old earnings previews or repeated T-1/T-2/T-3 countdowns.
- broad macro items with no relation to market-moving countries or indicators.
- low-quality global news pushed to a portfolio-only actor.

## Useful Filtered-Out Checks

Review omitted, queued, capped, cooled-down, filtered, failed, and `no_actor` rows. Focus first on High/Medium severity, portfolio symbols, trusted sources, and explicit mainline-aligned tickers.

```bash
sqlite3 -header -column data/events.sqlite3 "
WITH bounds AS (
  SELECT CAST(strftime('%s','now','-24 hours') AS INTEGER) AS start_ts
)
SELECT datetime(d.sent_at_ts,'unixepoch','localtime') AS logged_local,
       d.actor, d.channel, d.status, d.severity,
       e.symbols_json, e.source, e.title, e.summary, e.url
FROM delivery_log d
LEFT JOIN events e ON e.id = d.event_id, bounds
WHERE d.sent_at_ts >= bounds.start_ts
  AND (
    d.status IN ('filtered','no_actor','failed','omitted','capped','cooled_down','price_cooled_down')
    OR d.channel IN ('prefs','router','digest_item')
  )
ORDER BY d.actor, d.severity DESC, d.sent_at_ts DESC
LIMIT 200;
"
```

Load holdings when judging actor relevance:

```bash
ruby -rjson -e 'puts JSON.parse(File.read("data/portfolio/portfolio_telegram__direct__8039067465.json"))["holdings"].map { |h| h["symbol"] }.uniq.join(",")'
```

Potential misses:

- trusted-source High/Medium news for a holding was filtered as Low/noise.
- same-day hard signal should have upgraded a holding-related Low news item.
- useful global macro floor was absent from a personalized digest.
- delivery failed for channel-specific reasons after generation succeeded.
- digest cap omitted better items while lower-value items were sent.

## Output Format

Keep the final review concise and evidence-backed:

```text
Window: 2026-05-01 00:00-24:00 Asia/Shanghai
Actors reviewed: telegram::::8039067465, ...

Summary:
- sent immediate: N
- digest batches: N
- digest items sent / omitted: N / N
- filtered / failed / no_actor: N

Findings:
1. [duplicate] actor / time / event_id / title / evidence / recommendation
2. [noise] ...
3. [should_digest] ...

Representative good pushes:
- ...

Recommended follow-up:
- code fix / baseline sample / prefs change / no action
```

Include exact event ids, actor ids, local send times, titles, statuses, and URLs when available. Do not overgeneralize from one sample; mark uncertain cases as "needs more samples".

## Scheduled Daily Report

For the daily 10:00 review job, write one committed Markdown report for the previous `Asia/Shanghai` date:

```text
docs/event-review/YYYY-MM-DD.md
```

The report should include:

- exact local date window and UTC bounds;
- actors reviewed, limited to actors with at least one valid sent/dryrun push in that local date;
- per-actor summary counts for immediate pushes, digest batches, digest items sent/omitted, filtered/no_actor/failed rows;
- findings grouped by `duplicate`, `noise`, `bad_timing`, `should_filter`, `should_digest`, and `should_immediate`;
- representative good pushes so later reviewers know what "expected" looked like;
- recommended follow-up: no action, prefs tweak, code fix, or baseline candidate.

Keep committed reports concise. Do not commit raw `data/exports/event-engine-calibration/` files, private DB files, full sent bodies, or long article excerpts. If the day has no actor with valid pushes, still write a short `docs/event-review/YYYY-MM-DD.md` report saying so.

## Escalation

If the review identifies a durable bug or baseline case:

- Use `event-engine-baseline-testing` before changing event-engine code, fixtures, or LLM classifier expectations.
- Add only durable public sample fields to baseline fixtures.
- Keep raw daily exports ignored and out of commits.
