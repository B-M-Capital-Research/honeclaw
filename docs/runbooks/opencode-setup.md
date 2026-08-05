# Runbook: OpenCode Setup

Last updated: 2026-08-05

## When to Use

- Installing the official `opencode` on a new machine
- Connecting your preferred provider in local OpenCode
- Preparing a reusable local environment for Hone's `opencode_acp` runner

## Prerequisites

- `curl` is installed
- You already have credentials for the provider you want OpenCode to use
- Your macOS / Linux shell can write to `~/.config` and `~/.local/share`

## 1. Install Official OpenCode

```bash
curl -fsSL https://opencode.ai/install | bash
```

Verify:

```bash
opencode --version
```

If the command still is not found in your shell, reload the shell config or confirm that the installer updated `PATH` correctly.

## 2. Connect Your Provider In OpenCode

Start the TUI:

```bash
opencode
```

In the TUI, run:

```text
/connect
```

- Choose the provider you actually want to use
- Finish the provider-side auth flow inside OpenCode

After a successful connection, credentials usually land in:

- `~/.local/share/opencode/auth.json`

Recommended default:

- Let OpenCode itself own the provider, auth, and default model
- Let Hone only set `agent.runner=opencode_acp`
- Only add Hone-side `agent.opencode.*` overrides if you explicitly want Hone to force a different model or route than your local OpenCode default

## 3. Inspect Available Models

Use the provider you connected above:

```bash
opencode models <provider>
```

For example, if you connected OpenRouter:

```bash
opencode models openrouter
```

To inspect detailed metadata and variants:

```bash
opencode models openrouter --verbose
```

Common OpenRouter examples:

- `openrouter/openai/gpt-5.4`
- `openrouter/openai/gpt-5.4-pro`

## 4. Write a Global Default Config

Config file:

- `~/.config/opencode/opencode.json` or `~/.config/opencode/opencode.jsonc`

Minimal generic example:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "model": "<provider>/<model>"
}
```

OpenRouter example:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "model": "openrouter/openai/gpt-5.4",
  "provider": {
    "openrouter": {
      "options": {
        "baseURL": "https://openrouter.ai/api/v1"
      }
    }
  }
}
```

## 5. Pin the Default Variant Explicitly

If you want the default reasoning strength for `build` and `plan` to stay at `medium`:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "model": "openrouter/openai/gpt-5.4",
  "agent": {
    "build": {
      "model": "openrouter/openai/gpt-5.4",
      "variant": "medium"
    },
    "plan": {
      "model": "openrouter/openai/gpt-5.4",
      "variant": "medium"
    }
  },
  "provider": {
    "openrouter": {
      "options": {
        "baseURL": "https://openrouter.ai/api/v1"
      }
    }
  }
}
```

Common variants:

- `none`
- `minimal`
- `low`
- `medium`
- `high`
- `xhigh`

## 6. Temporarily Override the Model For One Run

If you only want to try one model temporarily:

```bash
opencode -m <provider>/<model>
```

## 7. Verify the Setting Took Effect

```bash
opencode run "Reply with exactly: provider=<provider> model=<model> variant=<variant or none>" --print-logs
```

Check the following:

- The terminal header shows `build · openai/gpt-5.4`
- The logs show `providerID=openrouter modelID=openai/gpt-5.4`

Note: the model's spoken `variant` is not always trustworthy. If you need protocol-level truth, prefer the logs or an exported session.

### Verify the ACP entrypoint itself

The production boundary is newline-delimited JSON-RPC over stdio:

```bash
opencode acp
```

Drive `initialize -> session/new -> session/prompt` from a small local client and require a final `stopReason=end_turn`; do not treat a successful `initialize` alone as provider/model proof. The 2026-08-01 reference probe used official stable OpenCode `1.18.11` and returned `OPENCODE_ACP_OK`.

The observed `1.18.11` stream included:

- `available_commands_update`
- object-shaped `agent_thought_chunk.content.text`
- object-shaped, possibly split `agent_message_chunk.content.text`
- `usage_update` with `used`, `size`, and `cost { amount, currency }`

These fields are a versioned compatibility sample, not a promise that OpenCode and codex-acp will emit identical events. When the installed version changes, rerun the real probe before updating the fixture.

Hone reads `initialize.agentInfo.version` from the same ACP process that will receive the prompt. OpenCode `1.18.11` is the current validated fixture. A newer `1.x` release uses that dialect conservatively and is reported as `compatible_newer`; an older release, missing/unparseable version, or a new major fails before `session/prompt` until a real capture establishes a new dialect.

The versioned external capture is `tests/fixtures/acp/opencode-1.18.11.json`. It records adapter/version/capture date plus thought, split answer, and usage shapes; it is not generated from Hone's private Rust structs.

## 8. Wire It Into Hone

Recommended minimal Hone config:

File:

- `config.yaml`

Example:

```yaml
agent:
  runner: "opencode_acp"
