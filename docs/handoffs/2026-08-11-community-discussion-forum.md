# HONE Community Discussion Forum

- status: done_locally
- date: 2026-08-11
- scope: authenticated member discussion, moderation and safe local attachments

## Result

The existing `/community` page now has two explicit tabs. “官方动态” preserves the existing read-only HONE archive and its object-store/edge contracts. “讨论区” is a separate member forum with posts, comments, likes, reports, author/comment deletion and administrator hide/restore. Public authors are stable SHA-derived aliases rather than phone, email or internal user identifiers.

A post may include tickers, topics, an optional original-source URL and one attachment. The backend limits files to 10 MB and accepts only PDF, UTF-8 Markdown/text, PNG, JPEG or WebP when MIME, extension and magic agree. Attachment reads require the authenticated forum visibility rule and re-check SHA-256. HTML/SVG/executable uploads are not admitted.

Forum material is never research authority. No forum route is connected to prompts, ratings, red/green signals, key-event chains, portfolio news or research-library retrieval. The UI links members to “我的知识源” when they want a document to enter the existing candidate → administrator → HONE official trust ladder. Three distinct reports auto-hide a post into `pending_review`; one actor cannot report twice or report their own post.

## Verification

- Focused forum Rust: 7/7.
- Full Web API: 286 passed, 2 ignored.
- Focused Web forum/community contracts: 10/10.
- Full Web: 451/451.
- TypeScript, Rust formatting/diff check and public production build passed.
- `tests/regression/ci/test_community_forum_research_boundary.sh` passed.
- Authenticated local browser acceptance created a post, confirmed anonymized author and normalized tags, liked and commented, then removed the acceptance runtime data. Desktop and 390×844 layouts had no horizontal overflow.

## Risks / Follow-up

- The local atomic JSON/attachment implementation is not a production multi-instance authority. Before deployment, add PostgreSQL tables, object storage, backup/retention/deletion, moderation queue audit and abuse observability.
- The first version has no following, private messages, ranking/recommendation algorithm, notifications or automatic research ingestion. Those should not be added until moderation and privacy operations are proven.
- The legacy official timeline can still be unavailable locally when the cloud community runtime is absent; the forum tab remains independently usable.

## Rollback

Remove the `/community/forum*` routes and the `CommunityForum` tab/component. Local forum data remains isolated under `community-forum` and does not affect official community or research data.
