# Mobile Finance Calendar And Navigation Polish

- title: Mobile Finance Calendar And Navigation Polish
- status: done
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files: `packages/app/src/components/finance-calendar-message.tsx`, `packages/app/src/components/finance-calendar-mobile-card.tsx`, `packages/app/src/pages/chat.tsx`, `packages/app/src/pages/public-site.css`, `packages/app/src/lib/finance-calendar.ts`, `crates/hone-web-api/src/routes/public_finance_calendar.rs`
- related_docs: `docs/archive/plans/mobile-finance-calendar-nav-polish.md`, `docs/archive/plans/mobile-finance-calendar-dual-layout.md`, `docs/handoffs/2026-06-29-public-finance-calendar.md`, `docs/runbooks/backend-deployment.md`
- related_prs: main commits `31081106`, `e95b1049`, `2a6e7572`, `a4af378d`, `1a72b918`, `6ab39ee3`

## Summary

The mobile finance-calendar and navigation fixes are live on hone-claw.com. Calendar messages now reserve their image footprint, expose loading/error feedback, open into a zoomable viewer, and provide save/share fallbacks. The HONE header and menu use aligned geometry and a deliberate display type hierarchy.

## What Changed

- Generated calendar messages use a dedicated component with progress, retry, full-screen preview, tap/pinch zoom, direct download, file-based Web Share, and long-press fallback.
- Calendar-specific actions replace the previous duplicate generic message controls.
- Mobile navigation uses a 60px shell, matching 42px notification/menu controls, 8px spacing, 54px menu rows, and Avenir Next / PingFang typography.

## Verification

- Local: typecheck passed; 206 frontend tests passed; public production build passed; 390 x 844 geometry QA passed.
- Deployment: main commit `31081106` pushed; Cloudflare Pages switched from `index-C92cekvx.js` to `index-CAwvfWGR.js` at 17:19:37 CST.
- Production: `chat-CP613W7-.js` contains the calendar message/loading/retry implementation; `/`, `/chat`, and `/roadmap` return 200; unauthenticated auth check returns `401 {"error":"未登录"}`.
- Production 390px check confirms the new dimensions, no horizontal overflow, and the intended menu font stack.

## Risks / Follow-ups

- Run one authenticated real-iPhone smoke after the user next generates a calendar: throttle the network, open/zoom the image, save it, and invoke system share.
- Older iOS versions may ignore the anchor `download` attribute; Web Share and long-press saving remain available.

## Production Follow-up 2026-07-11 17:22 CST

The first live full-screen viewer allowed both Safari page pinch zoom and a component-level `210vw` tap zoom. On iPhone this could enlarge the calendar to a cropped three-column region and push the application header/footer controls outside the visual viewport. The task is reactivated to replace that mixed zoom model with controlled in-canvas levels and permanently fixed chrome.

The follow-up shipped in `e95b1049`. The viewer now uses explicit fit/125/150/200 percent levels, pans only its image viewport, keeps the close/zoom/save/share controls fixed, and restores the application-wide Safari gesture guard. Typecheck, the public build, and all 207 frontend tests passed. Cloudflare Pages switched to `index-D4wSdzNX.js` / `chat-ByxolQgf.js`; the production chunk contains the new zoom bar and no longer contains `210vw`. Core routes return 200, the unauthenticated auth probe returns the expected 401 JSON, and a 390 x 844 production check has no horizontal overflow.

## Dual Layout And Gesture Follow-up 2026-07-11 18:06 CST

The bounded button-only viewer did not satisfy iPhone pinch or native long-press behavior, and one desktop-oriented image remained too dense on a narrow screen. Commit `2a6e7572` now renders and uploads both the existing 1080 x 1350 desktop card and a dedicated 750 x 1334 mobile card. The portrait card keeps a readable monthly dot overview and moves important macro/earnings events into chronological agenda rows. The backend independently validates both upload paths and persists them in one backward-compatible message; old one-image messages still fall back to their existing image.

