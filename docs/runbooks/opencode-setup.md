# Runbook: OpenCode Setup

Last updated: 2026-03-17

## When to Use

- Installing the official `opencode` on a new machine
- Connecting OpenRouter and explicitly selecting a default model
- Preparing a reusable local environment for Hone's `opencode_acp` runner

## Prerequisites

- `curl` is installed
- You already have an OpenRouter API key
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

## 2. Connect OpenRouter

Start the TUI:

```bash
opencode
```

In the TUI, run:

```text
/connect
```

- Choose `OpenRouter`
- Enter or paste the OpenRouter API key

After a successful connection, credentials usually land in:

- `~/.local/share/opencode/auth.json`

## 3. Inspect Available Models

```bash
opencode models openrouter
```

To inspect detailed metadata and variants:

```bash
opencode models openrouter --verbose
```

Common models in current use:

- `openrouter/openai/gpt-5.4`
- `openrouter/openai/gpt-5.4-pro`

## 4. Write a Global Default Config

Config file:

- `~/.config/opencode/opencode.jsonc`

Minimal example:

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

Common OpenAI / OpenRouter variants:

- `none`
- `minimal`
- `low`
- `medium`
- `high`
- `xhigh`

## 6. Temporarily Override the Model for One Run

If you only want to try one model temporarily:

```bash
opencode -m openrouter/openai/gpt-5.4
```

## 7. Verify the Setting Took Effect

```bash
opencode run "Reply with exactly: provider=<provider> model=<model> variant=<variant or none>" --print-logs
```

Check the following:

- The terminal header shows `build · openai/gpt-5.4`
- The logs show `providerID=openrouter modelID=openai/gpt-5.4`

Note: the model's spoken `variant` is not always trustworthy. If you need protocol-level truth, prefer the logs or an exported session.

## 8. Wire It Into Hone

If the machine also runs Hone, add one explicit layer in the repo config:

File:

- `config.yaml`

Example:

```yaml
agent:
  runner: "opencode_acp"
  opencode:
    command: "opencode"
    args: ["acp"]
    model: "openrouter/openai/gpt-5.4"
    variant: "medium"
```

Notes:

- When `agent.opencode.model` is empty, Hone only uses the local `opencode` default config
- When `agent.opencode.model` is non-empty, Hone explicitly calls `session/set_model` in the ACP session
- `agent.opencode.variant` is appended to `modelId`, for example `openrouter/openai/gpt-5.4/medium`

## 9. Troubleshooting

### `opencode models openrouter` does not show the model

- First confirm that `/connect` succeeded
- Check whether `~/.local/share/opencode/auth.json` contains `openrouter`

### The TUI switched models, but Hone did not pick it up

- The UI switch may only be a temporary session state
- The Hone process does not reuse that temporary state by default
- Either write `~/.config/opencode/opencode.jsonc`
- Or set `agent.opencode.model` / `agent.opencode.variant` in Hone's `config.yaml`

### Hone reports that ACP set-model failed

- Confirm `opencode --version`
- Confirm that the current version supports ACP `session/set_model`
- Confirm that `agent.opencode.model` uses `<provider>/<model>` or `<provider>/<model>/<variant>`

## 10. Delivery Check

- `opencode --version` works
- `opencode models openrouter` works
- `~/.config/opencode/opencode.jsonc` contains the default model
- `opencode run ... --print-logs` shows the target model
- If Hone is involved, `config.yaml` is also configured with `agent.runner=opencode_acp`
