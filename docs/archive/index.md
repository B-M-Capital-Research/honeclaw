# Archive Index

Last updated: 2026-09-05

## 2026-09-05

### 3D 数据中心与行业分析导航

- Status: local implementation done; production deployment in_progress by user request
- Date: 2026-09-05
- Plan: `docs/current-plans/3d-data-center-explorer.md`
- Handoff: `docs/handoffs/2026-09-05-3d-data-center-explorer.md`
- Decision: `docs/decisions.md#d-2026-09-05-01-ai-infrastructure-explorer-and-authenticated-industry-reading`
- Regressions: `packages/app/e2e/public-data-center.spec.ts`; `packages/app/src/lib/data-center-{model,geometry}.test.ts`; `packages/app/src/lib/industry-map-navigation.test.ts`; `industry_map::tests`
- Current conclusion: chat shortcut now opens a responsive six-zone 3D scene linked to eight canonical industries. All logged-in users may read the industry map; edits and internal edit identities/notes remain administrator-only. Typecheck, public build, 564 Web tests, 11 backend route tests and 8 browser tests passed.
- Next entry point: local `/data-center` preview; coordinated frontend/backend delivery and Safari/iPhone acceptance remain release-time work.


## 2026-08-23

### Inactive User Scheduled Push Cleanup

- Status: done; production live with two documented iMessage delivery gaps
- Date: 2026-08-23
- Plan: `docs/archive/plans/inactive-user-scheduled-push-cleanup-2026-08-23.md`
- Handoff: `docs/handoffs/2026-08-23-inactive-user-scheduled-push-cleanup.md`
- Decision / ADR: no new ADR; this exposes the existing `CronJobUpdate.enabled` capability and preserves existing storage/scheduler boundaries
- Related PRs / commits: direct `main` commit `011d73118dbf1d6b0cc09a793882dc796f23aa9f`; no PR, release, or tag
- Related runbooks / regressions: isolated PostgreSQL cron tool regression; Runtime Image `32612578070`; GCE exact-meta/cloud/public-auth/channel reconnect acceptance
- Current conclusion: 26 inactive direct users initially had 45 jobs paused without deletion. One notified Feishu user then actively removed 2 jobs, leaving 25 users and 43 preserved disabled jobs. Feishu 18/18 and Web 6/6 notices were acknowledged; two inactive iMessage recipients remain undelivered because no macOS channel is online. Production now accepts conversational `enabled=false/true` updates so preserved tasks can be resumed by request.
- Next entry point: restore tagged jobs only after the owning user asks; if a real iMessage channel returns, recheck current state before sending the two missed notices.

## 2026-08-22

### Market Move Same-Day News And Research Activation

- Status: done; production live
- Date: 2026-08-22
- Plan: `docs/archive/plans/market-move-date-grounding-2026-08-22.md`
- Handoff: `docs/handoffs/2026-08-22-market-move-date-grounding.md`
- Decision / ADR: no new ADR; this reuses the existing investment research flow and market-move date anchor
- Related PRs / commits: direct `main` implementation commit `e08bb4607a5a8cd559c4320db220063fc021e0b4`; no PR, release, or tag
- Related runbooks / regressions: `hone-agent` 152/152; investment response guard 136/136; WebSearch 19/19; focused three-crate compile check; immutable Runtime Image run `32544207247`; GCE cloud-authority/public-auth/channel reconnect acceptance
- Current conclusion: market-move preturn Web searches use Tavily `day/news`, preserve provider `published_date`, and enter the existing finance research loop before the first model response. Production runs exact revision `e08bb460…` from digest `sha256:314d82c…cbf96`; PostgreSQL/S3 authority and Feishu connectivity are healthy. GitHub CI's sole Rust failure is the unchanged `soul.md` character-budget baseline already present on the parent commit, not this diff.
- Next entry point: obtain explicit authorization before sending an MRVL canary from a logged-in user account; separately handle the pre-existing `soul.md` mechanical-budget failure and audit the stale `origin.hone-claw.com` ngrok alias while preserving the healthy public Worker path.

## 2026-08-17

### Macro Indicator First-Class Entity Track C

- Status: implementation done locally; PostgreSQL gate and commit blocked by the current sandbox; no push
- Date: 2026-08-17
- Plan: `docs/current-plans/macro-indicator-entity-2026-08-17.md` (overall multi-track plan remains active)
- Handoff: `docs/handoffs/2026-08-17-macro-indicator-entity-track-c.md`
- Related PRs / commits: none; linked-worktree Git metadata is read-only in the current sandbox
- Related runbooks / regressions: macro dictionary scanner; ADP label collision; cluster-quorum and
  forced-tentative mutation checks; two production scheduler prompt paths; five protected ticker tests
- Current conclusion: `hone-core` now owns an independent bilingual macro-indicator dictionary and
  scanner. Channels use it only to lower candidate confidence and remove macro spans from symbol-cluster
  quorum; candidates are never denied, explicit ticker labels win, and `SecurityIdentifierKind` is unchanged.
- Next entry point: rerun the full workspace test gate with the documented PostgreSQL service live,
  create the scoped Track C commit from a Git-writable environment, then merge it into the still-active
  multi-track plan.

## 2026-08-16

### PostgreSQL Storage API Async Conversion

- Status: done locally; no push
- Date: 2026-08-16
- Plan: `docs/archive/plans/storage-api-async-2026-08-16.md`
- Handoff: intentionally omitted per task contract
- Related PRs / commits: local module sequence `7e3f0731` through `c58e8991`; no PR,
  release, tag, or deployment
- Related runbooks / regressions: full workspace all-target tests; 93 ignored
  PostgreSQL memory tests; CI-safe regression suite; 486 Web tests; delivered-push
  cross-connection claim regression; real `events.jsonl` replay diff
- Current conclusion: runtime PostgreSQL storage operations and constructors are async through
  memory, event-engine, scheduler, tools, channels, web API and runtime entry points. The shared
  sync-to-async runtime/channel bridge was deleted, schema initialization remains once-per-process
  with retry after failure, and event replay output is unchanged.
- Next entry point: keep the three documented synchronous boundaries narrow: two test-only
  PostgreSQL cleanup destructors and the one-time channel bootstrap retained solely because this
  task prohibited editing `bins/hone-imessage`.

### Runtime Timezone Without Geographic Defaults

- Status: done locally; no push and no historical database rewrite
- Date: 2026-08-16
- Plan: `docs/archive/plans/runtime-timezone-2026-08-16.md`
- Handoff: intentionally omitted per task contract
- Related PRs / commits: local `7ca42198`, `23208d2c`, `87ec8536`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; non-`+08:00`
  cron/date/render regression; full workspace tests; PostgreSQL ignored integration tests;
  CI-safe regression suite; Web tests; hardcoded-timezone closing grep
- Current conclusion: the process timezone now resolves from top-level config, then
  `HONE_TIMEZONE`, host IANA/current offset, and finally UTC. Cron, date keys, rendering,
  prompts and generic runtime clocks no longer assume Beijing; IANA DST is evaluated per
  instant. Offset-bearing TEXT timestamps compare as `timestamptz` without rewriting the
  existing `+08:00` rows.
- Next entry point: set top-level `timezone` explicitly in every production container; actor
  notification zones and exchange-calendar zones remain explicit domain overrides rather than
  process defaults.

## 2026-08-15

### Oldwang Research Platform Integration

- Status: done; merged to `main` (`b65a7cc1`) and pushed; no production deployment in this task
- Date: 2026-08-15
- Plan: `docs/archive/plans/oldwang-research-platform-integration.md`
- Handoff: `docs/handoffs/2026-08-15-oldwang-research-platform-integration.md`
- Related PRs / commits: merge `b65a7cc1` (integration branch `integrate/oldwang`)
- Related runbooks / regressions: full workspace cargo test (excl. local hone-desktop); Web 483/483; `tests/regression/ci` 23/23; local authenticated browser acceptance of `/research`
- Current conclusion: oldwang's ten research products now live on the `/research` research desk as URL-addressable panels fed by one `research-overview` aggregate call; navigation grew to five sections; dashboards became controlled panels sharing one modal shell/state/prompt envelope; CSS returned to `--hone-*` tokens with new traffic-light semantic tokens; backend gained `research_store` dedup, a weekly-brief pre-generation worker, the research-library body-limit fix, chat-path spawn_blocking, CORS DELETE, the email-token legacy-name fallback, and a defused date-pinned position-management test.
- Next entry point: worker migration into hone-scheduler, forum deletion semantics/attachment GC, history retention, public-router auth middleware, new-surface i18n, and large-payload pagination/ETag remain follow-ups listed in the handoff.

## 2026-08-11

### Weekly Brief Readable Calendar

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/weekly-brief-readable-calendar.md`
- Handoff: `docs/handoffs/2026-08-11-weekly-brief-readable-calendar.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-15-separate-weekly-review-from-the-industry-event-chain`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: Web API 289/2 ignored; Web 454/454; focused weekly and key-event contracts; TypeScript; public production build; authenticated API and desktop browser acceptance
- Current conclusion: the key-event-chain ten-day view has been replaced by a standalone structured weekly brief. It shows the prior and next Beijing natural weeks as readable date-grouped agendas, reuses macro/earnings JSON instead of the finance-calendar PNG, accepts only confirmed industry changes as completed facts, and labels past schedules, future reminders and missing earnings coverage explicitly. The local 2026-08-11 report correctly shows four prior-week schedules, three next-week macro events and no guessed earnings while FMP is unconfigured.
- Next entry point: configure the existing FMP key pool for covered-company earnings, then add a separately verified official-release ingestion path if prior-week macro actuals are required; do not infer results from schedule dates.

### Company Research Corpus Dialogue Priority

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/company-research-corpus-dialogue-priority.md`
- Handoff: `docs/handoffs/2026-08-11-company-research-corpus-dialogue-priority.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-11-compose-covered-company-dialogue-from-historical-thesis-cards-and-current-evidence`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: Skill Creator validation; `hone-channels` 793/1 ignored; company research dialogue contract; Hari conversation contract; Rust format/diff checks; console build; local skill-enabled smoke
- Current conclusion: covered-company questions now privately project only the relevant cards from the 52-company transcript corpus and require both the company-thesis and Hari Skills. Historical business-model, fundamental, moat and falsifier logic is preferred over generic memory, while all current prices, filings, guidance, orders, news, industry state and valuation inputs remain on the current evidence chain.
- Next entry point: configure the target environment's actor-safe function-calling model, then run MSFT, SNDK, APP+BE and uncovered NVDA answer canaries without weakening the public actor sandbox.

### Multi-method Daily Valuation And Rating Integration

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/multi-method-valuation-rating-integration.md`
- Handoff: `docs/handoffs/2026-08-11-multi-method-valuation-rating-integration.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-10-route-daily-valuation-by-business-model-and-feed-ratings-only-from-cross-checked-results`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: Web API 276/2 ignored; focused Web 22/22; TypeScript; Rust formatting/diff checks; public production build; console binary build; local fail-closed snapshot smoke
- Current conclusion: HONE valuation v2 now selects a multi-method model by business type, adds cycle normalization, scenario probabilities, method-level results and reverse valuation, and only sends fresh cross-checked values into company-rating v3. The 19:20 valuation run immediately refreshes ratings and the independent 19:30 run remains. Missing FMP data produces explicit unavailable rows and cannot reuse v1 or mock values.
- Next entry point: configure the target environment's existing FMP key pool, manually review at least one cyclical and one profitable-growth company, then version any profile-specific weight changes without weakening the quality gate.

### Hari Invest Conversation Decision Layer

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/hari-invest-conversation-agent.md`
- Handoff: `docs/handoffs/2026-08-11-hari-invest-conversation-agent.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-09-make-decisiveness-an-evidence-backed-decision-zone-contract`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: Skill Creator validation; Hari conversation contract; skill-runtime consistency/tools; `hone-tools` 185 passed / 1 ignored; `hone-channels` 789 passed / 1 ignored; console binary build; live admin dialogue with Skill tool trace
- Current conclusion: HONE's public `hari-invest` is now a conversation decision Skill rather than an exposed copy of the internal research package. Natural Chinese investment questions can discover it, its first paragraph must choose opportunity/hold/risk/data-insufficient with confidence and reason, and the rest separates time horizons, evidence, the strongest counterargument and observable change conditions. A real local dialogue loaded the Skill and its references before producing a decisive, bounded answer.
- Next entry point: configure an actor-safe server-side function-calling model or `hone_cloud` provider for ordinary public users, then add a golden-dialogue evaluation gate. Do not grant public actors host-capable Codex ACP access merely to make the button work.

### Daily Valuation Lab

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/daily-valuation-lab.md`
- Handoff: `docs/handoffs/2026-08-11-daily-valuation-lab.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-08-keep-numeric-valuation-hone-owned-reproducible-and-fail-closed`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: valuation Rust 5/5; Web API 268/2 ignored; Web 444/444; TypeScript; Rust formatting; public production build; console binary build; authenticated API and desktop browser acceptance
- Current conclusion: HONE now has a signed-in `/valuation-lab` and a 19:20 Beijing daily worker for the existing 52-company coverage. It uses current quote, quarterly free cash flow, balance-sheet and analyst-estimate evidence to compute transparent bear/base/bull DCF values, a forward-EPS cross-check and reverse-DCF implied growth. Only fresh, positive-FCF, convergent and cross-validated rows may enter company ratings; all other rows remain explicit no-valuation states. Exact numeric thresholds are HONE model defaults rather than claimed Hari formulas.
- Next entry point: configure the existing FMP key pool in the target environment, review the first live eligible cohort, then add versioned business-model-specific methods instead of weakening the missing-data gate.

### Unified Research Library

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/unified-research-library.md`
- Handoff: `docs/handoffs/2026-08-11-unified-research-library.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-07-keep-imported-research-actor-scoped-and-evidence-only`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: Web API 262/2 ignored; Web 441/441; TypeScript; Rust formatting; public production build; authenticated desktop/mobile browser acceptance; multipart API upload/list smoke
- Current conclusion: HONE now has a signed-in `/research-library` for daily personal or administrator-owned global research. It keeps source/date/hash/ticker/topic provenance and explicit per-item authorization for chat, key-event chains and portfolio news. Personal data stays actor-scoped; imported prose is evidence-only and cannot override instructions or become an automatic trade action. The current single-node local implementation uses manifest files plus stored bytes and does not claim a production vector database.
- Next entry point: migrate metadata to PostgreSQL and bytes to object storage before a cloud rollout, then implement one documented official connector at a time. Knowledge Planet/IMA private-session scraping remains out of scope.

### Continuous Research Ten-day Brief

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/continuous-research-ten-day-brief.md`
- Handoff: `docs/handoffs/2026-08-11-key-event-chain-and-serenity-source.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-06-treat-the-next-ten-days-as-a-verification-queue-not-a-prediction-calendar`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: key-event chain 9/9; Web API 258/2 ignored; Web 438/438; TypeScript; public production build; authenticated local browser acceptance
- Current conclusion: the key-event-chain dialog now includes a secondary ten-day brief without adding another homepage launcher. The current source-only snapshot reviews one attributable Rubin event for 2026-08-02–08-11 and keeps four questions for 2026-08-12–08-21; two HBM questions correctly wait for primary evidence. Review-by dates are not event predictions, broad titles use topic-specific evidence excerpts, and every available item links back to an admitted original.
- Next entry point: enrich the verification queue only from lawful primary calendars or filings, and keep explicit event dates separate from review-by dates. Configure the existing digest model only if bounded impact analysis is desired.

### Key-event Chain And Serenity Source

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/key-event-chain-and-serenity-source.md`
- Handoff: `docs/handoffs/2026-08-11-key-event-chain-and-serenity-source.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-05-build-key-event-chains-from-attributed-source-events`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: core 153/153; Web API 254/2 ignored; influencer adapter 7/7; key-event chain 5/5; Web 436/436; TypeScript; production build; fresh local source snapshots
- Current conclusion: the chat home now includes a seventh “关键事件链” Button. It builds 30-day Rubin and HBM timelines from attributable public-source events, preserves each original URL, labels deterministic topic admission separately from optional model analysis, and refreshes daily at 19:55 Beijing. The current local snapshot contains four Rubin events and three HBM events. Serenity is also connected to “大V速报” through the user-confirmed public aggregation feed while each item retains its exact X original. With no digest model configured, both products correctly remain `source_only` and do not invent impact judgments.
- Next entry point: configure the existing digest model profile to add ID-bound impact analysis, or add new event-chain topics through explicit keyword and source contracts; do not promote aggregation translations to primary evidence.

### Daily Influencer Brief Dashboard

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/influencer-daily-brief-dashboard.md`
- Handoff: `docs/handoffs/2026-08-11-influencer-daily-brief-dashboard.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-04-treat-influencer-content-as-attributed-opinion-not-market-truth`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: core 153/153; Web API 254/2 ignored; Web 436/436; TypeScript; production build; fresh local source snapshot
- Current conclusion: the chat home includes a cached “大V速报” that refreshes at 19:50 Beijing, preserves author/source/time, and separates original opinion, HONE summary and counterpoint. SemiAnalysis is connected through its official feed. The user-confirmed aichainmap public Serenity feed is connected as a named translation/aggregation layer while every row retains and validates the exact X original; Jukan remains “源待配置”. Missing sources, models and updates never become fabricated content or investment actions.
- Next entry point: configure a lawful Jukan bridge and digest model if desired; retain exact Serenity endpoint and X-original identity checks.

### Hari Daily Position-management Dashboard

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/position-management-daily-dashboard.md`
- Handoff: `docs/handoffs/2026-08-11-position-management-daily-dashboard.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-03-make-position-advice-evidence-gated-deterministic-and-non-executing`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: position-policy Rust 7/7; Web API 242/2 ignored; Web 430/430; focused Web 23/23; TypeScript; production build; authenticated local mobile browser acceptance
- Current conclusion: the fifth chat-home Button now produces an actor-scoped daily research report from real portfolio structure plus current company, macro, valuation and news evidence. It separates Hari logic from HONE concentration controls, fails closed on transcript-only/stale/uncovered data, and never executes or mutates a position. With current local FMP and same-day valuations unavailable, the real actor portfolio truthfully remains “数据不足”; no baseline score was promoted to advice and no private symbols or weights are retained here.
- Next entry point: configure current FMP evidence and reviewed daily Hari valuations to unlock current actions. The original five-Button scope is now implemented; future大V速报 or关键事件链 should start as separate product plans rather than weakening these evidence gates.

### Actor-scoped Daily Portfolio News Dashboard

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/portfolio-news-daily-dashboard.md`
- Handoff: `docs/handoffs/2026-08-11-portfolio-news-daily-dashboard.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-02-keep-portfolio-context-inside-hone-when-analyzing-news`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: portfolio-news Rust 6/6; Web API 235/2 ignored; Web 425/425; TypeScript; production build; authenticated local browser acceptance
- Current conclusion: the fourth chat-home Button now reads each actor's real positions, excludes watchlist rows, merges option exposure by underlying, filters the last 48 hours to attributable news, and optionally asks HONE's configured model for a validated impact analysis without disclosing portfolio context. Actor-local weights affect only ranking; source/model gaps remain explicit and never become invented actions.
- Next entry point: configure the existing FMP key pool and digest model in the deployed HONE environment, then let the 20:00 Asia/Shanghai worker populate real actor snapshots. The next product item is仓位管理, but it has not been started in this change set.

### Daily Signal Data Quality And Company Valuation Guardrails

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/daily-signal-data-quality-and-valuation.md`
- Handoff: `docs/handoffs/2026-08-11-daily-signal-data-quality-and-valuation.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-01-fail-closed-on-unsupported-daily-signals-and-stale-valuation`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: `docs/runbooks/source-web-startup.md`; Web API 229/2 ignored; full Web 420/420 plus focused contracts 21/21; TypeScript; production build; authenticated local browser acceptance
- Current conclusion: macro v2 now includes US 10Y/30Y Treasury yields, effective Fed funds, employment-population ratio and VIX with correct risk direction. AI v2 keeps only seven verifiable company-financial factors and removes unsupported specialized/hardware placeholders. Company valuation no longer uses transcript-era or generic P/E scores: only a same-day verified Hari three-scenario artifact can contribute, otherwise the UI says “今日不计估值分” and normalizes the remaining weights.
- Next entry point: populate `data/company_ratings/valuations/latest.json` only from a separately reviewed daily Hari research run; do not restore static valuation or begin the remaining homepage features until the user prioritizes the next one.

## 2026-08-10

### Local Public UI Dev Login Without SMS

- Status: done locally; no commit or production deployment
- Date: 2026-08-10
- Plan: `docs/archive/plans/local-public-dev-login.md`
- Handoff: `docs/handoffs/2026-08-10-local-public-dev-login.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-10-02-make-local-dev-login-explicit-server-owned-and-fail-closed`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: `docs/runbooks/source-web-startup.md`; backend gate test; frontend contract test; TypeScript; real Vite-proxied cookie/auth smoke
- Current conclusion: an explicitly enabled local/local HONE runtime now offers a server-owned test-account login button and normal HttpOnly session without SMS. The capability is off by default, does not render when disabled, and cannot activate in remote or cloud mode. Local ports 3000/3001/8077/8088 are healthy and only Web is running.
- Next entry point: refresh `http://127.0.0.1:3001/chat` and click “进入本地测试账号”; never set `HONE_PUBLIC_DEV_LOGIN` in production.

### Hari Invest Default Investment-Q&A Skill

- Status: done locally; no commit, public release, production deployment, or external publication
- Date: 2026-08-10
- Plan: `docs/archive/plans/hari-invest-hone-default.md`
- Handoff: `docs/handoffs/2026-08-10-hari-invest-hone-default.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-10-01-load-hari-invest-before-every-investment-answer`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: Skill validator; HONE prompt tests 14/14; discover tests 2/2; skill-tool tests 13/13; real MCP discovery/load; local runtime/API/channel smoke
- Current conclusion: HONE now installs the public-facing `hari-invest` Skill and requires every investment-related answer to load it before final judgment, while current facts remain evidence-backed and non-investment questions remain unaffected. The internal distiller and source transcripts are not exposed. The local runtime lists `hari-invest enabled=true`, Web is the only running channel, and both Vite frontends remain available.
- Next entry point: ask any investment question in the local HONE user UI at `http://127.0.0.1:3001/`; public release or production deployment requires separate explicit authorization.

### Macro And AI Daily Traffic-light Dashboards

- Status: done locally; no commit or deployment
- Date: 2026-08-10
- Plan: `docs/archive/plans/macro-ai-daily-signals.md`
- Handoff: `docs/handoffs/2026-08-10-macro-ai-daily-signals.md`
- Decision / ADR: no new ADR; reuses authenticated public API, worker lifecycle, FMP key pool and atomic snapshot boundaries
- Related PRs / commits: uncommitted local change set
- Related runbooks / regressions: Web API 218/2 ignored; daily-signals 7/7 and `cargo check -p hone-web-api`; Web full tests, TypeScript and production build; authenticated local 8077/8088/3001 runtime smoke
- Current conclusion: authenticated chat now has two cached daily-report launchers with gauges, evidence, history, CSP/hardware detail and saved-report Q&A. The worker preserves prior success, retries incomplete snapshots every 15 minutes, and never converts missing data to zero. FRED single-series CSV is live at 11/11 coverage (66.3 yellow); with FMP unconfigured, SEC Company Facts supplies all four CSP financial baselines (72.6 yellow) while hardware quotes remain explicitly unknown.
- Next entry point: refresh the local chat at `http://127.0.0.1:3001/chat`; configure the existing FMP key pool only if hardware market-confirmation quotes are required, and never hard-code a score to satisfy the visual gate.

### Transcript-informed Company Skill And Daily Ratings Dashboard

- Status: done locally; no commit, deployment, release, or tag
- Date: 2026-08-10
- Plan: `docs/archive/plans/company-thesis-daily-ratings.md`
- Handoff: `docs/handoffs/2026-08-10-company-thesis-daily-ratings.md`
- Decision / ADR: no new ADR; the dashboard uses an original explainable score and existing authenticated public/FMP boundaries
- Related PRs / commits: uncommitted local change set
- Related runbooks / regressions: Skill validation; transcript source coverage; Web API 214/2 ignored; Web 412; TypeScript; production build; local 8077/8088/3000/3001 runtime smoke
- Current conclusion: 51 private transcripts now resolve to 52 US-traded company research cards plus four cross-company evidence notes. Authenticated users get a searchable, filterable red/yellow/green dashboard above chat, backed by a 19:30 Beijing snapshot worker and explicit live/partial/stale/transcript-only provenance. No FMP key exists locally, so the current runtime correctly labels all 52 rows as research baselines rather than current-market ratings.
- Next entry point: configure the existing FMP key pool, restart the backend, and perform authenticated desktop/mobile acceptance from `http://127.0.0.1:3001/chat`; unsupported new listings and OTC rows must remain partial/low-confidence instead of receiving invented values.

## 2026-08-10

### Earnings OpenCode Signature And Renderer Recovery

