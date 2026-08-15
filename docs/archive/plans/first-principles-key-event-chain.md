# First-principles Key-event Chain

- title: 第一性原理关键事件链扩展
- status: completed
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/key_event_chain.rs`
  - `packages/app/src/lib/types.ts`
  - `packages/app/src/components/key-event-chain-dashboard.tsx`
  - `packages/app/src/components/key-event-chain-dashboard.css`
  - `packages/app/src/components/key-event-chain-dashboard.test.ts`
- related_docs:
  - `docs/decisions.md#d-2026-08-11-13-model-key-events-as-a-first-principles-industry-state-chain`
  - `docs/repo-map.md`
  - `docs/handoffs/2026-08-11-key-event-chain-and-serenity-source.md`

## Goal

把仅覆盖 Rubin/HBM 的事件列表升级为按 AI 产业第一性原理组织的状态链，覆盖模型、应用、数据中心、ASIC、系统、光互连、存储与电力，并用来源等级和核验状态阻止线索、观点或聚合翻译直接成为事实。

## Delivered

- 十二条明确主线：前沿模型、AI 应用、数据中心、ASIC、Rubin、HBM、HBF、NAND/SSD、800G/1.6T、CPO、NPO、SOFC。
- 主题与里程碑双重准入，以及 schedule/specification/launch/qualification/mass-production/order/capacity/deployment/financial/policy 闭集。
- 主题相关官方公司域名和 SEC 才能成为一手确认；研究库、大 V、聚合翻译和二手报道保持线索状态。
- 官方源的 handle、HTTPS host 和正文链接 host 均校验；默认仅启用已实际验证的 OpenAI RSS。
- UI 展示产业层级、第一性原理、确认/线索数量、证据等级、核验说明和仅看一手筛选。
- 十日简报限制为每主题一个验证问题；没有一手证据只能等待，不能提前给出结论。

## Validation

- Key-event-chain Rust: 12/12.
- Full Web API: 281 passed, 2 ignored.
- Focused dashboard: 6/6; full Web: 446/446.
- TypeScript typecheck, Rust formatting and public production build passed.
- Fresh local snapshot: 40 source-linked milestones, 3 confirmed and 37 clues; numeric prices did not become tickers.
- Authenticated desktop and 390×844 browser acceptance passed with twelve topics, confirmed-only filtering, twelve bounded ten-day questions and zero page/dialog overflow.

## Remaining Operational Work

Individually validate and add stable official feeds or APIs for NVIDIA, memory, optical, hyperscaler and SOFC companies. Until then, a missing official event is reported as coverage absence, not “nothing happened”; clue sources must not be promoted to confirmation.
