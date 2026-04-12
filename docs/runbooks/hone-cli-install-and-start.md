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
- Writes a `hone-cli` wrapper to the first writable user-facing bin dir already present in `PATH` when possible
  - Preferred matches are `~/.local/bin`, `~/bin`, `~/.cargo/bin`, `~/.bun/bin`, then other writable `~/...` PATH entries
  - If none of those are available, it falls back to `~/.local/bin/hone-cli` and prints both the immediate `export PATH=...` command and a shell-specific `~/.zshrc` or `~/.bashrc` / `~/.bash_profile` persistence hint when it can identify the login shell
- Seeds `~/.honeclaw/config.yaml` and `~/.honeclaw/soul.md` if they do not already exist
- Ships built Web assets under the install bundle and points the runtime at them through `HONE_WEB_DIST_DIR`
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

## Uninstall

Homebrew uninstall only removes the package files:

```bash
brew uninstall honeclaw
```

If the formula was installed via the fully qualified tap path, this also works:

```bash
brew uninstall B-M-Capital-Research/honeclaw/honeclaw
```

If you also want to remove local Hone config, runtime data, and downloaded bundles under `~/.honeclaw`, run cleanup first:

```bash
hone-cli cleanup
```

For non-interactive full cleanup:

```bash
hone-cli cleanup --all --yes
```

If you already uninstalled Homebrew and no longer have `hone-cli`, remove the install home manually:

```bash
rm -rf ~/.honeclaw
```

## Installed Layout

- Bundle root: `~/.honeclaw/current`
- User config: `~/.honeclaw/config.yaml`
- Generated effective config: `~/.honeclaw/data/runtime/effective-config.yaml`
- Runtime data: `~/.honeclaw/data`
- Skills dir: `~/.honeclaw/current/share/honeclaw/skills`
- Web assets: `~/.honeclaw/current/share/honeclaw/web`

The wrapper exports:

- `HONE_HOME=~/.honeclaw`
- `HONE_INSTALL_ROOT=~/.honeclaw/current`
- `HONE_USER_CONFIG_PATH=~/.honeclaw/config.yaml`
- `HONE_DATA_DIR=~/.honeclaw/data`
- `HONE_SKILLS_DIR=~/.honeclaw/current/share/honeclaw/skills`
- `HONE_WEB_DIST_DIR=~/.honeclaw/current/share/honeclaw/web`

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
- If you accidentally enable a channel and then hit a required field with no value to keep, the wizard offers:
  - retry the current field
  - go back and disable that channel
- Require an explicit choice for `FMP` and `Tavily` API keys: configure now or skip for this run
  - If you configure them now, the wizard writes `fmp.api_keys` and `search.api_keys`
  - `FMP` onboarding also clears the legacy single-key field `fmp.api_key`

Runner install references shown by onboarding:

- `Codex CLI`
  - Install: `npm install -g @openai/codex`
  - Update: `codex --upgrade`
  - Official guide: [OpenAI Codex CLI – Getting Started](https://help.openai.com/en/articles/11096431)
- `Codex ACP`
  - Install `codex` first, then install `codex-acp`
  - Minimum requirement: `codex-acp >= 0.9.5`
  - Recommended update command: `npm install -g @zed-industries/codex-acp@latest`
  - If you need to pin to the minimum validated floor: `npm install -g @zed-industries/codex-acp@0.9.5`
  - Official adapter repo: [zed-industries/codex-acp](https://github.com/zed-industries/codex-acp)
- `OpenCode ACP`
  - Install: `curl -fsSL https://opencode.ai/install | bash`
  - Official docs: [OpenCode Docs](https://opencode.ai/docs/)

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

Check where the installer placed the wrapper:

```bash
command -v hone-cli || ls -l ~/.local/bin/hone-cli
```

If it fell back to `~/.local/bin`, add that directory to `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

If you installed with Homebrew and `hone-cli` is still missing, the more likely issue is that your shell has not loaded Homebrew's environment yet:

```bash
eval "$($(command -v brew) shellenv)"
```

### `hone-cli start` says a runtime binary is missing

- Reinstall with the latest GitHub bundle
- Confirm that `~/.honeclaw/current/bin/` contains `hone-console-page`, `hone-mcp`, and any enabled channel binaries

### The backend starts but the web page says assets are missing

- Reinstall with the latest GitHub bundle or the latest Homebrew formula
- Confirm that the install root contains `share/honeclaw/web/index.html`
- Confirm that `HONE_WEB_DIST_DIR` points at the bundled `share/honeclaw/web`

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