- Status: done; exact GHCR/GCE deployment and authenticated production CRWV canary complete; no formal release or tag
- Date: 2026-08-10
- Plan: `docs/archive/plans/earnings-opencode-signature-recovery.md`
- Handoff: `docs/handoffs/2026-08-10-earnings-opencode-signature-recovery.md`
- Decision / ADR: no new decision; this restores the existing isolated earnings replay, safe-side-effect and PDF terminal contracts
- Related PRs / commits: direct `main` commits `4dd76971d7b9985e281c3632db17b2936e0f91ce`, `185504bc03d8be32bfcc1f851200e411ed8a8238`, and deployed `2a6aecf33936e85c7b34130fc2f8f2a8ab3eb9c6`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/opencode-setup.md`; `docs/runbooks/backend-deployment.md`; hone-channels/web-api suites; earnings PDF regression; full CI-safe regression; timestamp-bounded ACP and GCE artifact inspection
- Current conclusion: dedicated earnings turns recover once from the exact OpenRouter/Gemini corrupted-signature failure in a fresh isolated session, deny executed `task`/`bash`, retain safe renderer recovery material, and require a persisted PDF. Production CRWV message `7762a98e-b1e9-4e48-b20a-8f5175535346` exercised the real signature recovery and produced the four-page, 612,805-byte `CRWV_Q2_Earnings_Preview-1109374d.pdf`; attachment collection/persistence succeeded, active chats returned to zero, and the service stayed at `NRestarts=0`.
- Next entry point: `docs/current-plans/earnings-workflow-content-parity.md`; the runtime incident is closed, but the CRWV sample is not content-approved because it contains placeholder/untraceable evidence and required 22 renderer attempts.

## 2026-08-09

### Public Push Inbox And Unread State Closure

- Status: done; frontend deployed to Cloudflare Pages; no backend restart, formal release, or tag
- Date: 2026-08-09
- Plan: `docs/archive/plans/public-push-inbox-state-closure.md`
- Handoff: `docs/handoffs/2026-08-09-public-push-inbox-state-closure.md`
- Decision / ADR: no new decision; the implementation reuses the existing public push list/open and server read-through contracts
- Related PRs / commits: direct `main` commit `e451dd3b9a20f98777f888bf6b0e040c7fcdc386`; no PR, release, or tag
- Related runbooks / regressions: Web 407 tests; TypeScript typecheck; `tests/regression/ci/test_navigation_responsiveness_contract.sh`; public production build; authenticated production mobile browser acceptance
- Current conclusion: `/pushes` defaults to real push messages with summary, full Markdown detail and stable per-job categories, while subscription management remains a second view. Top, sidebar and bottom entries share one route-stable unread value, never clear it optimistically, and accept only server-returned `unread_count`; read failures retain the badge and retry. Cloudflare Pages serves the new message/manage views and the mobile workspace still has four primary tabs.
- Next entry point: `docs/handoffs/2026-08-09-public-push-inbox-state-closure.md`; production acceptance account had no historical pushes, so use a real push-bearing actor for any future visual review of category chips and full detail while retaining the automated model/API contracts.

## 2026-08-08

### Reviewed GCE Production Rollout

- Status: done; exact GHCR/GCE and Cloudflare Pages deployment complete; no formal release or tag
- Date: 2026-08-08
- Plan: `docs/archive/plans/production-gce-rollout-2026-08-08.md`
- Handoff: `docs/handoffs/2026-08-08-production-gce-rollout.md`
- Decision / ADR: no new decision; the review fixed violations of the existing bounded-enrichment and authenticated-session isolation contracts
- Related PRs / commits: direct `main` review-fix commit and exact deployed runtime `d379cccc6e909129d02e726c04919e7c7ec250e1`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; complete Rust/Web/Worker/CI-safe gates; Runtime Image run `31241183454`; CI run `31241183462`; Cloudflare Pages exact-commit check
- Current conclusion: production runs exact GHCR digest `sha256:adc956533d59e46cd50a44ff6380019aa5ece64de5013a2a6eb95646bbd1ca05` with healthy PostgreSQL/R2 cloud authority, zero local durable dependencies, zero active chats and `NRestarts=0`. Review fixed an outer preturn deadline that could erase completed branch evidence and a public route cache that could survive logout/account replacement. Pages serves the reviewed active-run recovery protocol and required security headers.
- Next entry point: `docs/handoffs/2026-08-08-production-gce-rollout.md`; use retained `beaf05c360a7397ce6335ce177fdb74380756662-ghcr-runtime` for immediate rollback. The stale `origin.hone-claw.com` tunnel alias remains a known separate risk while the public Worker API route is healthy.

## 2026-08-06

### Stripe Wallet Fixed-term Pass Deployment

- Status: blocked only on external Stripe wallet approval; code and production deployment complete
- Date: 2026-08-06
- Plan: `docs/current-plans/stripe-wallet-one-time-pass.md` (kept active until both live wallets are available)
- Handoff: `docs/handoffs/2026-08-06-stripe-wallet-one-time-pass.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-06-02-offer-recurring-and-fixed-term-stripe-memberships-as-separate-products`
- Related PRs / commits: direct `main` implementation `c99babc1e1ea3c54db41256331eb65dcefa7bd1d`; public-payment-copy correction `b905130158e12138fc1170c7de7e1adb54f0f08d`; no PR, formal release, or tag
- Related runbooks / regressions: `docs/runbooks/stripe-billing.md`; complete repository gates; signed Billing HTTP E2E; official Stripe test-mode Alipay and WeChat Pay payment lifecycles; Runtime Image runs `31082512757` and `31675804261`
- Current conclusion: HONE exposes the existing USD 199.99/year subscription and the USD 229.99/12-month non-renewing pass, and creates a correct authenticated live fixed-term Checkout. No live payment was submitted. On 2026-08-13 Stripe still reported both wallets `available=false`; exact correction runtime `e4e1e3e9` at digest `sha256:7d43450c4559fbf2a9dcf7d41faaa475627b9dc330f653f8fc18a1651deff351` therefore makes the production offer list server-authoritative and card-only by default. Production config and external Chrome both showed two card-only claims and no wallet claim; the external approval blocker remains unchanged.
- Next entry point: after Stripe approval, require both methods `available=true`, create one fresh no-payment live fixed-term Checkout, verify card plus Alipay plus WeChat Pay, retain a redacted screenshot, then archive the active plan.

## 2026-08-05

### Earnings PDF Terminal Enforcement And AAOI Production Repair

- Status: done; deployed to production from an exact GHCR image; no formal release or tag
- Date: 2026-08-05
- Plan: `docs/archive/plans/earnings-pdf-terminal-enforcement.md`
- Handoff: `docs/handoffs/2026-08-05-earnings-pdf-terminal-enforcement.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-05-02-route-earnings-workflows-to-gemini-31-pro-through-opencode` terminal-enforcement and OpenCode result-envelope consequences
- Related PRs / commits: direct `main` commits `c70be6c0`, `f24d0f76`, `bb002a43`, `82c835b7`, `9935d383`, and deployed `f5a384b2932b6602840968bc8c0a910f154008ee`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; full workspace check/test; complete CI-safe regressions; OpenCode 1.18.13 JSON-string result fixtures; real authenticated AAOI browser/download/refresh/PDF visual acceptance
- Current conclusion: a dedicated earnings turn cannot publish renderer-failure prose or a text-only report as success. The exact renderer trace and a persisted PDF are required, validated Markdown is projected deterministically, safe pre-write validation failures can recover once, and OpenCode's JSON-string MCP output is decoded before artifact/side-effect decisions. Production runs exact `f5a384b2` from digest `sha256:d7a11aef6b4b968bd172692ddfd5a29e4cfcd2a0d0f262f10afce499fcfab4ff`; real AAOI message `12fb473d-c2c3-4db7-ba40-e6b3a756e2f1` generated the four-page A4 `AAOI-preview-fdb23cd7.pdf`, downloaded successfully, survived chat refresh, and passed all-page watermark/news/share-image visual inspection.
- Next entry point: `docs/handoffs/2026-08-05-earnings-pdf-terminal-enforcement.md`; preserve strict renderer gates and use decoded tool traces, not user-facing fallback copy, for future diagnosis.

### Gemini 3.1 Pro Earnings Workflows And AAOI Production Sample

- Status: done
- Date: 2026-08-05
- Plan: `docs/archive/plans/gemini-earnings-workflows.md`
- Handoff: `docs/handoffs/2026-08-05-gemini-earnings-workflows.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-05-02-route-earnings-workflows-to-gemini-31-pro-through-opencode`
- Related PRs / commits: `115db5fb` through deployed runtime `2c2cd1db`; skill/renderer follow-ups `02641103`, `34b3ca49`, `f10175a9`, `105ca177`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/opencode-setup.md`; `docs/runbooks/backend-deployment.md`; focused runner/session/side-effect tests; `tests/regression/ci/test_earnings_research_pdf_markdown.sh`; real authenticated AAOI browser/PDF acceptance
- Current conclusion: both administrator earnings workflows use OpenCode ACP through OpenRouter's exact `google/gemini-3.1-pro-preview` while ordinary chat remains unchanged. The original Dify Workflow prompt is the primary analysis path; a real AAOI preview called a beat from the 189/190/203 USD-million management/consensus/independent bridge, compared Rosenblatt and Needham as actual issuing institutions, rendered eight company-relevant news paragraphs with no links, produced the exact watermark and Knowledge Planet share page, and remained downloadable after chat refresh. Structured `script_payload` removes model-authored JSON escaping, and the renderer rejects publisher-as-institution, conference, pure-price, generic-sector and unrelated-customer padding.
- Next entry point: `docs/handoffs/2026-08-05-gemini-earnings-workflows.md`; rotate the OpenRouter key that was pasted into chat, then use the same administrator buttons for future preview or attachment-backed analysis acceptance.

### Earnings Routing, Durable Attachments And Production Acceptance

- Status: done
- Date: 2026-08-05
- Plan: `docs/archive/plans/earnings-routing-minimax-hotfix.md`
- Handoff: `docs/handoffs/2026-08-04-production-deployment-dede2d61.md`
- Decision / ADR: no new ADR; this restores the existing trusted-runner and authenticated actor-owned attachment boundaries
- Related PRs / commits: `078b0883`, `cfb75481`, `ee250d72`, `50aa8b23`, deployed `9d64c5967bf74a5126948c7b49f6b918128f951a`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; focused routing/protocol/attachment/actor-OSS tests; complete CI-safe regression; real authenticated CRCL browser acceptance before and after service restart
- Current conclusion: canonical administrator earnings requests are promoted to the native `earnings-research` Skill and Codex ACP, provider-private tool tags are suppressed, generated attachments are promoted to actor-owned OSS even when the model already emitted an attachment marker, and the authenticated download proxy accepts only the current actor's upload/generated prefixes. A real CRCL preview survived a full service restart and downloaded from its historical chat card. The pre-fix local-only CRCL artifact is unrecoverable.
- Next entry point: use the retained old runtime release for rollback; keep Skill backups outside the configured discovery root, and use a fresh real earnings run—not a synthetic marker-only prompt—for future attachment acceptance.

## 2026-08-04

### Discord Stripe-only Community Migration

- Status: done; external Discord state updated and documentation completed; no application release or deployment
- Date: 2026-08-04
- Plan: `docs/archive/plans/discord-stripe-community-migration.md`
- Handoff: `docs/handoffs/2026-08-04-discord-stripe-community-migration.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-04-04-make-discord-community-operations-stripe-only`; refines the production consequences of `D-2026-08-04-01`
- Related PRs / commits: direct `main` documentation commit; no PR, release, deployment, or tag
- Related runbooks / regressions: `docs/runbooks/discord-stripe-community.md`; retired `docs/runbooks/whop-discord-fulfillment.md`; Discord REST identity/guild/member/role/channel/integration/webhook probes; external Chrome QA
- Current conclusion: `HONE 社区助手` is installed in `巴芒投研美股社群` with owner-approved Administrator after narrow permissions were proven insufficient for existing channel denies. The only public membership pin now uses HONE `/activate`, Stripe Checkout, and `/me`; the old Whop pin is deleted. `📋｜whop` became restricted `📋｜历史支付日志`, no active Whop Discord integration/webhook remains, and historical PII-bearing logs were neither exported nor deleted. Discord role state remains outside HONE Billing authority.
- Next entry point: `docs/handoffs/2026-08-04-discord-stripe-community-migration.md`; use the runbook before any bot permission, copy, role, or historical-log change.

### Stripe-only Production Billing Cutover

