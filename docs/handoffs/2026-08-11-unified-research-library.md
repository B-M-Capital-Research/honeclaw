# Unified Research Library

- status: done_locally
- date: 2026-08-11
- scope: authenticated research ingestion, actor/global isolation, and three downstream evidence paths

## Result

HONE now has a signed-in `/research-library` page reachable from chat. A user can upload a daily research file, record its origin and source date, tag tickers/topics, and explicitly authorize it for chat, key-event-chain and/or portfolio-news use. Personal material is actor-isolated; server-authoritative administrators may maintain a shared HONE global library. SHA-256 deduplication, bounded extraction, provenance, safe download, update and delete are enforced by the backend.

Relevant authorized material is injected into both public chat paths as untrusted research evidence and hidden from the visible conversation history. The global Rubin/HBM event chain may admit only matching global-library material. Portfolio news may use the current actor's personal plus global items only when an exact held ticker is tagged and the source date falls inside the existing 48-hour report window. Imported material cannot itself issue a trade action or override HONE instructions.

The current local store is intentionally a single-node manifest/file implementation under `storage.sessions_dir/research-library`. It is not described as a production vector database. Multi-instance deployment requires a follow-up PostgreSQL/object-storage migration before rollout.

## External Source Path

- Knowledge Planet: use its official Web export now; an official `zsxq-skill` can become a later authorized connector.
- IMA: accept user exports now; later bind only an official Skill/API-key authorization path when its stable contract and permissions are confirmed.
- Do not scrape private/logged-in content or copy browser cookies into HONE.

## Verification

- Web API: 262 passed, 2 ignored.
- Web: 441 passed; TypeScript and public production build passed.
- Rust formatting passed.
- Authenticated browser acceptance passed on desktop and 390px mobile with no horizontal overflow.
- Multipart API smoke created and listed an actor-scoped IMA-labelled sample with the expected provenance and use permissions.

## Follow-up

Move metadata and bytes onto HONE's cloud authorities before production. Then choose one official connector at a time, beginning with the source that provides the clearest documented OAuth/API-key scope, incremental cursor and deletion semantics.
