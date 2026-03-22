---
name: Knowledge Memory Search
description: Search the knowledge base by company name or ticker, read archived file summaries, and load the full text when needed
tools:
  - kb_search
  - web_search
aliases:
  - Knowledge Search
  - Knowledge Base Search
  - Look Up Knowledge
  - Knowledge Memory
---

## Knowledge Memory Search Skill

This skill searches the local knowledge memory for archived files related to a company or stock, avoiding repeated network lookups for information that is already stored.

---

### Tool Notes

| Tool call | Purpose |
|---|---|
| `kb_search(action="search", query="...")` | Search the knowledge table by company name or ticker and return matching files plus a one-sentence summary for each file |
| `kb_search(action="load_file", kb_id="...")` | Load the full parsed text of a specific file (use only when the summary is not enough) |
| `web_search(query="...")` | Fill in fresh data or recent developments that are not in the knowledge base |

### Workflow

#### Step 1: Extract the Search Keyword

Identify the company name or ticker from the user's question. Supported examples:
- Chinese company names such as "Apple", "Moutai", or "NVIDIA"
- English company names such as "Apple" or "NVIDIA"
- Tickers such as "AAPL", "600519.SH", or "NVDA"

If the request is vague, such as "the report I uploaded", prefer the company name or ticker mentioned earlier in the same conversation.

#### Step 2: Search the Knowledge Table

Call `kb_search(action="search", query="<keyword>")`.

Example result shape:
```json
{
  "matched": 2,
  "matches": [
    {
      "company_name": "Apple",
      "stock_code": "AAPL",
      "related_files": [
        {
          "kb_id": "uuid-xxxx",
          "filename": "Apple-2024-annual-report.pdf",
          "summary": "This document is Apple's 2024 annual report and covers revenue, profit, and business-line breakdowns"
        }
      ],
      "updated_at": "2026-03-10T12:00:00Z"
    }
  ]
}
```

#### Step 3: Decide Whether to Load the Full Text

**If the summary is enough to answer the question** -> answer directly from the summary and cite the source file name plus upload time.

**If you need original-document details** such as exact financial numbers, contract terms, or deeper analysis -> call:

```text
kb_search(action="load_file", kb_id="<uuid>")
```

> Note: `load_file` returns at most 20,000 characters. If the file is long, it may be truncated, so read the most relevant sections first.
> Do not load every file. Only load one or two of the most relevant files when needed.

#### Step 4: Answer the User

- Prefer the local knowledge base and cite the source
- If the knowledge base is stale or incomplete, supplement it with `web_search`
- Clearly separate information from the archived knowledge base and information from live web search

### No-Result Handling

If `kb_search(action="search")` returns `matched: 0`:
1. Tell the user that no matching company or ticker was found in the knowledge base
2. Suggest uploading a relevant document such as an annual report or research note and syncing it into knowledge
3. Optionally use `web_search` as a live fallback

### Notes

- Search keywords do not have to be exact tickers; substring matching works as well
- The same company may have multiple archived files, and each summary should be judged independently
- Do not dump the full document text back to the user; summarize the key points instead
