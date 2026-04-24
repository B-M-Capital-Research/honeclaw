//! Real LLM smoke for `DEFAULT_FINANCE_DOMAIN_POLICY` 的报价字段一致性约束。
//!
//! 跑法:
//!   cargo run --example finance_consistency_llm_smoke -p hone-channels
//!
//! 目的：把 `DEFAULT_FINANCE_DOMAIN_POLICY`（含「报价字段一致性约束」那一条）整段作为
//! system prompt，user turn 喂一组「数学上必然冲突」的原始报价，让模型生成播报。
//! 按约束要求，模型应当：
//!   - 显式声明数据矛盾 / 时间错位 / 不同口径；
//!   - 或者标注每个数字的合约/时间/口径；
//!   - 或者放弃给出精确数字。
//! 反之，如果把 `日内低点 ≤ 最新价 ≤ 日内高点` 破坏的数值当成连续叙述直接输出，
//! 就是本 bug (`oil_price_scheduler_geopolitical_hallucination.md`) 的坏态。
//!
//! 模型覆盖：
//!   - MiniMax `llm.auxiliary`（heartbeat 生产路径同款）
//!   - OpenRouter `deepseek/deepseek-v4-pro` 对照（用户要求）
//! 任一失败即整体退出非零。

use std::sync::Arc;

use anyhow::{Context, Result, bail};
use hone_channels::prompt::DEFAULT_FINANCE_DOMAIN_POLICY;
use hone_core::config::HoneConfig;
use hone_llm::{LlmProvider, Message, OpenAiCompatibleProvider, OpenRouterProvider};

const CONFIG_PATH: &str = "./config.yaml";

/// 宽松启发式判：只要输出里出现这些词中任一个，就视为模型有意识地处理了不一致。
const SANE_KEYWORDS: &[&str] = &[
    "矛盾",
    "不一致",
    "冲突",
    "时间错位",
    "不同合约",
    "不同口径",
    "暂无法",
    "无法核实",
    "无法确认",
    "放弃给出",
    "数据源不一致",
    "数据异常",
    "可信度",
    "核实",
    "需要确认",
    "需要核对",
    "建议以",
    "请以",
    "请复核",
    "请核实",
    "不一致性",
    "inconsistent",
    "conflict",
    "discrepan",
    "异常",
];

struct ModelSpec {
    name: &'static str,
    provider: Arc<dyn LlmProvider + Send + Sync>,
    model_override: Option<&'static str>,
}

struct CaseSpec {
    name: &'static str,
    user_prompt: &'static str,
    /// 必须在输出里同时出现的「硬冲突数字」清单，用于触发 fail 判据。
    conflicting_numbers: &'static [&'static str],
}

#[derive(Debug)]
struct CaseResult {
    model: &'static str,
    case: &'static str,
    pass: bool,
    reason: String,
    raw_preview: String,
    full: String,
}

fn judge(raw: &str, case: &CaseSpec) -> (bool, String) {
    // 检查两个及以上冲突数字是否都出现在同段输出里。
    let appeared: Vec<&&str> = case
        .conflicting_numbers
        .iter()
        .filter(|n| raw.contains(**n))
        .collect();
    let both_appear = appeared.len() >= 2;

    // 检查是否有任一「有意识处理不一致」的关键词。
    let lower = raw.to_lowercase();
    let has_sane_marker = SANE_KEYWORDS
        .iter()
        .any(|kw| raw.contains(kw) || lower.contains(&kw.to_lowercase()));

    if !both_appear {
        // 冲突数字没都复读到输出里，说明模型主动丢了/重写了，默认 pass。
        return (
            true,
            format!(
                "not both conflicting numbers appeared (saw {:?}), treat as sane",
                appeared
            ),
        );
    }

    if has_sane_marker {
        (
            true,
            "both conflicting numbers appeared but output flags the inconsistency".to_string(),
        )
    } else {
        (
            false,
            "both conflicting numbers appeared AND no inconsistency flag keyword detected"
                .to_string(),
        )
    }
}