- Status: done; deployed to production from exact GHCR image; no formal release or tag
- Date: 2026-08-04
- Plan: `docs/archive/plans/stripe-whop-parallel-billing.md`; superseded email task `docs/archive/plans/whop-email-delivery.md`
- Handoff: `docs/handoffs/2026-08-04-stripe-only-production-cutover.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-04-01-make-stripe-the-only-external-billing-provider`; `docs/decisions.md#d-2026-08-04-03-build-managed-linux-runtimes-in-actions-and-deliver-them-through-ghcr`
- Related PRs / commits: direct `main` commits `9961652f`, `91e93b51`, and exact deployed `edddfc5b890d124d76d8c6eddc9aa85f2e94b807`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/stripe-billing.md`; `docs/runbooks/backend-deployment.md`; `tests/regression/ci/test_billing_contract.sh`; `tests/regression/ci/test_billing_http_e2e.sh`; `tests/regression/ci/test_runtime_image_contract.sh`; GitHub Runtime Image run `30893733765`
- Current conclusion: HONE production is Stripe-only and runs the exact source revision above from GHCR digest `sha256:0dcd14a825a124344908b34f6cab19f83eca1f614a40eb2bdf08df2f093f0eee`. Live account/catalog/Portal/eight-event webhook/minimal restricted key, Cloudflare email verification, official open/unpaid USD 199.99/year Checkout, fail-closed unpaid entitlement, Stripe-only database constraints, and public auth boundaries passed. In the correct `bamang_research` profile, the Whop product and annual plan were hidden and the HONE webhook deleted. No real live payment was submitted.
- Next entry point: `docs/handoffs/2026-08-04-stripe-only-production-cutover.md`; obtain explicit owner authorization before any live-money proof, then separately address reconciliation, refunds/disputes, and Stripe Tax policy.

### Administrator Earnings Research Chat Entry And PDF Delivery

- Status: done; committed to `main`, not released or deployed
- Date: 2026-08-04
- Plan: `docs/archive/plans/earnings-research-chat-entry.md`; `docs/archive/plans/earnings-preview-expectation-model.md`; `docs/archive/plans/earnings-preview-news-page.md`; `docs/archive/plans/earnings-preview-news-freshness.md`; `docs/archive/plans/earnings-native-e2e-runtime-validation.md`
- Handoff: `docs/handoffs/2026-08-04-earnings-research-chat-entry.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-04-01-run-earnings-research-as-an-actor-scoped-native-skill`
- Related PRs / commits: direct `main` feature commit; no PR, release, or deployment
- Related runbooks / regressions: `tests/regression/ci/test_earnings_research_pdf_markdown.sh`; `tests/regression/manual/test_earnings_research_pdf.sh`; focused Web/Rust contracts; real SNDK analysis and preview acceptance; old Dify Workflow format comparison
- Current conclusion: The final-output contract matches the old Workflow structure and tone, while the expectation engine requires a same-quarter dated consensus snapshot, provider disagreement, comparable guidance bias, call/deck evidence, catalyst inclusion status, an independent revenue/profit bridge, and neutral bands. A private `preview_audit` binds published values and recomputes the call before PDF rendering. Current-source real-browser acceptance now completes from the administrator button through native Codex, host `skill_tool`, full Workflow-form report, five-page branded PDF, same-message download card, click path, and all-page inspection. The accepted 2026-08-04 evidence snapshot calls SNDK `与分析师持平` because both independent metrics remain inside the audited neutral bands; this supersedes older static examples but does not hard-code SNDK.
- Next entry point: `docs/handoffs/2026-08-04-earnings-research-chat-entry.md`; production rollout remains separate, and a deployed attachment-backed `财报分析` is the remaining environment-specific acceptance case.

## 2026-08-03

### One Codex Native Session Per Persistent Hone Conversation

- Status: done
- Date: 2026-08-03
- Plan: `docs/current-plans/acp-runtime-refactor.md` (active umbrella plan remains open)
- Handoff: `docs/handoffs/2026-08-03-codex-single-native-session.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-03-11-make-the-persisted-codex-native-id-the-sole-conversation-identity`; `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: delivered directly through `main`; exact revision remains discoverable from Git history and the source-runtime release manifest; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`; Codex CLI `0.146.0` / codex-acp `1.1.7` executable boundary; isolated manual Codex probe home
- Current conclusion: a nonempty persisted native ID is the sole identity binding. Missing ID alone permits `session/new`; the returned ID is checkpointed before prompt, mode/fingerprint changes resume in place, resume failure cannot fork, and native transport failures are never automatically resent.
- Next entry point: `docs/handoffs/2026-08-03-codex-single-native-session.md`. The umbrella ACP runtime refactor remains active and is not archived by this completed subphase.

### Bounded Local Build Storage And Source Release Retention

- Status: done
- Date: 2026-08-03
- Plan: `docs/archive/plans/build-storage-optimization.md`
- Handoff: `docs/handoffs/2026-08-03-build-storage-optimization.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-03-03-bound-local-build-storage-without-sharing-writable-targets`
- Related PRs / commits: pushed directly to `main`; no PR, deployment, release, or tag
- Related runbooks / regressions: `docs/runbooks/source-web-startup.md`; `tests/regression/ci/test_source_runtime_deploy_contract.sh`; complete workspace, Web, Edge Worker, and CI-safe gates
- Current conclusion: dev/test/source-runtime keep line-level debuginfo and no incremental state while each worktree retains an isolated writable target. Source deployment records its profile, preserves current + previous, and prunes only strict known old release directories after rollback is disarmed. Two audited Codex worktrees were removed, stale targets were cleaned, and the fully rebuilt/validated active target is `8.3G` with zero incremental payload instead of the prior roughly 50GB multi-worktree footprint.
- Next entry point: `docs/handoffs/2026-08-03-build-storage-optimization.md`; the current local runtime was deliberately not redeployed, so the next successful revision-bound deployment is where live release retention first applies.

### Confirmed Proactive Deliveries In The Next Agent Turn

- Status: done
- Date: 2026-08-03
- Plan: `docs/archive/plans/delivered-push-agent-context.md`
- Handoff: `docs/handoffs/2026-08-03-delivered-push-agent-context.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-03-01-project-confirmed-proactive-deliveries-into-the-next-interactive-turn`
- Related PRs / commits: pushed directly to `main`; no PR, deployment, release, or tag
- Related runbooks / regressions: `docs/runbooks/opencode-setup.md`; Codex ACP `1.1.7` executable JSON-RPC contract, OpenCode ACP `1.18.11` versioned prompt/stream contract and real free-model Hone MCP probe, 556 event-engine tests, 725 channel tests, full workspace check/test, Web 347, Edge 45, and complete CI-safe regressions
- Current conclusion: only an explicit channel ACK/durable-delivery call creates an actor-scoped journal fact. The next eligible Interactive turn atomically claims an ordered bounded batch, reuses it across retries, consumes on Agent success, and releases on failure. Native runners receive an explicit fact block before current user input; replay runners receive assistant/context. User persistence, developer instructions, historical transcript, tools/results, compact behavior, and adapter-specific streaming remain separate. Ordinary audit `sent`, queued/failed/dry-run output, compact, quota rejection, scheduler, and heartbeat cannot consume or inject by accident.
- Next entry point: `docs/handoffs/2026-08-03-delivered-push-agent-context.md`; start at `EventStore::log_confirmed_delivery`, then follow AgentSession claim and `RunnerConversationInput::prepare` projection.

## 2026-08-02

### Actor Price Ladder And Effective Rule Explanation

- Status: done
- Date: 2026-08-02
- Plan: `docs/archive/plans/price-ladder-effective-rules.md`
- Handoff: `docs/handoffs/2026-08-02-price-ladder-effective-rules.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-02-04-separate-price-candidate-bands-from-actor-notification-ladders`
- Related PRs / commits: committed and pushed directly to `main`; no PR, deployment, release, or tag
- Related runbooks / regressions: Core 136; event-engine 538/13 ignored; tools 163/1 ignored; Web API focused 6; Web 343; TypeScript typecheck; workspace all-target check; complete CI-safe regressions; 43-item offline classifier fixture; real local browser QA
- Current conclusion: PricePoller keeps one shared 6% + 2-point candidate grid, while each actor can own the first threshold and repeat step. The Router and all rule-query surfaces resolve one domain policy, so an 8%/4-point request executes and explains ±8/12/16 consistently. Lower system candidates and insufficient advances are not immediate, `immediate_kinds=price_alert` cannot bypass the threshold, min severity uses the actor-final severity, and system-floor intervention is explicit. The query also fails loud when the event engine or `price_alert` is globally disabled, states that the per-category daily High cap still applies, and explains that intraday price bands remain exempt from ordinary same-symbol cooldown.
- Next entry point: `docs/handoffs/2026-08-02-price-ladder-effective-rules.md`; start with `NotificationPrefs::effective_price_alert_policy`, then inspect router policy/dispatch and the shared schedule overview builder.

### Public Administrator Usage Analytics

- Status: done
- Date: 2026-08-02
- Plan: `docs/archive/plans/public-admin-usage-analytics.md`, `docs/archive/plans/public-admin-usage-analytics-refinement.md`, `docs/archive/plans/public-admin-usage-trend-charts.md`, `docs/archive/plans/public-admin-usage-date-options.md`
- Handoff: `docs/handoffs/2026-08-02-public-admin-usage-analytics.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-02-03-keep-public-usage-analytics-inside-the-web-administrator-boundary`
- Related PRs / commits: uncommitted local change set; no deployment, release, or tag
- Related runbooks / regressions: `docs/runbooks/public-user-admin.md`; Web API 164/164 with two credentialed live smokes ignored, Web 343/343, TypeScript typecheck, Public production build, and authenticated local browser QA
- Current conclusion: `/me` mounts a server-authorized usage report, two native-SVG 14-day charts, and one bounded, scrollable per-user/day table before collapsible whitelist management. The report removes message automation and all `codex*` actor rows, masks phone labels, and never exposes other channels. Daily charts show distinct question users and question totals on the same zero-filled two-week axis; the date selector also lists every one of those 14 dates, including zero-activity days, and the headline follows the selected range/date. Both administrator panels collapse independently; ordinary users remain hidden/`403`. Local read-only data and interaction QA passed; deployment was not performed.
- Next entry point: `docs/handoffs/2026-08-02-public-admin-usage-analytics.md`; use `crates/hone-web-api/src/routes/public_admin.rs` for aggregation/auth changes and `packages/app/src/components/public-admin-usage-panel.tsx` for presentation.

### Codex ACP Tool Status Cross-channel Projection Repair

- Status: done
- Date: 2026-08-02
- Plan: `docs/current-plans/acp-runtime-refactor.md` (tool-status subphase done; shared ACP runtime plan remains active)
- Handoff: `docs/handoffs/2026-08-02-acp-tool-status-projection.md`
- Decision / ADR: N/A; this restores the existing adapter-lifecycle and user-visible projection boundary without changing module ownership, runner routing, or storage authority
- Related PRs / commits: uncommitted local change set; no release or tag
- Related runbooks / regressions: exact Codex ACP 1.1.7 lifecycle/MCP/shell fixtures, secret-redaction and metadata-loss regressions, full `hone-channels` library suite, iMessage tests, four IM channel-bin compile checks, isolated live Discord MCP/shell probes, and source LaunchAgent health verification
- Current conclusion: cancelled `mcp_startup.<server>` watcher events remain available in raw diagnostics but no longer appear or count as business tools. Structured Hone MCP calls and actual shell calls have distinct, bounded labels; shell start/done labels remain stable even when Codex completion metadata is sparse. Discord, Telegram, Feishu, and iMessage consume the corrected shared event according to their Full/Compact/pending-state UX, without exposing command arguments, full paths, URLs, or secrets.
- Next entry point: `docs/handoffs/2026-08-02-acp-tool-status-projection.md`; use raw ACP payload shape plus the shared runner/outbound projection before adding any channel-specific exception.

## 2026-08-01

### Public Whole-site Theme Surface Audit And Repair

- Status: done
- Date: 2026-08-01
- Plan: `docs/archive/plans/public-theme-surface-audit.md`
- Handoff: `docs/handoffs/2026-08-01-public-theme-surface-audit.md`
- Decision / ADR: N/A; this completes the existing Public light/dark surface contract without changing routing ownership, authentication semantics, data flow, or module boundaries
- Related PRs / commits: this change set is committed and pushed directly to `main`; no release or tag
- Related runbooks / regressions: 20 focused theme contracts, Web typecheck, 334 Web tests, Public production build, `git diff --check`, and real light/dark browser QA across every Public UI route plus `/portfolio` and `/invest` redirect verification
- Current conclusion: first-time Public visitors still default to light, while explicit light, dark, and auto remain supported. Home, roadmap, plan, Blog/article, signed-in community surfaces, shared navigation overlays, mobile menus, and the footer now use paired semantic surfaces and foregrounds. Logged-out account, Whop, community gate, legal, chat, and share-preview pages were rechecked; all audited routes had zero horizontal overflow. Authentication, purchases, production data, deployment, release, and tags were not exercised.
- Next entry point: `docs/handoffs/2026-08-01-public-theme-surface-audit.md`; if production differs, verify the Pages commit/cache first, then reproduce under explicit `light` and `dark` before changing CSS.

### Public Logged-out Theme And Whop Layout/Scroll Repair

- Status: done
- Date: 2026-08-01
- Plan: `docs/archive/plans/public-login-theme-contrast.md`
- Handoff: `docs/handoffs/2026-08-01-public-auth-theme-whop-layout.md`
- Decision / ADR: N/A; this restores the existing Public theme and responsive-layout contract without changing authentication semantics, module boundaries, or data authority
- Related PRs / commits: this change set is committed and pushed directly to `main`; no release or tag
- Related runbooks / regressions: Public theme preference, login-surface, legal-page, and Whop direct-route contract tests, Web typecheck and 325 Web tests, Public production build, Edge Worker typecheck and 45 tests, full workspace check/test excluding Apple clients, complete CI-safe regressions, and real light/dark browser QA with computed contrast, overflow, TOC navigation, and Whop scroll proof
- Current conclusion: first-time Public visitors default to light while explicit `light`, `dark`, and `auto` preferences remain valid. Logged-out HONE branding, controls, links, status text, user agreement, privacy policy, shared navigation, and back-to-top action use theme-aware semantic surfaces with readable contrast. The Whop direct route loads Public CSS, renders a correctly sized brand, fits narrow viewports without horizontal overflow, and allows the full activation form to scroll. Authentication and verification behavior were not changed or exercised.
- Next entry point: `docs/handoffs/2026-08-01-public-auth-theme-whop-layout.md`; if production differs, verify the deployed Pages commit and inspect root theme attributes plus the Whop lazy chunk's Public CSS imports.

## 2026-07-31

### Public User Administrator And Whitelist Management

- Status: done
- Date: 2026-07-31
- Plan: `docs/archive/plans/public-user-admin-whitelist-management.md`
- Handoff: `docs/handoffs/2026-07-31-public-user-admin-whitelist-management.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-31-03-keep-public-user-administration-separate-and-database-authoritative`
- Related PRs / commits: implementation/deployment commit `5eacfe98c0b2b3bdaac11fc23830c0ab91b14f3d` on `main`; no release or tag
- Related runbooks / regressions: `docs/runbooks/public-user-admin.md`, `docs/runbooks/backend-deployment.md`; CLI 85, memory 132, core cloud-runtime 20, Web API 159 plus two credentialed ignores, Web typecheck/tests/public build, finance contracts 44/44, and the complete CI-safe regression suite
- Current conclusion: production public users now have a PostgreSQL-authoritative administrator role. `13871396421` is uniquely verified as an active administrator; `/me` exposes a responsive management panel only for the server-projected role, while every list/create/disable request rechecks PostgreSQL. Successful creates are atomically limited to five per administrator per Beijing day and audited; disable clears sessions and protects self/administrator targets. Cloudflare Pages and exact immutable runtime `5eacfe98` are live, PostgreSQL/R2 remain authoritative and healthy, local/origin/public auth boundaries fail closed, Feishu and the origin tunnel remain supervised, and active chats are zero. No prompt answer format was changed and no synthetic production whitelist mutation was performed.
- Next entry point: `docs/handoffs/2026-07-31-public-user-admin-whitelist-management.md`; on the administrator's next normal login, visually confirm “我的 → 管理”, and use only an explicitly controlled member for any live create/disable canary.

### Codex ACP Static-System, Minimal-Turn, And Native-Skill Follow-up

- Status: done
- Date: 2026-07-31
- Plan: `docs/current-plans/acp-runtime-refactor.md` (system/reseed, minimal-turn, and native-skill subphases done; shared ACP runtime plan remains active)
- Handoff: `docs/handoffs/2026-07-30-codex-acp-session-continuity-diagnosis.md` (2026-07-31 follow-up section)
- Decision / ADR: `D-2026-07-30-01` updated with first-turn/post-compaction seed semantics; `D-2026-07-31-02` records native skill discovery
- Related PRs / commits: this change set
- Related runbooks / regressions: focused ACP/Codex/sandbox tests, real CLI/ACP prompt inspection, Codex `skills/list`, a real native skill activation probe, full workspace check/test excluding Apple clients, Web `309/309`, Edge Worker `45/45` plus typecheck, finance contracts `44/44`, and complete CI-safe regressions
- Current conclusion: one persistent Codex thread no longer receives Hone's complete static system prompt on every message. The first native prompt sends it once; a structured native `contextCompaction` event schedules exactly one successful reseed. Ordinary trusted Interactive resumes contain only current Beijing time and current normalized user/attachment content—no Hone session/history metadata, receive-routing metadata, related-skill hints, entity-loop instructions, answer contracts, or generic user-input wrapper. Enabled Hone skills are exposed as per-skill symlinks under the actor workspace's `.agents/skills`; Codex natively discovered all 16 without errors and read `Market Analysis` on demand without a Hone MCP skill-loading call.
- Next entry point: use `codex_acp_should_seed_system_prompt` and `acp_needs_sp_reseed` in `crates/hone-channels/src/runners/codex_acp.rs`, structured compaction handling in `crates/hone-channels/src/runners/acp_common/ingest.rs`, minimal current-turn assembly in `crates/hone-channels/src/turn_builder.rs` / `agent_session/core.rs`, and native skill projection in `crates/hone-channels/src/execution.rs` / `sandbox.rs`.

### Conversational Notification Time And Numeric Controls

- Status: done
- Date: 2026-07-31
- Plan: `docs/archive/plans/notification-prefs-time-numeric-controls.md`
- Handoff: `docs/handoffs/2026-07-31-notification-prefs-time-numeric-controls.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-31-01-keep-conversational-notification-controls-deterministic-and-domain-owned`
- Related PRs / commits: committed directly to `main` in the 2026-07-31 notification repair change set; no release or tag
- Related runbooks / regressions: full `hone-core`, `hone-event-engine`, and `hone-tools` library suites; focused notification-prefs Web API tests; Web typecheck and 309 Web tests; local source runtime rebuild/restart
- Current conclusion: normal channel Agents can set or independently inherit actor-scoped timezone, named digest slots and macro floor, quiet hours, generic/up/down price thresholds, and large-position weight. A typed three-state patch, a composite multi-field action, and shared event-engine validator keep Agent/API behavior atomic and aligned; the tool publishes its real union input schema. Prompt, model, classifier, and investment-mainline edits remain outside this conversational surface.
- Next entry point: `docs/handoffs/2026-07-31-notification-prefs-time-numeric-controls.md`

### SEC Filing Summary JSON Cross-Channel Normalization

- Status: done
- Date: 2026-07-31
- Plan: `docs/archive/plans/sec-filing-summary-json-normalization.md`
- Handoff: `docs/handoffs/2026-07-31-sec-filing-summary-json-normalization.md`
- Decision / ADR: N/A; this restores the existing plain-text summary contract without changing channel ownership, routing, storage authority, or module boundaries
- Related PRs / commits: committed directly to `main` in the 2026-07-31 notification repair change set; no release or tag
- Related runbooks / regressions: full `hone-event-engine` lib tests, full `hone-core` lib tests plus the final config-example guard, `hone-web-api` check, exact changed-file rustfmt, cross-channel immediate renderer and Digest JSON-wrapper regressions, local source runtime rebuild/restart
- Current conclusion: SEC filing provider responses are normalized before persistence and again when existing events are rendered. Plain text, JSON objects/strings, and JSON code blocks produce user-facing prose; invalid structures fail closed to the filing fallback. Discord, Telegram, Feishu, iMessage, immediate delivery, and Digest share the corrected behavior. The source runtime is active with `filing_summary.response_format` removed from the effective config.
- Next entry point: `docs/handoffs/2026-07-31-sec-filing-summary-json-normalization.md`; inspect `delivery_log.body`, `events.payload_json.llm_summary`, and `MarketEvent::normalized_llm_summary` if another structured-response envelope becomes visible.

## 2026-07-30

### Codex ACP Session Continuity Diagnosis And Persistent-Session Follow-up

- Status: done
- Date: 2026-07-30
- Plan: `docs/current-plans/acp-runtime-refactor.md` (diagnostic subphase done; shared ACP runtime plan remains active)
- Handoff: `docs/handoffs/2026-07-30-codex-acp-session-continuity-diagnosis.md`
- Decision / ADR: `D-2026-07-30-01` supersedes the Codex fresh-session portion of the earlier ACP continuity contract
- Related PRs / commits: uncommitted local follow-up; the now-superseded fresh-session behavior was introduced by `be5d7414`
- Related runbooks / regressions: Codex ACP initialize/event-stream manual regressions, raw cross-process resume probe, real two-turn Discord-path probe, persistent-session metadata/cold-start/context-overflow regressions
- Current conclusion: one deterministic Hone logical session now maps to one persistent native Codex session. The first turn calls `session/new`; later turns call non-replaying `session/resume`. Hone seeds its durable transcript only when entering persistent mode, then Codex owns history and automatic compaction. A live Discord probe used native ID `019fb3c2-f2f7-7140-8140-7520409d79be` for both turns and the single Codex rollout contained two complete user/assistant turns. Codex CLI and adapter are `0.146.0` / `1.1.7`; model and effort are process config.
- Next entry point: `docs/handoffs/2026-07-30-codex-acp-session-continuity-diagnosis.md`; diagnose a continuity issue from the Hone logical session metadata, the `session/resume` request/result, and the one matching native Codex rollout. Do not silently replace an unresumable native session.

## 2026-07-28

### P0 Experience Core Capabilities

- Status: done
- Date: 2026-07-28
- Plan: `docs/archive/plans/p0-experience-core-capabilities.md`
- Handoff: `docs/handoffs/2026-07-28-p0-experience-core-capabilities.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-27-01-make-evidence-admission-and-mutation-completion-capability-level`
- Related PRs / commits: `c2edceb7269476c39a3eb23efd25d14d4675aa93`, `e07eadb3a7af01bc71e1240ddd351c8805313eff`, `f56631072a38f32f8f02efa49c5a268156612219`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, `tests/regression/ci/test_finance_automation_contracts.sh`
- Current conclusion: nineteen workbook records were reduced to reusable evidence, mutation-completion, channel-projection, session/task, and responsive-workspace capabilities. Three were solved in this task, four prior systemic fixes were reverified, six are covered by current contracts, one remains an explicit provider-wide residual risk, four brittle one-off proposals were deferred, and one non-product request needs no action. The `c2edceb7` capability fix is live inside exact runtime `f5663107`; its lightweight no-proxy supervisor readiness path remains stable beyond the previous self-exit window, with authoritative PostgreSQL/R2 storage, healthy auth/public routes, established Feishu connectivity, and zero active chats.
- Next entry point: `docs/handoffs/2026-07-28-p0-experience-core-capabilities.md`

## 2026-07-27

### Automatic Reminder Cancellation And Feishu Table Production Activation

- Status: done
- Date: 2026-07-27
- Plan: `docs/archive/plans/reminder-cancellation-and-feishu-table-activation.md`
- Handoff: `docs/handoffs/2026-07-27-reminder-cancellation-and-feishu-table-activation.md`
- Decision / ADR: N/A; this closes missing durability and delivery checks inside the existing actor-scoped scheduler contract and records the resulting invariant without changing module ownership or prompt answer format
- Related PRs / commits: fix commit `caa45733819a404ebc7e383f8b830b0a26bcff80` on `main`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; targeted reminder/storage/tool/scheduler/channel regressions; `hone-feishu` `69/69`; full workspace check/test excluding Apple clients; Web `302/302`; Edge Worker `45/45`; finance contracts `44/44`; complete CI-safe regressions; 500-payload immutable manifest and production cloud/auth/process/connection probes
- Current conclusion: `remove_all` and `disable_all` now make actor-scoped automatic-reminder cancellation durable and idempotent, propagate storage failures, and suppress cancelled queued work both before model execution and before outbound channel delivery while preserving claimed one-shot semantics. Production Web/Feishu moved from `0.15.2` to exact `caa45733` (`0.15.3`), activating the already released Feishu native Markdown/raw-table renderer. Cloud authority, PostgreSQL/OSS, ports, origin/public auth boundaries, Feishu TLS connections, and zero active chats are healthy. Prompt answer formatting was not changed, Discord's pre-existing credential stop remains unchanged, and the prior immutable package remains available for rollback.
- Next entry point: `docs/handoffs/2026-07-27-reminder-cancellation-and-feishu-table-activation.md`, `docs/bugs/cancel_all_automatic_reminders_leaves_scheduled_jobs_active.md`, and `docs/invariants.md`; if a real client reproduces either issue, capture the actor/job identity or exact Feishu payload without sending a broad canary.

### v0.15.3 Feishu Native Table Fix Release

- Status: done
- Date: 2026-07-27
- Plan: `docs/archive/plans/v0.15.3-formal-release.md`
- Handoff: `docs/handoffs/2026-07-27-v0.15.3-formal-release.md`
- Decision / ADR: N/A; this patch corrects Feishu card serialization without changing module, runner, route, tool, channel-binary, crate, or storage topology
- Related PRs / commits: release commit `9b75868fb202da58ef0559d57834510f0af7a694`; annotated tag `v0.15.3`; GitHub Actions Release run `30249078543`
- Related runbooks / regressions: workspace check excluding Apple clients; `hone-feishu` `69/69`; `hone-channels` prompt `55/55`; two shared raw-table regressions; release-note validation; commit-state rustfmt and diff checks; successful Linux/macOS/Apple/Homebrew jobs; checksum manifests and eight-asset inventory
- Current conclusion: `v0.15.3` is published as a non-draft, non-prerelease GitHub Release. Standard Markdown and parseable legacy raw tables now become root-level Feishu JSON 2.0 tables; malformed or constrained paths do not expose component source. All expected CLI and Apple assets plus checksum manifests are available. Production was not deployed or restarted.
- Next entry point: `https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.15.3`, then `docs/handoffs/2026-07-27-v0.15.3-formal-release.md`; use the normal deployment workflow before the controlled direct/scheduler Feishu client recheck.

### Feishu Native Table Rendering

- Status: done
- Date: 2026-07-27
- Plan: `docs/archive/plans/feishu-native-table-rendering.md`
- Handoff: `docs/handoffs/2026-07-27-feishu-native-table-rendering.md`
- Decision / ADR: N/A; this corrects the existing Feishu card protocol implementation without changing module ownership or cross-module architecture
- Related PRs / commits: release commit `9b75868fb202da58ef0559d57834510f0af7a694`; annotated tag `v0.15.3`; GitHub Actions Release run `30249078543`
- Related runbooks / regressions: `hone-feishu` markdown `18/18`, outbound `6/6`, and full `69/69`; `hone-channels` prompt `55/55`; existing shared sanitizer and scheduler raw-table regressions; scoped Rust format and diff checks
- Current conclusion: standard Markdown tables and parseable legacy raw-table payloads are now emitted as root-level Feishu JSON 2.0 `table` elements before message splitting. Direct, scheduler, and placeholder final updates share the renderer; malformed, over-limit, and Markdown-only paths remain readable without exposing `<table .../>` source. No runtime restart, deployment, or live Feishu send was performed.
- Next entry point: deploy through the normal release/runtime workflow, then replay one direct and one scheduler Markdown table in a controlled Feishu account and append the client result to `docs/bugs/feishu_raw_table_component_code_leak.md`.

### v0.15.2 Formal Release And Production Deployment

- Status: done
- Date: 2026-07-27
- Plan: `docs/archive/plans/v0.15.2-formal-release-deploy.md`
- Handoff: `docs/handoffs/2026-07-27-v0.15.2-formal-release-deploy.md`
- Decision / ADR: N/A; this release activates the already accepted `v0.15.1` cumulative security baseline without changing module topology, answer format, channel ownership, or cloud storage authority
- Related PRs / commits: release commit `8491a3c2aabac28e9cd8411a8b2adbc61c7799c5`; annotated tag `v0.15.2`; GitHub Actions Release run `30230535235`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; `docs/runbooks/desktop-release-app-runtime.md`; full workspace check/test excluding Apple clients; Web `302/302`; Edge Worker `45/45`; finance contracts `44/44`; complete CI-safe regressions; main CI `30230221660`; Apple Clients `30230221669`; 500-file immutable manifest; production cloud/auth/channel probes; downloaded Apple checksum, DMG, App, Simulator, and Xcode inspection
- Current conclusion: production moved from the old exact `0.14.1` runtime to `target/deploy-8491a3c2` and now reports `0.15.2`, healthy PostgreSQL/R2, authoritative cloud storage, zero local durable dependencies and active chat runs, and running Web/Feishu channels. The non-draft, non-prerelease GitHub Release contains all eight expected assets; downloaded Apple artifacts are checksum-valid and version/architecture-correct. The prior immutable runtime remains available for rollback. macOS remains ad-hoc signed/not notarized, iOS remains Simulator/Xcode-only, and the already stopped Discord credential state is unchanged.
- Next entry point: `https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.15.2`, then `docs/handoffs/2026-07-27-v0.15.2-formal-release-deploy.md` for deployment evidence, checksums, rollback, and Apple distribution limitations.

### Security Audit Remediation And v0.15.1 Release

- Status: done
- Date: 2026-07-27
- Plan: `docs/archive/plans/security-audit-remediation-release.md`
- Handoff: `docs/handoffs/2026-07-27-security-audit-remediation-release.md`
- Decision / ADR: N/A; the release strengthens existing browser, auth, archive, tool-process, config, and secret-lifecycle boundaries without changing module topology or storage authority
- Related PRs / commits: security release commit `a24ba01b` plus the dependency-advisory follow-up; annotated tag `v0.15.1`
- Related runbooks / regressions: sealed Codex Security scan `c771f7ab-01ee-4f83-9a14-19e6df35834b`; full workspace check/test excluding Apple clients; Web tests and public build; Edge Worker typecheck plus `45/45`; complete CI-safe regressions; release-note validation; live Pages security-header probe
- Current conclusion: all 12 validated findings (6 Medium, 6 Low) have targeted code fixes and regression proof. The release adds browser framing/transport headers, bounded SMS admission and uniform membership responses, chart/archive resource budgets, owner-only credential files and truthful token rotation, hidden Discord token entry, digest-only cloud API-key persistence with legacy cleanup, and patched QUIC/Serde/Tauri/Telegram dependencies. Two additional transitive alerts were proved non-reachable by exact feature/API/target analysis before dismissal. No Critical or High source finding remains.
- Next entry point: `docs/handoffs/2026-07-27-security-audit-remediation-release.md`, `docs/releases/v0.15.1.md`, and `docs/runbooks/backend-deployment.md`; backend activation remains an external-supervisor rollout step after the formal release.

### Public Workspace Visual Acceptance Fixes

- Status: done
- Date: 2026-07-27
- Plan: `docs/archive/plans/public-workspace-visual-acceptance-fixes.md`
- Handoff: `docs/handoffs/2026-07-27-public-workspace-visual-acceptance-fixes.md`
- Decision / ADR: N/A; public auth, session ownership, API boundaries, and module boundaries are unchanged
- Related PRs / commits: this change set
- Related runbooks / regressions: 285 frontend tests with 863 assertions, TypeScript check, public frontend build, visual contract tests, desktop and 390 × 844 Chromium QA, delayed-bootstrap skeleton probe, authenticated PDF success/failure fixtures, dark-theme share and attachment checks, and a 16-image visual evidence set
- Current conclusion: logged-in users can reach theme and font controls across the Public Workspace; research history and push center behave consistently across pages; new research, search empty states, recovery loading, attachment positioning/accessibility, focus visibility, contrast, PDF fallback, share preview, and community empty states now have explicit and verified behavior. A same-day evidence review also removed the mobile dark chat's mixed gray/green message palette: page, assistant, user bubble, and text now share one coherent workspace token hierarchy. The local source is ready, but the installed `/Applications/HONE.app` still loads the older deployed `hone-claw.com` bundle because no release or deployment was requested.
- Next entry point: `docs/handoffs/2026-07-27-public-workspace-visual-acceptance-fixes.md`, `packages/app/src/pages/chat.tsx`, and `packages/app/src/components/public-workspace-shell.tsx`

## 2026-07-26

### v0.15.0 Formal Release

- Status: done
- Date: 2026-07-26
- Plan: `docs/archive/plans/v0.15.0-formal-release.md`
- Handoff: `docs/handoffs/2026-07-26-v0.15.0-formal-release.md`
- Decision / ADR: N/A; existing release and Apple artifact contracts remain unchanged
- Related PRs / commits: release commit `a96425bad98ffc08c64c35718bd8b85245d43e51`; annotated tag `v0.15.0`; GitHub Actions run `30207643379`
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`; full workspace check/test excluding Apple clients; Web `294/294`; Edge Worker `45/45`; finance contracts `44/44`; complete CI-safe regressions; user-app test/check; iOS contract; local and CI Apple builds; Release asset/checksum inspection
- Current conclusion: HONE `v0.15.0` is published as a non-draft, non-prerelease GitHub Release with all eight expected assets. The downloaded Apple manifest verified the Universal macOS DMG, iOS Simulator App, and Xcode archive; the DMG passed image/signature checks and both downloaded Apps report `0.15.0` arm64/x86_64 executables. macOS remains ad-hoc signed/not notarized, and iOS remains a Simulator/Xcode distribution because Developer ID and iOS provisioning credentials are not configured.
- Next entry point: `https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.15.0`, then `docs/handoffs/2026-07-26-v0.15.0-formal-release.md` for validation evidence and Apple signing limitations.

### Interactive Business No-Refusal Finalization

- Status: done
- Date: 2026-07-26
- Plan: `docs/archive/plans/interactive-business-no-refusal-finalization.md`
- Handoff: `docs/handoffs/2026-07-26-interactive-business-no-refusal-finalization.md`
- Decision / ADR: `D-2026-07-26-06` and `docs/adr/0004-agent-owned-research-loop.md`
- Related PRs / commits: `75ca1957`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; workspace check/test excluding Apple clients; Agent `135/135`; Channels `680 passed` plus one ignored; Web API `140 passed` plus two ignored; Web `294/294`; Edge Worker `45/45`; finance contracts `43/43`; complete CI-safe regressions; real-host Apple Vision OCR; 501-file immutable deployment manifest and three production replays
- Current conclusion: repeated generic failures came from irreversible early finance-header publication, fixed failure-tail synthesis, non-executable skill misclassification, a path-only text model receiving no image bytes, and a separate non-finance hard refusal. Exact `75ca1957` keeps the answer format but publishes only a complete usable final, gives blocked/read-only failures one same-Agent no-tools answer, supplies local OCR text, removes canned failure copy, and lets ordinary questions answer directly. Production Web/Feishu, cloud authority, storage, ports and auth are healthy; market, CPU and OCR attachment-block canaries each ended once with byte-identical two-row history and zero active chats.
- Next entry point: use the handoff and D-2026-07-26-06. Reopen the fixed image bug on any generic refusal or missing production OCR block; otherwise continue only the independent scheduler entity-guard P2 in the shared ticker plan.

### Interactive Market-Move Explanation Reliability

- Status: done
- Date: 2026-07-26
- Plan: `docs/current-plans/ticker-resolution-architecture.md` (market-move subphase done; shared umbrella remains active only for the scheduler entity-guard P2)
- Handoff: `docs/handoffs/2026-07-26-interactive-market-move-explanations.md`
- Decision / ADR: `D-2026-07-26-01`, `D-2026-07-26-02`, `D-2026-07-26-03`, `D-2026-07-26-05`, and `docs/adr/0004-agent-owned-research-loop.md`
- Related PRs / commits: `27ea2f53`, `cd78375`, `f0281adb`, `e46a6bf4`, `ec06485a`, `12f1a924`, `4139e12c`, `84ca1f21`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; workspace check/test excluding Apple clients; Agent `134/134`; Channels `680/680`; Web API `133 passed, 2 ignored`; Tools `143 passed, 1 ignored`; Web `286/286`; Edge Worker `45/45`; finance contracts `42/42`; complete CI-safe regressions; immutable deployment manifest and four exact production replays
- Current conclusion: the failure class was not a single weak model answer. Exact-header finalization, requested-date/scope preservation, exact-symbol call compatibility, representative broad-market evidence, quote-field consistency, source quality, and repeated research rounds each contributed distinct failures. Exact `84ca1f21` now requires two verified full representative quotes, rejects `quote_short` as complete evidence, matches percentages/exchange/close semantics to structured fields, refuses snippet-only definitive causes, and ends after one source attempt. Four fresh rumor/broad/Friday/HIMS actors completed in `45.597–58.917s` with one successful terminal, no reset/error/partial/generic failure, byte-identical two-row history, correct `2026-07-24` scope/premise handling, honest cause gaps, and zero active chats.
- Next entry point: use `docs/handoffs/2026-07-26-interactive-market-move-explanations.md` for this completed subphase. Continue the still-active umbrella only at `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`.

### Scheduler Entity-Guard Hardening

- Status: done
- Date: 2026-07-26
- Plan: `docs/current-plans/ticker-resolution-architecture.md` (shared umbrella remains active only for live recheck)
- Handoff: `docs/handoffs/2026-07-26-scheduler-entity-guard-hardening.md`
- Decision / ADR: N/A; this is a guard hardening inside the existing ticker-resolution contract
- Related PRs / commits: this change set
- Related runbooks / regressions: `cargo test -p hone-channels scheduler_and_heartbeat_skip_macro_regulatory_and_name_components --lib -- --nocapture`; `cargo test -p hone-channels heartbeat_subject_markers_count_as_security_context --lib -- --nocapture`; `cargo test -p hone-channels scheduled_ticker_subject_is_available_without_parsing_the_envelope --lib -- --nocapture`; `cargo test -p hone-channels operational_checks_and_scheduler_conditions_do_not_become_tickers --lib -- --nocapture`; `cargo test -p hone-channels collision_policy_accepts_real_short_tickers_only_with_strong_binding --lib -- --nocapture`; `cargo check -p hone-channels --tests`
- Current conclusion: scheduler/heartbeat deterministic preflight no longer treats bare non-interactive `TitleCase` company names as closed-form securities, no longer splits institution-name fragments such as `ARK Invest` into ticker `ARK`, and no longer lets `PCE/CPI/GDP/FOMC/NFP/PMI/SEC/FDA/NASA/PDUFA/ARK` enter securities resolution without explicit ticker binding. The bug ledger is updated to code-level `Fixed`, but this automation did not restart runtime processes, so a future live scheduler/heartbeat window still needs to confirm the fix has loaded.
- Next entry point: `docs/handoffs/2026-07-26-scheduler-entity-guard-hardening.md` and `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`

## 2026-07-26

### Whop → HONE International Activation

- Status: done
- Date: 2026-07-26
- Plan: `docs/archive/plans/whop-hone-activation.md`
- Handoff: `docs/handoffs/whop-hone-activation.md`
- Decision / ADR: `D-2026-07-26-04` and `D-2026-07-26-06` in `docs/decisions.md`
- Related PRs / commits: implementation `4632dfa9`; Cloudflare email
  `92cad045`; portable verification `c12e95a6`; current Whop signing and
  production runtime `482c34d54aef4f0d9726acea0b753d751a5973be`
