# Hari Invest Conversation Agent Handoff

Date: 2026-08-11
Status: done locally; not committed or deployed

## Outcome

HONE now has a public, implicit dialogue Skill that turns the distilled investment framework into decisive but bounded answers. It does not expose the internal team Skill or claim to be old Wang.

## Source Review

- The WeChat temporary attachment had expired, so the byte-identical package was recovered from the persistent WeChat message directory.
- Package SHA-256: `adcda7e5ca2abae7dd2ad27fa50cceea0c1869644655dccf4c40e86344e71c42`.
- Supplied version: internal v0.4.0. Installed maintainer version: v0.4.1. The public conversion keeps v0.4.1's provider-agnostic current-research rule.

## Runtime Contract

1. Investment intent discovers and loads `hari-invest`, including natural Chinese phrases inside a longer question.
2. Current price, filings, news, industry and holdings data come from HONE tools, not from the Skill package.
3. After the mandatory data-time line, the first paragraph chooses `机会区 / 持有区 / 风险区 / 数据不足`, states confidence and gives the core reason.
4. The answer then provides short/mid/long views, evidence, the strongest counterargument and observable change conditions.
5. Missing critical evidence fails closed; missing secondary evidence lowers confidence. No automatic trade execution is introduced.

## Validation Evidence

- Skill validation and contract regressions passed.
- Full relevant Rust libraries passed: `hone-tools` 185/1 ignored; `hone-channels` 789/1 ignored.
- A live administrator dialogue caused the Agent to read the Skill, soul, decision rubric, boundaries and conversation contract before returning a high-confidence `风险区` answer to an overvalued AI-chip scenario.

## Production Blocker And Next Work

The public user route currently has no configured actor-safe model provider. Its refusal is intentional: public actors may not inherit host-capable Codex ACP. Configure a server-side function-calling provider or `hone_cloud`, expose readiness in the UI, and run golden prompts through the actual public route.

After that, prioritize an evaluation gate over another persona Skill: assert Skill loading, current-data dates/citations, conclusion zone in paragraph one, contradiction handling and stable latency. The next useful domain modules are a current-evidence gate, business-model-specific valuation methods and actor-scoped portfolio decisions; each should remain separate from voice/persona.
