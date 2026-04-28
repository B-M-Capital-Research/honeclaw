//! Real LLM smoke for the heartbeat JSON contract.
//!
//! 跑法:
//!   cargo run --example heartbeat_prompt_llm_smoke -p hone-channels
//!
//! 目的：直接喂 `build_scheduled_prompt` 构造出来的 heartbeat prompt 给生产
//! 真实走的辅助 LLM (`llm.auxiliary`，OpenAI-compatible，当前为 MiniMax)，
//! 再用生产路径上的 `inspect_heartbeat_result` 去判定 `parse_kind`。
//!
//! 为什么不是 OpenRouter：`scheduler.rs` 里 heartbeat 的 `model_override` 走的是
//! `core.auxiliary_model_name()`，也就是 `llm.auxiliary.model`。config.yaml 注释
//! 明写：「When configured, it overrides the legacy openrouter.sub_model route」。
//! 所以这里必须用 `OpenAiCompatibleProvider` 指向 `llm.auxiliary` 才算覆盖生产。
//!
//! 覆盖的回归：
//!   - `scheduler_heartbeat_unknown_status_silent_skip.md`：模型把推理塞进
//!     `<think>...</think>` 外再包 JSON，老解析直接判 noop；新 prompt 规则 6a +
//!     `strip_internal_reasoning_blocks` 应当让最终 parse_kind 落入
//!     `JsonNoop` / `JsonEmptyStatus` / `JsonTriggered` / `SentinelNoop`。
//!
//! 不做的事：不真的连定时任务落账、不发 Feishu/Telegram。只验契约。

use std::sync::Arc;

use anyhow::{Context, Result, bail};
use hone_channels::scheduler::{
    HeartbeatOutcome, HeartbeatParseKind, build_scheduled_prompt, inspect_heartbeat_result,
};
use hone_core::ActorIdentity;
use hone_core::config::HoneConfig;
use hone_llm::{LlmProvider, Message, OpenAiCompatibleProvider};
use hone_scheduler::SchedulerEvent;

const CONFIG_PATH: &str = "./config.yaml";

fn make_event(job_name: &str, task_prompt: &str) -> SchedulerEvent {
    SchedulerEvent {
        actor: ActorIdentity::new("smoke", "smoke_user", None::<String>)
            .expect("valid actor identity"),
        job_id: "smoke-job".into(),
        job_name: job_name.into(),
        task_prompt: task_prompt.into(),
        channel: "smoke".into(),
        channel_scope: None,
        channel_target: "smoke".into(),
        delivery_key: "smoke-key".into(),
        push: serde_json::Value::Null,
        tags: Vec::new(),
        heartbeat: true,
        schedule_hour: 0,
        schedule_minute: 0,
        schedule_repeat: "heartbeat".to_string(),
        schedule_date: None,
        last_delivered_previews: Vec::new(),
        bypass_quiet_hours: false,
    }
}

#[derive(Debug)]
struct CaseOutcome {
    name: &'static str,
    parse_kind: HeartbeatParseKind,
    outcome_kind: &'static str,
    raw_preview: String,
    contract_ok: bool,
}

fn summarize_outcome(outcome: HeartbeatOutcome) -> &'static str {
    match outcome {
        HeartbeatOutcome::Noop => "noop",
        HeartbeatOutcome::Deliver(_) => "deliver",
    }
}

fn is_contract_ok(kind: &HeartbeatParseKind) -> bool {
    matches!(
        kind,
        HeartbeatParseKind::Empty
            | HeartbeatParseKind::SentinelNoop
            | HeartbeatParseKind::JsonNoop
            | HeartbeatParseKind::JsonEmptyStatus
            | HeartbeatParseKind::JsonTriggered
    )
}

async fn run_case(
    provider: &(dyn LlmProvider + Send + Sync),
    name: &'static str,
    task_prompt: &str,
) -> Result<CaseOutcome> {
    let event = make_event(name, task_prompt);
    let prompt = build_scheduled_prompt(&event);

    let messages = vec![Message {
        role: "user".into(),
        content: Some(prompt),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];

    // model=None 让 provider 使用自身配置里的默认 model（即 llm.auxiliary.model）。
    let result = provider
        .chat(&messages, None)
        .await
        .with_context(|| format!("LLM 调用失败 case={name}"))?;

    let (outcome, parse_kind) = inspect_heartbeat_result(&result.content);
    let raw_preview: String = result.content.chars().take(180).collect();
    let contract_ok = is_contract_ok(&parse_kind);

    Ok(CaseOutcome {
        name,
        outcome_kind: summarize_outcome(outcome),
        parse_kind,
        raw_preview,
        contract_ok,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = HoneConfig::from_file(CONFIG_PATH).with_context(|| format!("加载 {CONFIG_PATH}"))?;
    let aux = &cfg.llm.auxiliary;
    if !aux.is_configured() {
        bail!(
            "llm.auxiliary 未配置（base_url/model/api_key 至少缺一项），heartbeat 生产路径不可用"
        );
    }
    let api_key = aux.resolved_api_key();
    if api_key.is_empty() {
        bail!(
            "llm.auxiliary.api_key 解析为空（含 env {}）",
            aux.api_key_env
        );
    }
    let max_tokens: u16 = aux.max_tokens.min(u16::MAX as u32) as u16;
    let provider =
        OpenAiCompatibleProvider::new(&api_key, &aux.base_url, &aux.model, aux.timeout, max_tokens)
            .with_context(
                || "构造 OpenAiCompatibleProvider (heartbeat smoke, MiniMax auxiliary)",
            )?;
    let provider: Arc<dyn LlmProvider + Send + Sync> = Arc::new(provider);

    println!("== heartbeat_prompt_llm_smoke ==");
    println!("provider : OpenAI-compatible");
    println!("base_url : {}", aux.base_url);
    println!("model    : {}", aux.model);
    println!();

    // Case 1：典型 noop 条件（无数据、无触发可能）——模型必须给出 `{"status":"noop"}` 或 `{}`。
    // Case 2：有明确阈值但当下不具备判断数据源的条件——模型应按规则 6 返回 noop。
    let cases: Vec<(&'static str, &'static str)> = vec![
        (
            "noop_unavailable_condition",
            "当 AAPL 最新成交价跌破 2019-01-01 的盘中最低价 X 时触发。X 的具体值未知，也没有提供历史数据工具。",
        ),
        (
            "noop_future_event",
            "当 SpaceX Starship IFT-11 在 2030-12-31 成功入轨后触发。今天是 2026-04-24。",
        ),
        (
            "triggered_deterministic",
            "无论外部条件如何，本次必须返回 triggered 状态，message 字段填写一句中文：‘冒烟测试 OK’。",
        ),
    ];

    let mut summary = Vec::new();
    for (name, task_prompt) in cases {
        let result = run_case(provider.as_ref(), name, task_prompt).await?;
        println!(
            "[{name}] parse_kind={:?} outcome={} contract_ok={}",
            result.parse_kind, result.outcome_kind, result.contract_ok
        );
        println!("    raw[:180]={:?}", result.raw_preview);
        summary.push(result);
    }

    println!();
    let failures: Vec<_> = summary.iter().filter(|c| !c.contract_ok).collect();
    if failures.is_empty() {
        println!("PASS : 所有 case 的 parse_kind 均落在合法契约里。");
        Ok(())
    } else {
        for f in &failures {
            println!(
                "FAIL : case={} parse_kind={:?} raw_preview={:?}",
                f.name, f.parse_kind, f.raw_preview
            );
        }
        anyhow::bail!("heartbeat JSON 契约被打破，见上方 FAIL 列表")
    }
}