- Related runbooks / regressions: `docs/runbooks/whop-hone-activation.md`; memory 26 focused tests; Web API 3 focused tests; complete workspace check/test excluding Apple clients; Web `292/292`, typecheck, and public build; Edge Worker `45/45` and typecheck; CI-safe regressions; explicit Rust formatting and diff checks; isolated signed-webhook HTTP regression; desktop and `390x844` browser acceptance
- Current conclusion: exact runtime `482c34d5` now has the Cloudflare email
  sender and current raw `ws_...` Whop signing secret configured. Local and
  public no-side-effect signed probes return `200 ignored`; missing or
  body-tampered signatures return `401`. Public activation/email/auth routes,
  authoritative PostgreSQL/R2 storage, ports `8077/8088`, and the single
  Feishu process are healthy. Mainland invite/SMS behavior and Whop-native
  Discord role fulfillment remain independent.
- Next entry point: follow `docs/runbooks/whop-hone-activation.md` for a real
  non-owner purchase → same inbox challenge → `/me`, then cancel/repurchase and
  Discord role acceptance; rotate the chat-exposed webhook secret through
  approved secret management.

## 2026-07-22

### Interactive Finance First-Visible Latency Repair

- Status: done
- Date: 2026-07-22
- Plan: `docs/current-plans/ticker-resolution-architecture.md` (shared umbrella remains active for scheduler entity-guard P2)
- Handoff: `docs/handoffs/2026-07-21-interactive-first-visible-latency.md`
- Decision / ADR: `D-2026-07-21-01`, `D-2026-07-22-01`, and `docs/adr/0004-agent-owned-research-loop.md`
- Related PRs / commits: `b06de76a`, `820a7240`, `2563f7ad`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; Agent `124/124`; Channels `670/670`; Web API `128/128` with two credentialed tests ignored; Tools `143` with one optional test ignored; Web `280/280`; Edge Worker `45/45`; finance contracts `39/39`; complete workspace check/test, CI-safe regressions, formatting/diff checks; immutable deployment manifest and exact production replay
- Current conclusion: the exact incident query previously withheld all visible text for about two minutes because synchronous context work and an unbounded Agent research fan-out preceded a buffered final. Production now ACKs one neutral typed Web-finance line at a safe irreversible boundary, enforces three finance batches plus 24-total/20-DataFetch/6-Web/6-route ceilings, and gives the same Agent a tool-disabled natural final at exhaustion. Exact deployment `2563f7ad` passed cloud/runtime/auth/static health and replayed the query with the first exact line at `179ms`; four model calls, 14 executed tools and two routes ended once at `117.189s`, with no reset/error/partial/failure suffix and byte-identical 8,167-byte visible/persisted output.
- Next entry point: the latency subtask is closed. Continue the shared ticker plan only for the scheduler `800G` / `NAND` / `AST` / `SEC` entity-guard P2; immutable `target/deploy-b06de76a` remains the immediate runtime rollback.
## 2026-07-19

### Public Community Private-R2 Edge Delivery

- Status: done
- Date: 2026-07-19
- Plan: `docs/archive/plans/public-community-edge-delivery.md`
- Handoff: `docs/handoffs/2026-07-19-public-community-edge-delivery.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-19-09-deliver-authenticated-community-archives-from-private-r2-at-the-edge`
- Related PRs / commits: `385e35b0`, `100f5608`; docs-only follow-up `cb796cce`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md#public-community-private-r2-edge-rollout`; workspace check/test excluding Apple clients; CI-safe regressions; Web `280/280`, typecheck, and discovery-off/on builds; Worker `45/45`, typecheck, frozen install, and Wrangler dry-run; publisher `8/8`; production snapshot dry-run/apply/idempotent dry-run
- Current conclusion: the authenticated community edge path is implemented behind backend, Worker, and Pages gates while every legacy API remains available. The private production R2 snapshot contains `662` contents, `833` resources, `719` edge descriptors, `34` feed pages, and `754` publication objects; the final dry-run reports `existing_objects=754`, `would_write=0`, `no_op=true`, and zero conflicts. The exact-route Worker is deployed as version `e01c1603-7c34-476a-b63b-33ac74244108` with private `honeclaw` binding, no secret, and fail-closed `503`. Commits are pushed and Pages rebuilt automatically with discovery compiled out. Exact backend build `100f5608` is staged but not running; backend configuration, user traffic, and the old process remain unchanged.
- Next entry point: let the external supervisor restart the prepared `target/deploy-100f5608` build, require the Step 1 cloud-health and `mode=off` `200 enabled=false` probes, and only then proceed to Step 4. Step 5 is complete unless the canonical archive changes.

## 2026-07-17

### RKLB Entity, Market Data, And Deep Valuation Repair

- Status: done
- Date: 2026-07-17
- Plan: `docs/archive/plans/rklb-data-resolution-regression.md`
- Handoff: `docs/handoffs/2026-07-17-rklb-entity-resolution-repair.md`
- Decision / ADR: `D-2026-07-16-01`, `D-2026-07-17-01`, `D-2026-07-17-02`, and `D-2026-07-17-03` in `docs/decisions.md`
- Related PRs / commits: `ff3852c3`, `7d14c87f`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; `hone-channels` 569/569; `hone-tools` 136/136 with one expected ignored test; finance contracts 24/24; live RMBS/RKLB/NBIS/INTL/BTCUSD and mixed-market provider probes; GitHub CI `29570821727`; three final production Web/SSE RKLB cases; controlled restart and runtime/storage/auth health checks
- Current conclusion: FMP/DataFetch was healthy. The failure came from an explicit RKLB ticker still depending on auxiliary prose parsing, provider-sensitive alias rewriting, missing ticker-binding syntax, and then safe-range wording falling through to the quote-only contract. Exact tickers now keep their provider query and bypass auxiliary extraction when complete; semantic-empty search has a strict same-symbol profile fallback; safe-range and entry-decision questions require the deep nine-section contract. Final production returned Rocket Lab USA, Inc. / RKLB at `67.35 USD` with all nine sections, one answer/terminal, zero reset/error, and zero active chats.
- Next entry point: `crates/hone-channels/src/investment_response_guard.rs` and `docs/handoffs/2026-07-17-rklb-entity-resolution-repair.md`

### Investment Response Template And Deterministic Repair

- Status: done
- Date: 2026-07-17
- Plan: `docs/archive/plans/investment-response-template-regression.md`
- Handoff: `docs/handoffs/2026-07-17-investment-response-contract-repair.md`
- Decision / ADR: `D-2026-07-15-02`, `D-2026-07-17-01`, and `D-2026-07-17-02` in `docs/decisions.md`
- Related PRs / commits: `922007fa`, `d5f1dca0`, `3880d623`, `ce25d0ea`, `010dbae9`, `b0f50a77`, `d75451c3`, `ae8ebc11`, `340b9ee1`, `24c4c48d`, `dea3303d`, `4869ac5c`, `b4874a2c`, `020c678a`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; DataFetch `27/27`; `hone-channels` `565/565` in a fresh target; full workspace check/test excluding Apple clients; Web typecheck and `265/265`; finance contracts `24/24`; CI-safe suite; GitHub CI run `29547741054`; live provider regression; four initial production Web/SSE E2E turns plus a final post-restart RMBS turn; controlled restart and health probes; Cloudflare Pages deployment `53103ef2-eb25-4caa-aafc-f2f8c7a42afd`
- Current conclusion: `71a4498e` reduced the 291-line investment contract to 36 lines and explicitly discouraged the fixed response templates. FMP/DataFetch and Tavily were healthy; RMBS/NBIS/INTL failures came from entity/asset routing and a second whole-answer repair path. The service now owns the first Beijing data time, exact entity, same-symbol current/extended quote and full asset template; unknown/persistent tool traces block automatic replay paths, and covered explicit portfolio/trade/deep-research intents remain execute-once even without a trace; extended-hours and historical/OHLC prose cannot launder an unverified price. The final runtime is supervisor/backend `23199`/`23210`, started at Beijing `09:39`, with healthy Postgres/S3, zero local durable dependencies and zero active chats. The final RMBS turn returned `101.42 USD`, all nine required sections, one terminal stream and no reset/error or false live-data denial.
- Next entry point: `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-channels/src/tool_trace.rs`, and `docs/handoffs/2026-07-17-investment-response-contract-repair.md`

## 2026-07-16

### Investment Guard Scheduler Routing Fix

- Status: done
- Date: 2026-07-16
- Plan: `docs/archive/plans/investment-guard-scheduler-routing-fix.md`
- Handoff: `docs/handoffs/2026-07-16-investment-guard-scheduler-routing-fix.md`
- Decision / ADR: follow-up scope clarification to `D-2026-07-15-03` in `docs/decisions.md`
- Related PRs / commits: this change set; follows incomplete mitigation `c776b808`
- Related runbooks / regressions: investment guard unit tests, full channel library tests, 12 CI-safe finance contracts, `hone-cli` build, isolated live scheduled-envelope probe, and runtime/API/storage/channel health checks
- Current conclusion: `repeat=daily/trading_day` was incorrectly parsed as ticker `REPEAT` because the direct single-stock guard scanned scheduler envelopes. Scheduler and heartbeat envelopes now bypass that interactive guard, generic report acronyms and multi-security inputs cannot masquerade as a single ticker, and search requires an exact symbol match. A live envelope containing `repeat=daily` plus “财报分析” completed successfully without any market-data preflight.
- Next entry point: `crates/hone-channels/src/investment_response_guard.rs` and `docs/handoffs/2026-07-16-investment-guard-scheduler-routing-fix.md`

## 2026-07-15

### Deep Single-Stock Evidence And Response Contract

- Status: done
- Date: 2026-07-15
- Plan: `docs/archive/plans/response-contract-enforcement.md`
- Handoff: `docs/handoffs/2026-07-15-response-contract-enforcement.md`
- Decision / ADR: `D-2026-07-15-01`, `D-2026-07-15-02`, and `D-2026-07-15-03` in `docs/decisions.md`
- Related PRs / commits: `c29de55c`
- Related runbooks / regressions: 117 core tests, 7 function-calling agent tests, full channel library tests, 12 CI-safe finance automation contracts, `hone-cli` build, and isolated live NBIS Web regression
- Current conclusion: the real NBIS incident was caused by the model ignoring a full prompt that had already been injected, not by prompt omission. Canonical `soul.md` and its runtime sync are restored, non-admin native-runner configuration routes through the actor-bound safety runner, and deep single-stock turns now prefetch same-symbol quote/profile/financial/news/calendar evidence before enforcing a nine-section final answer. Incomplete drafts are reset and retried once, then fail closed. The exact live question completed successfully with all sections after restart.
- Next entry point: `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/core.rs`, and `docs/handoffs/2026-07-15-response-contract-enforcement.md`

## 2026-07-13

### Public Community Original Assets And Navigation Repair

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-community-assets-navigation-fix.md`
- Handoff: `docs/handoffs/2026-07-12-public-community-readonly.md`
- Decision / ADR: N/A; source access controls remain the hard download boundary and immutable full-SHA objects remain the rollback boundary
- Related PRs / commits: `879e9722`, `af3cb605`, `7ab36682`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 118 core tests, 106 Web API tests with two credentialed tests ignored, 67 CLI tests, 242 Web tests, TypeScript check, public production build, 11 CI-safe regression scripts, desktop/390px browser QA, and the 651-file magic/size/SHA/OOXML audit
- Current conclusion: this follow-up supersedes the file/image counts in the 2026-07-12 community entry. The complete archive now has 649 content rows and 818 resources: 53 original-resolution images plus 765 file resources. Of those files, 651 verified originals (2,614,811,800 bytes) are stored in immutable R2 objects and linked from PG, 113 remain explicitly source-protected, and only resource 834 remains unresolved after independent visible-UI search. Desktop/mobile navigation includes a compact first-class Community tab, the backend runs the current `0.14.1` code, and production Pages serves `assets/index-BB8Wrwbl.js`.
- Next entry point: `docs/handoffs/2026-07-12-public-community-readonly.md`, `bins/hone-cli/src/cloud.rs`, `crates/hone-web-api/src/routes/public_community.rs`, and `packages/app/src/pages/public-community.tsx`

## 2026-07-12

### Public Read-only Community

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/public-community-readonly.md`, `docs/archive/plans/public-community-deployment-qa.md`
- Handoff: `docs/handoffs/2026-07-12-public-community-readonly.md`
- Decision / ADR: N/A; source-protected resources remain a hard no-download boundary
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, cloud doctor schema apply, 218 Rust tests (2 credentialed ignored), workspace check, 236 Web tests, TypeScript check, public production build, CI-safe regression suite, desktop/390px browser QA
- Current conclusion: the 616-row user-authorized archive is deployed as the shared authenticated `/community` timeline across Web/macOS/iOS. Content is newest-first and read-only; every source post remains one row with ordered media. The runtime is `0.14.1`, Cloudflare Pages serves `index-D-q3AOum.js`, R2-backed passive images/PDF use the hardened private preview route, and 764 protected source files deliberately remain metadata-only.
- Next entry point: `crates/hone-web-api/src/routes/public_community.rs` and `packages/app/src/pages/public-community.tsx`

### Public Chat Native Runner Streaming

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/public-chat-native-runner-streaming.md`
- Handoff: `docs/handoffs/2026-07-12-public-chat-startup-experience.md`
- Decision / ADR: `D-2026-07-12-03` in `docs/decisions.md`
- Related PRs / commits: `6d5075a4`, this CI follow-up change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 13 LLM tests, 7 function-calling tests, 500 channel tests, 101 Web API tests with two credentialed tests ignored, 218 frontend tests, frontend typecheck/public build, workspace all-target check excluding Apple clients, production asset/API/runtime checks
- Current conclusion: Codex/OpenCode ACP retains native ACP message chunks for trusted administrators, while ordinary public users remain actor-isolated and now receive real upstream OpenAI-compatible/OpenRouter tool-capable SSE through the strict function-calling runner. Fragmented parallel tool calls are assembled by index, internal reasoning stays hidden, transient tool preambles reset in place, final persistence remains one assistant turn, and the public client frame-batches deltas in the existing thinking card.
- Next entry point: `crates/hone-llm/src/provider.rs`, `agents/function_calling/src/lib.rs`, `crates/hone-channels/src/runners/tool_reasoning.rs`, and `packages/app/src/pages/chat.tsx`

### Server-owned Finance Calendar Images

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/server-owned-finance-calendar-images.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: `D-2026-07-12-02` in `docs/decisions.md`
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 216 frontend tests, frontend typecheck, public production build, 100 Web API tests with two credentialed tests ignored
- Current conclusion: each new finance-calendar message now persists validated desktop and mobile PNG paths as structured session metadata. Public bootstrap/history selects one path from the request User-Agent, legacy two-marker messages are selected server-side, and the client renders one stable authenticated image URL without calendar refetching, Canvas rebuilding, blob replacement, or source swapping. Image responses use private immutable browser caching.
- Next entry point: `crates/hone-web-api/src/routes/public_finance_calendar.rs`, `crates/hone-web-api/src/routes/history.rs`, and `packages/app/src/components/finance-calendar-message.tsx`

### Public Chat Startup Experience

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/public-chat-startup-experience.md`, `docs/archive/plans/public-chat-mobile-gesture-share-polish.md`
- Handoff: `docs/handoffs/2026-07-12-public-chat-startup-experience.md`
- Decision / ADR: N/A; public auth and actor-scoped history ownership remain unchanged
- Related PRs / commits: `22af864b`, `2f0c0e9e`, this follow-up change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 216 frontend tests, frontend typecheck, 96 Web API tests from the pagination phase, public production build, 390 x 844 browser QA
- Current conclusion: `/chat` now uses one recovery shell, starts from the newest 20 projected messages, cursor-loads older history upward, reserves media layout, represents every assistant run as one in-thread card, blocks accidental browser-level pinch outside the controlled calendar viewer, and centers user queries inside exported share bubbles.
- Next entry point: `docs/handoffs/2026-07-12-public-chat-startup-experience.md`

### Agent And Data Security Hardening

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/agent-data-security-hardening.md`
- Handoff: `docs/handoffs/2026-07-12-agent-data-security-hardening.md`
- Decision / ADR: `D-2026-07-12-01` in `docs/decisions.md`
- Related PRs / commits: `dbabbe77`, `a99bf096`
- Related runbooks / regressions: 495 channel tests, 115 core tests, 121 memory tests, 123 tool tests, 95 Web API tests, 12 Discord tests, 211 frontend tests, workspace check, CI-safe regression suite, cloud doctor, production/origin/CORS/runtime probes
- Current conclusion: non-admin actors can no longer use host-capable ACP/CLI runners and instead use actor-bound function calling; runtime/config/sandbox permissions and skill child environments are owner-only/secret-free; public CORS and actor-key data isolation are verified; production dependency alerts fell from 10 to two Tauri-only residual alerts. Admin ACP remains a trusted boundary and all credentials should be rotated.
- Next entry point: `docs/handoffs/2026-07-12-agent-data-security-hardening.md`

## 2026-07-11

### Public Web Visual Architecture And Finance Calendar V4

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/public-web-visual-architecture-refactor.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: `D-2026-07-11-03` in `docs/decisions.md`
- Related PRs / commits: `5b7b1d67`, `a3e0dbaa`
- Related runbooks / regressions: 211 frontend tests, typecheck, public build, direct Canvas dense-fixture review at 1500 x 2668 and 390px, 390 x 844 production browser QA, production asset/route/runtime checks
- Current conclusion: public visual ownership is split into foundation, shared component polish, chat shell, and component-local artifact layers. Mobile finance-calendar PNGs use one Canvas 2D renderer when a calendar is created, eliminating iOS html2canvas glyph clipping. The 2026-07-12 server-owned image follow-up supersedes client-side lazy upgrades: history now receives one backend-selected persisted image and never rebuilds v1-v3 artifacts in the viewer.
- Next entry point: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

### Mobile Finance Calendar Dual Layout And Gestures

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/mobile-finance-calendar-dual-layout.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: superseded in part by `D-2026-07-12-02` in `docs/decisions.md`; creation still renders/uploads both variants, while persistence and history selection are backend-owned
- Related PRs / commits: `2a6e7572`, `a4af378d`, `1a72b918`, `6ab39ee3`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 209 frontend tests, 7 focused Rust tests, 14 focused migration tests, typecheck, public build, rendered 390 x 844 portrait/fit/300 percent/legacy-upgrade reviews, 15-event editorial design review, production asset/route/origin checks
- Current conclusion: new finance-calendar messages carry independently validated desktop and mobile PNGs. Since the 2026-07-12 follow-up, backend metadata owns both paths and history selects one by device; the viewer no longer lazily rebuilds or replaces portrait blobs. The controlled viewer gestures and the HONE monthly-brief artifact composition remain in place.
- Next entry point: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

### Mobile Finance Calendar And Navigation Polish

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/mobile-finance-calendar-nav-polish.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: N/A; APIs, persistence, and module boundaries are unchanged
- Related PRs / commits: `31081106`, `e95b1049`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, `bun run typecheck:web`, `bun run test:web` (207 passed), `bun run build:web:public`, local and production 390 x 844 browser QA
- Current conclusion: hone-claw.com now serves a bounded fit/125/150/200 percent finance-calendar viewer with fixed controls; Safari page zoom no longer combines with calendar canvas zoom. Production uses `index-D4wSdzNX.js` / `chat-ByxolQgf.js`; core routes and the public API proxy passed smoke checks.
- Next entry point: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

### HONE Client Brand And iOS Release

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/hone-client-brand-ios-release.md`
- Handoff: `docs/handoffs/2026-07-11-hone-client-brand-ios-release.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-02-use-one-hone-brand-and-remote-boundary-for-public-apple-clients`
- Related PRs / commits: `e33a467a`, `dc889ffa`, `aa32c818`, `6a14e3e7`; tag `v0.13.0`
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `docs/runbooks/public-user-ios-app.md`, `bash tests/regression/ci/test_hone_ios_contract.sh`, Apple Clients run `29139331210`, Release run `29139409377`
- Current conclusion: Public Web, focused macOS, and standalone iOS clients now use one uppercase HONE brand and polished navigation language. v0.13.0 ships a Universal macOS DMG, Xcode-built iOS Simulator app, complete iOS Xcode project, and Apple checksum manifest; device IPA/TestFlight and notarized macOS distribution still require Apple signing credentials.
- Next entry point: `docs/handoffs/2026-07-11-hone-client-brand-ios-release.md`

### Standalone Public User macOS App

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/standalone-public-user-macos-app.md`
- Handoff: `docs/handoffs/2026-07-11-standalone-public-user-macos-app.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `cargo test -p hone-user-app`, `cargo check -p hone-user-app`, `bash scripts/build_user_app.sh`, Universal architecture/bundle/signature inspection, packaged `/chat` launch smoke
- Current conclusion: Hone now ships a focused Universal macOS user client that enters production `/chat` through a polished local startup shell and intentionally excludes local runtime, ACP, MCP, channels, config, skills, and data directories. The 16 MB app / 5.7 MB DMG are ad-hoc signed on this machine and require Developer ID signing plus notarization before public distribution.
- Next entry point: `docs/runbooks/public-user-macos-app.md`

## 2026-07-10

### Web Scheduled Push Mobile Hotfix

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/web-scheduled-push-mobile-hotfix.md`
- Handoff: `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-10-01-project-web-scheduled-results-into-a-durable-push-inbox`
- Related PRs / commits: `383058fe`
- Related runbooks / regressions: `cargo test -p hone-web-api --lib`, targeted memory legacy-push tests, `bun run test:web`, `bun run typecheck:web`, `bun run build:web:public`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`, actor-scoped HTTP backfill smoke
- Current conclusion: Production mobile blank message shells were caused by deploying the scheduled-push backend without the matching public bundle. The mobile push bell now sits outside the hidden desktop nav, and pre-upgrade scheduled messages are lazily imported into the durable inbox; the affected actor imported 79 messages in 93ms without clearing existing sessions or read state. Cloudflare Pages now serves `index-BeqwKSm5.js`, and the production Worker returns the same 79-message inbox through authenticated public API traffic.
- Next entry point: `crates/hone-web-api/src/routes/public_pushes.rs`, `packages/app/src/components/public-nav.tsx`, and `packages/app/src/components/public-push-center.tsx`

### Web Scheduled Push Inbox

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/web-scheduled-push-inbox.md`
- Handoff: `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-10-01-project-web-scheduled-results-into-a-durable-push-inbox`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api --lib`, `cargo check -p hone-channels -p hone-web-api`, `bun run test:web`, `bun run typecheck:web`, `bun run build:web:public`, local PostgreSQL + HTTP mark-through smoke
- Current conclusion: Web scheduled results now render as compact summary cards, open full content on demand, persist actor-scoped read state, and collect in a rail/mobile push center with an aggregate unread dot; reading the latest push clears the dot while Feishu and other channels remain unchanged.
- Next entry point: `crates/hone-web-api/src/routes/public_pushes.rs`, `memory/src/cron_job/history.rs`, and `packages/app/src/components/public-push-center.tsx`

### Public Finance Calendar Polish

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/public-finance-calendar-polish.md`
- Handoff: `docs/handoffs/2026-06-29-public-finance-calendar.md`
- Decision / ADR: N/A; module boundaries and upload/send architecture are unchanged
- Related PRs / commits: N/A
- Related runbooks / regressions: finance-calendar helper smoke and changed TS/TSX syntax parse passed; `bash tests/regression/run_ci.sh` passed available checks before stopping at missing `cargo`; Rust/Bun suites remain pending in a provisioned environment
- Current conclusion: the public finance calendar now opens on the current month with an immediate image preview, compact month navigation, explicit loading/error/source states, a redesigned 1080 x 1350 share image, and 17 verified July 2026 macro events in Beijing time.
- Next entry point: `packages/app/src/pages/chat.tsx`, `packages/app/src/components/finance-calendar-card.tsx`, and `crates/hone-web-api/src/routes/public_finance_calendar.rs`

## 2026-06-29

### Public Finance Calendar

- Status: done
- Date: 2026-06-29
- Plan: `docs/archive/plans/public-finance-calendar.md`
- Handoff: `docs/handoffs/2026-06-29-public-finance-calendar.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api finance_calendar` and `bun run test:web` still need to be rerun where Rust/Bun are installed; `bash scripts/ci/check_fmt_changed.sh` skipped because no base ref was discoverable in this workspace.
- Current conclusion: public chat now includes a “我的财经日历” quick action that fetches actor-scoped macro/FMP earnings data, renders a month-view PNG in the browser, uploads it through the current-user public upload root, appends an assistant image message, and broadcasts `push_message`.
- Next entry point: `crates/hone-web-api/src/routes/public_finance_calendar.rs`, `packages/app/src/pages/chat.tsx`, and `packages/app/src/components/finance-calendar-card.tsx`

## 2026-06-24

### ACP `hone-mcp` Process Cleanup

- Status: done
- Date: 2026-06-24
- Plan: `docs/current-plans/acp-runtime-refactor.md`
- Handoff: `docs/handoffs/2026-06-24-acp-hone-mcp-process-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels acp_child_guard_terminates_grandchild_process_group -- --nocapture`, `cargo test -p hone-channels codex_acp -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: ACP CLI children now run in an isolated process group and are cleaned up through `AcpChildGuard`, so `codex_acp` / `opencode_acp` success, error, and timeout paths terminate stdio MCP grandchildren such as `hone-mcp` instead of leaving local process leaks.
- Next entry point: `crates/hone-channels/src/runners/acp_common/process.rs`

## 2026-06-21

### Feishu Direct Cron Result Recovery

