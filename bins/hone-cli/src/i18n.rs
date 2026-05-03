//! `hone-cli` bilingual prompt strings.
//!
//! Two-locale (zh / en) lookup used by the `onboard` wizard and other
//! interactive subcommands. The flow:
//!
//! 1. `detect_initial_lang()` reads `LC_ALL` / `LANG` from the host
//!    environment and returns `Lang::Zh` for `zh*` variants, `Lang::En`
//!    otherwise. This is the *default selection* shown on Step 1; the user
//!    can always flip it.
//! 2. The chosen `Lang` threads through the rest of onboard via explicit
//!    parameters. `t(lang, key)` returns the localized string.
//! 3. Missing keys fall back to the key itself so a stray `t(lang, "foo.bar")`
//!    surfaces visibly in dev rather than silently rendering empty.
//!
//! The string table is intentionally compact — only Step 1 and a handful of
//! shared chrome strings (banner, apply summary) are translated here. Other
//! prompts in `onboard.rs` keep their original Chinese for now; they can be
//! migrated to `t!()` in a follow-up commit without touching this module's
//! shape.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lang {
    Zh,
    En,
}

impl Lang {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
        }
    }

    /// Map `hone_core::config::Locale` (the persisted enum) to the in-CLI
    /// `Lang` used by the `t()` lookup. Lets non-onboard subcommands localize
    /// their messages from whatever the user picked during onboarding.
    pub(crate) fn from_locale(locale: hone_core::config::Locale) -> Self {
        match locale {
            hone_core::config::Locale::Zh => Self::Zh,
            hone_core::config::Locale::En => Self::En,
        }
    }
}

impl Default for Lang {
    fn default() -> Self {
        Self::Zh
    }
}

/// Best-effort lookup of the persisted CLI language. Tries the canonical
/// config first; falls back to host-locale detection so subcommands run
/// before onboarding still pick a sensible language.
pub(crate) fn resolve_lang(config_path: Option<&std::path::Path>) -> Lang {
    if let Ok(paths) = crate::common::resolve_runtime_paths(config_path, false) {
        if let Ok(config) = hone_core::HoneConfig::from_file(&paths.canonical_config_path) {
            return Lang::from_locale(config.language);
        }
    }
    detect_initial_lang()
}

/// Read the host locale from `LC_ALL` then `LANG`. Anything starting with
/// `zh` (case-insensitive) maps to `Lang::Zh`; everything else (and unset)
/// maps to `Lang::En`. Used to seed the default selection on Step 1; the
/// user can override and the override is what gets persisted.
pub(crate) fn detect_initial_lang() -> Lang {
    for var in ["LC_ALL", "LANG"] {
        if let Ok(raw) = std::env::var(var) {
            let head = raw.split('.').next().unwrap_or("").to_ascii_lowercase();
            if head.starts_with("zh") {
                return Lang::Zh;
            }
            if !head.is_empty() {
                return Lang::En;
            }
        }
    }
    Lang::En
}

/// Look up a localized string for the given (lang, key). Returns the key
/// itself for missing entries — that's louder than returning an empty string
/// and helps catch typos during dev.
pub(crate) fn t(lang: Lang, key: &str) -> &'static str {
    let pair = STRINGS.iter().find(|(k, _, _)| *k == key);
    match (pair, lang) {
        (Some((_, zh, _)), Lang::Zh) => zh,
        (Some((_, _, en)), Lang::En) => en,
        // Leak missing keys into the UI so they're visible, not invisible.
        (None, _) => leak(key),
    }
}

fn leak(key: &str) -> &'static str {
    Box::leak(key.to_string().into_boxed_str())
}

