---
name: Hone Administrator
description: Admin-only skill for viewing or modifying Hone source code and configuration, and for performing restarts. Available only to users listed in config.yaml admins
tools:
  - restart_hone
---

## Hone Administrator Skill (hone_admin)

> **Security note:** This skill is available only to administrators listed in `config.yaml` under `admins`.
> If an unauthorized user tries to trigger admin actions, politely refuse.

---

### Project Structure Overview

```text
<PROJECT_ROOT>/               <- project root (current working directory)
├── config.yaml               <- main config file (LLM, channels, admin list, and so on)
├── launch.sh                 <- the only startup script; it builds and then launches
├── bins/
│   ├── hone-imessage/        <- iMessage channel entrypoint
│   ├── hone-discord/         <- Discord channel entrypoint
│   └── hone-telegram/        <- Telegram channel entrypoint
├── crates/
│   ├── hone-core/            <- core config and Agent interfaces
│   ├── hone-channels/        <- shared channel logic (`HoneBotCore`)
│   ├── hone-tools/           <- tool implementations, including `restart_hone`
│   ├── hone-llm/             <- LLM call layer
│   └── hone-memory/          <- session persistence
├── agents/
│   ├── gemini_cli/           <- Gemini CLI agent
│   ├── function_calling/     <- function-calling agent
│   └── codex_cli/            <- Codex CLI agent
├── skills/                   <- built-in system skills (`.md` files)
│   └── hone_admin/SKILL.md   <- this file
├── data/
│   ├── runtime/current.pid   <- PID of the current `launch.sh` process
│   └── logs/restart.log      <- restart log
└── Cargo.toml                <- Rust workspace config
```

---

### Operation 1: View or Modify Source Code or Config

**Use the gemini_cli agent (default):**
Gemini CLI runs in `--yolo` mode and can read and write files directly. Tell the user what you are about to do, then operate on the target files.

**Common edit cases:**
- Modify `config.yaml`: adjust the LLM model, timeout, prompts, admin list, and so on
- Modify `skills/<name>/SKILL.md`: update a skill's instructions or prompt
- Modify Rust source (`bins/`, `crates/`): a rebuild is required before the change takes effect

**Edit flow:**
1. Read the target file first and confirm its current content
2. Make the change
3. Show the diff or a summary to the user and confirm it looks correct
4. If Rust source changed, restart the service so the change can take effect
5. If only `config.yaml` or `skills/` changed, restart is still required because Hone loads them at startup

---

### Operation 2: Restart Hone

**When to use:** after modifying Rust source or config so the change can take effect.

**Restart tool:** `restart_hone(confirm="yes")`

**Flow:**
1. Confirm the edits are done
2. Tell the user: "Hone will restart shortly and be unavailable for about 1-3 minutes, including rebuild time"
3. Call the tool:
   ```text
   restart_hone(confirm="yes")
   ```
4. After the tool returns `success=true`, tell the user:
   - "Hone will stop and restart in about 3 seconds"
   - "The restart includes a rebuild and is expected to take about 1-3 minutes"
   - "The service will recover automatically after the restart; if it is still unresponsive after 5 minutes, check the machine manually"

**Restart mechanism:**
- The tool reads `data/runtime/current.pid` to get the current `launch.sh` PID
- It waits 3 seconds in the background so the reply can be sent first, then kills the process
- It immediately runs `nohup bash launch.sh &` in the project root, including rebuild
- The new PID is written to `data/runtime/current.pid`

---

### Operation 3: Check Runtime Status

You can use the following to check the current Hone state:
- Read `data/runtime/current.pid` to see the process PID
- Read `data/logs/restart.log` to see recent restart logs
- Read `data/logs/hone.log` to see the runtime logs

---

### Strict Rules

1. **Confirm before changing**: show a diff or summary before restarting
2. **Restart is risky**: during restart, iMessage and Discord replies may be unavailable
3. **Do not delete critical files**: never delete `config.yaml`, `launch.sh`, or the `data/` directory
4. **You must use the tool**: restarts must go through `restart_hone`; do not kill the process directly
5. **Admins only**: if the user is not in the admin list, refuse all admin actions politely
