# P0 Experience Core Capabilities

- title: P0 Experience Core Capabilities
- status: done
- created_at: 2026-07-27
- updated_at: 2026-07-28
- owner: Codex
- related_files:
  - `crates/hone-core/src/provider_symbol.rs`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-channels/src/`
  - `bins/hone-feishu/src/`
  - `crates/hone-web-api/src/`
  - `packages/app/src/`
  - `tests/regression/`
- related_docs:
  - `docs/handoffs/2026-07-28-p0-experience-core-capabilities.md`
  - `docs/current-plans/ticker-resolution-architecture.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/bugs/README.md`
  - `docs/bugs/feishu_scheduler_watchlist_stockanalysis_abnormal_prices.md`
  - `docs/proposal/auto_p0_investment_output_safety_gate.md`
  - `docs/runbooks/backend-deployment.md`

## Goal

Turn the user-supplied workbook's nineteen experience records into a small set of enforceable product capabilities rather than nineteen phrase-, ticker-, screen-, or channel-specific patches. Preserve the established finance answer format and Agent ownership while making identity resolution, mutation confirmation, evidence completeness, numeric/unit coherence, structured rendering, and responsive information architecture reliable across user wording and channels.

## Scope

- Treat all workbook records as P0 review inputs, not as an instruction to implement every suggestion.
- Mark non-product requests such as a reusable logo asset as `no_action`.
- Do not implement a fix whose only safe form is a one-off company alias, prompt phrase, CSS coordinate, output regex, or channel-only rewrite.
- Revalidate already deployed systemic fixes for generic refusal recovery, context overflow, Feishu native tables, and authoritative reminder cancellation.
- Consolidate unresolved behavior into five capability domains:
  - canonical cross-market identity and user-confirmable ambiguity
  - session, mutation, and scheduled-task reliable completion
  - typed evidence completeness plus numeric/unit/source coherence
  - one canonical final answer with channel-native structured projection
  - responsive, theme-safe, task-oriented public workspace composition
- Produce a replacement workbook with every source record, status, capability domain, decision, implementation evidence, verification evidence, and a compact overall status summary. Do not commit user screenshots or private workbook contents to the public repository.
- Push code and repository documentation, build an immutable package from the exact tested source commit, drain active chats, restart all currently managed production services, and verify storage/channel/API/public health.

## Validation

- Workbook extraction reconciles all `19` source records and `20` embedded images without modifying the source file.
- Every implemented behavior is backed by a capability-level matrix spanning wording variants, symbols/markets, channels, session age, tool success/failure, and mobile/desktop presentation where applicable.
- Targeted module tests plus changed-file formatting.
- Full workspace check/test excluding Apple clients, Web tests, Public Community Edge typecheck/tests, and CI-safe regressions.
- Replacement workbook value/formula inspection, formula-error scan, and rendered visual review of every sheet.
- Immutable deployment manifest verification.
- Two zero-active-chat checks before restart; post-restart version/path, ports, PostgreSQL/OSS authority, local durable dependency count, source/public auth boundary, managed channel connections, and repeated active-chat checks.

## Documentation Sync

- Keep this plan indexed in `docs/current-plan.md` while active.
- Update `docs/invariants.md` and `docs/decisions.md` for durable evidence/coherence, canonical-final/projection, and mutation-completion behavior changes.
- Update `docs/repo-map.md` only if module ownership or main data flow changes.
- Update affected bug records and `docs/bugs/README.md` from the capability-level result, not by creating one bug per workbook row.
- On completion, add one reusable handoff, move this plan to `docs/archive/plans/`, remove the active index entry, and add an archive-index entry.
- Keep the source and final workbook outside the public repository because they contain user screenshots; record only a redacted checksum/count/status summary in the handoff.

## Risks / Open Questions

- A generic post-generation semantic gate would conflict with the accepted Agent-owned Interactive answer architecture and can create new refusals; prefer typed evidence admission and deterministic dimensional/coherence checks before evidence reaches the model.
- Cross-channel equality means one canonical semantic answer plus channel-native projection, not byte-identical Markdown/Feishu card payloads.
- Historic tasks and actor state must remain actor-scoped; no global cleanup is allowed to solve one user's stale reminders.
- UI recommendations that imply a product redesign need measurable task-flow benefit and responsive contracts; screenshot-specific pixel fixes are out of scope.
- The requested spreadsheet engine is unavailable in this runtime. The final workbook must therefore use the local LibreOffice engine as an explicit fallback, preserve source evidence, and pass exported-XLSX plus rendered-PDF inspection before delivery.

## Progress

- 2026-07-27: Source workbook was parsed read-only. It contains one sheet, nineteen issue rows, and twenty embedded screenshots. Apple Vision OCR and rendered-sheet inspection recovered the evidence without modifying the source.
- 2026-07-27: Initial clustering identified five capability domains. Rows for generic refusal, context overflow, raw Feishu table source, and bulk reminder cancellation correspond to already deployed systemic repairs and require regression/live verification rather than new patches. The logo request is non-product/no-action. The primary unresolved trust issue is typed numeric/source coherence: the same evidence set can contain a quote near `$920.95`, a bullish consensus, and a target near `$158`, while scheduler/direct paths have repeatedly admitted high-risk price anchors.
- 2026-07-27: Added symbol-scoped `earnings_outlook`, which combines quote, company earnings timing/estimates, quarterly analyst estimates, target consensus, ratings snapshot, and financials while preserving partial component failures and coverage. Agent routing and the typed scheduled/heartbeat evidence path both recognize the composite; the global date-window calendar is no longer used as symbol-specific outlook evidence.
- 2026-07-27: Quote evidence now publishes independent usability flags for price, change, range, and market-cap claims. Target consensus with an invalid internal range is quarantined; an extreme target/current-price ratio requires independent corroboration. This is an upstream evidence contract and does not add a final-answer judge or alter the prompt answer layout.
- 2026-07-27: Added a central read-after-write reconciliation contract for ambiguous `portfolio`, `cron_job`, and `notification_prefs` mutation failures. The write remains execute-once; only the corresponding actor-scoped read may run, followed by one tools-disabled same-Agent completion from observed state.
- 2026-07-27: Shared Web Markdown rendering now preserves accidental paired-tilde spans as inert text instead of striking whole finance paragraphs. The current calendar code already binds month labels, preview payloads, overflow presentation, and mobile rendering to selected state; old screenshot-specific calendar/style patches were not added. Anonymous production smoke reached the public site and login boundary; authenticated workspace visual inspection remains unavailable without a test actor and is not being bypassed.
- 2026-07-27: The abnormal-price bug was promoted to `P0 / In Progress`. Internal dimensional checks cover a large class of incoherent provider payloads but do not prove a provider-wide split-adjustment payload that is internally self-consistent; this limitation will remain explicit in the final workbook rather than being mislabeled solved.
- 2026-07-27: Generated the private replacement workbook outside the repository without committing user screenshots. It reconciles all `19` rows and `20` source images, contains status overview / issue status / capability matrix / original evidence sheets, passed a LibreOffice open-recalculate-export cycle, formula-error scan, and visual review of all `18` rendered PDF pages. SHA-256: `6b08ba7ff500718373d877cd7ed6e68d15ba65cccc995cd14efe7f9f3f792b0b`.
- 2026-07-27: All repository gates passed: changed-diff check, workspace `cargo check`, full workspace Rust tests excluding Apple clients, Web `303/303` plus typecheck, Public Community Edge `45/45` plus typecheck, and every CI-safe regression. Regression contract 37 now explicitly locks the new write-once / actor-scoped read-once / timeout-fail-closed reconciliation boundary.
- 2026-07-28: Commit `c2edceb7269476c39a3eb23efd25d14d4675aa93` was pushed to `main`, built into `target/deploy-c2edceb7`, and verified against a `500`-payload SHA-256 manifest. Two independent active-chat checks returned zero before both managed supervisors received SIGINT. Web and Feishu restarted from the exact package; ports `8077/8088`, repository-root supervisor working directories, authoritative cloud storage, PostgreSQL, R2, JSON `401` auth boundaries, public routes/security headers, two Feishu established connections, and repeated zero-active-chat checks passed.
- 2026-07-28: The first post-restart R2 probe exposed a host-network fault: the active Clash global proxy node rejected TLS for both the old and new binaries, while Clash's direct delay test was healthy. Switching the reversible `GLOBAL` selector to `DIRECT` restored R2 immediately; this establishes an environment incident rather than a code regression and is recorded as an operational follow-up.