/// (key, zh, en). Keys are dotted namespaces — `step.<n>` for step labels,
/// `lang.*` for the language step itself, `apply.*` for the apply summary.
const STRINGS: &[(&str, &str, &str)] = &[
    // ── Banner / chrome ──────────────────────────────────────────────────
    (
        "banner.title",
        "Hone onboarding",
        "Hone onboarding",
    ),
    (
        "banner.subtitle",
        "约 3–5 分钟,全程键盘即可。Ctrl+C 可安全退出:mutation 只在最后一步才写盘。",
        "About 3–5 minutes, fully keyboard-driven. Ctrl+C is safe — mutations only write on the final step.",
    ),
    (
        "banner.hint",
        "每个环节都可以跳过,之后再通过 `hone-cli onboard` 或其他 CLI 子命令补配。",
        "Every step can be skipped; you can rerun `hone-cli onboard` or use other CLI subcommands later.",
    ),
    // ── Step 1 — Language ────────────────────────────────────────────────
    ("step.language", "Language / 界面语言", "Language / 界面语言"),
    (
        "lang.prompt",
        "请选择控制台与 CLI 默认语言 / Choose console + CLI default language",
        "Choose console + CLI default language / 请选择控制台与 CLI 默认语言",
    ),
    ("lang.option_zh", "中文 (zh)", "中文 (zh)"),
    ("lang.option_en", "English (en)", "English (en)"),
    (
        "lang.note",
        "保存后写入 config.yaml.language;Web 控制台首次加载使用此默认值,各设备仍可单独覆盖。",
        "Saved into config.yaml.language. The web console picks this up on first load; devices can still override locally.",
    ),
    // ── Step labels (shifted by +1 due to language step at front) ────────
    ("step.runner", "Runner", "Runner"),
    ("step.channels", "Channels", "Channels"),
    ("step.admins", "Admins", "Admins"),
    ("step.providers", "Providers", "Providers"),
    ("step.notifications", "Notifications", "Notifications"),
    ("step.apply", "Apply", "Apply"),
    // ── Apply summary ────────────────────────────────────────────────────
    (
        "apply.fields_written",
        "(共写入 {n} 条字段)",
        "({n} fields written)",
    ),
    (
        "apply.canonical_path",
        "canonical config → {p}",
        "canonical config → {p}",
    ),
    (
        "apply.effective_path",
        "effective config → {p}",
        "effective config → {p}",
    ),
    (
        "apply.run_doctor",
        "Run `hone-cli doctor` now?",
        "Run `hone-cli doctor` now?",
    ),
    ("apply.start_now", "Start Hone now?", "Start Hone now?"),
    (
        "apply.complete",
        "Onboarding complete",
        "Onboarding complete",
    ),
    ("apply.next_steps", "下一步:", "Next steps:"),
    (
        "apply.tip_status",
        "`hone-cli status`   快速查当前配置",
        "`hone-cli status`   inspect current config",
    ),
    (
        "apply.tip_doctor",
        "`hone-cli doctor`   深度体检(路径 / binary / auth)",
        "`hone-cli doctor`   deep healthcheck (paths / binaries / auth)",
    ),
    (
        "apply.tip_start",
        "`hone-cli start`    启动 Hone + 启用渠道",
        "`hone-cli start`    start Hone + enabled channels",
    ),
    // ── yaml_io::apply_message ───────────────────────────────────────────
    (
        "yaml.apply.saved_live",
        "配置已保存，已立即生效",
        "Config saved and applied live",
    ),
    (
        "yaml.apply.saved_restart_required",
        "配置已保存，需重启运行时",
        "Config saved; runtime restart required",
    ),
    (
        "yaml.apply.saved_restart_components",
        "配置已保存，并需重启组件：{components}",
        "Config saved; restart required for components: {components}",
    ),
    // ── Runner step ──────────────────────────────────────────────────────
    (
        "runner.choose_default",
        "Choose the default runner",
        "Choose the default runner",
    ),
    (
        "runner.badge_no_binary",
        "no binary needed",
        "no binary needed",
    ),
    (
        "runner.badge_installed",
        "{binary} installed",
        "{binary} installed",
    ),
    (
        "runner.badge_missing",
        "{binary} missing",
        "{binary} missing",
    ),
    (
        "runner.binary_detected",
        "{binary} 已检测到可用。",
        "{binary} detected and ready to use.",
    ),
    (
        "runner.binary_missing_detail",
        "{binary} 未检测到({detail})。",
        "{binary} not detected ({detail}).",
    ),
    (
        "runner.keep_without_binary",
        "缺少 binary,仍然保留这个 runner?(no = 返回重新选择 runner)",
        "Binary missing — keep this runner anyway? (no = pick another runner)",
    ),
    (
        "runner.multi_agent.description",
        "默认推荐:search + answer 两段式,纯 HTTP 走 OpenRouter,不需要本机 CLI。",
        "Recommended default: two-stage search + answer over OpenRouter HTTP, no local CLI required.",
    ),
    (
        "runner.multi_agent.note_1",
        "前置:一把可用的 OpenRouter API key(后面 Providers 环节会让你填)。",
        "Requires: one working OpenRouter API key (collected later in the Providers step).",
    ),
    (
        "runner.multi_agent.note_2",
        "原理:第一段 search 用小模型拉证据,第二段 answer 用主模型总结。",
        "How it works: stage 1 (search) pulls evidence with a small model; stage 2 (answer) summarizes with the primary model.",
    ),
    (
        "runner.multi_agent.note_3",
        "适合:只有 API key、不想装 CLI 的用户。",
        "Best for: users who only have an API key and don't want to install a local CLI.",
    ),
    (
        "runner.multi_agent.note_4",
        "需要在本机切换模型时,之后用 `hone-cli models set ...` 即可,不必重跑 onboard。",
        "To switch models later, run `hone-cli models set ...` — no need to rerun onboard.",
    ),
    (
        "runner.codex_cli.description",
        "优先复用本机 codex CLI 登录态；适合已经能直接运行 codex 的用户。",
        "Reuses your local codex CLI login. Best for users who can already run `codex`.",
    ),
    (
        "runner.codex_cli.note_1",
        "前置：本机可执行 `codex --version`。",
        "Requires: `codex --version` executes locally.",
    ),
    (
        "runner.codex_cli.note_2",
        "优点：不需要单独填写 OpenAI-compatible base URL / API key。",
        "Upside: no separate OpenAI-compatible base URL / API key to fill in.",
    ),
    (
        "runner.codex_cli.note_3",
        "安装：`npm install -g @openai/codex`；已安装可用 `codex --upgrade` 更新。",
        "Install: `npm install -g @openai/codex`; if already installed, `codex --upgrade` will update.",
    ),
    (
        "runner.codex_cli.note_4",
        "官方说明：https://help.openai.com/en/articles/11096431",
        "Official docs: https://help.openai.com/en/articles/11096431",
    ),
    (
        "runner.codex_acp.description",
        "通过 codex-acp 接入 ACP 协议；需要本机同时具备 codex 与 codex-acp。",
        "Talks to ACP via `codex-acp`. Requires both `codex` and `codex-acp` locally.",
    ),
    (
        "runner.codex_acp.note_1",
        "前置：本机可执行 `codex --version` 与 `codex-acp --help`。",
        "Requires: `codex --version` and `codex-acp --help` both execute locally.",
    ),
    (
        "runner.codex_acp.note_2",
        "可额外配置 model / variant / sandbox policy。",
        "Optional: customize model / variant / sandbox policy.",
    ),
    (
        "runner.codex_acp.note_3",
        "安装：先装 `codex`，再装 `codex-acp`；Hone 当前最低要求是 `codex-acp >= 0.9.5`。",
        "Install `codex` first, then `codex-acp`. Hone requires `codex-acp >= 0.9.5`.",
    ),
    (
        "runner.codex_acp.note_4",
        "更新：`npm install -g @zed-industries/codex-acp@latest`。",
        "Update: `npm install -g @zed-industries/codex-acp@latest`.",
    ),
    (
        "runner.codex_acp.note_5",
        "官方说明：https://github.com/zed-industries/codex-acp",
        "Official docs: https://github.com/zed-industries/codex-acp",
    ),
    (
        "runner.opencode_acp.description",
        "通过 `opencode acp` 接入本机 OpenCode；优先复用你已经在 opencode 里配好的 provider / model。",
        "Talks to your local OpenCode via `opencode acp`. Reuses the provider / model you've configured in opencode.",
    ),
    (
        "runner.opencode_acp.note_1",
        "前置：本机可执行 `opencode --version`。",
        "Requires: `opencode --version` executes locally.",
    ),
    (
        "runner.opencode_acp.note_2",
        "默认不在 Hone 首装里填写 provider / base URL / API key。",
        "Hone onboarding doesn't ask for provider / base URL / API key for this runner.",
    ),
    (
        "runner.opencode_acp.note_3",
        "安装：`curl -fsSL https://opencode.ai/install | bash`。",
        "Install: `curl -fsSL https://opencode.ai/install | bash`.",
    ),
    (
        "runner.opencode_acp.note_4",
        "官方说明：https://opencode.ai/docs/",
        "Official docs: https://opencode.ai/docs/",
    ),
    (
        "runner.opencode_acp.note_5",
        "请先在 `opencode` 里通过 `/connect` 或全局 `opencode.json` / `opencode.jsonc` 配好默认模型。",
        "Configure the default model in `opencode` first via `/connect` or a global `opencode.json` / `opencode.jsonc`.",
    ),
    (
        "runner.opencode_acp.note_6",
        "如果需要 Hone 显式覆盖 opencode 默认模型，再用 `hone-cli models set ...`。",
        "Use `hone-cli models set ...` later if you want Hone to override opencode's default model.",
    ),
    (
        "runner.multi_agent.setup_title",
        "Multi-Agent setup",
        "Multi-Agent setup",
    ),
    (
        "runner.multi_agent.setup_note_1",
        "本 runner 只会写入 `agent.runner = \"multi-agent\"`;",
        "This runner only writes `agent.runner = \"multi-agent\"`.",
    ),
    (
        "runner.multi_agent.setup_note_2",
        "实际跑起来需要一把 OpenRouter API key,Providers 环节会让你填。",
        "It needs an OpenRouter API key to actually run — you'll fill it in during the Providers step.",
    ),
    (
        "runner.multi_agent.setup_note_3",
        "进阶:`multi-agent.search` / `answer` 两段模型可用 `hone-cli models set ...` 微调。",
        "Advanced: tune the `multi-agent.search` / `answer` stage models via `hone-cli models set ...`.",
    ),
    (
        "runner.codex_cli.model_prompt",
        "Codex CLI model（留空则使用 codex 默认模型）",
        "Codex CLI model (leave blank to use codex's default)",
    ),
    (
        "runner.codex_acp.model_prompt",
        "Codex ACP model",
        "Codex ACP model",
    ),
    (
        "runner.codex_acp.variant_prompt",
        "Codex ACP variant",
        "Codex ACP variant",
    ),
    (
        "runner.opencode_acp.setup_title",
        "OpenCode ACP setup",
        "OpenCode ACP setup",
    ),
    (
        "runner.opencode_acp.setup_note_1",
        "Hone 首装默认只切换 runner，不在这里强行写 provider / API key / model。",
        "Hone onboarding only switches the runner here — it won't overwrite your provider / API key / model.",
    ),
    (
        "runner.opencode_acp.setup_note_2",
        "请先用 `opencode` 自己完成 `/connect`、provider 选择和默认模型配置。",
        "Run `opencode` first to handle `/connect`, provider selection, and default model setup.",
    ),
    (
        "runner.opencode_acp.setup_note_3",
        "如果之后需要 Hone 显式覆盖 opencode 默认模型，再运行 `hone-cli models set ...`。",
        "Run `hone-cli models set ...` later if you need Hone to override opencode's default model.",
    ),
    (
        "runner.opencode_acp.confirm_connected",
        "你已经在 opencode 里 `/connect` 并选好默认模型了吗?",
        "Have you already run `/connect` in opencode and picked a default model?",
    ),
    (
        "runner.opencode_acp.warn_not_connected",
        "继续写入 runner 配置;请记得稍后执行 `opencode` 并 `/connect` 配好 provider,否则 Hone 起 chat 会立刻失败。",
        "Writing the runner config anyway. Remember to run `opencode` and `/connect` later — Hone chat will fail immediately otherwise.",
    ),
    // ── Channels step ────────────────────────────────────────────────────
    (
        "channel.hint_skip",
        "每个渠道可以先跳过,之后用 `hone-cli onboard` / `hone-cli configure` / `hone-cli channels ...` 补配。",
        "Channels can be skipped now and configured later via `hone-cli onboard` / `hone-cli configure` / `hone-cli channels ...`.",
    ),
    (
        "channel.imessage.skipped_non_macos",
        "iMessage 渠道仅 macOS 可用,当前平台跳过。",
        "iMessage is macOS-only — skipping on this platform.",
    ),
    (
        "channel.enable_prompt",
        "Enable {label} channel?",
        "Enable {label} channel?",
    ),
    (
        "channel.prerequisites_title",
        "{label} prerequisites",
        "{label} prerequisites",
    ),
    (
        "channel.allow_warn",
        "{label} 渠道默认 allow 白名单为空,即所有联系人都能触发 Hone。",
        "{label} channel ships with an empty allowlist, so any contact can trigger Hone.",
    ),
    (
        "channel.allow_hint",
        "如需限定,onboard 完成后用 `hone-cli configure --section channels` 或直接编辑 config.yaml。",
        "To restrict access, run `hone-cli configure --section channels` after onboarding or edit config.yaml directly.",
    ),
    (
        "channel.disabled_via_recovery",
        "已返回并禁用 {label} 渠道。",
        "Disabled {label} channel.",
    ),
    (
        "channel.required_field_empty",
        "该字段为必填项，不能为空。",
        "This field is required and cannot be empty.",
    ),
    (
        "channel.chat_scope_prompt",
        "{label} chat scope",
        "{label} chat scope",
    ),
    (
        "channel.imessage.label",
        "iMessage",
        "iMessage",
    ),
    (
        "channel.imessage.status_note",
        "仅 macOS 可用。",
        "macOS only.",
    ),
    (
        "channel.imessage.note_1",
        "需要 macOS。",
        "Requires macOS.",
    ),
    (
        "channel.imessage.note_2",
        "需要给运行 hone-cli 的终端应用授予“完全磁盘访问权限”。",
        "Grant Full Disk Access to the terminal app running hone-cli.",
    ),
    (
        "channel.imessage.note_3",
        "Hone 会轮询 `~/Library/Messages/chat.db`，并通过 AppleScript 发消息。",
        "Hone polls `~/Library/Messages/chat.db` and sends messages via AppleScript.",
    ),
    (
        "channel.imessage.target_handle_prompt",
        "iMessage target handle（可选；留空表示监听所有会话）",
        "iMessage target handle (optional; leave blank to watch all conversations)",
    ),
    (
        "channel.feishu.label",
        "Feishu",
        "Feishu",
    ),
    (
        "channel.feishu.note_1",
        "需要飞书开放平台应用的 `app_id` 与 `app_secret`。",
        "Requires `app_id` and `app_secret` from a Feishu Open Platform app.",
    ),
    (
        "channel.feishu.note_2",
        "平台侧需要完成 Bot / 事件接入与长连接相关配置。",
        "On the platform side, finish Bot setup, event subscription, and long-connection configuration.",
    ),
    (
        "channel.feishu.note_3",
        "本地只负责写入必填配置，不会替你开通平台权限。",
        "Hone only writes the required local config — it can't grant platform-side permissions for you.",
    ),
    (
        "channel.feishu.app_id_prompt",
        "Feishu app id",
        "Feishu app id",
    ),
    (
        "channel.feishu.app_secret_prompt",
        "Feishu app secret",
        "Feishu app secret",
    ),
    (
        "channel.telegram.label",
        "Telegram",
        "Telegram",
    ),
    (
        "channel.telegram.status_note",
        "当前仍偏实验/placeholder 模式，不建议当成熟生产渠道使用。",
        "Currently experimental / placeholder — not yet recommended for production.",
    ),
    (
        "channel.telegram.note_1",
        "需要 BotFather 创建的 bot token。",
        "Requires a bot token created via BotFather.",
    ),
    (
        "channel.telegram.note_2",
        "需要把 bot 加入目标私聊或群聊。",
        "Add the bot to the target DM or group chat.",
    ),
    (
        "channel.telegram.note_3",
        "如果想处理群聊普通消息，通常还需要检查 BotFather 的 privacy mode 设置。",
        "To process plain group messages, you usually also need to check the BotFather privacy mode setting.",
    ),
    (
        "channel.telegram.bot_token_prompt",
        "Telegram bot token",
        "Telegram bot token",
    ),
    (
        "channel.discord.label",
        "Discord",
        "Discord",
    ),
    (
        "channel.discord.note_1",
        "需要 Discord bot token。",
        "Requires a Discord bot token.",
    ),
    (
        "channel.discord.note_2",
        "需要把 bot 邀请进目标 server/channel。",
        "Invite the bot to the target server / channel.",
    ),
    (
        "channel.discord.note_3",
        "至少要给 bot 查看频道、读取历史消息、发送消息等基础权限。",
        "Grant the bot basic permissions: view channel, read message history, send messages.",
    ),
    (
        "channel.discord.bot_token_prompt",
        "Discord bot token",
        "Discord bot token",
    ),
    // ── Admin step ───────────────────────────────────────────────────────
    (
        "admin.hint_purpose",
        "管理员白名单决定谁能触发 `/register-admin` / `/report` / 重启 Hone 等管理指令。",
        "The admin allowlist controls who can trigger `/register-admin`, `/report`, restart Hone, and similar admin commands.",
    ),
    (
        "admin.hint_empty",
        "不配就没人是 admin,本机所有人都触发不到管理能力。",
        "Leave it empty and nobody will be admin — no one can invoke admin commands.",
    ),
    (
        "admin.add_self_prompt",
        "把自己加为已启用渠道的 admin 白名单?",
        "Add yourself to the admin allowlist for the enabled channels?",
    ),
    (
        "admin.skipped_hint",
        "已跳过 admin 配置;之后可用 `hone-cli configure` 或直接编辑 config.yaml 的 `admins.*`。",
        "Skipped admin setup. Run `hone-cli configure` later or edit `admins.*` in config.yaml.",
    ),
    (
        "admin.imessage.handle_prompt",
        "iMessage admin handle(手机号带国家码,如 +8613800138000 或 Apple ID 邮箱;留空跳过)",
        "iMessage admin handle (phone number with country code, e.g. +8613800138000, or Apple ID email; blank to skip)",
    ),
    (
        "admin.telegram.user_id_prompt",
        "Telegram admin user id(数字 ID,如 8039067465;可通过 @userinfobot 获取;留空跳过)",
        "Telegram admin user id (numeric ID, e.g. 8039067465; get it from @userinfobot; blank to skip)",
    ),
    (
        "admin.discord.user_id_prompt",
        "Discord admin user id(数字 ID,18 位数,可在 Discord 开发者模式下右键用户头像复制;留空跳过)",
        "Discord admin user id (18-digit numeric ID, copy via right-click on the user avatar in Developer Mode; blank to skip)",
    ),
    (
        "admin.feishu.choice_email",
        "邮箱(admin@example.com)",
        "Email (admin@example.com)",
    ),
    (
        "admin.feishu.choice_mobile",
        "手机号(+8613800138000)",
        "Mobile number (+8613800138000)",
    ),
    (
        "admin.feishu.choice_open_id",
        "open_id(ou_xxx)",
        "open_id (ou_xxx)",
    ),
    (
        "admin.feishu.choice_skip",
        "跳过",
        "Skip",
    ),
    (
        "admin.feishu.kind_prompt",
        "Feishu admin 用哪种 id 添加?",
        "Which kind of id should be used for the Feishu admin?",
    ),
    (
        "admin.feishu.email_prompt",
        "Feishu admin 邮箱",
        "Feishu admin email",
    ),
    (
        "admin.feishu.mobile_prompt",
        "Feishu admin 手机号(推荐带国家码,如 +8613800138000)",
        "Feishu admin mobile (country code recommended, e.g. +8613800138000)",
    ),
    (
        "admin.feishu.open_id_prompt",
        "Feishu admin open_id",
        "Feishu admin open_id",
    ),
    // ── Provider step ────────────────────────────────────────────────────
    (
        "provider.hint_explicit",
        "OpenRouter / FMP / Tavily 都会要求你明确选择:现在填写,或本轮跳过。",
        "OpenRouter / FMP / Tavily each require an explicit choice: fill now, or skip for this run.",
    ),
    (
        "provider.hint_skip_later",
        "跳过不会阻塞 onboarding,之后仍可用 `hone-cli configure --section providers` 补配。",
        "Skipping won't block onboarding — you can configure providers later via `hone-cli configure --section providers`.",
    ),
    (
        "provider.api_keys_title",
        "{label} API keys",
        "{label} API keys",
    ),
    (
        "provider.configure_prompt",
        "Configure {label} API keys now?",
        "Configure {label} API keys now?",
    ),
    (
        "provider.skip_message",
        "已跳过 {label} API key 配置。",
        "Skipped {label} API key configuration.",
    ),
    (
        "provider.saved_message",
        "已保存 {label} API keys。",
        "Saved {label} API keys.",
    ),
    (
        "provider.keep_existing_message",
        "保留现有 {label} API key 配置。",
        "Kept existing {label} API key configuration.",
    ),
    (
        "provider.keys_required_or_skip",
        "请至少输入一个有效 key，或选择跳过。",
        "Provide at least one valid key, or choose to skip.",
    ),
    (
        "provider.openrouter.label",
        "OpenRouter",
        "OpenRouter",
    ),
    (
        "provider.openrouter.prompt",
        "OpenRouter API keys（逗号分隔）",
        "OpenRouter API keys (comma-separated)",
    ),
    (
        "provider.openrouter.note_1",
        "LLM 主路由。multi-agent / codex_* / nano_banana 都默认走这里。",
        "Primary LLM route. multi-agent / codex_* / nano_banana all default to this provider.",
    ),
    (
        "provider.openrouter.note_2",
        "如果你 runner=opencode_acp 且已在 opencode 里配好 provider,可以在下一步跳过。",
        "If your runner is opencode_acp and the provider is already configured inside opencode, you can skip this step.",
    ),
    (
        "provider.openrouter.note_3",
        "支持一次填写多个 key,运行时会自动 fallback。",
        "Multiple keys are supported; runtime will fall back automatically.",
    ),
    (
        "provider.fmp.label",
        "FMP",
        "FMP",
    ),
    (
        "provider.fmp.prompt",
        "FMP API keys（逗号分隔）",
        "FMP API keys (comma-separated)",
    ),
    (
        "provider.fmp.note_1",
        "用于 `data_fetch` 等金融数据能力。",
        "Used by `data_fetch` and other financial-data capabilities.",
    ),
    (
        "provider.fmp.note_2",
        "支持一次填写多个 key，运行时会自动 fallback。",
        "Multiple keys are supported; runtime will fall back automatically.",
    ),
    (
        "provider.tavily.label",
        "Tavily",
        "Tavily",
    ),
    (
        "provider.tavily.prompt",
        "Tavily API keys（逗号分隔）",
        "Tavily API keys (comma-separated)",
    ),
    (
        "provider.tavily.note_1",
        "用于 `web_search` 等联网搜索能力。",
        "Used by `web_search` and other web-search capabilities.",
    ),
    (
        "provider.tavily.note_2",
        "支持一次填写多个 key，运行时会自动 fallback。",
        "Multiple keys are supported; runtime will fall back automatically.",
    ),
    // ── Notifications step ───────────────────────────────────────────────
    (
        "notifications.defaults_title",
        "新用户默认行为",
        "Defaults for new users",
    ),
    (
        "notifications.defaults_1",
        "Global digest:默认对所有新用户**开启**,LLM 精读后每天按窗口推送到 chat。",
        "Global digest: enabled by default for all new users — an LLM curates and pushes it to chat once per window.",
    ),
    (
        "notifications.defaults_2",
        "Per-event 通知:默认开启(Severity::Low 起、不限 portfolio)。",
        "Per-event notifications: enabled by default (from Severity::Low, all portfolios).",
    ),
    (
        "notifications.defaults_3",
        "投资主线自动蒸馏:后台 cron 周扫 sandbox `company_profiles/*/profile.md`,无需用户操作。",
        "Investment mainline auto-distillation: a background cron weekly-scans sandbox `company_profiles/*/profile.md` — no user action needed.",
    ),
    (
        "notifications.user_adjust_title",
        "终端用户如何调整",
        "How end-users adjust",
    ),
    (
        "notifications.user_adjust_1",
        "用自然语言告诉 bot 即可,例如「关闭 digest」「不要每天推送」「只看 portfolio」。",
        "Tell the bot in natural language, e.g. \"disable digest\", \"stop daily pushes\", \"portfolio only\".",
    ),
    (
        "notifications.user_adjust_2",
        "对应 `notification_prefs_tool`,无需 Web UI。",
        "Backed by `notification_prefs_tool` — no web UI required.",
    ),
    (
        "notifications.change_default_title",
        "如何改默认值",
        "How to change defaults",
    ),
    (
        "notifications.change_default_1",
        "默认值在 `crates/hone-event-engine/src/prefs.rs::NotificationPrefs::default()`。",
        "Defaults live in `crates/hone-event-engine/src/prefs.rs::NotificationPrefs::default()`.",
    ),
    (
        "notifications.change_default_2",
        "目前没有 config.yaml 入口;想改默认只能改源码后重新编译。",
        "There's no config.yaml entry yet; changing the defaults requires editing source and rebuilding.",
    ),
    (
        "notifications.advance_hint",
        "此步骤纯告知,不写任何配置;下一步进入 Apply。",
        "This step is informational only — no config is written. Next step: Apply.",
    ),
    // ── Recovery prompts ─────────────────────────────────────────────────
    (
        "recovery.option_retry",
        "重试当前字段",
        "Retry this field",
    ),
    (
        "recovery.option_disable_channel",
        "返回并禁用 {label} 渠道",
        "Go back and disable {label} channel",
    ),
    (
        "recovery.channel_required_empty_prompt",
        "{label} 的必填项“{field}”为空，下一步？",
        "{label} requires \"{field}\" — what next?",
    ),
    (
        "recovery.option_provider_skip",
        "跳过 {label} API key 配置",
        "Skip {label} API key configuration",
    ),
    (
        "recovery.provider_empty_prompt",
        "{label} API key 为空，下一步？",
        "{label} API key is empty — what next?",
    ),
    (
        "recovery.option_discord_token_retry",
        "重新输入 Discord bot token",
        "Re-enter Discord bot token",
    ),
    (
        "recovery.discord_token_invalid_prompt",
        "{label} 的 Discord token 格式不合法，下一步？",
        "{label}: Discord token format is invalid — what next?",
    ),
    (
        "recovery.discord_token_retry_hint",
        "请重新输入 Discord bot token。",
        "Please re-enter the Discord bot token.",
    ),
    // ── Discord token validation ─────────────────────────────────────────
    (
        "discord_token.empty",
        "Token 不能为空。",
        "Token cannot be empty.",
    ),
    (
        "discord_token.bad_segments",
        "Token 必须是三段结构（形如 xxx.yyy.zzz）。",
        "Token must have three segments (xxx.yyy.zzz).",
    ),
    (
        "discord_token.bad_charset",
        "Token 包含非法字符，应为 base64url 字符集。",
        "Token contains illegal characters; expected the base64url charset.",
    ),
    (
        "discord_token.too_short",
        "Token 长度偏短，请确认是否粘贴完整。",
        "Token is suspiciously short — confirm you pasted it fully.",
    ),
    (
        "discord_token.too_long",
        "Token 长度异常偏长，请检查是否重复粘贴。",
        "Token is unusually long — check for an accidental double-paste.",
    ),
    (
        "discord_token.valid_with_len",
        "Token 格式有效(长度={len})。",
        "Token format is valid (length={len}).",
    ),
    (
        "discord_token.message_with_len",
        "{message}(长度={len})。",
        "{message} (length={len}).",
    ),
    (
        "discord_token.confirm_use",
        "仍然使用这个 Discord token?",
        "Use this Discord token anyway?",
    ),
    (
        "discord_token.confirm_save",
        "仍然保存这个 Discord token?",
        "Save this Discord token anyway?",
    ),
    (
        "discord_token.confirm_retry",
        "Token 格式异常,重新输入?",
        "Token format looks invalid — re-enter?",
    ),
    (
        "discord_token.doctor_ok",
        "Discord token 基本格式有效（长度={len}）。",
        "Discord token format looks valid (length={len}).",
    ),
    // ── Misc ─────────────────────────────────────────────────────────────
    (
        "tty.required",
        "`hone-cli onboard` 需要交互式终端（TTY）",
        "`hone-cli onboard` requires an interactive terminal (TTY)",
    ),
];

