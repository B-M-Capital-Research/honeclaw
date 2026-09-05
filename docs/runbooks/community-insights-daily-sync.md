# Community Insights Source Sync

- title: Community Insights Source Sync
- status: `active`
- created_at: `2026-08-22`
- updated_at: `2026-09-05`
- owner: `Codex / production operator`
- related_files:
  - `scripts/community_production.py`
  - `bins/hone-cli/src/cloud.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `workers/public-community-edge/wrangler.jsonc`
  - `data/community-imports/production-operator.json` (ignored, host identity only)
- related_docs:
  - `docs/runbooks/backend-deployment.md`
  - `docs/handoffs/2026-08-22-community-insights-refresh.md` (historical evidence, not a live authority)
- verification: managed-production wrapper and exact-anchor dry-run; remote argument/environment/redaction fixture; append transaction regression; publisher read-only concurrency regression; remote runtime smoke after deployment; saved automation readback after prompt or cadence changes
- risks: local runner or browser unavailable; source access restrictions; protected originals; changed production identity; immutable publication conflicts

## Purpose and schedule

Synchronize the user-authorized `巴芒科技` Knowledge Planet at `https://wx.zsxq.com/group/51115212285814` into the PostgreSQL archive used by the managed production Web service. The source is `zsxq` and the external ID is `51115212285814`. R2 feed pages and resource descriptors are derived from that same production database.

The cadence is **every two hours**, using the existing local scheduled task `bamang-community-daily`, the existing project target `/Users/fengming2/Desktop/honeclaw`, and its existing model/reasoning settings. The identifier and filename retain `daily` for continuity; they do not specify the cadence. Save and read back the existing task through the app's automation tool when changing the schedule or prompt. Do not create a duplicate task or hand-edit its runtime TOML. The September 5 update was saved and read back successfully: name `巴芒科技洞察同步`, ACTIVE, every two hours, with the prior model/reasoning/project preserved. The first future run still needs observation.

The local computer, app, project, GCP access, and authorized source browser session must remain available when a run needs them. The app's saved task is the scheduling authority. See the [official scheduled-task documentation](https://learn.chatgpt.com/docs/automations?surface=app) for local execution and task management.

## Production authority: mandatory entry point

All four community operations use the same wrapper. Inspect and publish execute on GCP; append and asset upload execute locally because their reviewed manifests and captured files are local:

```bash
python3 scripts/community_production.py --remote cloud community-inspect --anchor-only
python3 scripts/community_production.py --remote cloud community-inspect --limit 10
python3 scripts/community_production.py cloud community-append --manifest /absolute/path/append.json
python3 scripts/community_production.py cloud community-assets --manifest /absolute/path/assets.json
python3 scripts/community_production.py --remote cloud community-publish --feed-prefix CURRENT_FEED_PREFIX --asset-prefix CURRENT_ASSET_PREFIX
```

The prefix placeholders above mean the exact currently deployed prefixes verified as described below; they are not literal arguments. In local mode, `--cli /absolute/path/hone-cli` before `cloud` selects an operator-verified binary; the default is `target/debug/hone-cli`. Do not combine `--remote` with `--cli`, a manifest, or a config override. Remote mode only accepts `cloud community-inspect` and `cloud community-publish` and their operation-specific flags. `--apply` is still explicit for publisher writes; remote inspect never writes.

Both modes check the reviewed PG identity and use an isolated working directory and minimal configuration with `timezone: Asia/Shanghai`. Local mode reads only PG/OSS settings from the managed service through GCP IAP, opens a temporary loopback PG tunnel, removes inherited alternate PG/OSS settings, and keeps credentials in process memory. It never relies on the repository `.env`.

Remote mode obtains the active `hone-web.service` MainPID and resolves `/proc/<pid>/exe` to its root-owned executable at `/opt/hone/releases/<40-character-revision>-ghcr-runtime/bin/hone-cli`. It checks the process identity again before execution and never selects the mutable `current` symlink or uploads a local binary. It reads PG/OSS settings only inside the GCP process, verifies the same pinned PG fingerprint there, and supplies a minimal child environment. Neither credentials nor resource object bytes pass back through IAP; only the redacted CLI result and non-secret execution identity return. Record the returned runtime revision with the run evidence.

