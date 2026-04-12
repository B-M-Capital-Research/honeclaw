# Runbook: Hone CLI Install And Start

Last updated: 2026-04-12

## When To Use

- Install Hone from GitHub release assets without cloning the repo
- Prepare a local runtime that starts with `hone-cli start`
- Verify the installed bundle layout and wrapper environment

## Install From GitHub

```bash
curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash
```

Optional version pin:

```bash
HONE_VERSION=v0.1.1 curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash
```

Optional onboarding control:

```bash
HONE_RUN_ONBOARD=0 curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash
HONE_RUN_ONBOARD=1 curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash
```

The installer:

- Downloads the matching release asset such as `honeclaw-darwin-aarch64.tar.gz`
- Extracts it under `~/.honeclaw/releases/<bundle>`
- Maintains `~/.honeclaw/current` as the active symlink
- Writes a `hone-cli` wrapper to `~/.local/bin/hone-cli`
- Seeds `~/.honeclaw/config.yaml` and `~/.honeclaw/soul.md` if they do not already exist
- In an interactive terminal, asks whether to run `hone-cli onboard` immediately
  - `HONE_RUN_ONBOARD=0` skips the prompt
  - `HONE_RUN_ONBOARD=1` forces onboarding immediately

## Install With Homebrew

```bash
brew install B-M-Capital-Research/honeclaw/honeclaw
```

The standard Homebrew tap (`B-M-Capital-Research/homebrew-honeclaw`) installs the same GitHub release bundle under Homebrew `libexec`, then exposes a `hone-cli` wrapper in Homebrew `bin`.

On first run, the wrapper:

- Seeds `~/.honeclaw/config.yaml` and `~/.honeclaw/soul.md` if they do not exist
- Uses the same default `HONE_HOME`, `HONE_USER_CONFIG_PATH`, `HONE_DATA_DIR`, and `HONE_SKILLS_DIR` semantics as the `curl | bash` install
- Lets `hone-cli start` reuse the bundled runtime binaries from the Homebrew cellar without requiring `./launch.sh` or `hone-desktop`

## Installed Layout

- Bundle root: `~/.honeclaw/current`
- User config: `~/.honeclaw/config.yaml`
- Generated effective config: `~/.honeclaw/data/runtime/effective-config.yaml`
- Runtime data: `~/.honeclaw/data`
- Skills dir: `~/.honeclaw/current/share/honeclaw/skills`

The wrapper exports:

- `HONE_HOME=~/.honeclaw`
- `HONE_INSTALL_ROOT=~/.honeclaw/current`
- `HONE_USER_CONFIG_PATH=~/.honeclaw/config.yaml`
- `HONE_DATA_DIR=~/.honeclaw/data`
- `HONE_SKILLS_DIR=~/.honeclaw/current/share/honeclaw/skills`

`HONE_CONFIG_PATH` is no longer exported globally by the wrapper. It is generated and injected only for spawned runtime processes.

## First-Time Setup

Run the local checks first:

```bash
hone-cli doctor
```

Then run the guided onboarding:

```bash
hone-cli onboard
```

The onboarding flow will:

- Detect local runner binaries such as `codex`, `codex-acp`, and `opencode`
- Let you choose the default runner
- If you choose `opencode_acp`, tell you to finish provider / model setup in local `opencode` first
  - Hone defaults to inheriting `~/.config/opencode/opencode.json` or `opencode.jsonc`
- Ask whether to enable each channel
- If a channel is enabled, require its local mandatory fields and print the key permission / prerequisite notes

If you prefer the older section-by-section setup, use:

```bash
hone-cli configure --section agent --section channels --section providers
```

You can also edit individual values non-interactively:

```bash
hone-cli config set agent.runner opencode_acp
hone-cli channels set telegram --enabled true --bot-token "<token>"
```

If you want Hone to explicitly override the model used by `opencode_acp`, set it afterwards:

```bash
hone-cli models set --runner opencode_acp --model openrouter/openai/gpt-5.4 --variant medium
```

## Start The Local Runtime

```bash
hone-cli start
```

What `hone-cli start` does in the current MVP:

- Loads canonical `config.yaml`
- Generates `data/runtime/effective-config.yaml`
- Starts `hone-console-page`
- Waits for `/api/meta` on the configured web port
- Starts enabled channel listeners for iMessage / Discord / Feishu / Telegram
- Keeps the process tree in the foreground until `Ctrl-C`

Current limitation:

- `hone-cli start` is runtime-only. It does not replace all `launch.sh` desktop/web dev modes yet.

## Troubleshooting

### `hone-cli` not found

Add `~/.local/bin` to `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### `hone-cli start` says a runtime binary is missing

- Reinstall with the latest GitHub bundle
- Confirm that `~/.honeclaw/current/bin/` contains `hone-console-page`, `hone-mcp`, and any enabled channel binaries

### Config edits seem to affect the wrong file

Check:

```bash
hone-cli config file
```

The installed wrapper should point to `~/.honeclaw/config.yaml`.

### Homebrew install fails to resolve the formula

If you previously tapped the wrong remote, untap it and install again:

```bash
brew untap B-M-Capital-Research/honeclaw
brew install B-M-Capital-Research/honeclaw/honeclaw
```
