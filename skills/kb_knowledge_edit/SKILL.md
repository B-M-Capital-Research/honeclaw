# Skill: Record Key Knowledge

## Trigger Conditions

Activate this skill when the user clearly says they want to record or save an important fact about a company or ticker, for example:

- "Save that Moutai's moat is xxx"
- "Store this information in Apple's knowledge"
- "Remember that AAPL's cash reserve is xxx"
- "Record this conclusion in knowledge memory"

## Workflow

### Step 1: Confirm the Target

First use `kb_search(action="search", query="<company name or ticker>")` to confirm whether the target already exists in the knowledge table.

- If it exists: record its `company_name` and `stock_code`
- If it does not exist: extract the company name and ticker from the user's description and create it later

### Step 2: Record the Knowledge

Call `kb_search(action="update_knowledge", ...)` to append one key fact:

```json
{
  "action": "update_knowledge",
  "company_name": "Moutai",
  "stock_code": "600519.SH",
  "knowledge_text": "Moat: brand premium + production constraints + dealer barrier (source: user input, 2024-03)"
}
```

### Step 3: Confirm the Result

Tell the user that the knowledge has been recorded, for example:

> I have recorded "Moat: brand premium..." in Moutai (600519.SH) key knowledge.

## Tool Parameter Reference

| Parameter | Required | Description |
|---|---|---|
| `action` | Yes | Must be `"update_knowledge"` |
| `company_name` | One of the two | Company name, in Chinese or English |
| `stock_code` | One of the two | Ticker, such as `AAPL` or `600519.SH` |
| `knowledge_text` | Yes | The single knowledge item to record |

## Notes

- **Record one item at a time**: split multiple facts into multiple `update_knowledge` calls
- **Idempotency protection**: identical text will not be written twice
- **Do not overwrite existing content**: `update_knowledge` appends and does not clear history
- **Recommended format**: `<knowledge> (source: <short note>, <date>)` for future traceability
- The user can view and edit all key knowledge items later on the "Knowledge Memory" page