- Status: done
- Date: 2026-06-21
- Plan: N/A, single-session active bug fix did not need dynamic plan tracking
- Handoff: `docs/handoffs/2026-06-21-feishu-direct-cron-tool-result-recovery.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels finalize_agent_response_recovers_cron_job_ --lib -- --nocapture`, `cargo test -p hone-channels finalize_agent_response_recovers_portfolio_confirmation --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: Feishu direct 定时任务治理相关 turn 在真实 `cron_job` 工具已经返回结果时，最终回复现在会优先恢复任务列表、创建/更新确认或删除确认，而不是继续退化成过渡句或通用“定时任务管理暂时不可用”提示。
- Next entry point: `docs/bugs/feishu_direct_cron_management_tool_unavailable_internal_state_exposed.md`

## 2026-05-31

### Cloud PG / OSS Runtime Migration

- Status: done
- Date: 2026-05-31
- Plan: `docs/archive/plans/cloud-pg-oss-runtime-migration.md`
- Handoff: `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo test --offline -p hone-memory company_profile --lib`, `cargo test --offline -p hone-event-engine mainline_distill --lib`, `cargo test --offline -p hone-channels normalize_local_image_references --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-event-engine -p hone-channels -p hone-web-api -p hone-cli --tests`, `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json`, `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`
- Current conclusion: `cloud.mode=cloud` now uses PG/R2 for all current runtime durable dependencies covered by cloud doctor: sessions, web auth, quota, cron, skill registry, notification prefs, portfolio, LLM audit, company profiles, uploads/attachments, generated images, and cloud document indexing. The final doctor result is `local_durable_dependency_count=0`; local mode remains compatible and reports 0 cloud durable dependencies.
- Next entry point: `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md` for operational notes and `docs/runbooks/backend-deployment.md` for migration commands.

## 2026-05-27

### v0.12.4 Formal Release

- Status: done
- Date: 2026-05-27
- Plan: N/A, single-session formal release execution
- Handoff: `docs/handoffs/2026-05-27-v0.12.4-release.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bash tests/regression/run_ci.sh`, `bash scripts/prepare_release_notes.sh v0.12.4 /tmp/release-notes-v0.12.4.md`
- Current conclusion: `v0.12.4` ships the Cloud PG / OSS runtime config slice, public upload OSS proxy path, scheduler commodity guard false-positive fix, Feishu/external error diagnostics, guarded live smoke wrappers, refreshed architecture SVG, and release notes.
- Next entry point: `docs/releases/v0.12.4.md`, then `docs/archive/plans/cloud-pg-oss-runtime-migration.md` for the completed cloud storage follow-up record.

## 2026-05-23

### Heartbeat Structured Status Hardening

- Status: done
- Date: 2026-05-23
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-23-heartbeat-structured-status-hardening.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`, `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: heartbeat status parsing now tolerates common nonstandard noop/triggered status aliases and complete internal-only no-op reasoning, while the prompt blocks tool/task/profile configuration fragments as final output.
- Next entry point: `docs/bugs/scheduler_heartbeat_unknown_status_silent_skip.md`

### Heartbeat Context Overflow Status Boundary

- Status: done
- Date: 2026-05-23
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-23-heartbeat-context-overflow-status.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels heartbeat_context_overflow_error_is_not_classified_as_noop --lib -- --nocapture`, `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: heartbeat context-window overflow is no longer treated as a legitimate noop; it is classified as `context_window_overflow` and lands as `execution_failed + skipped_error` for auditability.
- Next entry point: `docs/bugs/scheduler_heartbeat_context_window_limit_no_recovery.md`

### Heartbeat Max-Iterations Budget

- Status: done
- Date: 2026-05-23
- Plan: N/A, single active-bug fix did not need dynamic plan tracking
- Handoff: `docs/handoffs/2026-05-23-heartbeat-max-iterations-budget.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels heartbeat_prompt_requires_noop_json_for_contract_conflicts --lib -- --nocapture`, `cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`, `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: heartbeat auxiliary function-calling now gets 18 iterations instead of 10, and the heartbeat prompt explicitly requires minimal tool use so sector/multi-symbol heartbeat jobs are less likely to burn their whole budget confirming noop.
- Next entry point: `docs/bugs/scheduler_heartbeat_iteration_exhaustion_skips_alert.md`

## 2026-05-21

### Public Blog Module

- Status: done
- Date: 2026-05-21
- Plan: `docs/archive/plans/public-blog-module.md`
- Handoff: `docs/handoffs/2026-05-21-public-blog-module.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bun --filter @hone-financial/app test`, `bun --filter @hone-financial/app typecheck`, `HONE_APP_OUT_DIR=dist-public HONE_APP_SURFACE=public bun --filter @hone-financial/app build`
- Current conclusion: hone-claw.com public surface now has a bilingual static Blog index and Rust article route, with navigation/homepage entry points and local Chinese/English article images copied from the provided source links.
- Next entry point: `packages/app/src/lib/public-blog.ts`, `packages/app/src/pages/public-blog.tsx`, and `packages/app/src/pages/public-blog-post.tsx`

### Public Blog Share Metadata

- Status: done
- Date: 2026-05-21
- Plan: `docs/archive/plans/public-blog-share-metadata.md`
- Handoff: `docs/handoffs/2026-05-21-public-blog-module.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bun --filter @hone-financial/app test`, `bun --filter @hone-financial/app typecheck`, `HONE_APP_OUT_DIR=dist-public HONE_APP_SURFACE=public bun --filter @hone-financial/app build`
- Current conclusion: Blog article pages now show both Chinese and English titles, include a card to switch language versions, inject article-specific metadata at runtime, and use Cloudflare Worker HTML metadata injection for crawlers that do not execute the SPA. README top navigation and Rust-stack sections now link to the Blog with matching language labels.
- Next entry point: `packages/app/public/_worker.js`, `packages/app/src/pages/public-blog-post.tsx`, and `README_ZH.md`

## 2026-05-20

### Heartbeat Mimo 429 Key-Pool Fallback

- Status: done
- Date: 2026-05-20
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-20-heartbeat-mimo-429-key-pool.md`
- Decision / ADR: N/A
- Related PRs / commits: GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)
- Related runbooks / regressions: `cargo test -p hone-llm chat_with_tools_falls_back_to_next_key_after_http_429 -- --nocapture`, `cargo test -p hone-channels heartbeat_provider_429_quota_error_is_classified --lib -- --nocapture`
- Current conclusion: OpenAI-compatible non-streaming routes now honor provider key pools for non-OpenRouter profiles, so a single exhausted mimo key no longer drops the whole heartbeat batch when fallback keys are configured.
- Next entry point: `docs/bugs/scheduler_heartbeat_mimo_429_quota_exhausted.md`

## 2026-05-12

### Public SMS Verification Login

- Status: done
- Date: 2026-05-12
- Plan: `docs/archive/plans/public-sms-login.md`
- Handoff: `docs/handoffs/2026-05-12-public-sms-login.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth::tests::active_invite_user_by_phone_is_sms_login_whitelist`, `cargo test -p hone-memory web_auth::tests::record_tos_acceptance_updates_public_login_terms`, `cargo test -p hone-web-api aliyun_sms::tests`, `cargo check -p hone-web-api`, `bun run --cwd packages/app typecheck`, `bun run --cwd packages/app test:e2e -- --project=public public-sms-login.spec.ts`, optional live SMS smoke `HONE_ALIYUN_SMS_LIVE_PHONE=13871396421 cargo test -p hone-web-api aliyun_sms::tests::live_send_verify_code_smoke -- --ignored --nocapture`
- Current conclusion: 用户端登录已切换为手机号 + 阿里云短信验证码；管理端现有 Web invite 用户手机号作为白名单来源，旧邀请码仅保留为兼容管理字段。
- Next entry point: `crates/hone-web-api/src/aliyun_sms.rs`, `crates/hone-web-api/src/routes/public.rs`, and `packages/app/src/components/public-login-form.tsx`

## 2026-05-11

### LLM Profile Registry POC

- Status: done
- Date: 2026-05-11
- Plan: N/A, single-session POC did not need active plan tracking
- Handoff: `docs/handoffs/2026-05-11-llm-profile-poc.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `tests/regression/manual/test_llm_profile_poc.sh`, `cargo test -p hone-core config::tests`, `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- Current conclusion: The proposed `llm.providers` + `llm.profiles` shape can parse model profiles with `reasoning`, `response_format`, and other generation params, and OpenRouter accepted a live profile-derived request with `reasoning_present=true`.
- Next entry point: Runtime migration is tracked in `docs/archive/plans/llm-profile-runtime-migration.md`.

### LLM Profile Runtime Migration

- Status: done
- Date: 2026-05-11
- Plan: `docs/archive/plans/llm-profile-runtime-migration.md`
- Handoff: `docs/handoffs/2026-05-11-llm-profile-poc.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core config::tests`, `cargo check -p hone-channels --tests`, `cargo check -p hone-web-api --tests`, `cargo test -p hone-llm resolver`, `cargo test -p hone-event-engine global_digest_llm_providers_can_be_wired_per_stage`, `cargo test -p hone-web-api validate_global_digest`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop --bin hone-desktop sidecar`, `bun run typecheck:web`, `bun run test:web`, `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- Current conclusion: `llm.providers` + `llm.profiles` is now a runtime-supported profile registry for event-engine and auxiliary LLM paths; Settings UI can edit profile routing and profile params; legacy OpenRouter/Auxiliary fields remain fallback-compatible.
- Next entry point: `crates/hone-llm/src/resolver.rs`, `crates/hone-web-api/src/lib.rs`, and `packages/app/src/pages/settings.tsx`

### LLM Config Env Removal

- Status: done
- Date: 2026-05-11
- Plan: `docs/archive/plans/llm-config-env-removal.md`
- Handoff: `docs/handoffs/2026-05-11-llm-profile-poc.md`
- Decision / ADR: `docs/decisions.md#d-2026-05-11-01-make-llm-credentials-config-only`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core config::tests`, `cargo test -p hone-llm resolver`, `cargo test -p hone-cli mutations`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop --bin hone-desktop sidecar`, `bun run test:web`, `bun run typecheck:web`, `cargo run -p hone-cli -- config validate --json`, `cargo run -p hone-cli -- status --json`, `cargo run -p hone-cli -- probe --channel cli --user-id cli_smoke --query '只输出 HONE_CLI_LLM_OK' --show-events false`, `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- Current conclusion: LLM credentials are now config-only. Runtime no longer consumes `api_key_env` or parent-process `*_API_KEY` fallback for LLM provider/profile/auxiliary paths; CLI/Desktop OpenRouter writes now target `llm.providers.openrouter.api_keys`, while legacy `llm.openrouter.*` remains a config-only fallback/migration path.
- Next entry point: `crates/hone-core/src/config/agent.rs`, `crates/hone-llm/src/resolver.rs`, and `config.example.yaml`

## 2026-05-10

### Source CLI Start And Launch Retirement

- Status: done
- Date: 2026-05-10
- Plan: `docs/archive/plans/source-cli-start-retire-launch.md`
- Handoff: `docs/handoffs/source-cli-start-retire-launch-2026-05-10.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `docs/runbooks/source-web-startup.md`, `docs/runbooks/desktop-dev-runtime.md`, `cargo test -p hone-cli start`, `bash tests/regression/ci/test_source_cli_start_contract.sh`, `bash tests/regression/ci/test_install_hone_cli_path_resolution.sh`, CLI channel configuration smoke, source startup smoke with `/api/meta` on port `19077`, `cargo test -p hone-cli`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bun run typecheck:web`, `bun run test:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: Source checkout startup now uses `cargo run -p hone-cli -- start --build`, installed users continue with packaged `hone-cli start`, active docs no longer recommend source launcher flows, and the previous channel configuration changes were verified through real CLI commands against a temporary config.
- Next entry point: `docs/runbooks/hone-cli-install-and-start.md` for install/source startup and `docs/runbooks/desktop-dev-runtime.md` for desktop dev lanes.

### Channel Delivery Config Borrowing

- Status: done
- Date: 2026-05-10
- Plan: `docs/archive/plans/channel-delivery-config-borrowing.md`
- Handoff: `docs/handoffs/channel-delivery-config-borrowing-2026-05-10.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-tools cron_job_tool_add_preserves_origin_channel_target`, `cargo test -p hone-cli build_channel_mutations_supports_allowlists`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop desktop_channel_settings`, `cargo test -p hone-cli`, `bun run test:web`, `cargo test -p hone-memory channel_target`, `cargo test -p hone-scheduler scheduler_records_missing_channel_target_without_dispatching`, `cargo test -p hone-cli cli_parses_channels_targets_command`, `cargo test -p hone-memory`, `cargo test -p hone-scheduler`, `cargo test -p hone-web-api cron`, `bun run typecheck:web`, `cargo check --workspace --all-targets --exclude hone-desktop`
- Current conclusion: Hermes-style channel improvements were borrowed without adding platforms or a `home_channel` default. Honeclaw now keeps origin-bound delivery, exposes existing allowlists / `chat_scope` / iMessage `target_handle` through CLI and Desktop/Web settings, rejects or records missing scheduled delivery targets deterministically, and provides a typed cron-backed channel-target directory through `hone-cli channels targets`.
- Next entry point: Add a Web/Desktop selector backed by `CronJobStorage::list_channel_targets()` if users need clickable target discovery; do not introduce `home_channel` unless a separate no-origin system task flow is designed.

## 2026-05-09

### Event Engine Poller Timeout Boundary

- Status: done
- Date: 2026-05-09
- Plan: `docs/archive/plans/event-engine-poller-timeout-boundary.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine spawner::tests --lib -- --nocapture`, `cargo test -p hone-event-engine pollers::earnings_surprise::tests::quality_review_applies_successful_earnings_event --lib -- --nocapture`, `cargo test -p hone-event-engine --lib`, `cargo check -p hone-event-engine --tests`, changed-file `rustfmt --edition 2024 --check`
- Current conclusion: event-engine unified poller ticks now have a bounded timeout, so a stuck `poll().await` / `run_once().await` records a failed tick and releases the loop for the next scheduled cadence instead of suppressing `poller ok` indefinitely
- Next entry point: `docs/bugs/archive/event_engine_poller_cadence_stall_without_restart.md`

### Event Engine Mainline Distill Token Cap

- Status: done
- Date: 2026-05-09
- Plan: `docs/archive/plans/event-engine-mainline-distill-token-cap.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api mainline_distill_uses_short_completion_budget --lib -- --nocapture`, `cargo check -p hone-web-api --tests`, changed-file `rustfmt --edition 2024 --check`
- Current conclusion: mainline distill cron now uses its own OpenRouter provider capped at 1200 completion tokens instead of inheriting global `llm.openrouter.max_tokens`, closing the HTTP 402 preauthorization failure for short investment-mainline summaries
- Next entry point: `docs/bugs/event_engine_mainline_distill_openrouter_402.md`

## 2026-05-08

### Event-engine Push Quality Hardening

- Status: done
- Date: 2026-05-08
- Plan: `docs/archive/plans/event-engine-push-quality-hardening.md`
- Handoff: `docs/handoffs/2026-04-23-event-engine-push-quality.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine --lib`, `cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, changed-file `rustfmt --edition 2024 --check`; full `cargo fmt --all -- --check` currently blocked by unrelated formatting debt
- Current conclusion: 基于近期 event review 与 POC 结论，event engine 已补 analyst 同源文章 fanout 降噪、RSS 标题级保守实体链接，以及 Zacks 泛化模板回归证明；本轮没有新增 LLM 调用或 summary/body 宽匹配
- Next entry point: `docs/handoffs/2026-04-23-event-engine-push-quality.md#2026-05-08-poc-后续收口`

### Event Engine Earnings Quality Review

- Status: done
- Date: 2026-05-08
- Plan: `docs/archive/plans/event-engine-earnings-quality-review.md`
- Handoff: `docs/handoffs/2026-05-08-event-engine-earnings-quality-review.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine pollers::earnings_surprise`, `cargo test -p hone-event-engine pollers::earnings_quality`, `cargo test -p hone-event-engine --lib`, `cargo test -p hone-core --lib`, `cargo check -p hone-web-api`, changed-file `rustfmt --edition 2024 --check`; full `cargo fmt --all -- --check` currently blocked by unrelated formatting debt
- Current conclusion: `EarningsReleased` 已移除 EPS-only 推送，并新增 best-effort LLM 综合财报 review；AAOI / CAI / CRWV POC 结论落地为 SEC 8-K 上下文 + `x-ai/grok-4.1-fast` 风格 JSON judgement，失败、缺上下文或低置信时跳过 candidate
- Next entry point: `docs/handoffs/2026-05-08-event-engine-earnings-quality-review.md`

## 2026-04-30

### Feishu P1 直聊与定时任务可靠性修复批次

- Status: done
- Date: 2026-04-30
- Plan: `docs/archive/plans/feishu-p1-reliability-batch.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels runners::multi_agent::tests`, `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries`, `cargo check -p hone-channels`
- Current conclusion: 活跃 Feishu `P1` 已全部移出活跃队列；multi-agent 对 `cron_job` / `portfolio` 可信本地结果的直返放宽到多行与较长正文，避免“我的定时任务”这类本地状态答案已生成却仍被硬送进容易空回复的 answer 阶段
- Next entry point: `docs/bugs/README.md#活跃待修复`

## 2026-04-29

### Admin Notification Log and Actor Picker

- Status: done
- Date: 2026-04-29
- Plan: `docs/archive/plans/admin-notification-log-actor-picker.md`
- Handoff: `docs/handoffs/2026-04-29-admin-notification-log-actor-picker.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api routes::notifications`, `cargo test -p hone-event-engine list_recent_delivery_logs`, `cargo test -p hone-event-engine store::tests::delivery_log_is_append_only_across_retries`, `bun --filter @hone-financial/app typecheck`, `git diff --check`
- Current conclusion: 管理端推送日志已从只读 cron 执行记录改为合并 cron 与 event-engine `delivery_log`；默认排除 no-actor router 与 digest item 内部行，避免真实 Discord / sink 送达记录被淹没；前端现在显示 `events.kind_json.type` 的业务事件类型，推送日志和推送日程均改为 actor 下拉选择
- Next entry point: `docs/handoffs/2026-04-29-admin-notification-log-actor-picker.md`

## 2026-04-26

### 后端部署文档与 public chat 顶部菜单修复

- Status: done
- Date: 2026-04-26
- Plan: `docs/archive/plans/backend-deployment-and-public-chat-nav.md`
- Handoff: `docs/handoffs/2026-04-26-backend-deployment-public-chat-nav.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, `bun run typecheck:web`, `bun run build:web:public`, Chrome/Playwright local preview screenshots for `/chat`
- Current conclusion: 后端部署流程已落到 runbook，公开文档统一使用后端 origin 口径；public chat 顶部菜单样式已收敛到共享 public CSS，Cloudflare Pages SPA fallback 已加入 public 静态资源
- Next entry point: `docs/runbooks/backend-deployment.md`

### Non-P1 Fixing Bug Batch

- Status: done
- Date: 2026-04-26
- Plan: `docs/archive/plans/non-p1-fixing-bug-batch.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: `0e917fe fix scheduler fixing bug batch`
- Related runbooks / regressions: `cargo test -p hone-channels scheduler::tests`, `cargo test -p hone-channels prompt::tests`, `cargo test -p hone-channels`, `cargo test -p hone-feishu failed_reply_text`, `git diff --check`
- Current conclusion: 非 P1 `Fixing` 批次已完成代码止血与文档同步；按新口径，已代码修复但只待真实窗口复核的缺陷统一标记为 `Later`，不再占活跃队列，后续复现时改回 `New`
- Next entry point: `docs/bugs/README.md#later--待复现`

### Remove Truth Social Source

- Status: done
- Date: 2026-04-26
- Plan: N/A, single-session deletion did not need dynamic plan tracking
- Handoff: `docs/handoffs/2026-04-26-remove-truth-social-source.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo check -p hone-web-api`
- Current conclusion: Truth Social 已从 event-engine 活跃 source 集合删除；`truth_social_accounts` 配置、`TruthSocialPoller` 模块、engine 装配、主配置启用项和本机 ignored effective config 均已移除，历史 403 断流 bug 标记为 Closed
- Next entry point: `docs/handoffs/2026-04-26-remove-truth-social-source.md`

## 2026-04-24

### Price Event Lane 增量改造

- Status: done
- Date: 2026-04-24
- Plan: `docs/archive/plans/price-event-lane.md`
- Handoff: `docs/handoffs/2026-04-24-price-event-lane.md`
- Decision / ADR: `docs/decisions.md#d-2026-04-24-01-route-price-alerts-through-directional-band-lanes`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine price --lib`, `cargo test -p hone-event-engine router --lib`, `cargo test -p hone-event-engine digest --lib`, `cargo test -p hone-core --lib`, `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`, `cargo test --workspace --all-targets --exclude hone-desktop`
- Current conclusion: 价格事件已从日级去重改为 low/band/close 分层 id；盘中 `price_band:{symbol}:{date}:{up|down}:{band_bps}` 可在同日多次跨新档时形成独立事件，router 使用价格专属 gap/cap 控频，digest 对同一 actor/symbol/date/window 保留最新价格态，收盘价格默认摘要化
- Next entry point: `docs/handoffs/2026-04-24-price-event-lane.md`

### Event Engine Close Price 与 Truth Social 后续修复

- Status: done
- Date: 2026-04-24
- Plan: `docs/archive/plans/event-engine-close-price-truth-social-followup.md`
- Handoff: `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine --lib`, `cargo fmt --all -- --check`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, `cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib`, `env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, `python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-23 --actor telegram::::8039067465`, `python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-24 --actor telegram::::8039067465 --include-body`
- Current conclusion: Truth Social poller 已补 status / content-type / body-prefix 失败诊断，`price_close` 高波动已恢复 High / immediate 路由；真实模型 baseline 已从 12 条 LLM 样本扩到 15 条并 15/15 matched；2026-04-24 Telegram digest 省略项已可通过 `digest_item omitted` 审计，低信号 news/social/macro/no-op analyst 噪声已降噪
- Next entry point: `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`

## 2026-04-23

### Event Engine 推送质量全量修复

- Status: done
- Date: 2026-04-23
- Plan: `docs/archive/plans/event-engine-push-quality.md`
- Handoff: `docs/handoffs/2026-04-23-event-engine-push-quality.md`
- Decision / ADR: N/A
- Related PRs / commits: `0ff23d4 feat(event-engine): improve push quality routing`, `df820ca feat(event-engine): add daily push calibration export`
- Related runbooks / regressions: `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo test -p hone-core --lib`, `cargo check -p hone-web-api`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- Current conclusion: event engine 的 24 项推送质量清单已全部收口，新增 digest 去重 / min-gap / topic memory、source/channel 偏好、分类预算、方向性价格阈值、macro/earnings 时窗、delivery observability，以及 `amazon/nova-lite-v1` 不确定来源新闻分类基线
- Next entry point: `docs/handoffs/2026-04-23-event-engine-push-quality.md`

### Core Runtime 职责与类型收敛

- Status: done
- Date: 2026-04-23
- Plan: `docs/archive/plans/core-runtime-type-consolidation.md`
- Handoff: `docs/handoffs/2026-04-23-core-runtime-type-consolidation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels agent_session`, `cargo test -p hone-channels runners::tests`, `cargo test -p hone-event-engine subscription`, `cargo test -p hone-web-api routes::history`, `bun run test:web`, `bun --filter @hone-financial/app typecheck`, `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`
- Current conclusion: `AgentSession` 的 prompt/skill turn 构建与 response finalization 已从主编排里拆出，runner/session 内部事件收敛到 canonical `run_event`，runner kind / CLI probe 逻辑有了统一 helper，前端历史附件类型已和 Rust 对齐，本地图片 marker 也补了 Rust/前端共享 fixture
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-22

### Git Hook Auto Format

- Status: done
- Date: 2026-04-22
- Plan: `docs/archive/plans/git-hook-auto-format.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `.githooks/pre-commit` hook smoke test with staged Rust formatting, `bash -n .githooks/pre-commit`, `bash -n scripts/install_gitleaks.sh`
- Current conclusion: 本地 Git hook 现在会在 commit 前自动格式化已暂存 Rust 文件并重新暂存，push 前的 rustfmt / gitleaks 仍作为兜底门禁；同一 Rust 文件如果同时有已暂存和未暂存改动，pre-commit 会停止以避免把未选择的内容混入 commit
- Next entry point: `.githooks/pre-commit`

## 2026-04-20

### Hone 内置技能高置信度收敛

- Status: done
- Date: 2026-04-20
- Plan: `docs/archive/plans/hone-skill-consolidation.md`
- Handoff: `docs/handoffs/2026-04-20-hone-skill-consolidation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_finance_automation_contracts.sh`, `cargo test -p hone-tools load_skill_and_direct_invocation_accept_aliases`, `cargo fmt --all --check`
- Current conclusion: Hone 的高重叠金融 skill 已收敛到更小的维护面：`one_sentence_memory` 被删除，`major_alert` 被并入 `scheduled_task`，`valuation` 与 `stock_selection` 被并入带兼容 alias 的 `stock_research`；finance regression 已改为验证新的 canonical skill 形态
- Next entry point: `skills/stock_research/SKILL.md`

## 2026-04-19

### Hone 半小时健康巡检补齐用户端静态资源检查

- Status: done
- Date: 2026-04-19
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-19-hone-health-automation-public-web-check.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `sed -n '1,220p' ~/.codex/automations/hone-health-30m/automation.toml`, `bun run build:web:public`, `curl http://127.0.0.1:8088/`, `ls packages/app/dist-public`
- Current conclusion: `hone-health-30m` 现在不会再把“`8088` 正在监听”误判成用户端健康；它新增了 `packages/app/dist-public/index.html` 与 `8088` HTML 返回检查，并在只缺用户端静态资源时优先执行 `bun run build:web:public` 做最小止血，只有仍不健康时才整套重启
- Next entry point: `.codex/automations/hone-health-30m/automation.toml`

### Web 邀请码手机号绑定与固定端口切换

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-invite-phone-and-fixed-ports.md`
- Handoff: `docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`, `bun run build:web`, `bun run build:web:public`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run tauri:prep:build`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`, `curl http://127.0.0.1:8077/api/meta`, `curl http://127.0.0.1:8088/api/public/auth/me`, `curl -I http://127.0.0.1:8088/chat`
- Current conclusion: bundled desktop 现在固定使用管理端 `8077` 与用户端 `8088`；Web 邀请码已改为与手机号强绑定，管理端发码必须填手机号，用户端登录必须同时提交邀请码和手机号。新的 release app 已按 runbook 切换到 `.app` runtime；`discord` / `feishu` 在线，`telegram` 仍因配置里的 `Invalid bot token` 处于 `degraded`
- Next entry point: `docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md`