The fixed viewer now interprets two-finger distance as bounded 1x-3x canvas zoom, keeps single-touch native scrolling for panning, and no longer suppresses iOS image touch callout/user selection. Local verification passed 207 frontend tests, 7 focused Rust tests, typecheck, public build, and a rendered 390 x 844 portrait review. Runtime PID `9767` started with the new backend, and production switched to `index-BcPNNntX.js` / `chat-DFgZWxOf.js`. The production chunk contains the mobile path, portrait dimensions, agenda content, and touch listeners; `/`, `/chat`, and `/roadmap` return 200, while public and origin auth probes return the expected 401 JSON.

## Smooth Gesture And Legacy Upgrade Follow-up 2026-07-11 18:47 CST

The first custom pinch implementation still changed canvas width and recomputed scroll offsets on every touch frame. Real iPhone feedback showed visible chasing/jitter at 300 percent, while legacy one-image messages continued to expose the clipped desktop cell labels. Commit `a4af378d` replaces this with a fixed, exact contain base and GPU `translate3d + scale`; pinch keeps the touched content anchored, one-finger pan is clamped to visible image bounds, and touch updates are coalesced to one animation frame. The viewer now measures the source image and available viewport through `ResizeObserver`, so 100 percent always contains the entire artifact before zoom starts.

Visible legacy desktop-only messages now lazily fetch their month payload and render a portrait blob locally, avoiding an eager rebuild of all history. Mobile agenda rows contain one important event each and wrap the complete title instead of using ellipsis. Real-component QA covered 100 through 300 percent, exact 342 x 610 fit in a 390 x 844 viewer, full long-title rendering, and legacy 0.8-to-0.562 aspect conversion. All 209 frontend tests, typecheck, and the public build passed. Production uses `index-C-0scCea.js` / `chat-qOMG9Bni.js`; the chunk contains the transform, resize, touch, lazy-upgrade, and wrapping contracts, and route/runtime health is normal.

## Editorial Visual System Follow-up 2026-07-11 20:22 CST

User review correctly identified that the first portrait artifact still looked like a compressed desktop dashboard: an oversized month/header, weak calendar event marks, repeated white agenda cards, and unused lower-canvas space. Commit `1a72b918` replaces that visual system rather than adjusting isolated spacing. The new 750 x 1334 artifact is an editorial HONE monthly investment brief with a dark brand cover, next-window callout, event-day/macro/earnings counts, a warm scanning calendar, and one continuous category-aware timeline carrying date, category, complete title, and Beijing time. A dark source/disclaimer footer closes the composition.

Visual QA used 15 dense July events at a 390 px viewport. The root remained exactly 750 x 1334, all six timeline rows and long titles stayed within bounds, and the full composition used the portrait canvas without dead space. Typecheck, 209 frontend tests, and the public build passed. Production uses `index-CZTxbnVu.js` / `chat-ThsBAbIe.js`; the chunk contains the monthly brief, next window, month scan, key-date timeline, palette, and disclaimer contracts. Core routes return 200, auth returns the expected 401 JSON, and runtime/tunnel/UI sessions remain healthy.

## Visual Version Migration Follow-up 2026-07-11 20:28 CST

The redesign initially upgraded desktop-only legacy messages but left already-persisted first-generation mobile PNGs selected forever. Commit `6ab39ee3` adds an explicit `mobile-v2` filename contract and treats both a missing mobile source and any pre-v2 mobile source as eligible for the existing in-view lazy rebuild. The rebuilt blob takes precedence immediately, so users see the editorial artifact without regenerating or mutating conversation history.

Fourteen focused finance-calendar tests, typecheck, and the public build passed. Cloudflare Pages switched to `index-BORXXQqy.js` / `chat-DwTyIjoF.js`; the production chunk contains two `mobile-v2` contracts plus the monthly-brief design markers. `/`, `/chat`, and `/roadmap` return 200, auth returns the expected 401 JSON, runtime PID `9767` and both local API surfaces return healthy responses, and a 390 x 844 production browser check reports a 390 px document with no horizontal overflow or console errors.

## Next Entry Point

For future calendar image work, start with `packages/app/src/components/finance-calendar-message.tsx` for actions, `packages/app/src/components/finance-calendar-mobile-card.tsx` for the portrait artifact, `packages/app/src/lib/finance-calendar.ts` for zoom/source selection, and `crates/hone-web-api/src/routes/public_finance_calendar.rs` for the dual-path persistence contract.