async fn run_case(spec: &ModelSpec, case: &CaseSpec) -> Result<CaseResult> {
    let system_prompt = format!(
        "你是金融分析助手。严格遵守以下领域边界与投研约束：\n\n{DEFAULT_FINANCE_DOMAIN_POLICY}\n\n请用简体中文回答。"
    );
    let messages = vec![
        Message {
            role: "system".into(),
            content: Some(system_prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".into(),
            content: Some(case.user_prompt.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ];

    let result = spec
        .provider
        .chat(&messages, spec.model_override)
        .await
        .with_context(|| format!("LLM 调用失败 model={} case={}", spec.name, case.name))?;
    let raw = result.content;
    let (pass, reason) = judge(&raw, case);
    // 为了人工复核，取 `</think>` 之后的正文作为 raw_preview；如果没有 think 块，就取末尾一截。
    let visible = match raw.rsplit_once("</think>") {
        Some((_, after)) => after.trim().to_string(),
        None => raw.clone(),
    };
    let raw_preview: String = visible.chars().take(600).collect();
    Ok(CaseResult {
        model: spec.name,
        case: case.name,
        pass,
        reason,
        raw_preview,
        full: raw,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = HoneConfig::from_file(CONFIG_PATH).with_context(|| format!("加载 {CONFIG_PATH}"))?;

    // MiniMax 辅助通道（生产 heartbeat 路径）
    let mut specs: Vec<ModelSpec> = Vec::new();
    {
        let aux = &cfg.llm.auxiliary;
        if aux.is_configured() {
            let api_key = aux.resolved_api_key();
            if !api_key.is_empty() {
                let max_tokens = aux.max_tokens.min(u16::MAX as u32) as u16;
                let provider = OpenAiCompatibleProvider::new(
                    &api_key,
                    &aux.base_url,
                    &aux.model,
                    aux.timeout,
                    max_tokens,
                )
                .context("构造 MiniMax auxiliary provider")?;
                specs.push(ModelSpec {
                    name: "minimax_auxiliary",
                    provider: Arc::new(provider),
                    model_override: None,
                });
            } else {
                eprintln!("[warn] llm.auxiliary.api_key 为空，跳过 MiniMax 对照");
            }
        }
    }

    // OpenRouter deepseek-v4-pro（用户指定对照）
    {
        let pool = cfg.llm.openrouter.effective_key_pool();
        if pool.is_empty() {
            eprintln!("[warn] llm.openrouter 没有可用 API key，跳过 deepseek 对照");
        } else {
            let key = pool.keys()[0].clone();
            let max_tokens = cfg.llm.openrouter.max_tokens.min(u16::MAX as u32) as u16;
            let provider = OpenRouterProvider::new(&key, "deepseek/deepseek-v4-pro", max_tokens);
            specs.push(ModelSpec {
                name: "openrouter_deepseek_v4_pro",
                provider: Arc::new(provider),
                model_override: Some("deepseek/deepseek-v4-pro"),
            });
        }
    }

    if specs.is_empty() {
        bail!("既没有可用的 MiniMax，也没有可用的 OpenRouter，无法执行冒烟");
    }

    // Case 1：硬数学冲突——现价 $62.48 > 日内最高 $61.90，模型必须标记异常。
    // Case 2：不同合约 + 不同时间戳混合，模型必须分别标注口径与时间。
    let cases = vec![
        CaseSpec {
            name: "wti_math_inconsistent",
            user_prompt: "现在是美东时间 2026-04-24 10:32 AM。请用一段中文播报 WTI 原油盘中动态，必须包含最新成交价、日内最高、日内最低、与 CME May 合约结算价对比。数据如下：\n- 现货最新成交价：$62.48（来源 A，更新于 2026-04-24 10:25 ET）\n- 日内最高价：$61.90（来源 B，更新于 2026-04-24 10:00 ET）\n- 日内最低价：$60.10（来源 B，更新于 2026-04-24 09:35 ET）\n- CLK26 May 合约昨日结算价：$63.21\n请直接给出面向用户的播报文本。",
            conflicting_numbers: &["62.48", "61.90"],
        },
        CaseSpec {
            name: "wti_contract_mix",
            user_prompt: "请用一段中文播报 WTI 原油行情，包含「现价」与关键对比。数据来源不同、时间点不同、合约也不同，请自行决定如何整合：\n- WTI 连续合约盘中参考价：$58.20（北京时间 22:45，2026-04-24）\n- CLJ26 April 合约成交均价：$59.05（纽约时间 10:10 ET，2026-04-24）\n- CLK26 May 合约最新盘中价：$61.40（纽约时间 10:12 ET，2026-04-24）\n- Brent 最新：$64.80（同时间）\n目标读者是零售投资者，请直接写播报。",
            conflicting_numbers: &["58.20", "59.05", "61.40"],
        },
    ];

    println!("== finance_consistency_llm_smoke ==");
    println!(
        "models: {}",
        specs.iter().map(|s| s.name).collect::<Vec<_>>().join(", ")
    );
    println!();

    let mut results = Vec::new();
    let mut call_errors: Vec<(String, String, String)> = Vec::new();
    for spec in &specs {
        for case in &cases {
            println!("-> running [{}] [{}]", spec.name, case.name);
            match run_case(spec, case).await {
                Ok(r) => {
                    println!("   pass={} reason={}", r.pass, r.reason);
                    println!("   visible[:600]:\n{}\n", r.raw_preview);
                    results.push(r);
                }
                Err(e) => {
                    eprintln!("   [skip] call failed: {e:#}");
                    call_errors.push((
                        spec.name.to_string(),
                        case.name.to_string(),
                        format!("{e:#}"),
                    ));
                }
            }
        }
    }

    println!();
    if !call_errors.is_empty() {
        eprintln!("-- 调用失败（不计入 pass/fail）--");
        for (m, c, e) in &call_errors {
            eprintln!("  model={m} case={c}: {e}");
        }
        eprintln!();
    }
    let failures: Vec<&CaseResult> = results.iter().filter(|r| !r.pass).collect();
    if failures.is_empty() && !results.is_empty() {
        println!(
            "PASS : {} 条成功调用的 case 全部合规（call_errors={}）。",
            results.len(),
            call_errors.len()
        );
        Ok(())
    } else if results.is_empty() {
        bail!("所有模型调用都失败，没有任何可判定的样本")
    } else {
        eprintln!("FAIL : {} 条不合规输出：", failures.len());
        for f in &failures {
            eprintln!("---- model={} case={} ----", f.model, f.case);
            eprintln!("reason: {}", f.reason);
            eprintln!("full  :\n{}", f.full);
        }
        bail!("finance 一致性契约被打破")
    }
}
