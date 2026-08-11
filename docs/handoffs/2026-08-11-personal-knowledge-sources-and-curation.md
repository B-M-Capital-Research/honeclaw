# Personal Knowledge Sources and Curation

- status: done_locally
- date: 2026-08-11
- scope: personal external sources, isolated community candidates, administrator promotion to HONE official research

## Result

`/research-library` is now the signed-in “我的知识源” surface and has a first-class entry under `/me`. Every user can import their own local, Knowledge Planet-exported or iMA-exported material with source, date, ticker/topic and downstream-use metadata. Knowledge Planet is accurately described as an official OAuth/Skill user-device path that produces a read-only import package; HONE does not collect browser cookies or reuse a single server-side CLI login across tenants. iMA remains explicit export/import.

The backend now has three separate trust domains: `personal`, `community_candidate` and `hone_global`. A user can submit only their own successfully parsed personal material. The copied candidate is visible only to that user and administrators and is deliberately absent from the retrieval function used by all Agents and daily products. An administrator can approve it, which copies bytes and provenance into the HONE global library, or reject it with an auditable note. Approval and rejection are explicit POST actions and normal users cannot call the review boundary.

## Verification

- `cargo test -p hone-web-api --lib --no-fail-fast`: 278 passed, 2 ignored before the final projection-only field; focused research-library rerun 3/3 passed afterward.
- `bun run test:web`: 445 passed.
- `bun run typecheck:web`: passed.
- `bun run build:web:public`: passed.
- `bash tests/regression/ci/test_research_curation_contract.sh`: passed.
- Local backend rebuilt/restarted; ports 8077/8088 healthy and local dev login remained enabled.
- Authenticated browser acceptance verified `/me` entry, connector disclosures, candidate controls, desktop layout and 390×844 layout with no horizontal overflow.

## Risks / Follow-up

- This is not cloud automatic synchronization. A true per-user Knowledge Planet connection needs an official multi-tenant OAuth client/API contract; the current official CLI Keychain design is intentionally kept on the user's device.
- Do not add iMA session scraping. Wait for an official API/Skill or keep using user exports.
- Public forum posts/comments/likes are a separate untrusted-content system and were not mixed into this research authority. The next forum slice should preserve moderation, financial-content governance, evidence/date fields and an explicit “submit for curation” action.
- Before production, migrate all three metadata scopes to PostgreSQL and bytes to object storage.

## Rollback

Remove the submit/review routes and `community_candidate` projection, then restore the previous research-library page. Existing candidate files are isolated under `storage.sessions_dir/research-library/community-candidates` and can be retained or archived without affecting personal/global retrieval.
