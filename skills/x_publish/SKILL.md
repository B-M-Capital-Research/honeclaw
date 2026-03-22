---
name: X Publishing
description: Generate X (Twitter) drafts, including copy, optional thread splitting, and images. Supports multi-round revision and only publishes after an explicit approval token
tools:
  - x_draft
  - x_publish
  - image_gen
---

## X Publishing Skill

Follow a strict **two-stage confirmation** flow: create a draft -> show it to the user -> wait for the approval token -> publish.

### Publishing Flow

```
1. x_draft(action="create", ...) create a draft
2. Show the user the post content, image list, and approval token
3. Wait for the user to enter: "Confirm X publish <approval_token>"
4. x_publish(approval_token="approval token")
```

> **Strict rule:** only call `x_publish` when the user explicitly says `Confirm X publish <token>`.

### Publishing Policy

- Only reply "published" or "publish succeeded" after `x_publish` returns `success=true` and `status=published/already_published`
- After a successful publish, you **must** return the `tweet_url` to the user (for threads, link the first post)
- After a failed publish, show the error directly and tell the user what to do next

### Tool Guide

| Action | Tool call |
|------|---------|
| Create a draft | `x_draft(action="create", content="...", thread=true, image_prompt="...", image_count=3)` |
| View pending drafts | `x_draft(action="get_pending")` |
| Revise copy | `x_draft(action="revise", draft_id="ID", content="...")` |
| Toggle thread mode | `x_draft(action="revise", draft_id="ID", thread=true/false)` |
| Regenerate images | `x_draft(action="revise", draft_id="ID", image_prompt="...", image_count=3, regenerate_images=true)` |
| Use local images | `x_draft(action="revise", draft_id="ID", media_paths=["/path/a.png"])` |
| Text only | `x_draft(action="revise", draft_id="ID", media_paths=[])` |
| Cancel draft | `x_draft(action="cancel", draft_id="ID")` |
| Publish | `x_publish(approval_token="token")` or `x_publish(draft_id="ID", approval_token="token")` |
| Generate an image separately | `image_gen(image_type="general", prompt="...")` |

### Multi-Round Revision

The user may revise the draft multiple times before confirming. After each revision, show the full updated draft again, including the new approval token, and wait for confirmation.
