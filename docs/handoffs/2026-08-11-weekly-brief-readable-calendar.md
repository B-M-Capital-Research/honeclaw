# HONE Weekly Brief Readable Calendar

- status: done_locally
- date: 2026-08-11
- scope: standalone previous-week review and next-week event agenda

## Result

The chat home has a standalone “周度简报” between 大V速报 and 关键事件链. It now uses the same compact launcher geometry as the other primary dashboards. Daily Valuation Lab and Research Library are a two-card secondary utility row instead of two dashed full-width rows.

The modal uses three compact agenda tabs — 上周复盘, 下周关注 and 未来30天 AI — and defaults to 下周关注. It consumes structured events rather than the generated finance-calendar PNG, grouping the selected view by date with full event text, source, importance, evidence state, analysis and “提醒关注”.

The public API calculates Beijing natural-week windows at read time. It combines curated macro schedules, provider-verified earnings dates for actor holdings plus HONE's research universe, and only confirmed key-event-chain milestones from the prior week. The legacy ten-day payload is no longer serialized by `/key-event-chains`, and the event-chain dashboard no longer contains ten-day navigation or copy.

Past schedule rows deliberately say that the date has passed but the result is not present. Future rows are reminders, not predictions. Category-specific analysis explains transmission and fields to verify without inventing actual values, consensus, guidance or company outcomes. The saved report preserves those boundaries before follow-up chat.

## Local Evidence

- Report date: 2026-08-11.
- Previous week: 2026-08-03–2026-08-09, four macro schedules, zero confirmed industry changes in that exact window.
- Next week: 2026-08-17–2026-08-23, three macro events; FOMC minutes and Jackson Hole marked high importance.
- Official AI calendar: Hot Chips 2026 opens on 2026-08-23 and runs through 2026-08-25; NVIDIA FY27 Q2 results are scheduled for 2026-08-26. Both keep direct official links and `official_schedule` status. Hot Chips also appears in next week, raising that view to four items.
- Earnings scope: 53 symbols for the local actor (HONE coverage plus the actor's extra holding). The local FMP key pool is absent, so no earnings date is shown and the UI says coverage is incomplete.

## Verification

- Web API: 290 passed, 2 ignored.
- Weekly focused Rust: 4 passed.
- Weekly focused Web: 5 passed. Repository-wide Web currently reports 436 pass plus four Markdown-rendering failures/two setup errors caused by `DOMPurify.sanitize` being undefined in the local Bun test runtime; this pre-existing shared-renderer issue is outside the weekly-brief change and is not reported as green.
- TypeScript typecheck and public production build passed.
- Authenticated in-app browser acceptance verified the unified launcher stack, two-column utility row, three agenda tabs, Hot Chips and NVIDIA official dates/links, and zero console errors.

## Follow-up

- Configure the existing FMP key pool in the target environment to populate covered-company earnings dates. Do not weaken the missing-data state.
- Keep the small official AI calendar current. A new pinned event must have an exact company/organizer URL and date; approximate dates remain undated until confirmed.
- To turn prior-week macro schedules into result summaries, add a separately verified official-release ingestion path; never infer an actual value from the scheduled date.
- The macro schedule is inherited from the finance-calendar source. Extend that source through a maintained official-calendar ingestion job before dates beyond the curated horizon are needed.

## Rollback

Remove `/api/public/weekly-brief`, `WeeklyBriefDashboard` and its chat-home mount. Re-enable no ten-day UI automatically; the key-event-chain timeline remains independently functional. The research-library utility CSS can be restored independently.
