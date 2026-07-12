# Public Chat Mobile Gesture And Share Polish

- title: Public Chat Mobile Gesture And Share Polish
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `packages/app/src/components/chat-share-card.tsx`
  - `packages/app/src/components/chat-share-export.ts`
  - `packages/app/src/components/chat-share-modal.test.ts`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/lib/public-chat.ts`
  - `packages/app/src/pages/chat.test.ts`
  - `docs/repo-map.md`
- verification:
  - Share-card layout contract test passed
  - Public-chat pinch policy tests passed
  - Frontend typecheck and 216 frontend tests passed
  - Public production build passed
  - 390 x 844 browser QA confirmed centered query text, runtime viewport scale 1, and no console warnings
- risks:
  - Browser automation cannot synthesize a real iOS multi-touch gesture; native zoom prevention is covered by the runtime viewport assertion, non-passive touch policy, and pure policy tests.

## Goal

Center user queries inside exported black bubbles and prevent accidental browser-level pinch zoom on the chat surface while retaining controlled finance-calendar zoom.

## Completed

- [x] Centralized and corrected the exported user-bubble layout.
- [x] Added a tested public-chat viewport and multi-touch policy.
- [x] Preserved custom calendar lightbox pinch gestures through an explicit surface allowlist.
- [x] Ran full frontend and production-build verification.
- [x] Updated repository context, handoff, archive plan, and archive index.
