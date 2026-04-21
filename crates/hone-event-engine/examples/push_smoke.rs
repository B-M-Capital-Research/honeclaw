//! Smoke test for per-actor notification prefs + real Telegram sink.
//!
//! Reads every JSON file under ./data/notif_prefs, loads the actor's
//! NotificationPrefs, runs a fake MarketEvent through `should_deliver`, and
//! for telegram actors that pass the filter actually calls the Bot API
//! `sendMessage`. This validates the end-to-end filtering+delivery chain
//! without booting pollers/registry/digest.
//!
//! Run:  cargo run --example push_smoke -p hone-event-engine

use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use hone_core::ActorIdentity;
use hone_event_engine::{
    event::{EventKind, MarketEvent, Severity},
    prefs::{FilePrefsStorage, PrefsProvider},
};

const PREFS_DIR: &str = "./data/notif_prefs";
const CONFIG_PATH: &str = "./config.yaml";

struct TelegramSink {
    token: String,
    client: reqwest::Client,
}

impl TelegramSink {
    async fn send(&self, chat_id: &str, text: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "chat_id": chat_id, "text": text }))
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("telegram API {status}: {body}");
        }
        Ok(())
    }
}

fn read_bot_token() -> Result<String> {
    let raw =
        std::fs::read_to_string(CONFIG_PATH).with_context(|| format!("read {CONFIG_PATH}"))?;
    let cfg: serde_yaml::Value = serde_yaml::from_str(&raw)?;
    cfg.get("telegram")
        .and_then(|t| t.get("bot_token"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .context("telegram.bot_token missing in config.yaml")
}

fn parse_actor_from_filename(stem: &str) -> Option<ActorIdentity> {
    let parts: Vec<&str> = stem.splitn(3, "__").collect();
    if parts.len() != 3 {
        return None;
    }
    let channel = parts[0];
    let raw_scope = parts[1];
    let user_id = parts[2];
    let scope = if raw_scope == "direct" {
        None
    } else {
        Some(raw_scope.to_string())
    };
    ActorIdentity::new(channel, user_id, scope).ok()
}

fn build_fake_event() -> MarketEvent {
    MarketEvent {
        id: format!("smoke-test-{}", Utc::now().timestamp()),
        kind: EventKind::NewsCritical,
        severity: Severity::High,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: "[E2E smoke] notification-prefs 链路验收".into(),
        summary: "这是一条 push_smoke example 生成的 fake High 事件。收到这条消息 = \
                  FilePrefsStorage + should_deliver + Telegram sendMessage 全链路通。"
            .into(),
        url: None,
        source: "push_smoke".into(),
        payload: serde_json::Value::Null,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let token = read_bot_token()?;
    let sink = TelegramSink {
        token,
        client: reqwest::Client::new(),
    };
    let storage =
        FilePrefsStorage::new(PREFS_DIR).with_context(|| format!("open prefs dir {PREFS_DIR}"))?;
    let event = build_fake_event();

    println!("== push_smoke ==");
    println!("event id   : {}", event.id);
    println!(
        "event kind : news_critical / High (symbols={:?})",
        event.symbols
    );
    println!();

    let dir_entries =
        std::fs::read_dir(PREFS_DIR).with_context(|| format!("read dir {PREFS_DIR}"))?;
    let mut actors: Vec<ActorIdentity> = Vec::new();
    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if let Some(actor) = parse_actor_from_filename(stem) {
            actors.push(actor);
        } else {
            eprintln!("!! skipped unparsable file: {}", path.display());
        }
    }
    actors.sort_by(|a, b| a.channel.cmp(&b.channel).then(a.user_id.cmp(&b.user_id)));

    let mut sent = 0;
    let mut filtered = 0;
    let mut non_telegram = 0;
    let mut failed = 0;

    for actor in &actors {
        let prefs = storage.load(actor);
        let scope_display = actor.channel_scope.as_deref().unwrap_or("-");
        let would_deliver = prefs.should_deliver(&event);

        print!(
            "- {ch:<10} scope={scope:<32} user={uid:<40} enabled={en:<5} allow={al:<5} block={bl:<5} → ",
            ch = actor.channel,
            scope = scope_display,
            uid = actor.user_id,
            en = prefs.enabled,
            al = prefs
                .allow_kinds
                .as_ref()
                .map(|v| v.len())
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".into()),
            bl = prefs.blocked_kinds.len()
        );

        if !would_deliver {
            println!("FILTERED by prefs");
            filtered += 1;
            continue;
        }

        if actor.channel != "telegram" {
            println!("PASS (非 telegram,example 未实现该 sink)");
            non_telegram += 1;
            continue;
        }
        if !matches!(actor.channel_scope.as_deref(), None | Some("direct")) {
            // skipping group chats for safety — smoke test targets DM only
            println!("PASS (telegram 群聊 scope,跳过实际发送)");
            non_telegram += 1;
            continue;
        }

        let body = format!(
            "{}\n\n{}\n\nactor: {}:{}",
            event.title, event.summary, actor.channel, actor.user_id
        );
        match sink.send(&actor.user_id, &body).await {
            Ok(()) => {
                println!("SENT via telegram");
                sent += 1;
            }
            Err(e) => {
                println!("FAIL: {e:#}");
                failed += 1;
            }
        }
    }

    println!();
    println!(
        "summary: {sent} sent / {filtered} filtered by prefs / {non_telegram} skipped non-DM / {failed} failed"
    );

    if failed > 0 {
        std::process::exit(2);
    }
    Ok(())
}

fn _assert_prefs_dir_exists() {
    // 保留静态断言占位,不让 unused lint 挂
    let _ = Path::new(PREFS_DIR);
}
