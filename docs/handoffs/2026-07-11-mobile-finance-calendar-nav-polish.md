# Mobile Finance Calendar And Navigation Polish

- title: Mobile Finance Calendar And Navigation Polish
- status: done
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files: `packages/app/src/components/finance-calendar-message.tsx`, `packages/app/src/pages/chat.tsx`, `packages/app/src/pages/public-site.css`, `packages/app/src/lib/finance-calendar.ts`
- related_docs: `docs/archive/plans/mobile-finance-calendar-nav-polish.md`, `docs/handoffs/2026-06-29-public-finance-calendar.md`, `docs/runbooks/backend-deployment.md`
- related_prs: main commit `31081106`

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

## Next Entry Point

Start with `packages/app/src/components/finance-calendar-message.tsx` for image actions and `packages/app/src/pages/public-site.css` for calendar/mobile-nav styling.
