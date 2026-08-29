//! Actor-scoped daily position-management research report.
//!
//! The engine combines current portfolio structure with already-cached HONE
//! evidence. It never places orders or mutates holdings. Exact concentration
//! bands are HONE product controls; Hari logic is cited separately.

use std::collections::HashMap;
use std::path::PathBuf;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use hone_core::ActorIdentity;
use hone_memory::portfolio::{Holding, Portfolio, PortfolioStorage, holdings_with_weights};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::routes::company_ratings::{CompanyRating, CompanyRatingSnapshot};
use crate::routes::daily_signals::DailySignalReport;
use crate::routes::investment_decisions::{
    ExposureAction, InvestmentDecisionSnapshot, ResearchZone,
};
use crate::routes::portfolio_news::{PortfolioNewsItem, PortfolioNewsSnapshot};
use crate::state::AppState;

const FRAMEWORK_VERSION: &str = "hari-invest-v1";
const MODEL_VERSION: &str = "hone-position-management-v3-hari-portfolio-gate";
const DECISION_GATE_POLICY_VERSION: &str = "hone-hari-confirmed-logic-gate-v2-applicability";
const PORTFOLIO_GATE_POLICY_VERSION: &str = "hone-hari-portfolio-readiness-v1";
const STALE_AFTER_HOURS: i64 = 36;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PositionActionCounts {
    pub increase_candidate: usize,
    pub hold: usize,
    pub reduce: usize,
    pub review: usize,
    pub insufficient_data: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ThemeExposure {
    pub theme: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConcentrationSummary {
    pub level: String,
    pub largest_symbol: String,
    pub largest_weight: f64,
    pub top_three_weight: f64,
    pub theme_exposures: Vec<ThemeExposure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PositionMacroContext {
    pub signal: String,
    pub score: Option<f64>,
    pub phase: String,
    pub report_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PositionAdviceItem {
    pub symbol: String,
    pub name: String,
    pub theme: String,
    pub weight: f64,
    pub current_price: Option<f64>,
    pub avg_cost: Option<f64>,
    pub unrealized_return_percent: Option<f64>,
    pub rating_score: Option<f64>,
    pub rating_light: String,
    pub rating_status: String,
    pub valuation_position: String,
    pub news_impact: String,
    pub news_attention: String,
    pub action: String,
    pub action_label: String,
    pub confidence: String,
    pub rationale: Vec<String>,
    pub risks: Vec<String>,
    pub falsifiers: Vec<String>,
    pub framework_logic: Vec<String>,
    pub evidence_as_of: Vec<String>,
    pub evidence_sources: Vec<String>,
    pub priority_score: f64,
    #[serde(default)]
    pub decision_gate: PositionDecisionGate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PositionDecisionGate {
    pub status: String,
    pub revision_id: String,
    pub decision_at: String,
    pub policy_version: String,
    pub skill_version: String,
    pub pre_methodology_action: String,
    pub final_action: String,
    pub confirmed_logic_ids: Vec<String>,
    pub candidate_logic_used: bool,
    pub increase_candidate_authorized: bool,
    pub portfolio_action_authorized: bool,
    pub blocking_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PositionPortfolioRuleApplication {
    pub logic_id: String,
    pub logic_version: String,
    pub label: String,
    pub status: String,
    pub evidence: Vec<String>,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PositionPortfolioGate {
    pub policy_version: String,
    pub skill_id: String,
    pub skill_version: String,
    pub confirmed_logic_ids: Vec<String>,
    pub candidate_logic_used: bool,
    pub status: String,
    pub rules: Vec<PositionPortfolioRuleApplication>,
    pub blocking_reasons: Vec<String>,
    pub increase_candidate_authorized: bool,
    pub portfolio_action_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub trade_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PositionManagementSnapshot {
    pub report_date: String,
    pub generated_at: DateTime<Utc>,
    pub generated_at_beijing: String,
    pub next_refresh_at: DateTime<Utc>,
    pub timezone: String,
    pub model_version: String,
    pub framework_version: String,
    pub status: String,
    pub portfolio_updated_at: String,
    pub holdings_count: usize,
    pub total_weight: f64,
    pub unallocated_weight: f64,
    pub concentration: ConcentrationSummary,
    pub macro_context: PositionMacroContext,
    #[serde(default)]
    pub portfolio_gate: PositionPortfolioGate,
    pub counts: PositionActionCounts,
    pub summary: String,
    pub items: Vec<PositionAdviceItem>,
    pub methodology_note: String,
    pub disclaimer: String,
}

#[derive(Debug, Clone)]
struct PositionInput {
    symbol: String,
    name: String,
    weight: f64,
    avg_cost: Option<f64>,
}

pub(crate) async fn handle_get_position_management(
    State(state): State<std::sync::Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match crate::routes::public::require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = match ActorIdentity::new("web", &user.user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => {
            return crate::routes::json_error(
                axum::http::StatusCode::BAD_REQUEST,
                error.to_string(),
            );
        }
    };
    let storage = PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let portfolio = storage.load(&actor).ok().flatten();
    let positions = portfolio.as_ref().map(real_positions).unwrap_or_default();
    if positions.is_empty() {
        return Json(empty_snapshot(
            portfolio
                .as_ref()
                .map(|value| value.updated_at.as_str())
                .unwrap_or(""),
            "no_portfolio",
            "请先在“我的”中添加真实持仓，系统才会生成仓位管理建议。",
        ))
        .into_response();
    }

    let ratings = crate::routes::company_ratings::current_snapshot(&state).await;
    let news = crate::routes::portfolio_news::current_snapshot(&state, &actor).await;
    let latest_upstream_at = news
        .as_ref()
        .map(|snapshot| snapshot.generated_at)
        .unwrap_or(ratings.generated_at)
        .max(ratings.generated_at);
    let existing = read_snapshot(&state, &actor).await;
    let updated_at = portfolio
        .as_ref()
        .map(|value| value.updated_at.as_str())
        .unwrap_or("");
    let needs_rebuild = existing
        .as_ref()
        .is_none_or(|snapshot| snapshot_needs_rebuild(snapshot, updated_at, latest_upstream_at));
    if needs_rebuild {
        let macro_report = crate::routes::daily_signals::current_macro_report(&state).await;
        let decisions = current_decisions(&state, &positions).await;
        let snapshot = build_snapshot(
            portfolio.as_ref().expect("portfolio exists"),
            &ratings,
            &macro_report,
            news.as_ref(),
            &decisions,
        );
        if let Err(error) = write_snapshot(&state, &actor, &snapshot).await {
            warn!(actor = %actor.storage_key(), %error, "on-demand position snapshot write failed");
        }
        return Json(snapshot).into_response();
    }

    match existing {
        Some(mut snapshot) => {
            if Utc::now() - snapshot.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS) {
                snapshot.status = "stale".to_string();
            }
            Json(snapshot).into_response()
        }
        None => Json(waiting_snapshot(
            portfolio.as_ref().expect("portfolio exists"),
            &positions,
        ))
        .into_response(),
    }
}

pub(crate) async fn refresh_all(state: &AppState) -> anyhow::Result<()> {
    let storage = PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let portfolios = storage
        .list_all()
        .into_iter()
        .filter(|(_, portfolio)| !real_positions(portfolio).is_empty())
        .collect::<Vec<_>>();
    if portfolios.is_empty() {
        info!("position management refresh skipped: no portfolios");
        return Ok(());
    }

    let ratings = crate::routes::company_ratings::current_snapshot(state).await;
    let macro_report = crate::routes::daily_signals::current_macro_report(state).await;
    for (actor, portfolio) in portfolios {
        let positions = real_positions(&portfolio);
        let decisions = current_decisions(state, &positions).await;
        let news = crate::routes::portfolio_news::current_snapshot(state, &actor).await;
        let snapshot = build_snapshot(
            &portfolio,
            &ratings,
            &macro_report,
            news.as_ref(),
            &decisions,
        );
        if let Err(error) = write_snapshot(state, &actor, &snapshot).await {
            warn!(actor = %actor.storage_key(), %error, "position management snapshot write failed");
        }
    }
    info!("position management snapshots refreshed");
    Ok(())
}

fn build_snapshot(
    portfolio: &Portfolio,
    ratings: &CompanyRatingSnapshot,
    macro_report: &DailySignalReport,
    news: Option<&PortfolioNewsSnapshot>,
    decisions: &HashMap<String, InvestmentDecisionSnapshot>,
) -> PositionManagementSnapshot {
    let positions = real_positions(portfolio);
    let rating_map = ratings
        .items
        .iter()
        .map(|item| (item.symbol.to_ascii_uppercase(), item))
        .collect::<HashMap<_, _>>();
    let news_map = latest_analyzed_news(news);
    let themes = positions
        .iter()
        .map(|position| {
            let theme = rating_map
                .get(&position.symbol)
                .map(|rating| rating.theme.as_str())
                .unwrap_or("未覆盖");
            (position.symbol.clone(), theme.to_string())
        })
        .collect::<HashMap<_, _>>();
    let concentration = concentration_summary(&positions, &themes);
    let macro_context = PositionMacroContext {
        signal: macro_report.signal.clone(),
        score: macro_report.score,
        phase: macro_report.phase.clone(),
        report_date: macro_report.report_date.clone(),
        status: macro_report.status.clone(),
    };
    let total_weight = round1(positions.iter().map(|item| item.weight).sum::<f64>());
    let portfolio_gate =
        position_portfolio_gate(&positions, &macro_context, &concentration, total_weight);
    let mut items = positions
        .iter()
        .map(|position| {
            advise_position(
                position,
                rating_map.get(&position.symbol).copied(),
                position_decision_gate(
                    decisions.get(&position.symbol),
                    rating_map.get(&position.symbol).copied(),
                ),
                news_map.get(&position.symbol).copied(),
                news_decision_ready(news, &position.symbol),
                &macro_context,
                &concentration,
                portfolio_gate.increase_candidate_authorized,
            )
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        b.priority_score
            .total_cmp(&a.priority_score)
            .then_with(|| b.weight.total_cmp(&a.weight))
    });
    let counts = action_counts(&items);
    let macro_current = macro_context.report_date
        == Utc::now()
            .with_timezone(&Shanghai)
            .format("%Y-%m-%d")
            .to_string()
        && !matches!(macro_context.status.as_str(), "stale" | "framework_only");
    let news_analysis_incomplete = positions
        .iter()
        .any(|position| !news_decision_ready(news, &position.symbol));
    let status = if counts.insufficient_data == items.len() {
        "data_unavailable"
    } else if counts.insufficient_data > 0
        || ratings.data_status != "live"
        || !macro_current
        || news_analysis_incomplete
    {
        "partial"
    } else {
        "live"
    };
    let now = Utc::now();
    PositionManagementSnapshot {
        report_date: now.with_timezone(&Shanghai).format("%Y-%m-%d").to_string(),
        generated_at: now,
        generated_at_beijing: now
            .with_timezone(&Shanghai)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        next_refresh_at: next_refresh(now),
        timezone: "Asia/Shanghai".to_string(),
        model_version: MODEL_VERSION.to_string(),
        framework_version: FRAMEWORK_VERSION.to_string(),
        status: status.to_string(),
        portfolio_updated_at: portfolio.updated_at.clone(),
        holdings_count: positions.len(),
        total_weight,
        unallocated_weight: round1((100.0 - total_weight).max(0.0)),
        concentration,
        macro_context,
        portfolio_gate,
        summary: format!(
            "{} 个持仓：加仓候选 {}、持有 {}、降低暴露 {}、立即复核 {}、数据不足 {}。",
            items.len(),
            counts.increase_candidate,
            counts.hold,
            counts.reduce,
            counts.review,
            counts.insufficient_data
        ),
        counts,
        items,
        methodology_note: "公司层增加暴露必须先通过同一点时决策大脑与 Hari LOG-V0001/2/6；组合层还必须完成 LOG-V0003/4/5 的市场状态、动态杠铃和板块预算复核。未确认的精确仓位、角色分类和相关性阈值不会由 HONE 补造；15%/25%/45% 仅为 HONE 风险提示线。".to_string(),
        disclaimer: "仓位建议仅供研究与复核，不构成个性化投资顾问、收益承诺或自动交易；HONE 不会修改持仓。".to_string(),
    }
}

fn advise_position(
    position: &PositionInput,
    rating: Option<&CompanyRating>,
    decision_gate: PositionDecisionGate,
    news: Option<&PortfolioNewsItem>,
    news_decision_ready: bool,
    macro_context: &PositionMacroContext,
    concentration: &ConcentrationSummary,
    portfolio_increase_authorized: bool,
) -> PositionAdviceItem {
    let current_evidence = rating.is_some_and(|item| {
        matches!(item.data_status.as_str(), "live" | "partial") && item.price.is_some()
    });
    let complete_company_evidence =
        current_evidence && rating.is_some_and(|item| item.financial_as_of.is_some());
    let high_concentration = position.weight >= 25.0
        || (concentration.largest_symbol == position.symbol && concentration.level == "high");
    let negative_news = news.is_some_and(|item| {
        item.analysis_status == "model_analyzed"
            && item.impact == "negative"
            && (item.thesis_effect == "weakens" || item.attention == "立即复核")
    });
    let macro_current = macro_context.report_date
        == Utc::now()
            .with_timezone(&Shanghai)
            .format("%Y-%m-%d")
            .to_string()
        && !matches!(macro_context.status.as_str(), "stale" | "framework_only");
    let macro_supportive = macro_current && macro_context.signal == "green";
    let valuation_high = rating
        .and_then(|item| item.valuation.as_ref())
        .is_some_and(|value| value.current_price > value.bull_case);
    let valuation_opportunity = rating
        .and_then(|item| item.valuation.as_ref())
        .is_some_and(|value| value.current_price <= value.base_case);

    let preliminary_increase = rating.is_some_and(|item| {
        item.light == "green"
            && item.dimensions.fundamentals >= 70.0
            && item.dimensions.scarcity >= 70.0
    }) && valuation_opportunity
        && macro_supportive
        && position.weight < 15.0;
    let (action, label, mut confidence) = if !current_evidence {
        ("insufficient_data", "数据不足", "low")
    } else if !news_decision_ready {
        ("review", "等待分析", "low")
    } else if negative_news {
        ("review", "立即复核", "high")
    } else if rating.is_some_and(|item| item.light == "red") && position.weight >= 15.0 {
        ("reduce", "降低暴露", "high")
    } else if valuation_high && position.weight >= 15.0 {
        ("reduce", "降低暴露", "medium")
    } else if high_concentration && !rating.is_some_and(|item| item.light == "green") {
        ("reduce", "降低暴露", "medium")
    } else if high_concentration {
        ("review", "立即复核", "medium")
    } else if preliminary_increase
        && decision_gate.status == "passed"
        && portfolio_increase_authorized
    {
        ("increase_candidate", "加仓候选", "medium")
    } else if preliminary_increase && decision_gate.status == "passed" {
        ("review", "组合门禁复核", "low")
    } else if preliminary_increase {
        ("review", "决策门禁复核", "low")
    } else {
        ("hold", "持有", "medium")
    };
    if current_evidence && !complete_company_evidence && confidence != "low" {
        confidence = "low";
    }

    let mut rationale = Vec::new();
    if let Some(item) = rating {
        if current_evidence {
            if let Some(as_of) = item.financial_as_of.as_deref() {
                rationale.push(format!(
                    "公司评级 {:.1}（{}），财务数据截至 {}。",
                    item.score,
                    light_label(&item.light),
                    as_of
                ));
            } else {
                rationale.push(format!(
                    "当前行情已取得，公司研究结构分 {:.1}（{}）；季度财务尚未接入，因此本动作降为低置信度且不触发加仓。",
                    item.score,
                    light_label(&item.light)
                ));
            }
        } else {
            rationale.push(
                "缺少可核实的当前行情或季度财务，研究基线不能升级为当前仓位动作。".to_string(),
            );
        }
        if let Some(valuation) = item.valuation.as_ref() {
            rationale.push(format!("当日估值位置：{}。", valuation.current_position));
        } else {
            rationale.push("当日 Hari 三情景估值未通过复核，本次不以估值触发加仓。".to_string());
        }
    } else {
        rationale.push("该标的不在每日公司评级覆盖池，当前证据不足。".to_string());
    }
    if negative_news {
        rationale.push("近 48 小时存在削弱原投资逻辑的负面新闻，先复核证伪条件。".to_string());
    }
    if !news_decision_ready {
        rationale.push(
            "近 48 小时新闻来源或模型影响分析尚未完整通过门禁；未分析不等于没有风险，本次禁止形成加仓或自动持有结论。"
                .to_string(),
        );
    }
    if high_concentration {
        rationale.push(format!(
            "单一标的权重 {:.1}% 触发 HONE 集中度预警。",
            position.weight
        ));
    }
    if !macro_supportive {
        rationale.push("宏观信号不是当日绿灯，暂停把优质公司自动等同于可加仓。".to_string());
    }
    match decision_gate.status.as_str() {
        "passed" if portfolio_increase_authorized => rationale.push(format!(
            "同一点时决策大脑与组合增加暴露门禁均已通过 {}；该结果仍只是研究候选，不授权交易。",
            decision_gate.policy_version
        )),
        "passed" if preliminary_increase => rationale.push(
            "公司层决策门禁已通过，但组合层 LOG-V0003/4/5 尚未完成；本轮只能复核，不能形成加仓候选。"
                .to_string(),
        ),
        "blocked" | "missing" | "mismatched" if preliminary_increase => rationale.push(format!(
            "公司评级与估值曾形成加仓前置条件，但统一决策大脑门禁未通过：{}。",
            decision_gate.blocking_reasons.join("；")
        )),
        _ => {}
    }

    let current_price = rating.and_then(|item| item.price);
    let unrealized_return_percent = current_price
        .zip(position.avg_cost)
        .filter(|(_, cost)| *cost > 0.0)
        .map(|(price, cost)| round1((price / cost - 1.0) * 100.0));
    let mut priority = position.weight * 1.2;
    priority += match action {
        "review" => 70.0,
        "reduce" => 55.0,
        "increase_candidate" => 35.0,
        "hold" => 15.0,
        _ => 25.0,
    };

    PositionAdviceItem {
        symbol: position.symbol.clone(),
        name: rating
            .map(|item| item.name.clone())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| position.name.clone()),
        theme: rating
            .map(|item| item.theme.clone())
            .unwrap_or_else(|| "未覆盖".to_string()),
        weight: round1(position.weight),
        current_price,
        avg_cost: position.avg_cost.map(round2),
        unrealized_return_percent,
        rating_score: current_evidence
            .then(|| rating.map(|item| item.score))
            .flatten(),
        rating_light: rating
            .map(|item| item.light.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        rating_status: rating
            .map(|item| item.data_status.clone())
            .unwrap_or_else(|| "uncovered".to_string()),
        valuation_position: rating
            .and_then(|item| item.valuation.as_ref())
            .map(|value| value.current_position.clone())
            .unwrap_or_else(|| "今日未复核".to_string()),
        news_impact: news
            .map(|item| item.impact.clone())
            .unwrap_or_else(|| "none".to_string()),
        news_attention: news
            .map(|item| item.attention.clone())
            .unwrap_or_else(|| "无重点新闻".to_string()),
        action: action.to_string(),
        action_label: label.to_string(),
        confidence: confidence.to_string(),
        rationale,
        risks: rating
            .map(|item| item.risks.iter().take(3).cloned().collect())
            .unwrap_or_default(),
        falsifiers: rating
            .map(|item| item.falsifiers.iter().take(3).cloned().collect())
            .unwrap_or_default(),
        framework_logic: vec![
            "LOG-V0001".to_string(),
            "LOG-V0002".to_string(),
            "LOG-V0003".to_string(),
            "LOG-V0004".to_string(),
            "LOG-V0005".to_string(),
            "LOG-V0006".to_string(),
        ],
        evidence_as_of: rating
            .into_iter()
            .flat_map(|item| [item.market_as_of.clone(), item.financial_as_of.clone()])
            .flatten()
            .collect(),
        evidence_sources: rating
            .map(|item| item.data_sources.clone())
            .unwrap_or_default(),
        priority_score: round1(priority.min(100.0)),
        decision_gate,
    }
}

fn snapshot_needs_rebuild(
    snapshot: &PositionManagementSnapshot,
    portfolio_updated_at: &str,
    latest_upstream_at: DateTime<Utc>,
) -> bool {
    snapshot.model_version != MODEL_VERSION
        || snapshot.portfolio_updated_at != portfolio_updated_at
        || snapshot.generated_at < latest_upstream_at
}

async fn current_decisions(
    state: &AppState,
    positions: &[PositionInput],
) -> HashMap<String, InvestmentDecisionSnapshot> {
    let mut result = HashMap::new();
    for position in positions {
        if let Some(decision) =
            crate::routes::investment_decisions::current_decision(state, &position.symbol).await
        {
            result.insert(position.symbol.clone(), decision);
        }
    }
    result
}

fn position_decision_gate(
    decision: Option<&InvestmentDecisionSnapshot>,
    rating: Option<&CompanyRating>,
) -> PositionDecisionGate {
    let Some(decision) = decision else {
        return PositionDecisionGate {
            status: "missing".to_string(),
            blocking_reasons: vec!["缺少同标的、已通过点时校验的公司决策快照".to_string()],
            ..PositionDecisionGate::default()
        };
    };
    let methodology = &decision.decision.methodology;
    let mut blocking_reasons = methodology.blocking_reasons.clone();
    let rating_matches = rating.is_some_and(|rating| {
        decision.source_rating_score.to_bits() == rating.score.to_bits()
            && decision.source_rating_light == rating.light
            && decision.symbol == rating.symbol
    });
    if !rating_matches {
        blocking_reasons.push("公司决策与当前评级快照不一致".to_string());
    }
    if methodology.policy_version != DECISION_GATE_POLICY_VERSION {
        blocking_reasons.push("公司决策未使用当前 Hari 门禁版本".to_string());
    }
    if methodology.candidate_logic_used {
        blocking_reasons.push("公司决策错误地使用了候选或未确认逻辑".to_string());
    }
    if methodology.portfolio_action_authorized {
        blocking_reasons.push("公司层不得携带组合动作授权".to_string());
    }
    let increase_passed = rating_matches
        && methodology.policy_version == DECISION_GATE_POLICY_VERSION
        && !methodology.candidate_logic_used
        && !methodology.portfolio_action_authorized
        && methodology.pre_methodology_action == "increase_candidate"
        && methodology.increase_candidate_authorized
        && decision.decision.action == ExposureAction::IncreaseCandidate
        && decision.decision.zone == ResearchZone::Opportunity;
    let status = if increase_passed {
        "passed"
    } else if methodology.pre_methodology_action == "increase_candidate" {
        if blocking_reasons.is_empty() {
            blocking_reasons.push("公司决策未形成最终增加暴露候选".to_string());
        }
        "blocked"
    } else if !rating_matches || methodology.policy_version != DECISION_GATE_POLICY_VERSION {
        "mismatched"
    } else {
        if blocking_reasons.is_empty() {
            blocking_reasons.push("本轮公司决策不是增加暴露候选".to_string());
        }
        "not_applicable"
    };
    PositionDecisionGate {
        status: status.to_string(),
        revision_id: decision.revision_id.clone(),
        decision_at: decision.decision_at.to_rfc3339(),
        policy_version: methodology.policy_version.clone(),
        skill_version: methodology.skill_version.clone(),
        pre_methodology_action: methodology.pre_methodology_action.clone(),
        final_action: match decision.decision.action {
            ExposureAction::IncreaseCandidate => "increase_candidate",
            ExposureAction::Maintain => "maintain",
            ExposureAction::ReduceCandidate => "reduce_candidate",
            ExposureAction::ResearchOnly => "research_only",
        }
        .to_string(),
        confirmed_logic_ids: methodology.confirmed_logic_ids.clone(),
        candidate_logic_used: methodology.candidate_logic_used,
        increase_candidate_authorized: methodology.increase_candidate_authorized,
        portfolio_action_authorized: methodology.portfolio_action_authorized,
        blocking_reasons,
    }
}

fn position_portfolio_gate(
    positions: &[PositionInput],
    macro_context: &PositionMacroContext,
    concentration: &ConcentrationSummary,
    total_weight: f64,
) -> PositionPortfolioGate {
    let today = Utc::now()
        .with_timezone(&Shanghai)
        .format("%Y-%m-%d")
        .to_string();
    let macro_current = macro_context.report_date == today
        && !matches!(macro_context.status.as_str(), "stale" | "framework_only")
        && matches!(macro_context.signal.as_str(), "green" | "yellow" | "red");
    let theme_evidence = concentration
        .theme_exposures
        .iter()
        .map(|item| format!("{} {:.1}%", item.theme, item.weight))
        .collect::<Vec<_>>();

    let market_rule = PositionPortfolioRuleApplication {
        logic_id: "LOG-V0003".to_string(),
        logic_version: "0.1".to_string(),
        label: "牛熊识别与组合总仓位控制".to_string(),
        status: if macro_current {
            "partial_evidence"
        } else {
            "missing_current_evidence"
        }
        .to_string(),
        evidence: [
            macro_current.then(|| {
                format!(
                    "当日宏观 {} / {}，健康分 {}",
                    macro_context.signal,
                    macro_context.phase,
                    macro_context
                        .score
                        .map(|value| format!("{value:.1}"))
                        .unwrap_or_else(|| "未取得".to_string())
                )
            }),
            Some(format!(
                "组合已配置 {:.1}%，未配置 {:.1}%",
                total_weight,
                (100.0 - total_weight).max(0.0)
            )),
        ]
        .into_iter()
        .flatten()
        .collect(),
        gaps: [
            (!macro_current).then(|| "缺少当日可用宏观环境快照".to_string()),
            Some("牛熊判定阈值、目标总仓位与切换节奏尚未获得老王确认".to_string()),
        ]
        .into_iter()
        .flatten()
        .collect(),
    };
    let barbell_rule = PositionPortfolioRuleApplication {
        logic_id: "LOG-V0004".to_string(),
        logic_version: "0.1".to_string(),
        label: "随市场状态切换的杠铃组合".to_string(),
        status: "missing_required_classification".to_string(),
        evidence: vec![format!("当前读取 {} 个真实持仓", positions.len())],
        gaps: vec![
            "持仓尚无经过确认的压舱石/高成长 Beta 角色标签及其点时证据".to_string(),
            "杠铃两端比例、相关性与状态切换条件尚未获得老王确认".to_string(),
        ],
    };
    let sector_rule = PositionPortfolioRuleApplication {
        logic_id: "LOG-V0005".to_string(),
        logic_version: "0.1".to_string(),
        label: "板块优先的仓位分配".to_string(),
        status: if theme_evidence.is_empty() {
            "missing_current_evidence"
        } else {
            "partial_evidence"
        }
        .to_string(),
        evidence: theme_evidence,
        gaps: vec![
            "板块机会强弱、预算档位与板块内相对质量尚未形成点时组合结论".to_string(),
            "同板块公司风险相关性和精确预算阈值尚未获得老王确认".to_string(),
            "HONE 集中度提示线不能冒充 Hari 板块预算规则".to_string(),
        ],
    };

    PositionPortfolioGate {
        policy_version: PORTFOLIO_GATE_POLICY_VERSION.to_string(),
        skill_id: "hari-invest".to_string(),
        skill_version: "0.1.0".to_string(),
        confirmed_logic_ids: vec![
            "LOG-V0003".to_string(),
            "LOG-V0004".to_string(),
            "LOG-V0005".to_string(),
        ],
        candidate_logic_used: false,
        status: "incomplete_confirmed_parameters".to_string(),
        rules: vec![market_rule, barbell_rule, sector_rule],
        blocking_reasons: vec![
            "LOG-V0003：缺少已确认的牛熊阈值、总仓位目标与切换节奏".to_string(),
            "LOG-V0004：缺少经确认的组合角色分类、比例与相关性检查".to_string(),
            "LOG-V0005：缺少经确认的板块预算、相对质量与风险相关性规则".to_string(),
        ],
        increase_candidate_authorized: false,
        portfolio_action_authorized: false,
        shadow_portfolio_authorized: false,
        trade_authorized: false,
        scope: "只组织 LOG-V0003/4/5 的当前组合证据和缺口；未确认参数不补造，不生成目标仓位、订单或交易授权。".to_string(),
    }
}

fn latest_analyzed_news<'a>(
    snapshot: Option<&'a PortfolioNewsSnapshot>,
) -> HashMap<String, &'a PortfolioNewsItem> {
    let Some(snapshot) = snapshot.filter(|value| {
        matches!(
            value.status.as_str(),
            "live" | "partial" | "source_only" | "no_material_news"
        )
    }) else {
        return HashMap::new();
    };
    let mut result = HashMap::new();
    for item in &snapshot.items {
        result
            .entry(item.symbol.to_ascii_uppercase())
            .and_modify(|current: &mut &PortfolioNewsItem| {
                if item.priority_score > current.priority_score {
                    *current = item;
                }
            })
            .or_insert(item);
    }
    result
}

fn news_decision_ready(snapshot: Option<&PortfolioNewsSnapshot>, symbol: &str) -> bool {
    let Some(snapshot) = snapshot.filter(|value| {
        matches!(
            value.status.as_str(),
            "live" | "partial" | "no_material_news"
        ) && value.source_status == "live"
            && Utc::now() - value.generated_at <= chrono::Duration::hours(36)
    }) else {
        return false;
    };
    let symbol = symbol.to_ascii_uppercase();
    let relevant = snapshot
        .items
        .iter()
        .filter(|item| item.symbol.eq_ignore_ascii_case(&symbol))
        .collect::<Vec<_>>();
    if !relevant.is_empty() {
        return relevant
            .iter()
            .all(|item| item.analysis_status == "model_analyzed");
    }
    snapshot
        .coverage_items
        .iter()
        .any(|item| item.symbol.eq_ignore_ascii_case(&symbol) && item.status == "no_material_news")
}

fn real_positions(portfolio: &Portfolio) -> Vec<PositionInput> {
    let mut result = HashMap::<String, PositionInput>::new();
    for (holding, weight) in portfolio
        .holdings
        .iter()
        .zip(holdings_with_weights(&portfolio.holdings))
        .filter(|(holding, weight)| !holding.tracking_only.unwrap_or(false) && weight.is_some())
    {
        let symbol = position_symbol(holding);
        if symbol.is_empty() {
            continue;
        }
        let value = weight.unwrap_or_default();
        result
            .entry(symbol.clone())
            .and_modify(|item| item.weight += value)
            .or_insert_with(|| PositionInput {
                symbol,
                name: holding
                    .name
                    .clone()
                    .unwrap_or_else(|| holding.symbol.clone()),
                weight: value,
                avg_cost: (!holding.asset_type.eq_ignore_ascii_case("option")
                    && holding.avg_cost.is_finite()
                    && holding.avg_cost > 0.0)
                    .then_some(holding.avg_cost),
            });
    }
    let mut positions = result.into_values().collect::<Vec<_>>();
    positions.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    positions
}

fn position_symbol(holding: &Holding) -> String {
    if holding.asset_type.eq_ignore_ascii_case("option") {
        holding.underlying.as_deref().unwrap_or(&holding.symbol)
    } else {
        &holding.symbol
    }
    .trim()
    .to_ascii_uppercase()
}

fn concentration_summary(
    positions: &[PositionInput],
    themes: &HashMap<String, String>,
) -> ConcentrationSummary {
    let mut sorted = positions.to_vec();
    sorted.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    let largest = sorted.first();
    let top_three = sorted.iter().take(3).map(|item| item.weight).sum::<f64>();
    let mut by_theme = HashMap::<String, f64>::new();
    for position in positions {
        *by_theme
            .entry(
                themes
                    .get(&position.symbol)
                    .cloned()
                    .unwrap_or_else(|| "未覆盖".to_string()),
            )
            .or_default() += position.weight;
    }
    let mut theme_exposures = by_theme
        .into_iter()
        .map(|(theme, weight)| ThemeExposure {
            theme,
            weight: round1(weight),
        })
        .collect::<Vec<_>>();
    theme_exposures.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    let largest_weight = largest.map(|item| item.weight).unwrap_or_default();
    let max_theme = theme_exposures
        .first()
        .map(|item| item.weight)
        .unwrap_or_default();
    let level = if largest_weight >= 25.0 || max_theme >= 45.0 {
        "high"
    } else if largest_weight >= 15.0 || top_three >= 60.0 {
        "elevated"
    } else {
        "balanced"
    };
    ConcentrationSummary {
        level: level.to_string(),
        largest_symbol: largest.map(|item| item.symbol.clone()).unwrap_or_default(),
        largest_weight: round1(largest_weight),
        top_three_weight: round1(top_three),
        theme_exposures,
    }
}

fn action_counts(items: &[PositionAdviceItem]) -> PositionActionCounts {
    let mut counts = PositionActionCounts::default();
    for item in items {
        match item.action.as_str() {
            "increase_candidate" => counts.increase_candidate += 1,
            "hold" => counts.hold += 1,
            "reduce" => counts.reduce += 1,
            "review" => counts.review += 1,
            _ => counts.insufficient_data += 1,
        }
    }
    counts
}

fn light_label(value: &str) -> &'static str {
    match value {
        "green" => "绿灯",
        "yellow" => "黄灯",
        "red" => "红灯",
        _ => "待定",
    }
}

fn waiting_snapshot(
    portfolio: &Portfolio,
    positions: &[PositionInput],
) -> PositionManagementSnapshot {
    let mut snapshot = empty_snapshot(
        &portfolio.updated_at,
        "waiting_refresh",
        "持仓已读取，等待每日 20:00 生成第一份仓位管理建议。",
    );
    snapshot.holdings_count = positions.len();
    snapshot.total_weight = round1(positions.iter().map(|item| item.weight).sum());
    snapshot.unallocated_weight = round1((100.0 - snapshot.total_weight).max(0.0));
    snapshot
}

fn empty_snapshot(
    portfolio_updated_at: &str,
    status: &str,
    summary: &str,
) -> PositionManagementSnapshot {
    let now = Utc::now();
    PositionManagementSnapshot {
        report_date: now.with_timezone(&Shanghai).format("%Y-%m-%d").to_string(),
        generated_at: now,
        generated_at_beijing: now
            .with_timezone(&Shanghai)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        next_refresh_at: next_refresh(now),
        timezone: "Asia/Shanghai".to_string(),
        model_version: MODEL_VERSION.to_string(),
        framework_version: FRAMEWORK_VERSION.to_string(),
        status: status.to_string(),
        portfolio_updated_at: portfolio_updated_at.to_string(),
        holdings_count: 0,
        total_weight: 0.0,
        unallocated_weight: 100.0,
        concentration: ConcentrationSummary {
            level: "unknown".to_string(),
            largest_symbol: String::new(),
            largest_weight: 0.0,
            top_three_weight: 0.0,
            theme_exposures: Vec::new(),
        },
        macro_context: PositionMacroContext {
            signal: "unknown".to_string(),
            score: None,
            phase: "等待数据".to_string(),
            report_date: String::new(),
            status: "not_run".to_string(),
        },
        portfolio_gate: PositionPortfolioGate {
            policy_version: PORTFOLIO_GATE_POLICY_VERSION.to_string(),
            skill_id: "hari-invest".to_string(),
            skill_version: "0.1.0".to_string(),
            confirmed_logic_ids: vec![
                "LOG-V0003".to_string(),
                "LOG-V0004".to_string(),
                "LOG-V0005".to_string(),
            ],
            candidate_logic_used: false,
            status: "waiting_for_portfolio".to_string(),
            rules: Vec::new(),
            blocking_reasons: vec!["尚无可评估的真实组合".to_string()],
            increase_candidate_authorized: false,
            portfolio_action_authorized: false,
            shadow_portfolio_authorized: false,
            trade_authorized: false,
            scope: "只组织 LOG-V0003/4/5 的当前组合证据和缺口；未确认参数不补造，不生成目标仓位、订单或交易授权。".to_string(),
        },
        counts: PositionActionCounts::default(),
        summary: summary.to_string(),
        items: Vec::new(),
        methodology_note: "公司层增加暴露必须通过同一点时决策大脑的 Hari LOG-V0001/2/6 门禁；LOG-V0003/4/5 留在组合层，且不授权交易。".to_string(),
        disclaimer:
            "仓位建议仅供研究与复核，不构成个性化投资顾问、收益承诺或自动交易；HONE 不会修改持仓。"
                .to_string(),
    }
}

fn snapshot_dir(state: &AppState, actor: &ActorIdentity) -> PathBuf {
    PathBuf::from(&state.core.config.storage.portfolio_dir)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("./data"))
        .join("position_management")
        .join(actor.storage_key())
}

async fn read_snapshot(
    state: &AppState,
    actor: &ActorIdentity,
) -> Option<PositionManagementSnapshot> {
    let bytes = tokio::fs::read(snapshot_dir(state, actor).join("latest.json"))
        .await
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

async fn write_snapshot(
    state: &AppState,
    actor: &ActorIdentity,
    snapshot: &PositionManagementSnapshot,
) -> anyhow::Result<()> {
    let dir = snapshot_dir(state, actor);
    let history = dir.join("history");
    tokio::fs::create_dir_all(&history).await?;
    let bytes = serde_json::to_vec_pretty(snapshot)?;
    atomic_write(dir.join("latest.json"), &bytes).await?;
    atomic_write(
        history.join(format!("{}.json", snapshot.report_date)),
        &bytes,
    )
    .await?;
    Ok(())
}

async fn atomic_write(path: PathBuf, bytes: &[u8]) -> anyhow::Result<()> {
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    tokio::fs::write(&temp, bytes).await?;
    tokio::fs::rename(temp, path).await?;
    Ok(())
}

fn next_refresh(now: DateTime<Utc>) -> DateTime<Utc> {
    let local = now.with_timezone(&Shanghai);
    let today = Shanghai
        .with_ymd_and_hms(local.year(), local.month(), local.day(), 20, 0, 0)
        .single()
        .expect("Shanghai local time is unambiguous");
    (if local < today {
        today
    } else {
        today + chrono::Duration::days(1)
    })
    .with_timezone(&Utc)
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::company_ratings::{DailyValuation, RatingDimensions};

    fn holding(symbol: &str, weight: Option<f64>, tracking: bool) -> Holding {
        Holding {
            symbol: symbol.to_string(),
            asset_type: "stock".to_string(),
            shares: 0.0,
            avg_cost: 100.0,
            underlying: None,
            option_type: None,
            strike_price: None,
            expiration_date: None,
            contract_multiplier: None,
            holding_horizon: None,
            strategy_notes: None,
            notes: None,
            weight,
            name: Some(symbol.to_string()),
            tracking_only: Some(tracking),
        }
    }

    fn portfolio(holdings: Vec<Holding>) -> Portfolio {
        Portfolio {
            actor: None,
            user_id: "u".to_string(),
            holdings,
            updated_at: "2026-08-11T10:00:00+08:00".to_string(),
        }
    }

    fn rating(
        symbol: &str,
        light: &str,
        score: f64,
        valuation: Option<DailyValuation>,
    ) -> CompanyRating {
        CompanyRating {
            name: symbol.to_string(),
            symbol: symbol.to_string(),
            market_scope: "US".to_string(),
            theme: "AI".to_string(),
            value_chain: "compute".to_string(),
            score,
            light: light.to_string(),
            confidence: "high".to_string(),
            data_status: "live".to_string(),
            price: Some(100.0),
            change_percent: Some(1.0),
            price_avg50: Some(95.0),
            price_avg200: Some(85.0),
            year_low: Some(70.0),
            year_high: Some(120.0),
            market_history: None,
            short_interest: None,
            options_positioning: None,
            news_attention: None,
            institutional_holdings: None,
            analyst_consensus: None,
            market_as_of: Some("2026-08-11 09:30 ET".to_string()),
            financial_as_of: Some("2026-06-30".to_string()),
            thesis_summary: "thesis".to_string(),
            business_model: "model".to_string(),
            moat: "moat".to_string(),
            valuation_method: "DCF".to_string(),
            valuation,
            valuation_unavailable_reason: String::new(),
            dimensions: RatingDimensions {
                moat: 90.0,
                scarcity: 90.0,
                fundamentals: 85.0,
                visibility: 80.0,
                growth_quality: Some(85.0),
                pricing_power: Some(80.0),
                financial_quality: Some(85.0),
                valuation: Some(80.0),
                market_confirmation: 80.0,
                timing: Some(80.0),
            },
            metrics: Default::default(),
            score_cap_reason: String::new(),
            factor_coverage: 8,
            watch_items: vec![],
            risks: vec!["risk".to_string()],
            falsifiers: vec!["falsifier".to_string()],
            research_updated_at: "2026-08-01".to_string(),
            data_sources: vec!["FMP".to_string()],
        }
    }

    fn valuation(current: f64, base: f64, bull: f64) -> DailyValuation {
        DailyValuation {
            as_of: "2026-08-11".to_string(),
            generated_at_beijing: "2026-08-11 19:30".to_string(),
            currency: "USD".to_string(),
            bear_case: 70.0,
            base_case: base,
            bull_case: bull,
            current_price: current,
            probability_weighted_value: base,
            expected_upside_percent: (base / current - 1.0) * 100.0,
            method_count: 3,
            confidence: "high".to_string(),
            current_position: if current > bull {
                "高于乐观值"
            } else {
                "悲观—基准之间"
            }
            .to_string(),
            position_percent: 40.0,
            method: "DCF".to_string(),
            assumptions: vec!["a".to_string()],
            sources: vec!["a".to_string(), "b".to_string()],
        }
    }

    fn macro_context(signal: &str, score: f64) -> PositionMacroContext {
        PositionMacroContext {
            signal: signal.to_string(),
            score: Some(score),
            phase: "phase".to_string(),
            report_date: Utc::now()
                .with_timezone(&Shanghai)
                .format("%Y-%m-%d")
                .to_string(),
            status: "live".to_string(),
        }
    }

    fn passed_decision_gate() -> PositionDecisionGate {
        PositionDecisionGate {
            status: "passed".to_string(),
            revision_id: "NVDA-test-decision".to_string(),
            decision_at: "2026-08-11T12:00:00Z".to_string(),
            policy_version: DECISION_GATE_POLICY_VERSION.to_string(),
            skill_version: "0.1.0".to_string(),
            pre_methodology_action: "increase_candidate".to_string(),
            final_action: "increase_candidate".to_string(),
            confirmed_logic_ids: vec![
                "LOG-V0001".to_string(),
                "LOG-V0002".to_string(),
                "LOG-V0003".to_string(),
                "LOG-V0004".to_string(),
                "LOG-V0005".to_string(),
                "LOG-V0006".to_string(),
            ],
            candidate_logic_used: false,
            increase_candidate_authorized: true,
            portfolio_action_authorized: false,
            blocking_reasons: Vec::new(),
        }
    }

    #[test]
    fn legacy_snapshot_is_rebuilt_for_the_decision_brain_gate() {
        let mut snapshot = empty_snapshot("portfolio-v1", "live", "test");
        let upstream_before_snapshot = snapshot.generated_at - chrono::Duration::seconds(1);
        assert!(!snapshot_needs_rebuild(
            &snapshot,
            "portfolio-v1",
            upstream_before_snapshot
        ));

        snapshot.model_version = "hone-position-management-v1".to_string();
        assert!(snapshot_needs_rebuild(
            &snapshot,
            "portfolio-v1",
            upstream_before_snapshot
        ));
    }

    #[test]
    fn watchlist_is_excluded_and_options_merge_by_underlying() {
        let mut option = holding("NVDA260918C200", Some(10.0), false);
        option.asset_type = "option".to_string();
        option.underlying = Some("NVDA".to_string());
        let positions = real_positions(&portfolio(vec![
            holding("NVDA", Some(30.0), false),
            option,
            holding("TSLA", None, true),
        ]));
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].symbol, "NVDA");
        assert_eq!(positions[0].weight, 40.0);
    }

    #[test]
    fn concentration_uses_single_and_theme_bands() {
        let positions = vec![
            PositionInput {
                symbol: "A".into(),
                name: "A".into(),
                weight: 26.0,
                avg_cost: None,
            },
            PositionInput {
                symbol: "B".into(),
                name: "B".into(),
                weight: 20.0,
                avg_cost: None,
            },
        ];
        let themes = HashMap::from([("A".into(), "AI".into()), ("B".into(), "AI".into())]);
        let summary = concentration_summary(&positions, &themes);
        assert_eq!(summary.level, "high");
        assert_eq!(summary.top_three_weight, 46.0);
        assert_eq!(summary.theme_exposures[0].weight, 46.0);
    }

    #[test]
    fn transcript_baseline_never_becomes_current_action() {
        let mut baseline = rating("NVDA", "green", 95.0, Some(valuation(90.0, 110.0, 140.0)));
        baseline.data_status = "transcript_only".to_string();
        baseline.price = None;
        baseline.financial_as_of = None;
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let advice = advise_position(
            &position,
            Some(&baseline),
            PositionDecisionGate::default(),
            None,
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );
        assert_eq!(advice.action, "insufficient_data");
        assert!(advice.rating_score.is_none());
    }

    #[test]
    fn green_current_company_requires_verified_valuation_for_increase_candidate() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let with_value = rating("NVDA", "green", 90.0, Some(valuation(90.0, 110.0, 140.0)));
        let advice = advise_position(
            &position,
            Some(&with_value),
            passed_decision_gate(),
            None,
            true,
            &macro_context("green", 80.0),
            &concentration,
            true,
        );
        assert_eq!(advice.action, "increase_candidate");
        let without_value = rating("NVDA", "green", 90.0, None);
        let advice = advise_position(
            &position,
            Some(&without_value),
            PositionDecisionGate::default(),
            None,
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );
        assert_eq!(advice.action, "hold");
    }

    #[test]
    fn company_candidate_cannot_bypass_the_confirmed_portfolio_gate() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let current = rating("NVDA", "green", 90.0, Some(valuation(90.0, 110.0, 140.0)));
        let advice = advise_position(
            &position,
            Some(&current),
            passed_decision_gate(),
            None,
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );

        assert_eq!(advice.action, "review");
        assert_eq!(advice.action_label, "组合门禁复核");
        assert_eq!(advice.confidence, "low");
        assert!(
            advice
                .rationale
                .iter()
                .any(|line| line.contains("LOG-V0003/4/5") && line.contains("不能形成加仓候选"))
        );
    }

    #[test]
    fn portfolio_gate_exposes_only_confirmed_logic_and_never_invents_parameters() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let gate = position_portfolio_gate(
            &[position],
            &macro_context("green", 80.0),
            &concentration,
            10.0,
        );

        assert_eq!(gate.policy_version, PORTFOLIO_GATE_POLICY_VERSION);
        assert_eq!(
            gate.confirmed_logic_ids,
            vec!["LOG-V0003", "LOG-V0004", "LOG-V0005"]
        );
        assert!(!gate.candidate_logic_used);
        assert_eq!(gate.status, "incomplete_confirmed_parameters");
        assert!(!gate.increase_candidate_authorized);
        assert!(!gate.portfolio_action_authorized);
        assert!(!gate.shadow_portfolio_authorized);
        assert!(!gate.trade_authorized);
        assert_eq!(gate.rules.len(), 3);
        assert!(
            gate.blocking_reasons
                .iter()
                .any(|line| line.contains("牛熊阈值"))
        );
        assert!(
            gate.blocking_reasons
                .iter()
                .any(|line| line.contains("组合角色分类"))
        );
        assert!(
            gate.blocking_reasons
                .iter()
                .any(|line| line.contains("板块预算"))
        );
    }

    #[test]
    fn company_rating_cannot_bypass_the_confirmed_hari_decision_gate() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let current = rating("NVDA", "green", 90.0, Some(valuation(90.0, 110.0, 140.0)));
        let blocked = PositionDecisionGate {
            status: "blocked".to_string(),
            blocking_reasons: vec!["LOG-V0002：公司价值捕获尚未验证".to_string()],
            ..passed_decision_gate()
        };
        let advice = advise_position(
            &position,
            Some(&current),
            blocked,
            None,
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );

        assert_eq!(advice.action, "review");
        assert_eq!(advice.action_label, "决策门禁复核");
        assert_eq!(advice.confidence, "low");
        assert!(advice.rationale.iter().any(|line| {
            line.contains("统一决策大脑门禁未通过") && line.contains("LOG-V0002")
        }));
        assert!(!advice.decision_gate.portfolio_action_authorized);
    }

    #[test]
    fn current_quote_and_research_baseline_produce_low_confidence_action_without_financials() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: Some(80.0),
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let mut partial = rating("NVDA", "green", 82.0, None);
        partial.data_status = "partial".to_string();
        partial.financial_as_of = None;
        partial.data_sources = vec!["Nasdaq 官方行情降级快照".to_string()];
        let advice = advise_position(
            &position,
            Some(&partial),
            PositionDecisionGate::default(),
            None,
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );
        assert_eq!(advice.action, "hold");
        assert_eq!(advice.confidence, "low");
        assert_eq!(advice.rating_score, Some(82.0));
        assert!(
            advice
                .rationale
                .iter()
                .any(|line| line.contains("不触发加仓"))
        );
    }

    #[test]
    fn macro_red_suppresses_increase_candidate() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let current = rating("NVDA", "green", 90.0, Some(valuation(90.0, 110.0, 140.0)));
        let advice = advise_position(
            &position,
            Some(&current),
            PositionDecisionGate::default(),
            None,
            true,
            &macro_context("red", 40.0),
            &concentration,
            false,
        );
        assert_eq!(advice.action, "hold");
    }

    #[test]
    fn expensive_concentrated_position_reduces_exposure() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 20.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let current = rating("NVDA", "green", 90.0, Some(valuation(150.0, 110.0, 140.0)));
        let advice = advise_position(
            &position,
            Some(&current),
            PositionDecisionGate::default(),
            None,
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );
        assert_eq!(advice.action, "reduce");
    }

    #[test]
    fn negative_thesis_news_forces_review_before_other_actions() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let current = rating("NVDA", "green", 90.0, Some(valuation(90.0, 110.0, 140.0)));
        let news = PortfolioNewsItem {
            id: "n".into(),
            symbol: "NVDA".into(),
            title: "title".into(),
            published_at: Utc::now(),
            published_at_beijing: "08-11 10:00".into(),
            source: "Reuters".into(),
            source_url: "https://reuters.com/n".into(),
            source_summary: "summary".into(),
            severity: "high".into(),
            impact: "negative".into(),
            horizon: "medium".into(),
            thesis_effect: "weakens".into(),
            summary: "summary".into(),
            why_it_matters: "why".into(),
            attention: "立即复核".into(),
            confidence: "high".into(),
            analysis_status: "model_analyzed".into(),
            priority_score: 90.0,
        };
        let advice = advise_position(
            &position,
            Some(&current),
            PositionDecisionGate::default(),
            Some(&news),
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );
        assert_eq!(advice.action, "review");
        assert_eq!(advice.confidence, "high");

        let mut partial = current;
        partial.financial_as_of = None;
        let partial_advice = advise_position(
            &position,
            Some(&partial),
            PositionDecisionGate::default(),
            Some(&news),
            true,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );
        assert_eq!(partial_advice.action, "review");
        assert_eq!(partial_advice.confidence, "low");
    }

    #[test]
    fn incomplete_news_analysis_blocks_increase_candidate_and_hold() {
        let position = PositionInput {
            symbol: "NVDA".into(),
            name: "NVDA".into(),
            weight: 10.0,
            avg_cost: None,
        };
        let concentration = concentration_summary(
            &[position.clone()],
            &HashMap::from([("NVDA".into(), "AI".into())]),
        );
        let current = rating("NVDA", "green", 90.0, Some(valuation(90.0, 110.0, 140.0)));
        let advice = advise_position(
            &position,
            Some(&current),
            PositionDecisionGate::default(),
            None,
            false,
            &macro_context("green", 80.0),
            &concentration,
            false,
        );
        assert_eq!(advice.action, "review");
        assert_eq!(advice.action_label, "等待分析");
        assert_eq!(advice.confidence, "low");
        assert!(
            advice
                .rationale
                .iter()
                .any(|line| line.contains("未分析不等于没有风险"))
        );
    }
}