Run `bash tests/regression/ci/test_community_production_wrapper.sh` when changing this wrapper. Its account-free fixture executes the actual generated remote runner and verifies argument restrictions, wrong-authority and runtime-change rejection, isolated environment/timezone, and credential redaction. In particular, `NO_PROXY=true`, token budgets, and enabled flags must not corrupt JSON booleans; real credentials remain redacted even when short. After a runtime deployment, separately verify the real remote anchor and a publisher dry-run against the served prefixes. Dry-run evidence covers metadata/existence checks, not the full byte verification performed by apply.

`data/community-imports/production-operator.json` contains only `project`, `instance`, `zone`, and `expected_pg_identity_sha256`. It is ignored and owner-only; it must contain no credential. Missing configuration, a different identity, a managed `DATABASE_URL` requiring review, or failed IAP access stops production operations. Do not edit the fingerprint to make a failed check pass, switch accounts, open an alternate tunnel, or substitute another database.

The September 5 investigation found the old local CLI's `.env` selected loopback port `55432`, while the managed GCP service used a different PostgreSQL authority. August append results and the shared R2 projection therefore did not prove that the live Web archive was current. Never use a local manifest's old numeric IDs, a former success report, or R2 `latest.json` as the live append anchor.

The active managed runtime must contain `community-inspect --anchor-only` and `community-publish`. If it lacks those commands, report a runtime deployment requirement; do not silently switch inspect/publish back to the slower local route. For local append/assets, use a reviewed binary with the current importer safety checks. If that binary is absent or lacks the options, build `cargo build -p hone-cli` only when the relevant CLI/core sources have no unrelated unreviewed changes. Otherwise use a maintainer-recorded verified binary or report the missing local CLI. Never fall back to `community-contents`: it reconciles a full historical timeline by positions and is unsuitable for incremental sync.

## Capture and exact identity

1. Check the worktree and preserve unrelated changes. Read this runbook, `AGENTS.md`, and the deployment runbook's community section. Historical handoffs explain earlier operations; their absence is not a reason to discard the current verified workflow.
2. Create a distinct ignored run directory, such as `data/community-imports/2026-09-05/143000/`, so two-hour runs do not overwrite each other's evidence. Read the latest prior run result for pending resource/publication work, using it only as a recovery hint. Save the wrapper's `--remote cloud community-inspect --anchor-only` JSON unchanged. It supplies `content_id`, `published_at_raw`, the exact UTF-8 `body_sha256`, and ordered `resource_names`; do not calculate the anchor in prose or reconstruct it from a previous run. Use a separate `--remote cloud community-inspect --limit 10` response if the source comparison needs readable text/resources, and keep it private in the run directory.
3. Open the fixed source with the user's existing authorized browser session. Confirm the visible group and account context. Read the normal newest-first timeline, accounting for pinned cards. Open details and activate normal `展开全部` controls wherever necessary. Capture every topic through the exact anchor, including file-only and image-only topics. Keep the complete visible body and original resource order; do not replace a collapsed body with a summary. Re-read expanded cards before finalizing the manifest to ensure navigation or expansion has not changed the captured text.
4. Match the anchor using its time, complete text/hash, and ordered resource names; compare stable image keys where needed. A minute alone is insufficient: the source can contain distinct posts in the same minute. Preserve their visible order. If continuity or a complete body cannot be established, do not append a guessed or partial slice. This does not prevent independent recovery of an already committed production publication.

When a real stable topic ID is exposed through normal source navigation it can identify a new topic. When it is unavailable, use the existing archive's deterministic identity convention over the **complete** observed topic:

```javascript
const canonical = JSON.stringify([
  published_at_raw,
  author_name,
  body_text,
  ordered_file_display_names,
  ordered_image_source_keys,
]);
const source_item_id = "topic_" + sha256Utf8(canonical).slice(0, 48);
```

