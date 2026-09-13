# Original Dify full-US branch

Captured from the authenticated editor on 2026-09-13 (Asia/Shanghai), by reading each visible node's system/user prompt and variable chips. No source workflow was edited or published.

- Main: http://bm.vsource.club:3004/app/22b10bba-7fa3-4a9e-b278-977a9fea3b1a/workflow — only `全跑完-美`.
- Financial subworkflow: http://bm.vsource.club:3004/app/3da6f4e4-625c-4eff-ae38-a2b214acb750/workflow.
- Search subworkflow: http://bm.vsource.club:3004/app/89489f85-c259-4fcf-bda1-d88bb15a9456/workflow.

`original-prompts.json` contains eight verbatim system/user pairs. `original-financial.json` contains the financial formatter pair and shares search. DOM variable chips are represented as `node\nvariable`; the runtime replaces those tokens once, without reinterpreting source data. Whitespace is retained. Search JSON contains the five original query templates; progress JSON is a reference of visible callback code, never executed.

Main node IDs: overview 1740841530496, products 1740841782462, management 1740842132267, finance 1740842287164, competition 1742657234767, illustration 1743920521771, full-US valuation 17474517088250, title 1742655089850.

The runtime preserves the DAG and report order. HONE adapters replace Dify authentication, FMP/Tavily transports, callbacks and external upload. Financial data and searches are acquired internally; the optional extra topic defaults to 新闻. The original Gemini 3.1 Pro model remains the default; the title node also uses this configured model instead of Dify's native qwen-plus transport. There is a separate minimal external-data safety instruction; core prompts are unchanged. Only matching outer triple-apostrophe transport delimiters are unwrapped. No original cleanup that deletes blockquotes is applied: quoted valuation assumptions and diagrams remain in the full report.

Snapshot SHA-256:

```json
{
  "original-financial.json": "80b04282ddb3776f373d4d2cdb08fc1156cb068b305dcafee4392dbedd9a6eaa",
  "original-search-queries.json": "dc630b67096aa47fd3aeab725d04499c5b9d80e0f6503d299f408a58548745b7",
  "original-prompts.json": "e2289dff84ae12d50bc966c6eccdc36f0fdfd034b0cb3df4dbfad0c30c0de2af",
  "original-progress.json": "ff77ff74c125acd9189235c145d89d47123e081104b7d25e3cad22a38f7af540"
}
```