/// Substitute `{name}` placeholders in a template with values from `vars`.
/// Used at the call site like `tpl(t(lang, "apply.fields_written"), &[("n", &n)])`.
pub(crate) fn tpl(template: &str, vars: &[(&str, &dyn std::fmt::Display)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{}}}", k), &v.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_key_returns_key() {
        assert_eq!(t(Lang::Zh, "no.such.key"), "no.such.key");
        assert_eq!(t(Lang::En, "no.such.key"), "no.such.key");
    }

    #[test]
    fn lang_str_round_trip() {
        assert_eq!(Lang::Zh.as_str(), "zh");
        assert_eq!(Lang::En.as_str(), "en");
    }

    #[test]
    fn tpl_substitutes_named_placeholders() {
        let out = tpl(
            "wrote {n} fields to {path}",
            &[("n", &7), ("path", &"/etc/hone")],
        );
        assert_eq!(out, "wrote 7 fields to /etc/hone");
    }

    #[test]
    fn detect_recognizes_zh_via_lang_env() {
        // Snapshot existing env, set deterministic values, restore on drop.
        struct EnvScope {
            lc_all: Option<String>,
            lang: Option<String>,
        }
        impl Drop for EnvScope {
            fn drop(&mut self) {
                if let Some(v) = self.lc_all.take() {
                    unsafe { std::env::set_var("LC_ALL", v) };
                } else {
                    unsafe { std::env::remove_var("LC_ALL") };
                }
                if let Some(v) = self.lang.take() {
                    unsafe { std::env::set_var("LANG", v) };
                } else {
                    unsafe { std::env::remove_var("LANG") };
                }
            }
        }
        let _scope = EnvScope {
            lc_all: std::env::var("LC_ALL").ok(),
            lang: std::env::var("LANG").ok(),
        };
        unsafe {
            std::env::remove_var("LC_ALL");
            std::env::set_var("LANG", "zh_CN.UTF-8");
        }
        assert_eq!(detect_initial_lang(), Lang::Zh);
        unsafe { std::env::set_var("LANG", "en_US.UTF-8") };
        assert_eq!(detect_initial_lang(), Lang::En);
    }
}
