# Bug: event-engine marks legal-ad style stock news as high severity

## Summary

The news poller classified many template shareholder-alert / class-action solicitation articles as `severity=high`, creating high-priority noise from broad keyword matches.

## Observed Symptoms

- `data/events.jsonl` contained 794 `severity=high` events in the last 24h; 97 matched legal-ad style titles such as `class action`, `shareholder alert`, `deadline alert`, `investors to act`, or `lost money`.
- Representative records from `data/events.jsonl`:

```text
line 5: {"id":"news:https://www.globenewswire.com/news-release/2026/04/21/3277913/0/en/SHAREHOLDER-ALERT-Bernstein-Liebhard-LLP-Announces-A-Securities-Fraud-Class-Action-Lawsuit-Has-Been-Filed-Against-Gartner-Inc-IT.html","kind":{"type":"news_critical"},"severity":"high","symbols":["IT"],"occurred_at":"2026-04-21T08:21:00Z","title":"SHAREHOLDER ALERT Bernstein Liebhard LLP Announces A Securities Fraud Class Action Lawsuit Has Been Filed Against Gartner, Inc. (IT)","source":"fmp.stock_news:globenewswire.com"}
line 6: {"id":"news:https://www.globenewswire.com/news-release/2026/04/21/3277912/0/en/SHAREHOLDER-ALERT-Bernstein-Liebhard-LLP-Announces-A-Securities-Fraud-Class-Action-Lawsuit-Has-Been-Filed-Against-Super-Micro-Computer-Inc-SMCI.html","kind":{"type":"news_critical"},"severity":"high","symbols":["SMCI"],"occurred_at":"2026-04-21T08:21:00Z","title":"SHAREHOLDER ALERT Bernstein Liebhard LLP Announces A Securities Fraud Class Action Lawsuit Has Been Filed Against Super Micro Computer, Inc. (SMCI)","source":"fmp.stock_news:globenewswire.com"}
line 17: {"id":"news:https://www.globenewswire.com/news-release/2026/04/21/3277908/0/en/SHAREHOLDER-ALERT-Bernstein-Liebhard-LLP-Announces-A-Securities-Fraud-Class-Action-Lawsuit-Has-Been-Filed-Against-Snowflake-Inc-SNOW.html","kind":{"type":"news_critical"},"severity":"high","symbols":["SNOW"],"occurred_at":"2026-04-21T08:21:00Z","title":"SHAREHOLDER ALERT Bernstein Liebhard LLP Announces A Securities Fraud Class Action Lawsuit Has Been Filed Against Snowflake Inc. (SNOW)","source":"fmp.stock_news:globenewswire.com"}
```

- The same pattern repeated across symbols; the top repeated symbols in the legal-like high set included `NAVN` (6), `NKTR` (5), `SNOW` (5), `LU` (4), `SMCI` (3), `GEMI` (3), and `IBRX` (3).
- `data/events.sqlite3` currently shows only two historical `sink/high/sent` rows, both from the earlier dryrun mismatch, but these events remain `high` at source and can bypass digest if actor caps/cooldowns are not active or reset.

## Hypothesis / Suspected Code Path

Suspected path: `crates/hone-event-engine/src/pollers/news.rs:15` defines broad high-impact keywords, including `fraud`, `lawsuit`, and `class action`. These match law-firm marketing templates and immediately return `Severity::High`.

```rust
/// Default high-impact keywords (lowercase match). Can be overridden from config later.
const DEFAULT_CRITICAL_KEYWORDS: &[&str] = &[
    "bankruptcy",
    "bankrupt",
    "delist",
    "halt trading",
    "trading halted",
    "sec investigation",
    "sec probe",
    "sec charges",
    "sec settles",
    "recall",
    "fraud",
    "lawsuit",
    "class action",
    "short report",
    "short-seller",
    "hindenburg",
    "muddy waters",
    "guidance cut",
    "cuts guidance",
    "lowers guidance",
    "ceo resigns",
    "ceo steps down",
    "cfo resigns",
    "cfo steps down",
];
```