### 用户可见内部工作说明泄露修复

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/user-visible-internal-working-note-fix.md`
- Handoff: `docs/handoffs/2026-04-19-user-visible-internal-working-note-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels runners::tests -- --nocapture`, `cargo test -p hone-channels agent_session -- --nocapture`, `cargo test -p hone-web-api -- --nocapture`, `bun run test:web`
- Current conclusion: public web 不再把 `company_profiles/`、actor 用户空间、目录结构这类内部工作说明直接作为最终答复或执行中状态暴露给用户；ACP runner 在本轮发生工具调用时只接受“最后一个 tool 之后的 assistant 文本”作为最终答复候选，session 成功态也会对明显的内部 working note 触发安全 fallback
- Next entry point: `crates/hone-channels/src/agent_session.rs`

### Company Profile Optional Frontmatter

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-optional-frontmatter.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-optional-frontmatter.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile -- --nocapture`
- Current conclusion: 公司画像与事件现在不再在读取、列出、bundle preview/import 时硬依赖 YAML frontmatter；legacy plain Markdown 本地画像与 plain-Markdown 画像包都会推断最小 metadata 继续工作，不再因为 `缺少 frontmatter` 直接失败
- Next entry point: `memory/src/company_profile/markdown.rs`

### 公司画像包导入导出与傻瓜式导入流

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-transfer.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-transfer.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile`, `cargo test -p hone-web-api`, `bun run test:web`, `bun run typecheck:web`, `bun run build:web`, `bun run --cwd packages/app test:e2e`, `cargo check -p hone-memory -p hone-web-api -p hone-channels`
- Current conclusion: 公司画像现在支持 actor 私有画像包导入导出；Memory 页面左侧已收敛成单一“目标用户空间”列表，当前空间里的公司切换放到右侧详情内部；右侧会先自动扫描导入包，只在存在冲突时要求逐家公司选择“保留当前”或“用导入版本替换”，并在存在替换时自动生成导入前备份供用户下载；legacy plain Markdown 画像即使缺少 frontmatter，也能被 transfer 导出、自动备份并参与冲突判断
- Next entry point: `packages/app/src/context/company-profiles.tsx`

### Company Profile 模块拆分

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-module-split.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-transfer.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo fmt --all`, `cargo test -p hone-memory company_profile`, `cargo test -p hone-web-api`, `cargo check -p hone-memory -p hone-web-api -p hone-channels`
- Current conclusion: `hone-memory` 里的 company profile 已按职责拆成 `types / markdown / storage / transfer / tests` 子模块，保留原有 `hone_memory::*` 导出面和导入导出语义，后续继续改画像能力时不需要再在单个超大文件里同时处理类型、Markdown、zip 和存储细节
- Next entry point: `memory/src/company_profile/mod.rs`

Use this file as the historical entry point for completed or paused work that should remain discoverable.

### Web 管理端 / 用户端端口隔离与公网暴露加固

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-admin-public-isolation.md`
- Handoff: `docs/handoffs/2026-04-19-web-admin-public-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`, `./launch.sh --web`, `curl http://127.0.0.1:8077/api/public/auth/me`, `curl http://127.0.0.1:8088/api/meta`
- Current conclusion: Web 管理端和 invite 用户端已按端口与可访问路由拆开；管理端默认监听 `8077` 并只提供 `/api/*` 与 console SPA，用户端默认监听 `8088` 并只提供 `/api/public/*` 与 `/chat`。后续安全加固已经补上 public 邀请码失败冷却、邀请码停用 / 恢复 / 重置与会话清退、单邀请码单活跃 session、HTTPS 场景 `Secure` cookie，以及 public API 默认去掉 `CORS: *`；公网暴露时仍必须确保管理端不被反代出去，并在反向代理 / WAF 层继续做 IP 级限流
- Next entry point: `crates/hone-web-api/src/routes/public.rs`

### Public Web 邀请码与公网暴露安全加固

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/public-web-security-hardening.md`
- Handoff: `docs/handoffs/2026-04-19-web-admin-public-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`
- Current conclusion: public 邀请码登录已从“无防刷、无撤销、无会话止血”状态提升到具备应用层失败冷却、邀请码停用 / 恢复 / 重置、旧 session 立即失效、HTTPS `Secure` cookie 和同源默认访问的基础安全面；剩余长期暴露风险主要转移到反向代理 / WAF 限流策略与管理端误暴露治理
- Next entry point: `crates/hone-web-api/src/routes/web_users.rs`

### Web 邀请码用户端与管理端入口拆分

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-invite-chat-user-surface.md`
- Handoff: `docs/handoffs/2026-04-19-web-invite-chat-user-surface.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth -- --nocapture`, `cargo test -p hone-web-api -- --nocapture`, `bun run test:web`, `cd packages/app && bun run typecheck && bun run build`
- Current conclusion: 管理端现在可以在设置页生成邀请码并复制，侧边栏“开始”旁新增了用户端跳转 icon；用户侧新增 `/chat` 页面，通过邀请码登录并进入单会话 SSE 聊天窗口，过程卡片会展示 `Hone 思考中 -> 工具执行 -> 最终回复`；后端新增 `/api/public/*` 与 SQLite `web_auth` 存储，公开接口严格从 cookie 登录态反解 `web` actor，不再接受外部传入的 `channel/user_id/session_id`
- Next entry point: `crates/hone-web-api/src/routes/public.rs`

## 2026-04-17

### 群聊中间进度改为 compact 可见

- Status: done
- Date: 2026-04-17
- Plan: `docs/archive/plans/group-chat-compact-progress-visibility.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels outbound::tests -- --nocapture`, `cargo test -p hone-feishu listener -- --nocapture`, `cargo check --workspace --all-targets --exclude hone-desktop`
- Current conclusion: Telegram / Discord / Feishu 群聊现在都会显示处理中间进度，但默认收敛到 compact 粒度，只暴露“搜索信息 / 获取数据 / 执行命令 / 执行技能”等阶段，不再把 query、命令行和目录路径这类细节直接刷进群消息；当 runner 只吐出 `Tool` 这类泛化标签时，会结合 reasoning 回退成粗粒度动作文案，且连续多轮相同类型的工具调用也会像单聊一样逐轮追加
- Next entry point: `crates/hone-channels/src/outbound.rs`

### 对话额度改为可配置并支持无限制

- Status: done
- Date: 2026-04-17
- Plan: `docs/archive/plans/conversation-quota-config.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core`, `cargo test -p hone-channels run_success_commits_daily_conversation_quota -- --nocapture`, `cargo test -p hone-channels run_rejects_over_daily_limit_without_persisting_user_message -- --nocapture`, `cargo test -p hone-channels run_zero_daily_conversation_limit_bypasses_quota -- --nocapture`, `cargo run -q -p hone-cli -- config validate`
- Current conclusion: 用户每日成功对话额度不再固定写死为 `12`；现在由 `agent.daily_conversation_limit` 控制，`0` 表示无限制。本地 repo `config.yaml` 已切到 `0`，当前运行环境不再限制用户每日对话数
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-16

### Feishu 直聊 placeholder 假启动收口

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/feishu-direct-busy-placeholder-gap.md`
- Handoff: `docs/handoffs/2026-04-16-feishu-direct-busy-placeholder-gap.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- Current conclusion: Feishu 私聊当前已不再在 session 冲突时先发 placeholder 再卡死等待，而是会在入口直接返回 busy 提示；这条修复针对的是“placeholder 假启动”问题，不等同于已经完全根除所有深层长时间持锁根因
- Next entry point: `docs/handoffs/2026-04-16-feishu-direct-busy-placeholder-gap.md`

### 搜索失败提示主根因修复与 Tavily 复核

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/search-failure-tavily-and-tool-call-fix.md`
- Handoff: `docs/handoffs/2026-04-16-search-failure-tavily-and-tool-call-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels sanitize_search_context -- --nocapture`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- Current conclusion: `web_search` 工具确实走 Tavily，但当前统一失败提示的主根因不是 Tavily 全局不可用，而是 multi-agent 搜索阶段历史上下文清洗不完整，遗留 assistant `tool_calls` 与被删除的 `tool` 结果失配，触发 OpenAI-compatible provider `tool call result does not follow tool call (2013)`；该问题现已修复并完成定向测试与 desktop release 打包验证
- Next entry point: `docs/handoffs/2026-04-16-search-failure-tavily-and-tool-call-fix.md`

### Desktop 启动坑位沉淀与会话列表恢复

- Status: done
- Date: 2026-04-16
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-16-session-list-runtime-recovery.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`, `cargo test -p hone-core actor::tests::session_identity_can_be_restored_from_actor_session_id -- --exact`, `cargo test -p hone-memory session_sqlite::tests::list_sessions_skips_unreadable_rows -- --exact`, `cargo test -p hone-web-api routes::users::tests::actor_session_id_is_enough_for_listing_identity -- --exact`, `curl http://127.0.0.1:8077/api/meta`, `curl http://127.0.0.1:8077/api/users`, `curl http://127.0.0.1:8077/api/channels`
- Current conclusion: repo-local `honeclaw/data` 并未丢失，会话为空的主因是 backend session-listing 在部分脏数据路径上直接失败，导致 `/api/users` 错误返回空数组；现在列表会跳过损坏的 `normalized_json` 并从 `session_id` 回推 actor identity，desktop release runtime runbook 和 `bug-2` automation 也已经把锁文件、detached 启动静默失败、desktop/backend 分离排障、正式接口验证等坑位写清楚
- Next entry point: `docs/handoffs/2026-04-16-session-list-runtime-recovery.md`

### Desktop Agent 配置隔离修复

