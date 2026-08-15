# Daily Influencer Brief Dashboard

- status: done_locally
- date: 2026-08-11
- scope: authenticated Web dashboard and background worker

## Result

HONE now exposes `GET /api/public/influencer-digest` and a sixth chat-home launcher, “大V速报”. The global worker refreshes at 19:50 Beijing, reads the prior 36 hours from exact registered RSS aliases, deduplicates source URLs and stores atomic latest/history snapshots. The UI filters by author and keeps author, handle, Beijing time, original link, source/HONE analysis state, stance, horizon, fact/opinion label, topics, source-verified tickers and counterpoint.

SemiAnalysis is configured through its official feed. By explicit user instruction, Serenity/白毛 now uses the public JSON feed declared by aichainmap as a translation/aggregation layer. HONE pins that endpoint, limits time/bytes and accepts only exact X originals for `@aleabitoreddit`; the UI links both the X original and aichainmap. Jukan stays unconfigured until a lawful bridge exists. No search result, unrelated mirror or repost is used as an original source. Without a model, items remain source-only; without recent items, the report says so. A snapshot older than 36 hours is labeled stale.

## Verification

- `hone-core --lib`: 153 passed.
- `hone-web-api --lib`: 248 passed, 2 credentialed live tests ignored.
- Web: 433 passed; TypeScript and production build passed.
- Fresh local Web-only runtime: one official source succeeded, zero items existed in the 36-hour window, two X sources were explicitly unconfigured, and the next refresh was 19:50 Beijing.
- Authenticated local mobile browser: launcher, dialog, source status, empty state, disclaimer and composer layout passed.

## Follow-up

Deployment configuration may add a lawful Jukan bridge and a configured digest model. Keep Serenity endpoint/original-URL identity verification and the source-only fallback intact. The key-event chain is now implemented separately.
