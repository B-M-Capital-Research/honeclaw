# Community freshness, assets and latency recovery

- title: Community freshness, assets and latency recovery
- status: `blocked`
- created_at: `2026-09-05`
- updated_at: `2026-09-05`
- owner: `Codex`
- related_files:
  - `bins/hone-cli/src/cloud.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `crates/hone-web-api/src/routes/public_community.rs`
  - `scripts/community_production.py`
  - `packages/app/src/pages/public-community.tsx`
  - `packages/app/src/lib/public-community-timeline.ts`
  - `packages/app/src/components/community-pdf-preview.tsx`
  - `packages/app/community-pdf-assets.ts`
  - `tests/regression/ci/test_community_production_wrapper.sh`
- related_docs:
  - `../current-plans/public-community-edge-production-rollout.md`
  - `../runbooks/community-insights-daily-sync.md`
  - `../runbooks/backend-deployment.md`
  - `2026-08-22-community-insights-refresh.md`
- related_prs: recovery commits `e0278ed1c07d689d7f89d7472d319e4489bbe415`, `719b38c5a187ca980afe78289606c729c7d3b082`; deployed backend descendant `505cf737170e8a80715d41c75fc05d794ce5c7c8`

## Summary

The September 5 production audit found two different community archives: the managed API had 718 topics ending July 31, while a local automation had updated a different PostgreSQL database and shared private R2 projection. Restored uncommitted August capture/append tooling, reviewed its identity and transaction boundaries, and bound future community commands to the managed production environment through a fingerprint-checked IAP wrapper.

A contiguous source recovery manifest added 151 topics and 194 resource metadata rows to the live database, reaching 869 topics and source head `2026-09-04 00:51` (`content_id=976`). Recovered historical source captures are evidence, not proof that the August task had updated production. Its handoff now carries an explicit correction.

## What Changed

- `community-inspect --anchor-only` emits exact machine-generated anchors. Append validates the original anchor on every replay, verifies existing stable identities, preserves same-minute source order and accepts legitimate file identity completion by backfill.
- Production wrapper keeps credentials in memory, pins the database identity and excludes the repository dotenv. The ignored operator configuration stores only host/identity metadata.
- Publisher metadata preflight uses 16 concurrent reads, preserving ordered writes, immutable-key conflict refusal, apply-time byte/SHA checks and latest-pointer-last semantics.
- Legacy HEAD and matching conditional GET use object metadata/size verification without downloading entire files. Full GET still verifies SHA-256.
- Frontend overlaps grant/state requests, rejects stale edge heads by equality with canonical IDs, refreshes on visible intervals/focus/online, merges pagination in source order, catches body-stream failures, cancels closed previews and reuses PDF bytes for download.
- Preserve the deployed Worker code and managed-origin fallback; repository July rollout configuration is behind the live deployment. Do not deploy it over production.

## Verification

- Import: first production dry-run 151 inserts; apply inserted 151 in one transaction, source newest September 4. Replay passed: all 151 already present, zero inserts. Post-append v1 publisher preflight is conflict-free; final asset/publication evidence follows below.
- Stored-object audit: all 769 existing stored objects exist with matching size; sampled file/image GETs pass full SHA. R2 HEAD p50 140 ms / p95 222 ms from the managed host. These figures are origin object probes, not browser page-load timings.
- Append: real PostgreSQL transaction regressions 5/5; CLI community tests 22/22. Legacy resource route tests 15/15, including HEAD/304/object missing/size/full-byte integrity boundaries.
- Frontend: 563 unit tests, typecheck and two Chromium community E2E scenarios passed. Worker typecheck and 45/45 tests passed. Public production-mode build passed.
- Workspace check passed. Full workspace tests with live isolated PostgreSQL: 2824 passed, 113 ignored, three unrelated failures in existing agent routing and the soul prompt contract. No affected community regression failed.
- CI-safe suite: 21/22 scripts passed. `finance_automation_contracts` has nine pre-existing checks failing identically when its script and 45 inputs are reconstructed from HEAD. Initial PG availability failures were rerun successfully after restoring the database.
- Ignored operational evidence: `data/community-imports/2026-09-05/`, including source/append manifests, production object audit, CLI reports and workspace log. Do not publish source bodies or credentials with the handoff.

## Risks / Follow-ups

- Normal source PDF download is blocked in Chrome at `files.zsxq.com` with `ERR_BLOCKED_BY_CLIENT`. The user has been asked to resolve the browser block. Protected/inaccessible files remain metadata-only; do not claim they are repaired or substitute invented content.
- Historical recovered images are rendered variants; newly captured originals are recorded separately with exact size and SHA before backfill. Production resource IDs must come from the production append report, never the old local database.
- The core/runtime rollout, managed environment repair, publication, image browser acceptance and PDF Web rollout are complete. Only inaccessible source attachments and foreground user-Chrome PDF acceptance remain open; no further implementation defect is currently reproduced.
- Automatic sync was updated through the app on September 5 and read back: existing task `bamang-community-daily`, now named `巴芒科技洞察同步`, every two hours, ACTIVE; model `gpt-5.5` / medium and existing project target preserved. The prompt is exactly the reviewed production-wrapper workflow and stays quiet without a meaningful change. Local app/browser/GCP availability remains required; a saved schedule is not proof of its first future run.
- Existing unrelated data-center/frontend edits in the shared checkout belong to another task and must remain outside the community recovery commit.

## Next Entry Point

Resume after normal authorized source downloads become available and the user unlocks the Mac for foreground Chrome PDF acceptance. Reuse the existing production wrapper and resource mapping; do not repeat append/apply when its replay and publisher are already no-op. The deployed binary/Web pair is `505cf737` / `719b38c5`. Do not interpret metadata-only topics as repaired attachments.

## September 5 deployment progress

- Pushed exact recovery `e0278ed1`; Rust pre-push formatting and gitleaks passed (no secrets detected).
- Pages production deployment `16c38377-47f0-4ee3-8414-1432629a2acf` succeeded. Served entry changed from `index-BMWMF4UX.js` to `index-CYYxI3Dc.js`; its SHA-256 is `243470ab98f6790d4cd28dd0df65eafed370487a98783e4f333ef5defe9819bf`, matching a clean detached exact-revision build. Its discovery-on community Chromium E2E passed 2/2.
- Source capture supplied 18 original PNG images (13,732,860 bytes), combined with 57 verified historical image renditions into 75 production image backfills. No image key in the new delta remains without captured bytes. A separate verified historical 8-page PDF maps to production resource 1122; larger already-stored historical images were preserved instead of replacing them with smaller old renditions.
- Actual origin access-log aggregation before recovery: 3007 successful feed requests had p50 956 ms / p95 2981 ms; 77 resource requests had p50 5782 ms / p95 11811 ms. These describe mixed historical requests and are not controlled before/after transfer benchmarks.

- Image backfill apply/replay completed: 75 uploaded and promoted, then zero uploads/updates. The recovered historical PDF also applied/replayed with zero remaining changes. Total newly repaired resources: 76. Stored resources now 845; 273 remain unavailable/protected, including 119 new file references.
- Real public-user browser login completed through the user-provided SMS flow; confirmed latest September 4 topic and a decoded 1468×1678 original in the image lightbox. No login credential is retained in operational artifacts.
- GHCR Runtime Image run `33939375270` succeeded for `e0278ed1`, digest `sha256:8ab26dd7f95663de06b6504edc5e6298526787ee5d272e73208a0232fa7c2df9`; exact immutable bundle staged with all payload hashes verified. This remains a verified fallback; the actual joint cutover uses its descendant below.

## September 5 publication and joint runtime acceptance

- Production publication applied successfully: 869 contents, 1118 resources, 845 edge-backed resources, 273 legacy/unavailable references; 44 pages and 890 planned objects, 171 objects written. Latest became `976`, and every published resource passed full-byte SHA-256 verification. Report: ignored `production-publish-apply.json`.
- The source was reopened after publication: newest visible topic remained `2026-09-04 00:51`. This is the latest verified source state, not a promise that a later source post cannot arrive.
- Fixed the managed service environment: the existing matching edge secret was absent from `/etc/hone/runtime.env`; added it without printing/persisting it in the repository. The obsolete public-Web directory was replaced with `/opt/hone/public-web/current`. Root-only environment backups and independently hash-verified immutable public builds preserve rollback.
- Coordinated one managed restart with the 3D task: actual backend `505cf737170e8a80715d41c75fc05d794ce5c7c8`, GHCR digest `sha256:7fc791b9a3fafb14bc753b3e861d51ac9f7639a16d3f2eaab480320daa9d0c57`. Both pre-switch idle samples were zero. PostgreSQL, OSS and cloud authority passed; local durable dependencies remained zero. No ad-hoc service was launched.
- At cutover, the public fallback was also `505cf737`; loopback `/chat` entry `index-DABemvNv.js` SHA-256 `990dfa0bf96bcab09e8b25675005bff2fd37a4cb212e027cf89ad8e7a7476dc9` matched the exact build. The paired fallback remains the previous `a2d76ea44d04ef307740e7d599d360f65dd3b6bc` binary and verified `e0278ed1` public Web. Preserve the repaired runtime environment during rollback; details are linked from the 3D handoff.
- Authenticated edge canary from GCP passed: feed 200 / 168 ms / 44.6 KB, latest `976` equals production PG; image 1301 HEAD 392 ms, GET 1.71 s for 1.30 MB and 304 603 ms; PDF 1122 HEAD 615 ms, GET 781 ms for 4.27 MB and 304 172 ms. Both GET bodies matched exact size and SHA. These are individual GCP-to-public-edge measurements, not user-device page-load percentiles or a controlled comparison with historical logs. Report: ignored `production-edge-canary-505-complete-report.jsonl`.
- A fresh logged-in Chrome community tab displayed the latest source topics and resource URLs under `/_community/v1/`. Opening resource 1301 decoded its 1468×1678 pixels successfully. The recovered PDF downloaded with exact 4,272,631-byte size and SHA `b507349ac48cf60c5f4f0a0fb2138ed61c3d6eb3ff081e8ec1f7bd9af3771987`, but Chrome blocked the native iframe viewer. This additional observed bug prompted the canvas preview follow-up; iframe presence alone is not rendering proof.
- The saved two-hour automation now uses `--remote` for inspect/publish, keeping object verification on the managed GCP runtime; append/assets retain the reviewed local manifest path. App-tool update and TOML readback matched the reviewed prompt after trailing-newline normalization, with model/reasoning/project/cadence preserved. Its first future execution remains unobserved.
- Final remote production inspect passed in 7.277 seconds and returned source head `976`. Publisher dry-run on the same verified `505cf737` runtime passed in 32.282 seconds: `ok=true`, `no_op=true`, `would_write=0`, `written=0`, `conflicts=[]`, and all 890 planned objects already existed. Its verification mode is `head_exists_only`; the earlier apply is the separate full-byte SHA proof. Reports: ignored `remote-inspect-acceptance.json` / `remote-publish-acceptance.json`.
- Remote-wrapper acceptance exposed and fixed an output-redaction bug: the non-secret `HONE_POSTGRES_NO_PROXY=true` value had been treated as a credential and corrupted JSON booleans. Credential-key suffixes and authenticated proxy URLs are now distinguished from routing/feature/budget settings without exempting short real secrets. The retained CI-safe wrapper script passes four tests covering argument rejection, exact production identity, runtime changes during preparation, environment isolation and output redaction. No production data was changed during these dry-runs.

## PDF display follow-up verification

- Replaced the observed blocked iframe with lazy PDF.js `6.3.289` display code and a same-origin worker, versioned fonts/CMaps/ICC/wasm assets. The component renders one bounded canvas page, provides navigation/zoom and accessible text, and destroys its worker on close. Preview and download join the same in-flight/completed Blob; this does not make protected source files available.
- Final frontend typecheck and 567 tests passed. Four community Chromium E2E scenarios passed against the discovery-enabled public production build, including actual two-page Chinese pixels/text, changed second-page color/content, zoom, mobile layout, one shared GET, worker teardown, malformed-PDF download fallback and cancellation/reopen without page errors. Desktop/mobile screenshots were visually reviewed; this supersedes the earlier iframe-presence test.
- The static-assets plugin has regression coverage for MIME types, including JavaScript codec fallbacks, and exact same-origin versioned paths. PDF.js is pinned to the official [6.3.289 release](https://github.com/mozilla/pdf.js/releases/tag/v6.3.289), containing [GHSA-hq66-cqwq-w95j's fix](https://github.com/mozilla/pdf.js/security/advisories/GHSA-hq66-cqwq-w95j); display code never instantiates the separate scripting manager or XFA.
- Screenshots/build/E2E logs are retained in the ignored task evidence directory at final delivery. Exact-revision production deployment and real-user PDF acceptance are recorded in the subsequent phase below.

## Final deployed Web and remaining browser acceptance

- Public Web `719b38c5a187ca980afe78289606c729c7d3b082` built from a clean detached checkout with discovery enabled: 671 files, exact production-bundle E2E 4/4. Pages deployment `757935e8-ee21-42e1-8057-baef20662409` succeeded. The real Chrome page loads `/assets/index-Boc1MGk1.js`; its reviewed build SHA is `c61a10ba8fb5246e7c60e2c6ca803363d4382585b03a8f440770515b32b6c5c9`.
- Public fallback independently verified the full manifest and every file, staged immutably, checked the previous link was `505cf737`, then atomically activated `719b38c5`. Live loopback `/chat` and entry JS bytes matched exact hashes; managed PID/starttime remained unchanged. No environment edit, binary switch or restart occurred. Archive SHA `c788db2d7c2b6ee966741fbd7bfa9415c0129c977df4e0435a75084867251919`; manifest SHA `eea69201606945f2bfb00c0454b1064764d10ee418c084b36577d08215b6ea2e`. Report/manifest/summary: ignored `public-fallback-719b38c5-*`. For a frontend-only rollback, retain backend `505cf737` and restore the fully verified prior `505cf737` public release through the same expected-current procedure.
- A separate unauthenticated Python probe from this workstation received Cloudflare `403 / 1010` for public asset requests. It is not a successful public-byte hash check; no WAF setting, user agent or authentication was changed to bypass it. Native Pages deployment status, clean artifact provenance, managed fallback HTTP hashes and the actual logged-in Chrome entry are the available deployment evidence.
- The new real Chrome PDF flow downloaded the exact 4,272,631-byte Google PDF again with SHA `b507349ac48cf60c5f4f0a0fb2138ed61c3d6eb3ff081e8ec1f7bd9af3771987`, recognized all eight pages and displayed the new controls. The screen-locked user browser still did not complete its display canvas; foreground pixel/turn-page acceptance is explicitly unverified and the user has been asked to unlock. Browser visibility reported `visible`, and captured warnings/errors were empty.
- Independent Chromium using that exact file and the exact production build rendered the first page in 201 ms (local mocked transfer, not network latency), with 79,055 nonblank pixels, eight pages, one resize event and no console/page errors. PDF.js display rendering schedules its continuation through `window.requestAnimationFrame`; a temporary test-only pause of rAF reproduces the same pending-canvas symptom while `document.visibilityState` stays visible, and restoring frames completes rendering. This supports the lock/scheduling explanation but does not prove the actual cause until the real browser is unlocked. No production change to print intent, browser security, or OS settings was made.
- Remaining attachment total is 273 protected/unavailable references, including 119 from the recovered new interval. Source PDF download remained `ERR_BLOCKED_BY_CLIENT`. The task remains `blocked` for these external conditions, with deployed work retained in the archive index and the active plan kept for continuation.