- Status: done
- Date: 2026-04-16
- Plan: `docs/current-plans/canonical-config-runtime-apply.md`
- Handoff: `docs/handoffs/2026-04-16-desktop-agent-config-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core promote_legacy_runtime_agent_settings`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop build_agent_setting_updates_keeps_opencode_and_multi_agent_answer_isolated`
- Current conclusion: desktop legacy agent config promotion no longer overwrites canonical `agent.opencode` when the canonical `api_key` is intentionally blank, and desktop settings save no longer lets `multi-agent.answer` silently overwrite `agent.opencode`; both P1 bug docs and the bug navigation table are now updated to `Fixed`
- Next entry point: `docs/handoffs/2026-04-16-desktop-agent-config-isolation.md`

## 2026-04-15

### Bug 台账导航页与自动化文档模式升级

- Status: done
- Date: 2026-04-15
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-15-bug-index-and-automation-doc-mode.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `ls docs/bugs`, `sed -n '1,40p' docs/bugs/*.md`, `sed -n '1,220p' .codex/automations/bug/automation.toml`, `sed -n '1,220p' .codex/automations/bug-2/automation.toml`
- Current conclusion: `docs/bugs/README.md` 现在作为 bug 目录导航和状态总表存在，集中展示活跃待修复、已修复/关闭和历史分析条目；`bug` 与 `bug-2` 两个 automation 都被要求在任何 bug 状态变化时同步维护这张表
- Next entry point: `docs/bugs/README.md`

### Bug 每小时巡检自动化升级

- Status: done
- Date: 2026-04-15
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-15-hourly-bug-audit-automation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `sqlite3 data/sessions.sqlite3 '.tables'`, `sqlite3 data/sessions.sqlite3 'pragma table_info(session_messages);'`, `find data/runtime -maxdepth 2 -type f`
- Current conclusion: 每小时 `bug` automation 现在会优先巡检最近一小时真实会话与运行日志，并把“AI 返回不及预期、结构/格式错误、返回质量不佳但不影响功能链路”的问题统一按 `P3` 建档；只有真正影响功能链路、正确性、稳定性或投递结果的问题，才继续提升到 `P0`-`P2`。2026-04-26 起，新增或确认仍活跃的 `P1` 还必须通过 `gh issue create` 创建脱敏 GitHub issue，正文标记 `Reporter: hone-scanner` 并 `CC: @chet-zzz @Finn-Fengming`
- Next entry point: `docs/handoffs/2026-04-15-hourly-bug-audit-automation.md`

### Desktop 日志接口与 multi-agent 运行态恢复

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/runtime-logs-runner-recovery.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api logs`, `cargo test -p hone-core from_file_applies_runtime_overlay`, `curl http://127.0.0.1:8077/api/logs`, `curl http://127.0.0.1:8077/api/channels`
- Current conclusion: `/api/logs` 现在能容忍非 UTF-8 日志内容与日志缓冲锁中毒，不再因为多字节明文切片直接断开连接；`HoneConfig::from_file()` 也会正确合并 runtime overlay，渠道与 desktop 运行态恢复后能够稳定回到 `multi-agent`
- Next entry point: `crates/hone-web-api/src/routes/logs.rs`

### 持仓记忆补齐持有期限与策略信息

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/portfolio-memory-horizon-strategy.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory portfolio`, `cargo test -p hone-tools portfolio_`, `cargo test -p hone-web-api portfolio`, `bun run typecheck:web`, `bun run test:web`
- Current conclusion: 持仓记忆现在除标的、数量、成本和备注外，还会稳定保留 `holding_horizon`（`long_term` / `short_term`）和 `strategy_notes`；前端表单已允许负成本价输入，底层存储 / tool / API / UI 都兼容负成本与新增策略字段
- Next entry point: `memory/src/portfolio.rs`

### GitHub Security / Quality 高优问题收口

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/security-quality-remediation.md`
- Handoff: `docs/handoffs/2026-04-15-security-quality-remediation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bun run build:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: 已收口 research proxy URL 校验、session / company profile 路径组件校验、console 明文 user id 日志、Actions workflow 权限与一批高优 transitive dependency；剩余值得关注但未继续深挖的主要是 desktop GTK/Tauri 链上的 `glib` 告警，以及 `feishu-sdk -> salvo_core` 带入的低优 `rand 0.10.0`
- Next entry point: `docs/handoffs/2026-04-15-security-quality-remediation.md`

### Pre-Compact KV Cache 稳定性收口

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/kvcache-stability-before-compaction.md`
- Handoff: `docs/handoffs/2026-04-15-kvcache-stability-before-compaction.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-agent-codex-cli`
- Current conclusion: Hone 现在不会在下一次 compact 之前，由自身更小的 recent restore window、按当前用户输入动态变化的 system prompt related-skill block，或 `codex_cli` 的额外 20 条裁剪，提前制造可避免的 cache miss；compact 之后 prefix 变化仍视为正常边界
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-13

### Multi-Agent 输出净化与 think/tool_call 泄漏修复

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/multi-agent-output-sanitization.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-feishu`, `cargo test -p hone-channels sanitize_user_visible_output -- --nocapture`, `cargo test -p hone-channels restore_context_sanitizes_polluted_assistant_history -- --nocapture`, `cargo test -p hone-channels internal_search_note_does_not_skip_answer_stage -- --nocapture`
- Current conclusion: 统一新增用户可见输出净化层后，multi-agent 搜索阶段不再把带 `<think>` / `<tool_call>` 的内部工作稿直接返回给用户；`AgentSession`、`restore_context`、`session_compactor` 会在持久化、恢复与压缩路径上拦截或清洗污染内容；Feishu / Telegram / Discord / iMessage 用户可见回复现统一隐藏 `<think>`，Feishu / iMessage 流式 formatter 也会吞掉 `<tool_call>` / `<tool_result>` / `<tool_use>` 内部块
- Next entry point: `crates/hone-channels/src/runtime.rs`

### 跨渠道富文本分段渲染修复

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/cross-channel-rich-text-segmentation.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-telegram -p hone-discord -p hone-feishu`, `cargo test -p hone-channels outbound::tests::split_html_segments_rebalances_open_tags_across_segments -- --exact`, `cargo test -p hone-channels outbound::tests::split_markdown_segments_rebalances_code_fences_across_segments -- --exact`
- Current conclusion: 共享分段层现在新增 HTML / Markdown 两种 format-aware segmenter；Telegram 长回复会在分段边界自动补全并重开 HTML tag，Discord / Feishu 会在 Markdown 代码块跨段时自动补全并重开 fence，避免富文本结构在长回复发送时被切坏后降级或回退纯文本
- Next entry point: `crates/hone-channels/src/outbound.rs`

### 飞书表格语法护栏

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/feishu-table-sanitization.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-feishu markdown`, `cargo test -p hone-channels prompt`
- Current conclusion: 飞书提示词已明确禁止模型手写原始 `<table .../>` 卡片标签；运行时会继续自动把标准 Markdown 表格转换成飞书表格，同时对损坏、截断或 schema 错误的 raw table 做规范化/降级，避免坏标签直接投递到用户侧
- Next entry point: `docs/archive/plans/feishu-table-sanitization.md`

### Skill Runtime 对齐 Claude Code 与 Multi-Agent 优化提案

- Status: done
- Date: 2026-04-13
- Plan: N/A
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 已完成一份 proposal，对比 Claude Code 官方 skill 模型与 Hone 当前实现差异，并分析 `multi-agent` runner 下 skill 的实际使用模式；提案建议把 active skill state 提升为 runner 一等状态，随后再补 `allowed-tools` / `context: fork` / supporting files 等执行与作者体验能力
- Next entry point: `docs/proposals/skill-runtime-multi-agent-alignment.md`

## 2026-04-14

### 会话上下文超限自动恢复与错误净化

- Status: done
- Date: 2026-04-14
- Plan: `docs/archive/plans/context-overflow-recovery.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-channels context_overflow_auto_compacts_and_retries_successfully -- --nocapture`, `cargo test -p hone-channels context_overflow_failure_is_rewritten_to_friendly_message -- --nocapture`
- Current conclusion: `AgentSession` 现在会识别上下文超限错误并在同一 turn 内先强制 compact 当前 session、再重新准备 execution 自动重试一次；若恢复后仍失败，用户只会看到稳定友好的提示，不再看到 `bad_request_error`、`invalid params`、`context window exceeds limit` 等底层 provider 原始报错
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-12

### v0.1.10 CLI Onboarding Provider 配置补齐

- Status: done
- Date: 2026-04-12
- Plan: N/A
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-cli`, `bash scripts/prepare_release_notes.sh v0.1.10 /tmp/release-notes-v0.1.10.md`
- Current conclusion: `hone-cli onboard` 现在会明确要求用户对 `FMP` 和 `Tavily` API key 做出“填写或跳过”的选择；`FMP` 首装写入改为优先使用 `fmp.api_keys`，并清空旧的 `fmp.api_key` 兼容字段；对应 release notes 已补齐到 `docs/releases/v0.1.10.md`
- Next entry point: [v0.1.10 release](https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.1.10)

### v0.1.9 Release 失败修复与补发

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/release-v0.1.9-publish-fix.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: `a505060` (`docs: restore v0.1.9 release notes`)
- Related runbooks / regressions: `bash scripts/prepare_release_notes.sh v0.1.9 /tmp/release-notes-v0.1.9.md`, GitHub Actions `Release` run `24307695528`
- Current conclusion: 已补齐 `docs/releases/v0.1.9.md` 并重推 `v0.1.9` tag；`ensure-release` 不再因缺失 release notes 失败，三套发布产物与 `SHASUMS256.txt` 已成功上传，Homebrew formula 同步发布完成
- Next entry point: [v0.1.9 release](https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.1.9)

### 公司画像与长期基本面追踪

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/company-portrait-tracking.md`, `docs/archive/plans/company-portrait-skill-framework.md`, `docs/archive/plans/company-research-actor-spaces.md`, `docs/archive/plans/remove-kb-memory-surface.md`
- Handoff: `docs/handoffs/2026-04-12-company-portrait-tracking.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile -- --nocapture`, `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`, `bun run --cwd packages/app typecheck`
- Current conclusion: Hone 已具备 Markdown 形式的公司画像与事件时间线、按 actor 展示的画像 Web 视图（允许彻底删除），以及更贴近投研档案的 `company_portrait` skill；画像文档现在直接落在 actor sandbox 的 `company_profiles/` 中，由 agent 使用 runner 原生文件读写维护，不再依赖专用 mutation tool、公共画像目录或 KB 记忆入口
- Next entry point: `docs/handoffs/2026-04-12-company-portrait-tracking.md`

### CLI 首装 Onboarding 与安装向导

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/cli-onboarding-install-wizard.md`
- Handoff: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `bash tests/regression/manual/test_install_bundle_smoke.sh`, `cargo check -p hone-cli`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已支持首装 `onboard/setup` TUI，能够探测本机 runner、在不强迫 Hone 侧填写 OpenCode provider 配置的前提下切到 `opencode_acp`，并按渠道逐个引导启用与填写本地必填字段；GitHub release 安装脚本在交互终端下会询问是否立即运行该向导
- Next entry point: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`

### Desktop Rust Check 与 IDE 语法检查解耦

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/desktop-rust-check-workflow.md`
- Handoff: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check --workspace --all-targets`
- Current conclusion: 默认 workspace Rust 检查继续排除 `hone-desktop`；desktop crate 新增开发态 sidecar 校验豁免开关，VSCode rust-analyzer 默认携带该 env，因此 IDE / 本地 `cargo check` 不再被缺失的 Tauri bundled binaries 阻塞
- Next entry point: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`

### Hone CLI Config MVP 与可安装启动流

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/hone-cli-config-mvp.md`
- Handoff: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test -p hone-core`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已具备 `config / configure / models / channels / status / doctor / start` 管理面；shared runtime overlay service 已供 CLI 与 desktop 共用；macOS / release 安装链路支持 `hone-cli start`，且已补齐首次 runtime config seed 行为
- Next entry point: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`

### Local 私有 Workflow Runner（公司研报 v1）

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/local-workflow-runner.md`
- Handoff: `docs/handoffs/2026-04-12-local-workflow-runner.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cd local/workflow && bun test`, `cd local/workflow && bun run bootstrap-config`, `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts --outdir /tmp/local-workflow-build`, `WORKFLOW_RUNNER_PORT=3213 bun run start`
- Current conclusion: 在 `local/workflow/` 下新增独立本地 workflow runner，并在后续迭代中补齐紧凑工作台、运行级 prompt override、SSE 去重续流、停止接口、单实例串行、Python UTF-8/旧版本注解兼容，以及结构化进度与节点详情观测；当前 `company_report` 入口既可在页面里运行/观察/停止，也可通过 `bun run client` 从本机其它位置发起并监听进度
- Next entry point: `docs/handoffs/2026-04-12-local-workflow-runner.md`

## 2026-04-11

### 金融自动化合同回归闭环

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/finance-automation-contract-loop.md`
- Handoff: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_finance_automation_contracts.sh`, `bash tests/regression/run_ci.sh`
- Current conclusion: finance 固定 9 样本合同切片已从 `success=5 review=1 fail=3` 收口到 `success=9 review=0 fail=0`；剩余 skill policy wording 漂移已全部修正
- Next entry point: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`

### 大文件物理拆分重构

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/large-files-refactor.md`
- Handoff: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`
- Decision / ADR: `docs/decisions.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: runtime override和渠道启动已收口到共享层；desktop sidecar、Feishu / Telegram 渠道热点与前端 settings 纯状态逻辑已按职责拆开，验证矩阵已跑通
- Next entry point: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`

## 2026-03-31

### macOS DMG Release 打包收口

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `make_dmg_release.sh`
- Current conclusion: 新增 `make_dmg_release.sh` 并真实产出 Apple Silicon / Intel 两套 DMG；release 包内置 `hone-mcp` 与 macOS `opencode`，并补齐 packaged/runtime 启动环境与启动锁重试路径
- Next entry point: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`

### 定时任务输出净化与 Tavily 失败隔离

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-tools`, `cargo test -p hone-channels`
- Current conclusion: heartbeat / 定时任务会抽出真正 JSON 结果；Tavily 临时失败会返回脱敏 unavailable 结构，且不再持久化进会话工具上下文
- Next entry point: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`

## 2026-03-29

### 额度与定时任务可靠性修复

- Status: done
- Date: 2026-03-29
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory`, `cargo test -p hone-channels`
- Current conclusion: 普通用户每日额度调整为 12；非 heartbeat 定时任务补上“同日单次补触发”；heartbeat JSON 解析失败会安全抑制
- Next entry point: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`

## 2026-03-27

### 单一聊天范围配置与群聊忙碌态控制

- Status: done
- Date: 2026-03-27
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-core -p hone-channels`
- Current conclusion: `dm_only` 收敛为 `chat_scope`；群聊忙碌态在显式触发场景具备统一控制
- Next entry point: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`

## 2026-03-26

### 子模型配置与心跳任务调度

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory -p hone-scheduler -p hone-tools -p hone-core -p hone-web-api -p hone-channels`, `cargo check -p hone-desktop`
- Current conclusion: Desktop 支持 OpenRouter 子模型配置，会话压缩切到子模型，cron 新增 heartbeat 任务类型
- Next entry point: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`

### Session SQLite 影子写入与运行时切换

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_session_sqlite_migration.sh`
- Current conclusion: SessionStorage 已支持 `json | sqlite` 切换；SQLite shadow write 与 runtime 主读都已接入
- Next entry point: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`

## 2026-03-24

### 群聊预触发窗口统一改造

- Status: done
- Date: 2026-03-24
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-channels -p hone-core`
- Current conclusion: Telegram / Discord / 飞书群聊统一为“未触发先静默缓存、显式触发再执行”的预触发窗口模型
- Next entry point: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`

## 2026-03-22

### 多渠道附件工程化卡点

- Status: archived
- Date: 2026-03-22
- Plan: `docs/archive/plans/channel-attachment-gate.md`
- Handoff: `docs/handoffs/2026-03-22-channel-attachment-gate.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: 共享附件 ingest 已统一拦截超限附件与异常图片，并把拦截原因透出到渠道 ack
- Next entry point: `docs/handoffs/2026-03-22-channel-attachment-gate.md`

## 2026-03-19

### 真群聊共享 Session 落地

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-shared-session.md`
- Handoff: `docs/handoffs/2026-03-19-group-shared-session.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`, `cargo test -p hone-memory -p hone-channels`
- Current conclusion: 群聊会话归属改为显式 `SessionIdentity`；三渠道群消息共享上下文，Web 控制台按真实 `session_id` 浏览
- Next entry point: `docs/handoffs/2026-03-19-group-shared-session.md`

### 群聊回复追加链路统一

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-reply-append-chain.md`
- Handoff: `docs/handoffs/2026-03-19-group-reply-append-chain.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-discord -p hone-telegram`
- Current conclusion: 群聊占位符、首条 `@用户` 与多段 reply 链已在 Discord / Telegram / Feishu 统一
- Next entry point: `docs/handoffs/2026-03-19-group-reply-append-chain.md`

## 2026-03-18

### 渠道运行态心跳替代 pid 判活

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-web-api -p hone-desktop -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage`, `cargo test -p hone-core -p hone-web-api`
- Current conclusion: `/api/channels` 已改为基于 `runtime/*.heartbeat.json` 的心跳新鲜度呈现状态
- Next entry point: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`

### launch.sh 真实进程清理修复

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash -n launch.sh`, `cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: `launch.sh` 已直接启动真实 debug 二进制，pid 文件改为记录真实服务进程
- Next entry point: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`

### Discord 重复“正在思考中”排查

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- Current conclusion: 结论偏向入口被多个 consumer / 进程重复消费，而不是单次 `opencode_acp` run 自行双发 thinking
- Next entry point: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`

### Runner 切换到 Gemini 3.1 Pro

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_gemini_streaming.sh`
- Current conclusion: 默认 runner 已切到 `gemini_acp`，模型固定为 `gemini-3.1-pro-preview`
- Next entry point: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`

## 2026-03-17

### IM 渠道共享入口收口

- Status: archived
- Date: 2026-03-17
- Plan: `docs/archive/plans/attachment-ingest-unify.md`
- Handoff: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`, `cargo test -p hone-channels`
- Current conclusion: 共享 `ingress` / `outbound` 抽象已收口；Discord / 飞书附件 ingest 与 KB 管线下沉到 `hone-channels`
- Next entry point: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`

### 文档计划与 handoff 清理

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-doc-context-cleanup.md`
- Decision / ADR: `docs/adr/0001-repo-context-contract.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 清空已完成计划、合并零碎 handoff，并把 `docs/current-plan.md` 恢复为活跃任务入口
- Next entry point: `docs/handoffs/2026-03-17-doc-context-cleanup.md`

### Legacy 兼容移除与数据迁移

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体迁移细节见 handoff
- Next entry point: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`

### 项目清理（会话稳定性 / 渠道收敛）

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-project-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体清理结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-project-cleanup.md`

### 架构收敛与稳定性审计

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体审计结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`

### Identity 限额策略

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-identity-quota-policy.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体策略结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-identity-quota-policy.md`

### 运行时管理员口令拦截

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-register-admin-intercept.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体拦截链路见 handoff
- Next entry point: `docs/handoffs/2026-03-17-register-admin-intercept.md`

### Telegram 管理员白名单支持

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/telegram-admin-whitelist.md`
- Handoff: `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core`, `cargo test -p hone-channels`
- Current conclusion: `admins` 正式支持 `telegram_user_ids`，共享管理员判定已接入 Telegram，当前私聊 identity `8039067465` 已写入本地配置
- Next entry point: `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`

### 活跃计划清理

- Status: done
- Date: 2026-04-16
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-16-current-plan-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: `docs/current-plan.md` 已从 10 个活跃任务收口到 4 个；6 个长期失焦或仅剩占位语义的计划已移入 `docs/archive/plans/`
- Next entry point: `docs/handoffs/2026-04-16-current-plan-cleanup.md`

### Public Website Mobile Responsive Pass

- Status: done
- Date: 2026-04-26
- Plan: `docs/archive/plans/public-mobile-responsive-pass.md`
- Handoff: `docs/handoffs/2026-04-26-public-mobile-responsive-pass.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bun run build:web:public`, `bun run typecheck:web`, Playwright mobile overflow audit
- Current conclusion: 公开站共享移动端样式已收口，首页、对话页、路线图和基础文档页在 360/390/430/768 宽度下不再横向撑宽，header 保持在视口内
- Next entry point: `packages/app/src/pages/public-site.css`

### Hone Cloud Runner + Web User API Key

- Status: done
- Date: 2026-05-04
- Plan: `docs/archive/plans/hone-cloud-runner-api-key.md`
- Handoff: `docs/handoffs/2026-05-04-hone-cloud-runner-api-key.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth -- --nocapture`, `cargo check -p hone-web-api`, `cargo check -p hone-desktop`, `tsc -p packages/app/tsconfig.json --noEmit`
- Current conclusion: 客户端新增可见 `Hone Cloud` runner，并隐藏 legacy multi-agent / standalone codex CLI 入口；Web 邀请码用户现在拥有只存 hash 的 per-user API Key，public app 提供 Bearer 鉴权的 OpenAI-compatible `/api/public/v1/chat/completions`
- Next entry point: `docs/handoffs/2026-05-04-hone-cloud-runner-api-key.md`

### Public Web Multi-Session Auth

- Status: done
- Date: 2026-05-05
- Plan: `docs/archive/plans/public-web-multi-session-auth.md`
- Handoff: `docs/handoffs/2026-05-05-public-web-multi-session-auth.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth -- --nocapture`, `cargo check -p hone-web-api -p hone-memory`, `cargo test -p hone-web-api public -- --nocapture`
- Current conclusion: public web 普通登录不再清除同一用户其它活跃 session，避免每小时健康检查自动化、用户浏览器和多设备登录互相踢掉 `hone_web_session`
- Next entry point: `memory/src/web_auth.rs`

### SEC Enrichment OpenRouter Token Cap

- Status: done
- Date: 2026-05-07
- Plan: `docs/archive/plans/sec-enrichment-openrouter-token-cap.md`
- Handoff: `docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api sec_filings_enrichment --lib`, `cargo test -p hone-event-engine sec_filings_enrichment --lib`, `cargo check -p hone-web-api`
- Current conclusion: SEC filing enrichment now uses a dedicated OpenRouter provider capped by `event_engine.sec_filings.enrichment.max_summary_tokens`, so short summary output no longer inherits the global 30k completion budget that triggered OpenRouter `HTTP 402`.
- Next entry point: `crates/hone-web-api/src/lib.rs`

### SEC Enrichment Section Excerpts

- Status: done
- Date: 2026-05-07
- Plan: `docs/archive/plans/sec-enrichment-section-excerpts.md`
- Handoff: `docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine sec_enrichment --lib`
- Current conclusion: SEC filing enrichment now selects filing-aware excerpts before the LLM call. 10-Q/10-K prioritize MD&A, strategic/capital/risk/legal windows and Risk Factors; 8-K prioritizes the front-loaded exhibit/news-release narrative. The default excerpt budget is now `10_000` chars, with `7_000` / `4_500` / `2_800` retries on `Prompt tokens limit exceeded`, covering the follow-up OpenRouter failures where TEM filings still hit `5198 > 3256` and `3956 > 3256` after the first section-aware pass.
- Next entry point: `crates/hone-event-engine/src/pollers/sec_enrichment.rs`

### Public Login Production Hotfix

- Status: done
- Date: 2026-05-13
- Plan: N/A
- Handoff: `docs/handoffs/2026-05-13-public-login-prod-hotfix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`, `bun --filter @hone-financial/app test -- chat.test.ts`, `cargo test -p hone-web-api routes::public::tests::sms_phone_candidates_accept_plus_86_and_local_numbers`, Chrome headless public chat smoke
- Current conclusion: Public chat now tolerates legacy malformed history rows without crashing on `content.split`; public SMS login accepts `+86...` numbers against local-number whitelist rows and sends Aliyun requests in local-number form; production was switched to rebuilt `0.11.2` release app, with `web`, `discord`, and `feishu` reporting running.
- Next entry point: `docs/handoffs/2026-05-13-public-login-prod-hotfix.md`

### Web Direct Sandbox Isolation Hotfix

- Status: done
- Date: 2026-05-14
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-14-web-direct-sandbox-isolation-hotfix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels sandbox --lib -- --nocapture`, `cargo test -p hone-channels prepare_ignores_repo_internal_sandbox_override --lib -- --nocapture`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop runtime_env -- --nocapture`, `cargo check -p hone-channels --tests`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- Current conclusion: Actor sandboxes no longer default to repo `data/agent-sandboxes`; repo-internal sandbox roots now fall back to a repo-external temp directory, desktop sidecar propagates that explicit sandbox root, and sandbox initialization removes legacy portfolio files before native-file runners can read them.
- Next entry point: `docs/handoffs/2026-05-14-web-direct-sandbox-isolation-hotfix.md`

### Public Login ToS Runtime Mismatch

- Status: done
- Date: 2026-05-20
- Plan: N/A
- Handoff: `docs/handoffs/2026-05-20-public-login-tos-runtime-mismatch.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`, `bun run build:web:public`, `cargo build --release -p hone-console-page`, `bun --filter @hone-financial/app test -- public-sms-login`
- Current conclusion: Public login failure came from runtime artifact skew: the public bundle and `hone-console-page` binary did not agree on `TOS_VERSION`. Port 8088 now serves the rebuilt public bundle with ToS `2.1`, and the rebuilt backend accepts `2.1` while rejecting stale `2.0`.
- Next entry point: `docs/handoffs/2026-05-20-public-login-tos-runtime-mismatch.md`

### FMP/Tavily Usage Throttle

- Status: done
- Date: 2026-06-21
- Plan: N/A
- Handoff: `docs/handoffs/2026-06-21-fmp-tavily-usage-throttle.md`
- Decision / ADR: N/A
- Related PRs / commits: this change set
- Related runbooks / regressions: `cargo test -p hone-tools --lib`, `cargo test -p hone-agent --lib`, `cargo test -p hone-event-engine pollers::price --lib`, `cargo test -p hone-channels heartbeat_tool --lib`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bash scripts/diagnose_fmp_tavily.sh --tavily-query 'health check'`
- Current conclusion: Tavily search now uses low-bandwidth Bearer requests, usage logging, and key cooldowns; FMP data_fetch now has TTL caching; heartbeat tool calls are capped; FMP price polling now runs only during US regular-session windows by default.
- Next entry point: `docs/handoffs/2026-06-21-fmp-tavily-usage-throttle.md`

### Public Mobile Overlay And Calendar Hotfix

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/public-mobile-overlay-calendar-hotfix.md`
- Handoff: `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-10-01-project-web-scheduled-results-into-a-durable-push-inbox`
- Related PRs / commits: this change set
- Related runbooks / regressions: `packages/app/e2e/public-mobile-overlays.spec.ts`, `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`
- Current conclusion: mobile push center and detail layers no longer collide with the fixed nav, inbox-open acknowledgement reliably clears the red dot without consuming future arrivals, and the finance calendar has a full-screen zoomable viewer above all page stacking contexts.
- Next entry point: `packages/app/src/pages/chat.tsx`

### v0.14.0 Apple User Client Release

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/v0.14.0-apple-user-client-release.md`
- Handoff: `docs/handoffs/2026-07-12-v0.14.0-apple-user-client-release.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
- Related PRs / commits: `60ef12c8`, tag `v0.14.0`, GitHub Actions run `29181306840`
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `cargo test -p hone-user-app`, `bash tests/regression/ci/test_hone_ios_contract.sh`, `bun run test:web`
- Current conclusion: v0.14.0 published verified macOS Universal DMG, iOS Simulator App, and Xcode assets; macOS now has a verifiable bundle-level ad-hoc signature, and Apple checksum files use portable basenames with generation-time self-validation.
- Next entry point: `docs/handoffs/2026-07-12-v0.14.0-apple-user-client-release.md`

### v0.14.1 macOS Session And Calendar Release

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/v0.14.1-macos-session-calendar-release.md`
- Handoff: `docs/handoffs/2026-07-12-v0.14.1-macos-session-calendar-release.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
- Related PRs / commits: `44b39aad`, tag `v0.14.1`, GitHub Actions run `29189572109`
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `cargo test -p hone-user-app`, `bash tests/regression/ci/test_hone_ios_contract.sh`, `bun run test:web`
- Current conclusion: v0.14.1 published a verified Universal macOS DMG whose stable named WebKit data store preserves login cookies across restarts/upgrades, while the PC finance-calendar modal now remains inside short viewports with internal scrolling.
- Next entry point: `docs/handoffs/2026-07-12-v0.14.1-macos-session-calendar-release.md`

### Public Agent Workspace Redesign

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-agent-workspace-redesign.md`
- Handoff: `docs/handoffs/2026-07-13-public-agent-workspace-redesign.md`
- Decision / ADR: N/A
- Related PRs / commits: `63e91795`
- Related runbooks / regressions: `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`, responsive browser QA at 1440 x 900 and 390 x 844
- Current conclusion: `/chat` now enters a responsive HONE Agent research workspace backed by existing community, calendar, push, account, and conversation data; desktop uses three columns, mobile uses five primary tabs, and history selection or prompt send returns to the unchanged single conversation runtime without navigation.
- Next entry point: `docs/handoffs/2026-07-13-public-agent-workspace-redesign.md`

### Public Workspace Page Unification

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-workspace-page-unification.md`
- Handoff: `docs/handoffs/2026-07-13-public-workspace-page-unification.md`
- Decision / ADR: N/A
- Related PRs / commits: `affa8836`
- Related runbooks / regressions: `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`, responsive browser QA at 1440 x 900 and 390 x 844
- Current conclusion: restore, Insights, Tracking/calendar, and Account now share the Agent workspace chrome; Insights is a continuous research stream, Tracking uses a desktop month grid plus a separate mobile agenda, and Account uses a lightweight action surface.
- Next entry point: `docs/handoffs/2026-07-13-public-workspace-page-unification.md`

### Public Chat Silent Restore And History Entry

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-chat-history-entry.md`
- Handoff: `docs/handoffs/2026-07-13-public-agent-workspace-redesign.md`
- Decision / ADR: N/A
- Related PRs / commits: this change set
- Related runbooks / regressions: `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`, responsive browser QA at 390 x 844 and 1365 x 850
- Current conclusion: authenticated chat now renders its full shell immediately, silently restores the latest 20 messages at the bottom, and exposes mobile conversation history with stable message navigation and cursor-based older-page loading; empty histories still land on the Agent overview.
- Next entry point: `packages/app/src/pages/chat.tsx`

### GPT-5.6 Codex ACP Runtime Simplification

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/gpt-5-6-codex-acp-simplification.md`
- Handoff: `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-13-01-retire-in-process-function-calling-and-multi-agent`
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `tests/regression/manual/test_codex_acp_initialize.sh`, `tests/regression/run_ci.sh`
- Current conclusion: the in-process function-calling crate and sequential multi-agent runner are removed; Codex ACP now defaults to GPT-5.6 Sol/xhigh on Codex 0.144.1 and Agent Client Protocol adapter 1.1.2; static prompts no longer carry the full skill catalog.
- Next entry point: `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`

### Entity-First Investment Pipeline

- Status: done
- Date: 2026-07-16
- Plan: `docs/archive/plans/entity-first-investment-pipeline.md`
- Handoff: `docs/handoffs/2026-07-16-entity-first-investment-pipeline.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-16-01-make-security-entity-resolution-the-first-investment-stage`
- Related PRs / commits: this change set
- Related runbooks / regressions: `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_entity_search_live.sh`, `scripts/diagnose_fmp_tavily.sh`
- Current conclusion: all named-security investment turns now resolve structured entities through current-turn DataFetch search and same-symbol quotes before generation; multi-security and typed scheduled/heartbeat turns share the gate, retries reuse the prepared contract, and unresolved/ambiguous entities fail closed without acronym denylists, hard-coded aliases, or first-result guessing.
- Next entry point: `docs/handoffs/2026-07-16-entity-first-investment-pipeline.md`

### Bare Ticker Entity Resolution Regression

- Status: done
- Date: 2026-07-16
- Plan: `docs/archive/plans/plain-ticker-entity-resolution.md`
- Handoff: `docs/handoffs/2026-07-16-entity-first-investment-pipeline.md#2026-07-16-普通-ticker-回归修复阶段`
- Decision / ADR: `docs/decisions.md#d-2026-07-16-01-make-security-entity-resolution-the-first-investment-stage`
- Related PRs / commits: `fa65dfef`, `335c4b73`, `4aa21b29`
- Related runbooks / regressions: `tests/regression/manual/test_entity_search_live.sh`, `tests/regression/ci/test_finance_automation_contracts.sh`
- Current conclusion: contextual bare `NBIS/nbis` and multi-ticker questions now enter DataFetch exact-symbol verification without depending on auxiliary JSON; scheduler ticker subjects share that path, while report periods, assignment metadata, industry acronyms, and unrelated lowercase words remain outside it. Complex prose may still use auxiliary extraction for aliases, but its partial result can only add to—not replace—the deterministic ticker set. The deployed NBIS probe completed all nine sections, and the MRVL live search/quote plus 10-symbol partial-auxiliary regression passed.
- Next entry point: `docs/handoffs/2026-07-16-entity-first-investment-pipeline.md#2026-07-16-普通-ticker-回归修复阶段`

### Asset-Aware Investment Preflight And Visible-Content Guard

- Status: done
- Date: 2026-07-16
- Plan: `docs/archive/plans/asset-aware-investment-preflight.md`
- Handoff: `docs/handoffs/2026-07-16-entity-first-investment-pipeline.md#2026-07-16-资产类型与可见正文门禁阶段`
- Decision / ADR: `docs/decisions.md#d-2026-07-16-01-make-security-entity-resolution-the-first-investment-stage`
- Related PRs / commits: this change set
- Related runbooks / regressions: `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_entity_search_live.sh`
- Current conclusion: exact securities now route independently as equity, ETF/fund, or crypto before evidence fetching; semantic-empty provider results are not confused with outages, inapplicable tools are audited across retries, and investment validation runs against the same sanitized visible content sent to users. The deployed INTL ETF probe completed all nine sections with the verified quote and holdings instead of failing on empty company financials or hidden reasoning numbering.
- Next entry point: `docs/archive/plans/asset-aware-investment-preflight.md`

### Validated Investment Streaming

- Status: done
- Date: 2026-07-16
- Plan: `docs/archive/plans/investment-validated-streaming.md`
- Handoff: `docs/handoffs/2026-07-16-entity-first-investment-pipeline.md#2026-07-16-投研回答校验后一次提交阶段`
- Decision / ADR: `docs/decisions.md#d-2026-07-16-02-commit-investment-replies-only-after-validation`
- Related PRs / commits: this change set
- Related runbooks / regressions: 510 channel tests, 24 DataFetch tests, 3 Web SSE tests, `tests/regression/ci/test_finance_automation_contracts.sh`, and deployed RMBS/INTL Web SSE probes
- Current conclusion: guarded investment attempts are now invisible until entity/evidence and final-answer validation succeed. Internal retries no longer send drafts or resets; Web emits one terminal event; RMBS valuation targets no longer conflict with its verified current quote. Deployed RMBS and INTL probes each produced one answer, zero resets, zero run errors, one successful terminal event, and complete asset-appropriate nine-section output.
- Next entry point: `docs/archive/plans/investment-validated-streaming.md`

### Server-Authoritative Web Chat Run Recovery

- Status: done
- Date: 2026-07-16
- Plan: `docs/archive/plans/chat-active-run-ux.md`; production follow-up: `docs/archive/plans/chat-active-run-production-followup.md`
- Handoff: `docs/handoffs/2026-07-16-chat-active-run-recovery.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-16-03-make-active-chat-runs-server-authoritative`
- Related PRs / commits: `335c4b73`, `92d776a8`, `4aa21b29`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md#drain-active-chats-before-a-controlled-restart`, 113 Web API tests, 14 CLI start tests, 263 Web tests, live disconnect/duplicate/NBIS/RMBS probes
- Current conclusion: Public chat now recovers one real server-owned run id, start time and safe phase across refresh; quota reservations no longer fabricate activity, interrupted turns render terminally, guarded investment drafts remain hidden, same-session duplicates are rejected, and controlled CLI shutdown drains live Web turns before terminating the backend. The missed public deployment was corrected: local, 8088 and Cloudflare Pages use `index-DmyhjLnz.js`; production chat chunks expose the full active-run protocol and no longer contain the old local pending reconstruction.
- Next entry point: `docs/handoffs/2026-07-16-chat-active-run-recovery.md`

### CRWV Exact-Ticker Versus Embedded-Product Repair

- Status: done
- Date: 2026-07-18
- Plan: `docs/current-plans/ticker-resolution-architecture.md` (this exact-ticker/CWY subrepair is done; Agent research-loop content acceptance and scheduler/entity P2 remain active)
- Handoff: `docs/handoffs/2026-07-18-crwv-entity-resolution-repair.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-17-04-resolve-securities-through-a-span-aware-exact-first-pipeline`
- Related PRs / commits: `4d419770`, `b87c4cb7`, `2d6b4be8`, `8d4fcdd6`, `fcca5a35`, `54b14068`
- Related runbooks / regressions: `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_entity_search_live.sh`, `docs/runbooks/backend-deployment.md`
- Current conclusion: the provider was healthy. Exact ticker, genuine natural-name conflict, embedded-product reference, iterative search refinement, and unlinked translated aliases are handled without a service-owned publication ban. `fcca5a35`/`54b14068` removed the fixed refusal and first-group freeze; both fresh CRWV+NBIS phrasings passed with verified `73.21`/`177.71`, one answer/terminal, two-message history, and zero active chats. This archived entry closes only that exact-ticker/CWY subrepair. The same umbrella plan is active again for Agent research-loop/content acceptance after later CRWV/NVIDIA canaries, in addition to scheduler task-prose P2 and first-visible latency.
- Next entry point: `docs/handoffs/2026-07-18-crwv-entity-resolution-repair.md`; remaining scheduler false positives stay in `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`.

### Whop Product Alignment With Knowledge Planet Membership

- Status: done
- Date: 2026-07-26
- Plan: `docs/archive/plans/whop-product-alignment.md`
- Handoff: `docs/handoffs/2026-07-26-whop-product-alignment.md`
- Decision / ADR: N/A
- Related PRs / commits: this change set
- Related runbooks / regressions: Whop CLI product/plan readback, authenticated dashboard and public-page browser QA, `bun run typecheck`, `bun run test:web`
- Current conclusion: one canonical visible Whop membership now offers the same four benefits as the Knowledge Planet membership at the repository-declared international price of USD 199.99/year, without a free trial or stale Discord fulfillment instructions. The English purchase page now links to the new route. All historical products remain archived because every attached historical plan is locked by Whop as non-deletable.
- Next entry point: `docs/handoffs/2026-07-26-whop-product-alignment.md`

### Whop Purchase To Discord VIP Fulfillment

- Status: done
- Date: 2026-07-26
- Plan: `docs/archive/plans/whop-discord-fulfillment.md`
- Handoff: `docs/handoffs/2026-07-26-whop-discord-fulfillment.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-26-04-let-whop-own-discord-membership-fulfillment`
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/whop-discord-fulfillment.md`; authenticated Whop product/settings readback; Whop member-preview claim-flow QA; repository Whop/Discord role-lifecycle audit; `git diff --check`
- Current conclusion: the canonical Whop product now includes the native Discord app connected to `巴芒投研美股社群`, includes `VIP 付费用户`, logs to `#whop`, and removes the role on cancellation. Whop—not a custom webhook or the HONE bot—owns Discord account linking and role lifecycle. A non-owner test membership remains required to prove the real join/grant/revoke/reactivation sequence because creator preview intentionally simulates rather than mutates access.
- Next entry point: `docs/runbooks/whop-discord-fulfillment.md`

### Invisible Context-Overflow Auto-Recovery

- Status: done
- Date: 2026-07-26
- Plan: `docs/archive/plans/context-overflow-invisible-auto-recovery.md`
- Handoff: `docs/handoffs/2026-07-26-context-overflow-invisible-auto-recovery.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-26-07-make-context-window-recovery-invisible-and-current-turn-aware`; `docs/adr/0004-agent-owned-research-loop.md`
- Related PRs / commits: `268561c4`, `620391d1`
- Related runbooks / regressions: `tests/regression/ci/test_finance_automation_contracts.sh` case 44; Agent oversized-result and 12,000-row calendar regressions; Channel compact/current-turn-only/public-sanitizer regressions; `docs/runbooks/backend-deployment.md`
- Current conclusion: current-turn tool growth is bounded inside the same read-only Agent without tool replay, repeated Session overflow falls through to current-turn-only recovery, and no context/compact/path/new-session diagnostic may cross a public boundary. Exact `620391d1` passed all repository gates and a fresh production replay with one successful answer, byte-identical two-row history, responsive concurrent health probes, and zero active chats.
- Next entry point: `docs/handoffs/2026-07-26-context-overflow-invisible-auto-recovery.md`

### Codex ACP Discord Runtime Recovery

- Status: done
- Date: 2026-07-29
- Plan: `docs/current-plans/acp-runtime-refactor.md` (parent remains active)
- Handoff: `docs/handoffs/2026-07-29-codex-acp-discord-runtime-recovery.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md` (no decision change)
- Related PRs / commits: this change set
- Related runbooks / regressions: `cargo test -p hone-channels --lib`; focused attachment and Codex ACP tests; admin-scoped no-side-effect ACP probe; real Discord image-turn timing and Discord API readback
- Current conclusion: Hone now sends the adapter-required `gpt-5.6-sol[xhigh]` selector, reaches `session/prompt`, and lets admin Codex ACP turns read images natively instead of blocking the first Discord placeholder on redundant Apple Vision helper compilation. The motivating real turn completed successfully with 19 tool calls and a 671-character reply; after deployment, a fresh Discord follow-up received its placeholder in 1.121 seconds and completed successfully in 112.473 seconds, with Discord API readback confirming the final edit. The source launchd runtime was rebuilt and restarted cleanly.
- Next entry point: `docs/handoffs/2026-07-29-codex-acp-discord-runtime-recovery.md`

### Event Engine Market-Session Digest Labels

- Status: done
- Date: 2026-07-31
- Plan: `docs/archive/plans/event-engine-market-session-digest-labels.md`
- Handoff: `docs/handoffs/2026-07-31-event-engine-market-session-digest-labels.md`
- Decision / ADR: N/A; behavior is recorded in `docs/invariants.md`
- Related PRs / commits: this change set
- Related runbooks / regressions: `cargo test -p hone-event-engine quiet_flush_ --lib`; `cargo test -p hone-event-engine --lib`
- Current conclusion: both local Discord actor preferences now map 07:30 to `postmarket / 盘后要闻` and 21:00 to `premarket / 盘前要闻`, with quiet hours ending at 07:30. A quiet flush coinciding with a named digest slot reuses that label, so the morning postmarket rollup is no longer drained or renamed before the configured slot. The rebuilt source runtime is healthy and Discord has reconnected.
- Next entry point: after the next natural 21:00 and 07:30 slots, query `data/events.sqlite3` as described in the handoff to confirm the live user-visible headers and event windows.

### Public User Admin Whitelist Production List Recovery

- Status: done
- Date: 2026-07-31
- Plan: `docs/archive/plans/public-admin-whitelist-production-followup.md`
- Handoff: `docs/handoffs/2026-07-31-public-user-admin-whitelist-management.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-31-03-keep-public-user-administration-separate-and-database-authoritative`
- Related PRs / commits: `49ef8dd4e2d5298ad69f01b73d7a1b9be7fa5b87`
- Related runbooks / regressions: `docs/runbooks/public-user-admin.md`; Core 22/22; Memory 133/133; Web API 160/160 plus 2 credentialed ignores; `tests/regression/run_ci.sh`
- Current conclusion: The production 500 came from binding a Rust string directly to a PostgreSQL `date` placeholder in the daily-count query, not from the frontend or administrator role. All count/audit paths now cast text to date, the list uses a minimal non-secret projection, and ancillary count failure no longer hides readable rows. The exact immutable production build is active with healthy PostgreSQL/R2, zero active chats, and the tunnel unchanged.
- Next entry point: refresh the authenticated “我的 → 管理” view; if live mutation acceptance is needed, use one explicitly controlled phone rather than synthetic production accounts.

### ACP Native-Turn Role Boundary And Versioned Stream Dialects

- Status: done
- Date: 2026-08-01
- Plan: `docs/current-plans/acp-runtime-refactor.md` (this role-boundary phase is done; parent ACP runtime plan remains active)
- Handoff: `docs/handoffs/2026-08-01-acp-native-turn-contract.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-01-01-separate-conversation-ownership-from-acp-stream-dialects`; `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/opencode-setup.md`; codex-acp `1.1.7` executable boundary regression; OpenCode `1.18.11` stream fixture; real two-turn Codex and one-turn OpenCode probes; complete repository gates
- Current conclusion: Runner conversation ownership is explicit. Codex instructions use native `developer_instructions`, all Codex `session/prompt` calls are current-turn-only even after compact, legacy/mismatched generations rotate, and the old seed/reseed/transcript execution path is deleted. Codex and OpenCode retain separate version-labelled stream mappings that preserve their available detail without claiming byte-identical channel output.
- Next entry point: `docs/handoffs/2026-08-01-acp-native-turn-contract.md`; the remaining umbrella runtime work stays in `docs/current-plans/acp-runtime-refactor.md`.

### ACP Version-aware Runtime And Revision-bound Local Deployment

- Status: done
- Date: 2026-08-02
- Plan: `docs/current-plans/acp-runtime-refactor.md` (this version/deployment phase is done; parent ACP runtime plan remains active)
- Handoff: `docs/handoffs/2026-08-02-acp-version-aware-runtime-deploy.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-02-01-select-acp-dialects-from-the-live-initialize-boundary`; `docs/decisions.md#d-2026-08-02-02-deploy-direct-source-runtimes-as-immutable-revision-units`; `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: `16699c7e`, `2b5ab70d`, `3e223a97`, `b8e18312`, `998496b1`, `ee9da19a`; feature branch `codex/acp-versioned-runtime-deploy`
- Related runbooks / regressions: `docs/runbooks/source-web-startup.md`; `docs/runbooks/opencode-setup.md`; `tests/regression/ci/test_source_runtime_deploy_contract.sh`; version-labelled Codex ACP `1.1.7` and OpenCode `1.18.11` fixtures; complete repository gates
- Current conclusion: Codex and OpenCode now select separate typed stream dialects from the exact live initialize identity/version, retain explicit compatibility status, and fail closed on mismatched identities, older versions, missing versions, or unknown majors. Codex prompts remain current-turn-only after compaction. The local source runtime deploys Web/Discord/MCP as an immutable revision unit with drain, PID/lock convergence, persisted runner PATH, fresh channel login, external provenance verification, and one complete rollback path. Exact implementation `ee9da19a` is active with four listeners, zero active chats, matching Web/ACP build provenance, and a successful no-tool ACP sentinel.
- Next entry point: `docs/handoffs/2026-08-02-acp-version-aware-runtime-deploy.md`; do not touch GCE without a new explicit instruction.

### Public Admin Usage Analytics Production Rollout

- Status: done
- Date: 2026-08-02
- Plan: `docs/archive/plans/public-admin-usage-production-rollout.md`
- Handoff: `docs/handoffs/2026-08-02-public-admin-usage-analytics.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-02-03-keep-public-usage-analytics-inside-the-web-administrator-boundary`
- Related plan: `docs/archive/plans/public-admin-all-channel-usage.md`
- Related PRs / commits: `39ce9ce54f5cbfea26e664459cb70edf3fd97292`, `c4c217236fae8bbe571f259cd46b6b4768178bcf`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; `docs/runbooks/public-user-admin.md`; Web 344/344; Web API 165/165 plus 2 credentialed ignores; Public-admin 8/8; Public production build
- Current conclusion: The administrator analytics table, dynamic date-bound summary, zero-filled 14-day charts, and collapsible whitelist are live across Web, Feishu, Telegram, Discord, and iMessage identity namespaces. Production currently shows Feishu, Web, and Discord data; same ids across channels remain separate, concrete group actors are safe to count, and `codex*` plus automation envelopes remain excluded. At the authenticated 2026-08-03 01:21 Beijing read-back, the latest 14 days contained 65 question actors, 401 questions, and 1,874 successful pushes. Exact GCE release `c4c21723` is active with authoritative PostgreSQL/R2, zero local durable dependencies, a connected Feishu stream, and zero active chats; Cloudflare Pages and the `181****4550` Chrome session both show the new channel column and summaries.
- Next entry point: `docs/handoffs/2026-08-02-public-admin-usage-analytics.md`; direct GCE rollback is `/opt/hone/releases/39ce9ce54f5cbfea26e664459cb70edf3fd97292-admin-usage-20260802`.

### GCE SMS Runtime Environment Persistence

- Status: done
- Date: 2026-08-02
- Plan: `docs/archive/plans/gce-sms-runtime-env-persistence.md`
- Handoff: `docs/handoffs/2026-08-02-gce-sms-runtime-env-persistence.md`
- Decision / ADR: N/A; operational persistence contract is recorded in the backend deployment runbook
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; `tests/regression/ci/test_backend_runtime_env_contract.sh`; complete `tests/regression/run_ci.sh`
- Current conclusion: Production SMS had returned generic acceptance while the detached provider call failed because the managed Web service lacked its Aliyun AccessKey. The canonical credential pair is now persisted in the protected host environment, and a root-owned systemd `ExecStartPre` gate blocks future starts with missing, empty, or placeholder credentials. Zero-chat restart, exact revision, PostgreSQL/R2, Web/Feishu, public canary acceptance, cleanup, and restored OS Login 2FA all passed.
- Next entry point: `docs/handoffs/2026-08-02-gce-sms-runtime-env-persistence.md`; use the committed validator before changing any managed backend environment.

### SNDK Current-listing Evidence And GCE Rollout

- Status: done
- Date: 2026-08-03
- Plan: `docs/current-plans/ticker-resolution-architecture.md` (this SNDK phase is done; the parent ticker/scheduler umbrella remains active)
- Handoff: `docs/handoffs/2026-08-03-sndk-listing-evidence-and-gce-rollout.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-03-02-reject-malformed-read-only-calls-individually-and-require-listing-evidence`
- Related PRs / commits: initial `116dc54b3540e30b8420aaacf007ede33f0b9f5d`; replacement `aed36044`; deployed main `5028870dcb341476e17b57fdfa84d72624b04200`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; focused SNDK snapshot/outlook, pre-turn and listing-conflict regressions; exact malformed-read-only batch/listing-final Agent regression; complete CI-safe suite; two independent strict production Web canaries
- Current conclusion: The exact strict-runner failure was one malformed read-only outlook call cancelling valid discovery siblings, followed by a tools-disabled final publishing stale acquisition memory. The replacement rejects that bad call individually, keeps valid siblings running, preloads current same-symbol identity/listing evidence before the first model call, and requires current `inactive_listing` before any explicit delisting denial can publish. Exact GCE revision `5028870d` is active with healthy authoritative PostgreSQL/S3 and effective per-user daily quota 100. The Web-only atomic restart restored listeners in about two seconds; two independent fresh SNDK canaries both recognized SanDisk/闪迪, exercised current quote/earnings evidence, and contained none of “已退市 / 未上市 / 无法提供当前财报前瞻”.
- Next entry point: `docs/handoffs/2026-08-03-sndk-listing-evidence-and-gce-rollout.md`; the parent ticker plan remains active only for scheduler `800G` / `NAND` / `AST` / `SEC` P2. Immediate rollback is the retained `116dc54b` release plus `config-pre-5028870d-20260803T151518Z.yaml`.

### Earnings PDF Workflow Style Parity And Chat Download

- Status: done
- Date: 2026-08-04
- Plan: `docs/archive/plans/earnings-pdf-workflow-style-parity.md`
- Handoff: `docs/handoffs/2026-08-04-earnings-research-chat-entry.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-04-02-keep-earnings-pdfs-workflow-dense-and-chat-downloads-direct`
- Related PRs / commits: direct `main` feature commit; no PR, release, or deployment
- Related runbooks / regressions: `tests/regression/ci/test_earnings_research_pdf_markdown.sh`; `tests/regression/manual/test_earnings_research_pdf.sh`; `packages/app/e2e/public-chat-pdf-download.spec.ts`
- Current conclusion: The SNDK preview PDF follows the old Workflow's dense A4 visual grammar while remaining searchable, uses the exact `知识星球：巴芒科技` watermark, includes two recent-news pages and the original final share image, and appears in the HONE assistant bubble as a direct authenticated download card. Current-source real-browser acceptance is complete: native Codex invoked the host-owned official renderer, the 620338-byte five-page artifact passed full-page inspection, and the card click path was exercised.
- Next entry point: production deployment is a separate decision; after deployment, repeat one attachment-backed `财报分析` with a real administrator file upload.

### Earnings PDF Download Recovery And Preview Call Calibration

- Status: done
- Date: 2026-08-04
- Plan: `docs/archive/plans/earnings-pdf-download-and-call-calibration.md`
- Handoff: `docs/handoffs/2026-08-04-earnings-pdf-download-and-call-calibration.md`
- Decision / ADR: no new ADR; the durable download, audit, display-unit, and prose constraints are recorded in `docs/invariants.md`
- Related PRs / commits: uncommitted local change set; no PR, release, or deployment
- Related runbooks / regressions: `cargo test -p hone-web-api`; `bun run test:web`; `tests/regression/ci/test_earnings_research_pdf_markdown.sh`; real local ANET browser/PDF acceptance
- Current conclusion: authenticated Blob downloads now provide visible success/failure state and safely recover sanitized generated-file placeholders only inside the current actor sandbox. Preview calls are recomputed from a guidance/segment anchor, exact bridge, historical bias and evidenced tolerance with base/display-unit validation. The old Workflow section skeleton remains, while company-specific opening and core prose are no longer locked to a common sentence template. A real ANET run remained inline for its own audited reasons rather than a global neutral default.
- Next entry point: `docs/handoffs/2026-08-04-earnings-pdf-download-and-call-calibration.md`; after deployment, rerun fresh ANET/ALAB/AMD samples under the final display-unit contract before delivering those PDFs.

### Reviewed Main Sync And GCE Production Deployment

- Status: done
- Date: 2026-08-04
- Plan: `docs/archive/plans/production-deployment-dede2d61.md`
- Handoff: `docs/handoffs/2026-08-04-production-deployment-dede2d61.md`
- Decision / ADR: no new architecture decision; runtime asset/dependency requirements were added to the backend deployment runbook
- Related PRs / commits: reviewed `ee7024b6..dede2d61`; deployed fix revision `3b01aa2c4567f80ebe2c77fc096887d46b4b634f`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; complete Rust/Web/Worker/Public/CI-safe gates; GHCR bundle verifier; production cloud-authority, skill registry and official PDF renderer smoke
- Current conclusion: the reviewed internationalization and extended-hours changes are live from the exact immutable GHCR digest with healthy authoritative PostgreSQL/S3, zero active chats, a healthy public API and the current Pages bundle. Production now has the exact `earnings-research` skill, Chromium and Noto CJK; a service-user renderer smoke passed full-page PDF inspection with readable Chinese, the exact `知识星球：巴芒科技` watermark and the Knowledge Planet share page. The stale Sunny-Ngrok `origin.hone-claw.com` alias remains a separately recorded legacy-fallback risk and was not changed during this deployment.
- Next entry point: `docs/handoffs/2026-08-04-production-deployment-dede2d61.md`; use the retained previous GHCR release for rollback, and do not enable community-edge legacy fallback until the origin alias contract is reconciled.

### Codex ACP Missing-rollout Recovery And Caris Production Acceptance

- Status: done
- Date: 2026-08-05
- Plan: `docs/archive/plans/codex-acp-missing-rollout-recovery.md`
- Handoff: `docs/handoffs/2026-08-05-codex-acp-missing-rollout-recovery.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-05-01-replace-a-codex-binding-only-when-the-adapter-proves-its-rollout-is-absent`; `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: `f819584cff2f5b386c89f0791f1488c149ad3dfe`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; Codex ACP `1.1.7` missing-rollout executable fixture; complete local and GitHub CI gates; real Caris report/PDF restart-persistence acceptance
- Current conclusion: HONE now replaces a Codex native binding only when the validated adapter proves that the exact persisted rollout is absent before prompt, checkpointing the replacement before execution while all ambiguous resume failures remain fail-closed. The affected Caris binding received a bounded backup/repair; exact production revision `f819584c` is healthy and a real Caris financial analysis persisted one PDF that remained downloadable after service restart. Five superseded reproducible GHCR releases were removed after a staging-time disk-capacity incident, leaving about 5GB free and preserving current, immediate rollback and secondary rollback releases.
- Next entry point: `docs/handoffs/2026-08-05-codex-acp-missing-rollout-recovery.md`; do not bulk-clear the remaining stale bindings, and check system-disk headroom before staging another runtime.

### Push Subscription / Email Production Rollout

- Status: done
- Date: 2026-08-09
- Plan: `docs/archive/plans/push-subscription-email-production-rollout.md`
- Handoff: `docs/handoffs/2026-08-09-push-subscription-email-production-rollout.md`
- Decision / ADR: no new architecture decision; the public unsubscribe route and provider/runtime boundary are recorded in the deployment runbook and regression tests
- Related PRs / commits: direct `main` implementation commit `9eff909aba898dfd12b268a75f71bc269f1e7c4d`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; core email/unsubscribe, scheduler and Web API focused tests; Web public subscription model tests; complete local/GitHub CI; immutable GHCR manifest and production public probes
- Current conclusion: push management and login-free unsubscribe are live on the public API/Pages path, with Monday-first schedule semantics and provider failures that do not expose response PII. Exact GCE revision `9eff909a` is active from digest `sha256:1335325fb6075c98a85ee585bbfc12a3e1073a2b90d3ad860e3fcafec13ba758`, Cloudflare email and unsubscribe secret names are configured in the protected runtime environment, PostgreSQL/S3 authority is healthy, and active chats are zero. Scheduled email pushes remain a follow-up because the scheduler does not yet call `EmailSender` or resolve verified recipient emails.
- Next entry point: `docs/handoffs/2026-08-09-push-subscription-email-production-rollout.md`; preserve the signing secret across normal deploys, rotate the chat-exposed provider token, and implement scheduler-to-email delivery before advertising email pushes.

### Mobile Bottom Navigation Four Tabs

- Status: done
- Date: 2026-08-09
- Plan: `docs/archive/plans/mobile-bottom-nav-four-tabs.md`
- Handoff: `docs/handoffs/2026-08-09-mobile-bottom-nav-four-tabs.md`
- Decision / ADR: none; this repairs the existing four-tab mobile UI contract
- Related PRs / commits: direct `main` implementation commit `959fca1600af5791118a17912cc944b6b9ca3464`; no PR, release, tag, or backend deployment
- Related runbooks / regressions: public chat/workspace style contracts; full Web suite; public production build; navigation responsiveness regression; real `390 × 844` production viewport acceptance
- Current conclusion: the component already rendered `Agent / 推送 / 洞察 / 我的`, but a stale three-column CSS grid wrapped `我的` below the fixed-height navigation and clipped it. Production now uses four equal columns, all four tabs remain inside the viewport, and `我的` navigates to `/me`.
- Next entry point: `docs/handoffs/2026-08-09-mobile-bottom-nav-four-tabs.md`; keep rendered tab count and CSS grid tracks synchronized.

### Personal Knowledge Sources And Curation

- Status: done locally
- Date: 2026-08-11
- Plan: `docs/archive/plans/personal-knowledge-sources-and-curation.md`
- Handoff: `docs/handoffs/2026-08-11-personal-knowledge-sources-and-curation.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-12-promote-external-research-through-an-explicit-trust-ladder`
- Related PRs / commits: uncommitted local change set; no PR, release or production deployment
- Related runbooks / regressions: `tests/regression/ci/test_research_curation_contract.sh`; Web API research tests; Web 445/445; public production build; authenticated desktop/mobile browser acceptance
- Current conclusion: HONE now exposes “我的知识源” from `/me`, accepts personal Knowledge Planet official-Skill exports and iMA exports without private-session scraping, and enforces a three-domain `personal → community_candidate → hone_global` trust ladder. Candidates cannot enter Agent or daily-product retrieval until an administrator explicitly approves and copies them into the official library.
- Next entry point: migrate the three scopes to PostgreSQL/object storage before production; implement public forum posts/comments/likes as a separate untrusted content domain with moderation and an explicit curation submission action.

### First-principles Key-event Chain

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/first-principles-key-event-chain.md`
- Handoff: `docs/handoffs/2026-08-11-key-event-chain-and-serenity-source.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-13-model-key-events-as-a-first-principles-industry-state-chain`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: key-event chain 12/12; Web API 281/2 ignored; focused Web 6/6; full Web 446/446; TypeScript; Rust formatting; public production build; authenticated desktop/mobile acceptance
- Current conclusion: the seventh home Button is now a twelve-topic first-principles industry state chain rather than a Rubin/HBM news list. It requires topic-plus-milestone admission, separates topic-specific company/SEC confirmation from research/opinion clues, exposes first principles and source status in the UI, and keeps the ten-day view to one evidence-gated question per topic. The final local snapshot retained 40 original-linked milestones: 3 confirmed and 37 clues; SOFC stayed empty instead of receiving fabricated filler.
- Next entry point: validate and configure stable official feeds/APIs company by company. Preserve the rule that 白毛/Serenity and research uploads are discovery clues, not factual confirmation.

### HONE Community Discussion Forum

- Status: done locally; no commit or production deployment
- Date: 2026-08-11
- Plan: `docs/archive/plans/community-discussion-forum.md`
- Handoff: `docs/handoffs/2026-08-11-community-discussion-forum.md`
- Decision / ADR: `docs/decisions.md#d-2026-08-11-14-keep-member-discussion-outside-the-research-authority`
- Related PRs / commits: local uncommitted change set
- Related runbooks / regressions: forum Rust 7/7; Web API 286/2 ignored; focused Web 10/10; full Web 451/451; TypeScript; Rust formatting; public production build; `tests/regression/ci/test_community_forum_research_boundary.sh`; authenticated desktop/mobile acceptance
- Current conclusion: `/community` now separates the existing read-only HONE archive from an authenticated member discussion forum. Members can post, comment, like, report and attach one bounded safe file under a pseudonymous identity; owners and administrators retain deletion/moderation controls. Forum material never enters Agent or investment evidence paths, and the UI sends curation candidates through “我的知识源”.
- Next entry point: migrate the forum to PostgreSQL/object storage with retention, moderation audit and abuse observability before any production enablement. Do not add ranking, DMs or investment retrieval to the local filesystem version.
### Earnings Original-workflow Migration And Mode Isolation

- Status: done
- Date: 2026-08-12
- Plan: `docs/archive/plans/earnings-workflow-content-parity.md`
- Handoff: `docs/handoffs/2026-08-10-earnings-opencode-signature-recovery.md`
- Decision / ADR: `docs/decisions.md`; repository generative-workflow rules in `AGENTS.md` and `docs/invariants.md`
- Related PRs / commits: direct `main` sequence through `bd2eb2f9`, `7516be88`, `0c6d0328`, `7beb53e9`, and final deployed `521c0787064d4bfbe18822c3cbc613b5d0390886`; no PR, release, or tag
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`; `tests/regression/ci/test_earnings_research_pdf_markdown.sh`; 49 finance automation contracts; production ACP prompt audit; real CRWV preview/analysis browser canaries
- Current conclusion: HONE directly follows the recovered BamangResearch/Dify earnings prompts without mechanical evidence/content gates. Preview and analysis are separate host-selected products, their prompts exist only in the current turn and never persist/compact/restore, and the renderer only owns technical PDF completion. Exact production `521c0787` is healthy; one CRWV preview and one CRWV analysis each received only its own prompt, used one renderer, emitted no compact, persisted a distinct PDF, and remained downloadable after refresh. A completed renderer artifact can survive an exact subsequent Gemini signature failure only when the trace proves no unrelated write.
- Next entry point: continue content improvement through retrieval, targeted search, original prompts/model choices and real-sample review; do not restore report schemas, source/number coverage gates or validator-driven rewrite loops. Immediate runtime rollback is retained `7beb53e9`.

### SQLite To PostgreSQL Full Migration

- Status: done
- Date: 2026-08-16
- Plan: `docs/archive/plans/sqlite-to-postgres-migration-2026-08-16.md`
- Implementation spec: `docs/archive/plans/sqlite-to-postgres-implementation-spec.md`
- Superseded plan: `docs/archive/plans/session-sqlite-migration-plan.md`
- Handoff: none added for Phase 4 by explicit user instruction; earlier migration evidence remains in the existing dated handoff
- Related commits: `524204f1`, `8191089b`, `8608c6d4`, plus the final verification/archive commit
- Verification: workspace tests 2575 passed / 0 failed across 30 targets; 92 ignored hone-memory PostgreSQL tests passed; 22 CI-safe regressions passed; Web 486 passed / 0 failed; cloud doctor reported PostgreSQL healthy and schema ensured; `cargo tree -i rusqlite --workspace` showed only `hone-cli` and `hone-imessage`
- Current conclusion: PostgreSQL is the only Hone runtime database backend. All alternate storage configuration, shadow/backfill tooling, stale fixtures, prompts, policies, and current architecture/runbook references are removed. `hone-cli` retains a read-only historical event-store importer, while `hone-imessage` independently reads macOS `chat.db`; neither is runtime persistence.
- Next entry point: no migration work remains. Future storage changes must preserve the PostgreSQL authority and treat the two read-only consumers as narrow compatibility boundaries.

### Structured Market Data Before Open Web Search

- Status: deployed to production
- Date: 2026-08-22
- Plans: `docs/archive/plans/market-data-source-priority.md`, `docs/archive/plans/financial-report-data-verification-guidance.md`, `docs/archive/plans/market-data-financial-guidance-production-rollout.md`
- Handoff: `docs/handoffs/2026-08-22-market-data-source-priority.md`
- Decision / ADR: no new ADR; the durable soft-priority rule is recorded in `docs/invariants.md`
- Related PRs / commits: direct `main` implementation commit `3678558483628b605aa927cfa168539a22eca84a`; no PR or tag
- Related runbooks / regressions: DataFetch 57/57 plus financial bundle guidance 1/1; WebSearch 19/19; registry 5/5; function-calling Agent 153/153; pure channel priority and financial guidance tests; workspace compile; GitHub frontend/Edge and Secret Scan; Runtime Image `32548881694`; production exact-meta/cloud/public soak
- Current conclusion: named-company/security research now resolves the entity and prefers a complete structured snapshot before open Web search, while Web remains available for announcements, relationships, events and causal evidence. Provider gaps remain non-blocking. Strict market-move evidence now uses server-computed `hone_change_basis.pct`, including the AAOI `129.10 → 124.82 = -3.32%` regression. Financial figures such as EBITA/EBITDA are additionally bound to the latest disclosed date/period and an explicit quarterly/TTM/forward window; stale or conflicting key figures prompt one targeted official-source check, without a dual-source or missing-data publication gate. Production runs exact revision `36785584…` from immutable digest `sha256:fc6029b4…`; PostgreSQL/OSS/cloud authority and public API acceptance are healthy, with `e08bb460…` retained for immediate rollback.
- Next entry point: with action-time confirmation to create user-visible messages, run named-company, relationship, AAOI-style move and fresh-report financial-metric canaries; preserve these as model/tool-selection and generation guidance, and do not turn them into missing-data gates, forced retry loops or automatic answer rewrites.