`crates/hone-event-engine/src/pollers/news.rs:170` performs a plain substring match over title/body, with no source, title-template, symbol-watchlist, or materiality guard:

```rust
fn classify_severity(title: &str, text: &str, keywords: &[String]) -> Severity {
    let t = title.to_lowercase();
    let body = text.to_lowercase();
    for kw in keywords {
        if t.contains(kw) || body.contains(kw) {
            return Severity::High;
        }
    }
    Severity::Low
}
```

`crates/hone-event-engine/src/router.rs:295` then treats high events as immediate-send candidates unless cap/cooldown demotes them:

```rust
match effective_sev {
    Severity::High => {
        let default_body = renderer::render_immediate(event, self.sink.format());
        let body = match self.polisher.polish(event, &default_body).await {
            Some(polished) => polished,
            None => default_body,
        };
        if let Err(e) = self.sink.send(&actor, &body).await {
            tracing::warn!("sink send failed: {e:#}");
            let _ = self.store.log_delivery(
                &event.id,
                &actor_key(&actor),
                "sink",
                sev,
                "failed",
                Some(&body),
            );
            continue;
        }
```

## Evidence Gap

- Need actor subscription snapshots to know exactly how many of the 97 high legal-like events resolved to each user.
- Need successful real-sink logs or Telegram API delivery receipts to prove whether any of these high events reached the user after the current non-dryrun sink was assembled.
- Need product criteria for whether class-action notices should always be digest-only, watchlist-only, source-weighted, or demoted when titles match law-firm solicitation templates.

## Latest巡检 Update

- 2026-04-22T02:09:55Z: `data/events.jsonl` contained 960 `severity=high` events in the last 24h; 201 matched legal-ad style patterns such as `class action`, `shareholder alert`, `deadline alert`, `investors to act`, `lost money`, `securities fraud`, or `law firm`.
- Latest representative samples are still the same law-firm template family: `data/events.jsonl:5` through `data/events.jsonl:23` are repeated `SHAREHOLDER ALERT Bernstein Liebhard LLP Announces A Securities Fraud Class Action Lawsuit Has Been Filed Against ...` records for many unrelated symbols, including `IT`, `SMCI`, `GEMI`, `NAVN`, `GO`, `IBRX`, `NKTR`, `DRVN`, `SLNO`, `BSX`, `PINS`, `PSIX`, `SNOW`, `CWH`, `ALIT`, `MNDY`, `COTY`, `STLA`, and `LU`.
- `data/events.sqlite3` shows the real high sink path is active in the current process (`price:AAOI:2026-04-22` at `2026-04-22 00:04:33` UTC), so high-severity keyword noise remains user-visible when caps/cooldowns do not demote it.
- 2026-04-21T22:08:04Z: `data/events.jsonl` contained 864 `severity=high` events in the last 24h; 159 matched legal-ad style patterns such as `class action`, `shareholder alert`, `deadline alert`, `investors to act`, `lost money`, `securities fraud`, `law offices`, or `investor deadline`.
- Latest representative samples are still the same template family, e.g. `data/events.jsonl:5` through `data/events.jsonl:19` are repeated `SHAREHOLDER ALERT Bernstein Liebhard LLP Announces A Securities Fraud Class Action Lawsuit Has Been Filed Against ...` records for `IT`, `SMCI`, `GEMI`, `NAVN`, `GO`, `IBRX`, `NKTR`, `DRVN`, `SLNO`, `BSX`, `PINS`, `PSIX`, `SNOW`, `CWH`, and `ALIT`.
- `data/events.sqlite3` showed a real high sink send at `2026-04-21 21:19:33` UTC for an AAPL news item, so the high path is no longer only dryrun. This increases the impact of high-severity noise because future keyword false positives can now reach the real Telegram sink.

## Severity

sev2. The issue can flood immediate high-priority routing with low-value legal advertising templates and consume the daily high cap, potentially delaying genuinely important alerts.

## Date Observed

2026-04-21T18:08:42Z
