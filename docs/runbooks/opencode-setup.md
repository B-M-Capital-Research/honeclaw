# Runbook: OpenCode Setup

Last updated: 2026-04-12

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

- `~/.config/opencode/opencode.jsonc`

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
- Either write `~/.config/opencode/opencode.jsonc`
- Or set `agent.opencode.model` / `agent.opencode.variant` in Hone's `config.yaml`

### Hone reports that ACP set-model failed

- Confirm `opencode --version`
- Confirm that the current version supports ACP `session/set_model`
- Confirm that `agent.opencode.model` uses `<provider>/<model>` or `<provider>/<model>/<variant>`

## 10. Delivery Check

- `opencode --version` works
- `opencode models <provider>` works
- `~/.config/opencode/opencode.jsonc` contains the default model
- `opencode run ... --print-logs` shows the target model
- If Hone is involved, `config.yaml` is also configured with `agent.runner=opencode_acp`
