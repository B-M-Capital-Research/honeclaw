# Skills

This directory contains the built-in skill definitions for Hone Financial.

Each skill is a subdirectory with a `SKILL.md` file that describes its purpose, trigger conditions, and workflow.

## Local Internal Research Skill

`company-thesis-ratings` is the local HONE runtime skill for transcript-informed company research and the explainable daily rating dashboard. Its 52 compressed company cards are derived from 51 authorized transcripts and four research workbooks. A per-turn alias index projects only cards named in the current question. When a covered company is matched, HONE must load this Skill before answering and use it as the preferred historical baseline for business model, fundamentals, moat, value-chain position, risks and falsifiers. Current prices, filings, guidance, orders, news and valuation inputs still come from the current evidence tools. Do not redistribute the cards or reconstruct the original transcripts from them.

`hari-invest` is the locally installed public-facing conversation and decision layer distilled from Hari's confirmed investment framework. HONE must load it for investment questions and combine it with current market, filing, news and portfolio evidence. Covered-company questions compose it with `company-thesis-ratings`: the company Skill supplies the historical company-specific thesis, while Hari supplies the current decision discipline. After the required data-time line it gives one explicit opportunity / hold / risk / insufficient-data research zone, confidence, time-horizon differences, the strongest counterargument and observable upgrade/downgrade conditions. The internal team Skill, distillation workspace and `laowang-investment-distiller` are not runtime Q&A skills and must never be exposed to ordinary users. Local installation does not authorize public redistribution or production deployment.

---

## Note on Open-Source Scope

To comply with open-source licensing requirements, a number of **professional valuation tools, investment research workflows, and proprietary knowledge bases** are **not included** in this public repository.

These cover areas such as:

- Advanced DCF and relative-valuation models
- Sector-specific deep-research workflows
- Curated investment research knowledge bases (e.g., earnings transcripts, analyst report libraries)

If you are interested in accessing these capabilities, feel free to reach out to us:

- **YouTube:** [巴芒投研美股频道](https://www.youtube.com/@%E5%B7%B4%E8%8A%92%E6%8A%95%E7%A0%94%E7%BE%8E%E8%82%A1%E9%A2%91%E9%81%93) — follow for investment research content
- **Discord:** see the invite link in the [root README](../README.md) to join our community channel
