---
name: PDF Understanding
description: Read and analyze user-uploaded PDFs, preferably using the system-extracted text fragments, and output summaries, key points, and risk notes
tools:
  - web_search
  - data_fetch
---

## PDF Understanding Skill

When the message contains a `【PDF extraction text】` block, the system has already attempted to extract text from the PDF. Follow the flow below.

### Standard Flow

1. Identify the user's goal first: summary, Q&A, risk scan, financial metric extraction, investment judgment, and so on
2. Prefer the content in `【PDF extraction text】` and answer with a structured conclusion:
   - Core conclusion (1-3 points)
   - Key evidence (important text points from the PDF)
   - Uncertain or missing items
3. If extraction failed or the text is insufficient, which is common for scanned PDFs:
   - State clearly that the current PDF text is insufficient
   - Ask the user for OCR text, a key-page screenshot, or a specific page number
4. If the user's question requires external fact checking, add tools:
   - `web_search(query="...")` for news, announcements, or industry context
   - `data_fetch(...)` for real-time market or company data

### Output Requirements

- Do not guess conclusions that are not present in the PDF
- When citing evidence, stay close to the original wording and avoid inventing numbers
- Keep the output concise and actionable for a chat setting
