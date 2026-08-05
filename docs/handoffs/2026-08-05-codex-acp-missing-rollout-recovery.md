# Codex ACP Missing-rollout Recovery And Caris Production Acceptance

- title: Codex ACP Missing-rollout Recovery And Caris Production Acceptance
- status: done
- created_at: 2026-08-05
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `crates/hone-channels/src/runners/acp_common/protocol.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
  - `docs/runbooks/backend-deployment.md`
- related_docs:
  - `docs/archive/plans/codex-acp-missing-rollout-recovery.md`
- related_prs: none; delivered directly through `main`

## Summary

Caris Life Sciences failed before Skill execution because authoritative HONE session metadata still referenced a Codex native thread whose rollout no longer existed in the service user's persistent Codex state. The validated adapter returned structured `error.data.details = "no rollout found for thread id <same persisted id>"`; the public UI collapsed that into the generic failure message.

The runtime now replaces a binding only for that exact, same-ID, pre-prompt proof. Every generic internal error, mismatched ID, stderr phrase, timeout, auth/permission failure, process exit, empty response and post-prompt failure remains fail-closed. The replacement ID is checkpointed before the current prompt, and no prompt is ever automatically resent.

## What Changed

- ACP protocol errors retain a bounded, redacted structured details suffix for internal diagnosis.
- Codex ACP `1.1.7` can execute `session/resume -> exact missing-rollout -> session/new -> authoritative checkpoint -> session/prompt` inside the same adapter process.
- The affected production Caris session's three ACP binding fields were backed up to owner-only `/opt/hone/session-binding-rollbacks/caris-codex-binding-20260805T0915CST.psv` with SHA-256 `09b9ad0586d8a1d11f47453bda22c4486e5da7c79765a09e9b657318a9f38b44`, then cleared without changing history, messages, uploads or actor identity.
- Before repair, authoritative storage had 154 distinct nonempty Codex bindings while current local Codex state had one thread and only one matching binding. After the targeted Caris change, 153 other bindings remained intentionally untouched; the runtime will recover one only when the adapter proves that exact bound rollout is absent.
- Commit `f819584cff2f5b386c89f0791f1488c149ad3dfe` was pushed to `main` and published as immutable runtime digest `sha256:370b61bf0a9262f25585970a61ff7f37b608e35a13f764cd6d8eca7ba4fa8721`.
- Production runs `/opt/hone/releases/f819584cff2f5b386c89f0791f1488c149ad3dfe-ghcr-runtime`. Immediate binary rollback remains `/opt/hone/releases/9d64c5967bf74a5126948c7b49f6b918128f951a-ghcr-runtime`; the Caris metadata backup above is the separate bounded data rollback artifact.

## Verification

- Focused executable-adapter tests cover exact recovery, wrong-ID rejection, stderr rejection, ordinary resume failure and checkpoint-before-prompt.
- `cargo test -p hone-channels --lib`: 757 passed, 1 ignored.
- Workspace check/test, Web 364 tests, Worker typecheck/45 tests, changed-file format checks and all CI-safe regressions passed locally.
- GitHub Actions: CI `30966104736`, Runtime Image `30966104744`, Secret Scan `30966104917`, Code Quality `30966105120`, and Release Cache Warm `30966104855` passed for `f819584c`.
- Exact GHCR bundle file list, payload SHA-256 values, embedded revision, source, profile and target were verified twice on the managed host before cutover.
- Production `/api/meta` returns exact `f819584c`, `ghcr_linux_oci`, `cloud_mode=cloud`, healthy PostgreSQL/OSS, cloud-authoritative storage and zero local durable dependencies. `hone-web.service` is active with `NRestarts=0`; active chats returned to zero.
- Real administrator flow submitted “请分析 Caris Life Sciences 的最新财报，并完成证据核验和可分享 PDF。” The run executed HONE data tools, Web evidence checks and `earnings-research`, persisted `generated_files=1`, and produced `Caris_Life_Sciences_Q1_2026_Financial_Analysis.pdf-2ccf2ec2.pdf`.
- The first PDF click showed “已开始下载”. After a zero-chat restart of the same release, restored history still contained the full Caris report and the same PDF card; a second click again showed “已开始下载”.
- Public `https://hone-claw.com/api/public/auth/me` returns JSON `401`, and `/chat` returns `200`. The already-recorded Sunny-Ngrok `origin.hone-claw.com` legacy alias still redirects to its not-found surface and was not changed by this task.

## Risks / Follow-ups

- Do not bulk-clear the remaining 153 bindings. Recovery requires the live adapter's exact same-ID structured proof; otherwise stop and diagnose.
- Staging the new 1.2GB runtime filled the 30GB system disk. Five superseded GHCR releases (`91e93b51`, `3b01aa2c`, `078b0883`, `cfb75481`, `ee250d72`) were removed after exact path/current-target checks; they are reproducible from immutable GHCR artifacts. Disk usage is now 83% with about 5GB free. Future deployments must check for at least 2GB free before cutover and keep the GHCR set to the current, immediate rollback and one secondary rollback release; separately governed legacy releases are not opportunistic prune targets.
- The affected original Caris actor received the bounded metadata repair but was not the Chrome actor used for the end-to-end administrator acceptance. Its next turn should create a fresh native binding; inspect that actor only if it reports another failure.
- No formal release or tag was created.

## Next Entry Point

For another missing-rollout incident, begin with `docs/runbooks/backend-deployment.md#Audit-Codex-ACP-Bindings-After-Rollout-State-Changes`, compare exact IDs only, and preserve the three-field owner-only backup before any repair. Use this handoff for the production revision, runtime digest, data backup, rollback points and Caris acceptance evidence.
