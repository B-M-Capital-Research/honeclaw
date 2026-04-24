//! Real LLM live smoke for `run_with_progress_ticks`（即生产 `agent.run.progress` 心跳 watchdog 核心循环）。
//!
//! 跑法:
//!   cargo run --example progress_watchdog_live_smoke -p hone-channels
//!
//! 验证目标：
//!   1. `run_with_progress_ticks` 在 `run_fut` 仍在 pending 期间，会按 tick 间隔触发 `on_tick`；
//!   2. 一旦 `run_fut` 返回，`on_tick` 不再继续触发；
//!   3. 不依赖 mock —— `run_fut` 就是一次真实的 MiniMax `chat()` 调用，模拟生产里
//!      `runner.run(...)` 的长阻塞场景。
//!
//! 之所以要这条 smoke：`docs/bugs/feishu_scheduler_run_stuck_without_cron_job_run.md`
//! 的修复依赖「runner 卡住期间 watchdog 会定期发 heartbeat 到 sidecar.log」。
//! 这条假设必须用真实 LLM 调用跨越 tick 窗口，单测里 mock 调用 0ms 就返回，验证不到。
//!
//! Tick 间隔默认 2 秒（smoke 自带常量，不经 env）。MiniMax `chat()` 通常 3-15 秒，
//! 所以预期 tick >= 1（更常见 2-6）。如果某次真的只花 <2s 返回，视作样本无效，
//! 允许 retry 一次更长的 prompt。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use hone_channels::agent_session::run_with_progress_ticks;
use hone_core::config::HoneConfig;
use hone_llm::{LlmProvider, Message, OpenAiCompatibleProvider};

const CONFIG_PATH: &str = "./config.yaml";
const TICK: Duration = Duration::from_secs(2);
const MIN_EXPECTED_TICKS: u64 = 1;

async fn one_long_chat(
    provider: Arc<dyn LlmProvider + Send + Sync>,
    prompt: &str,
) -> anyhow::Result<String> {
    let messages = vec![Message {
        role: "user".into(),
        content: Some(prompt.into()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];
    let res = provider
        .chat(&messages, None)
        .await
        .context("chat failed")?;
    Ok(res.content)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = HoneConfig::from_file(CONFIG_PATH).with_context(|| format!("加载 {CONFIG_PATH}"))?;
    let aux = &cfg.llm.auxiliary;
    if !aux.is_configured() {
        bail!("llm.auxiliary 未配置");
    }
    let api_key = aux.resolved_api_key();
    if api_key.is_empty() {
        bail!("llm.auxiliary.api_key 解析为空");
    }
    let max_tokens = aux.max_tokens.min(u16::MAX as u32) as u16;
    let provider: Arc<dyn LlmProvider + Send + Sync> = Arc::new(
        OpenAiCompatibleProvider::new(&api_key, &aux.base_url, &aux.model, aux.timeout, max_tokens)
            .context("build provider")?,
    );

    println!("== progress_watchdog_live_smoke ==");
    println!("model : {}", aux.model);
    println!("tick  : {:?}", TICK);
    println!();

    // 构造一个足够拖时间的 prompt（生成长文 + 思考链，MiniMax 通常会飙到 5-15s）。
    let prompt = "请用简体中文详细写一篇 1500 字左右的原油市场周度复盘分析，覆盖 WTI、Brent、LNG、OPEC+ 政策、地缘事件、美国库存数据、期货持仓结构、衍生品价差、需求端消费数据、供给端页岩油产能变化，最后给出投机与对冲视角下的操作建议。要有清晰的小标题和段落。";

    let tick_count = Arc::new(AtomicU64::new(0));
    let tick_count_for_closure = tick_count.clone();

    let chat_fut = one_long_chat(provider.clone(), prompt);
    let start = std::time::Instant::now();
    let content = run_with_progress_ticks(chat_fut, TICK, move |ticks, elapsed| {
        let counter = tick_count_for_closure.clone();
        counter.store(ticks, Ordering::SeqCst);
        println!("[tick {ticks}] elapsed={:?}", elapsed);
        async move {}
    })
    .await;
    let total_elapsed = start.elapsed();
    let observed_ticks = tick_count.load(Ordering::SeqCst);

    let content = content.context("chat failed")?;
    let preview: String = content.chars().take(180).collect();

    println!();
    println!("chat returned in {:?}", total_elapsed);
    println!("observed ticks : {observed_ticks}");
    println!("content preview[:180] : {:?}", preview);

    if total_elapsed < TICK {
        bail!(
            "chat returned in {:?} < tick {:?}; test sample invalid, 请重跑或加长 prompt",
            total_elapsed,
            TICK
        );
    }

    if observed_ticks < MIN_EXPECTED_TICKS {
        bail!(
            "观察到 {observed_ticks} 个 tick，但预期至少 {MIN_EXPECTED_TICKS} 个（chat 实际 {:?}, tick {:?}）。watchdog 没有在 run_fut 阻塞期间发 heartbeat！",
            total_elapsed,
            TICK
        );
    }

    // 同时验证 run_fut 完成后不再继续 tick：用 tokio::time::sleep(2*TICK) 再看 counter。
    let before = tick_count.load(Ordering::SeqCst);
    tokio::time::sleep(TICK * 2).await;
    let after = tick_count.load(Ordering::SeqCst);
    if after != before {
        bail!("run_fut 返回后 ticker 仍在跑（{before} -> {after}），watchdog 未正确退出循环");
    }
    println!("post-return sanity: ticker 已停止 (before={before} after={after})");

    println!(
        "\nPASS : watchdog 在真实 LLM 调用期间发出 {observed_ticks} 次 tick，返回后立即停止。"
    );
    Ok(())
}