The SHA is lowercase hexadecimal. Preserve Unicode, exact text, whitespace, file names, and order; do not translate, summarize, sort, or trim the body. `ordered_image_source_keys` contains the stable keys observed in rendered image paths, with no temporary host query/signature. This is a content-derived archive ID, not a claim that the source supplied a native topic ID. The convention was verified against all 164 records in the retained August 22 manifest. Do not block solely because a list card has no clickable native topic ID. Conversely, genuinely indistinguishable duplicate topics need source-side identity clarification; do not invent suffixes or silently deduplicate to pass validation.

Build UTF-8 JSON with `anchor` and newest-first `items`. Each item has consecutive `source_delta_index` starting at zero, `source_item_id`, `author_name`, `published_at_raw` (`YYYY-MM-DD HH:MM`), complete `body_text`, and ordered `resources`. A resource has consecutive `ordinal`, `resource_kind` (`file` or `image`), `display_name`, and `source_resource_id`. Files may have a null ID when the source does not expose one; images require an observed stable key. Retain the established files-then-images ordering used by the source cards and historical manifests, and preserve order within each group. Do not retain source cookies, authorization headers, signed download URLs, or temporary resource URLs.

## Append, resource recovery, and replay

Run `community-append` through the wrapper in local mode, without `--remote` or `--apply`. New data may be applied only when the command succeeds, the exact anchor matches, and every action is `would_insert` or `already_present`. Apply the same manifest, then replay it without `--apply`. Completion requires `anchor_matched=true`, all actions `already_present`, `existing` equal to the manifest item count, and `would_insert=0`.

The importer verifies the original anchor even on an all-existing replay. When anything is missing, the anchor must still be the current production head. Existing IDs must match the stored author, time, body, and ordered resource identity, and a source ID already used by a different import key is a conflict. Inserts allocate IDs oldest-first so the public timestamp/ID descending order preserves same-minute source order. An ID conflict or stale head is resolved by re-reading production and the source; never edit an old anchor or reorder source items merely to force acceptance.

Resource capture is independent of text freshness. Use normal source image viewers or download controls to obtain legitimately available bytes. Validate the actual file size, SHA-256, and MIME/magic; image dimensions help distinguish an original from a rendered thumbnail. A displayed thumbnail can be archived as that rendition, but cannot be labeled an original. Do not substitute a screenshot for the original file.

Map captures to **production** resource IDs returned by the successful append/replay or live inspect, using the topic identity plus ordered resource identity. For images, verify the stable image key; for files, use the actual topic and attachment identity rather than a filename alone. Never reuse a numeric resource ID from the former local database. Apply `community-assets` through the production wrapper in local mode only after its dry-run is conflict-free; re-run it to confirm no remaining upload/update. The importer permits a previously unknown file source ID to be completed by verified backfill without invalidating the original append replay.

A protected/app-only attachment, blocked download, or unavailable original does not invalidate complete visible topic text or other accessible images. Keep those resources metadata-only or in their already established protected state, record the reason, and continue archiving the other legitimate resources. Do not repeatedly click a blocked item or bypass a browser/source restriction. Preserve prior verified bytes if an attempted replacement is unavailable.

With no new topics, do not create an empty append manifest. Verified resources left pending by an earlier run can still be recovered using current production IDs. A prior manifest proven to belong to this production authority can be replayed without `--apply` to obtain current resource mappings: require its original anchor to match and every topic to be `already_present`. If that proof fails, do not use its numeric IDs. A source login/CAPTCHA failure stops new source capture; it does not authorize new authentication actions. Independent publication recovery from already verified production data can still proceed, with source freshness explicitly marked unverified.

## Publication and accumulated work

Every run evaluates the production publisher, including runs with no new source topic and runs recovering a previously committed append. Source freshness, asset availability, and publication freshness are separate results. A successful append followed by a failed publisher must be recoverable next time even if the source has not changed again.

Use the currently deployed `COMMUNITY_FEED_PREFIX`, with a matching `COMMUNITY_RESOURCE_PREFIX` ending in `/resources`, as recorded in `workers/public-community-edge/wrangler.jsonc` and the deployment runbook's current rollout evidence. Check that the latest operator evidence agrees before writing. The September 5 recovery retains `community/zsxq/51115212285814/delivery/v1`, after the production append preflight found no immutable-key conflict; a later reviewed prefix migration supersedes this record. Do not infer the active prefix from a CLI default, silently return to v1, or publish into an unserved namespace. Keep the asset prefix at the deployed `COMMUNITY_ASSET_PREFIX`.