```

Notes:

- When `agent.opencode.model` / `api_base_url` / `api_key` are empty, Hone inherits the local OpenCode config instead of overriding it
- When `agent.opencode.model` is non-empty, Hone explicitly calls `session/set_model` in the ACP session
- `agent.opencode.variant` is appended to `modelId`, for example `openrouter/openai/gpt-5.4/medium`

For the administrator-only earnings preview/analysis workflows, keep the
global runner unchanged and configure the dedicated per-turn route instead:

```yaml
agent:
  earnings_workflow:
    runner: "opencode_acp"
    model: "google/gemini-3.1-pro-preview"

llm:
  providers:
    openrouter:
      kind: "openrouter"
      base_url: "https://openrouter.ai/api/v1"
      api_key: ""
```

Put the real key only in the ignored canonical `config.yaml` through the
interactive provider configurator or another stdin-only secret path; never
commit it or pass it on a command line. Hone uses the existing OpenRouter key
pool to inject `OPENROUTER_API_KEY` only into the OpenCode child and converts
the workflow model to `openrouter/google/gemini-3.1-pro-preview` at the
transport boundary. Ordinary conversations still use `agent.runner` and its
normal model.

If you explicitly want Hone to override your local OpenCode default, then add:

```yaml
agent:
  runner: "opencode_acp"
  opencode:
    command: "opencode"
    args: ["acp"]
    model: "openrouter/openai/gpt-5.4"
    variant: "medium"
    api_base_url: "https://openrouter.ai/api/v1"
    api_key: ""
```

## 9. Troubleshooting

### `opencode models <provider>` does not show the model

- First confirm that `/connect` succeeded
- Check whether `~/.local/share/opencode/auth.json` contains the provider you just connected

### The TUI switched models, but Hone did not pick it up

- The UI switch may only be a temporary session state
- The Hone process does not reuse that temporary state by default
- Either write `~/.config/opencode/opencode.json` / `opencode.jsonc`
- Or set `agent.opencode.model` / `agent.opencode.variant` in Hone's `config.yaml`

### Hone reports that ACP set-model failed

- Confirm `opencode --version`
- Confirm that the current version supports ACP `session/set_model`
- Confirm that `agent.opencode.model` uses `<provider>/<model>` or `<provider>/<model>/<variant>`

### `initialize` succeeds but the first model prompt fails authentication

- A working ACP transport does not prove that the configured provider token is still valid
- Reconnect the provider with `/connect`, or repair the selected provider's own local auth/config
- Never paste a key into a checked-in probe fixture or capture it in protocol logs

For a worktree-safe transport/MCP probe, pass the real ignored runtime paths explicitly:

```bash
env \
  PATH="$HOME/.opencode/bin:$PATH" \
  HONE_CONFIG_PATH=/absolute/path/to/config.yaml \
  HONE_DATA_DIR=/absolute/path/to/data \
  bash tests/regression/manual/test_opencode_acp_hone_mcp.sh
```

`HONE_OPENCODE_ACP_MODEL=<provider>/<model>` optionally exercises `session/set_model` without changing the global default. A free probe model can prove ACP transport, MCP registration, reasoning/answer/usage events, and end-turn handling when the configured provider credential is temporarily unavailable; it does not prove that the production default provider is authenticated. Record both facts separately.

## 10. Delivery Check

- `opencode --version` works
- `opencode models <provider>` works
- `~/.config/opencode/opencode.json` or `opencode.jsonc` contains the default model
- `opencode run ... --print-logs` shows the target model
- If Hone is involved, `config.yaml` is also configured with `agent.runner=opencode_acp`
