//! 一次性脚本:为所有有 portfolio 且 holdings 非空的 direct actor 跑首波 thesis 蒸馏。
//!
//! 用途:新功能上线后手动触发首次蒸馏,跳过 cron 等待。
//! 用法:`cargo run --example distill_first_wave -p hone-event-engine`
//!
//! 行为:
//! - 读 ./config.yaml 拿 OpenRouter key + storage 路径
//! - 扫所有 portfolio,对每个 direct actor 跑 distill_and_persist_one
//! - 打印每只 ticker 的蒸馏结果(thesis 文本前 80 字)+ skipped 列表
//! - 失败的 actor 单独 warn,不阻断其它

use std::path::PathBuf;
use std::sync::Arc;

use hone_core::config::HoneConfig;
use hone_event_engine::global_digest::{LlmThesisDistiller, distill_and_persist_one};
use hone_event_engine::prefs::{FilePrefsStorage, PrefsProvider};
use hone_llm::{LlmProvider, OpenRouterProvider};
use hone_memory::PortfolioStorage;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    let config_path = std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".into());
    println!("loading config from {config_path}");
    let config = HoneConfig::from_file(&config_path)?;

    let prefs_dir = config.storage.notif_prefs_dir.clone();
    let portfolio_dir = config.storage.portfolio_dir.clone();
    let model = config.event_engine.global_digest.event_dedupe_model.clone();

    println!("model: {model}");
    println!("prefs_dir: {prefs_dir}");
    println!("portfolio_dir: {portfolio_dir}");

    let provider: Arc<dyn LlmProvider> = Arc::new(OpenRouterProvider::from_config(&config)?);
    let distiller = LlmThesisDistiller::new(provider, model);
    let prefs_storage = FilePrefsStorage::new(&prefs_dir)?;
    let portfolio_storage = PortfolioStorage::new(PathBuf::from(&portfolio_dir));
    // 复刻 hone-channels::sandbox_base_dir 的解析,避免循环依赖
    let sandbox_base = std::env::var("HONE_AGENT_SANDBOX_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("HONE_DATA_DIR").map(|p| PathBuf::from(p).join("agent-sandboxes"))
        })
        .unwrap_or_else(|_| PathBuf::from("./data/agent-sandboxes"));
    println!("sandbox_base: {}", sandbox_base.display());

    let mut total_actors = 0u32;
    let mut total_distilled = 0u32;
    let mut total_failed = 0u32;
    for (actor, portfolio) in portfolio_storage.list_all() {
        if !actor.is_direct() {
            continue;
        }
        let holdings: Vec<String> = portfolio
            .holdings
            .iter()
            .map(|h| h.symbol.clone())
            .collect();
        if holdings.is_empty() {
            continue;
        }
        total_actors += 1;
        let actor_label = format!(
            "{}/{}/{}",
            actor.channel,
            actor.channel_scope.clone().unwrap_or_default(),
            actor.user_id
        );
        println!(
            "\n────── actor: {actor_label} ({} holdings) ──────",
            holdings.len()
        );
        println!("  holdings: {}", holdings.join(", "));

        match distill_and_persist_one(&distiller, &prefs_storage, &sandbox_base, &actor, &holdings)
            .await
        {
            Ok(updated) => {
                total_distilled += 1;
                let theses_count = updated
                    .investment_theses
                    .as_ref()
                    .map(|m| m.len())
                    .unwrap_or(0);
                println!(
                    "  ✓ distilled {} theses, {} skipped, last_at = {:?}",
                    theses_count,
                    updated.thesis_distill_skipped.len(),
                    updated.last_thesis_distilled_at
                );
                if let Some(theses) = &updated.investment_theses {
                    let mut keys: Vec<&String> = theses.keys().collect();
                    keys.sort();
                    for ticker in keys {
                        let preview: String = theses[ticker].chars().take(80).collect();
                        println!("    [{ticker}] {preview}…");
                    }
                }
                if let Some(style) = &updated.investment_global_style {
                    let preview: String = style.chars().take(120).collect();
                    println!("    [STYLE] {preview}…");
                }
                if !updated.thesis_distill_skipped.is_empty() {
                    println!(
                        "    [skipped] {}",
                        updated.thesis_distill_skipped.join(", ")
                    );
                }
            }
            Err(e) => {
                total_failed += 1;
                eprintln!("  ✗ FAILED: {e:#}");
            }
        }

        // 验证写入
        let reloaded = prefs_storage.load(&actor);
        println!(
            "  verify: prefs.investment_theses has {} entries; last_at = {:?}",
            reloaded
                .investment_theses
                .as_ref()
                .map(|m| m.len())
                .unwrap_or(0),
            reloaded.last_thesis_distilled_at
        );
    }

    println!(
        "\n========\nfinished: {total_actors} actors, {total_distilled} distilled, {total_failed} failed"
    );
    Ok(())
}