For one fixed prefix:

1. Run `python3 scripts/community_production.py --remote cloud community-publish --feed-prefix <current-feed-prefix> --asset-prefix <current-asset-prefix>` without `--apply`. Require success, `ok=true`, and `conflicts=[]`.
2. If `would_write=0` and `no_op=true`, no publication apply is needed.
3. If it reports pending writes, apply with exactly the same source, group, page size, feed prefix, and asset prefix. This is allowed even when this run appended zero topics: it repairs the production projection or pending resource promotion.
4. Repeat the same dry-run. Require `ok=true`, `no_op=true`, `would_write=0`, and `conflicts=[]` before marking publication current.

Dry-run only checks resource object existence; apply retains full eligible-object size/type/SHA verification under a repeatable-read snapshot and publication advisory lock. The September 5 recovery had 845 eligible objects: local publication spent over ten minutes reading them through the distant operator route. Remote execution moves those same checks close to production PG and R2 and avoids transferring every object to the local computer. It does not reduce the set of verified objects, substitute HEAD for SHA verification, or relax immutable object checks. Metadata checks run with bounded concurrency; large attachment verification deliberately uses lower concurrency. A quiet stdout period is not itself a failure; remote output is returned when the CLI completes. Allow a healthy publisher to finish, record elapsed time, and use bounded retry after an actual transient error. Do not skip verification because a prior run took several minutes.

Descriptors and cursor pages are immutable. If an existing key has different bytes, stop that publication; retain the canonical append and available resources. Never delete or overwrite the conflicting object, bypass the conflict, or publish from the former database. A namespace change such as v1 to v2 requires a separately reviewed publisher/Worker routing and cache transition by the production operator; the scheduled task must report the required action without changing Worker/Pages settings. Mutable `active.json` and `latest.json` are updated only through the publisher's ordering/readback contract, with `latest.json` last.

## Results, recovery, and maintenance

Write a small, secret-free `result.json` or `result.md` in each run directory. Retain the production authority fingerprint, source check time, live/source head times, inserted topic/resource counts, newly archived resource count/bytes, remaining protected or unavailable resources, append replay result, publication result/prefix, elapsed time, and any recovery entry point. Keep complete source text and private manifests out of user-facing summaries.

- `SYNCED`: the observed source delta is in production and the projection is verified current. State separately whether all captured resources or only the accessible subset have bytes; never claim all files open when some remain protected.
- `PUBLISHED` / `RESOURCES_RECOVERED`: previously committed production content or legitimate resources were repaired without requiring a new source topic. State whether source freshness was actually checked.
- `NO_CHANGES`: source and production agree, no resource recovery occurred, and publisher is already a no-op. Record locally and produce no new user-facing finding.
- `ACTION_REQUIRED`: an authentication, browser, identity, runtime, or operator decision is needed. Describe the concrete next step and actual writes already completed; do not claim “no writes” if an earlier safe phase committed.
- `BLOCKED`: a source continuity/body/identity conflict or publication conflict prevents a phase. Preserve completed safe phases and report their exact state. An unchanged known blocker is not a new finding every two hours.

Only meaningful progress, a new failure requiring attention, or a change in a known blocker warrants a concise result for the user. Do not generate periodic “still current” or repeated unchanged-blocker messages. Once the source is unavailable, do not state that production matches the latest source merely because publisher is a no-op.

The scheduled task may write only its ignored run artifacts and legitimate downloaded bytes; compiling the reviewed CLI writes build artifacts. It must not modify source, tests, repository docs, operator identity, automation definitions, deployment settings, source accounts, or production service state, and must not commit/push or send messages to third parties. Review future workflow changes in the normal repository task, then update this runbook and the existing automation prompt using the app tool. Test the revised workflow and inspect the first saved run before calling the cadence healthy.
