//! One aggregated call that paints the research desk's first screen.
//!
//! Each section projects its latest durable snapshot into a compact card via
//! its own `overview_card` function; this module only assembles the grid. A
//! missing or unreadable snapshot degrades to a `waiting` card — the endpoint
//! never fabricates data and never triggers a section's live computation.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use hone_core::ActorIdentity;
use serde::Serialize;
use serde_json::json;

use crate::state::AppState;

const SUMMARY_MAX_CHARS: usize = 60;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OverviewCard {
    pub key: String,
    pub title: String,
    pub kicker: String,
    pub report_date: Option<String>,
    pub status: String,
    pub signal: Option<String>,
    pub score: Option<f64>,
    pub metric: Option<String>,
    pub summary: Option<String>,
    /// Snapshot generation time, used only for the payload-level
    /// `generated_at`; the card JSON itself stays compact.
    #[serde(skip)]
    pub generated_at: Option<DateTime<Utc>>,
}

impl OverviewCard {
    /// A card that has no data yet: `status: "waiting"`, everything else null.
    pub(crate) fn waiting(key: &str, title: &str, kicker: &str) -> Self {
        Self {
            key: key.to_string(),
            title: title.to_string(),
            kicker: kicker.to_string(),
            report_date: None,
            status: "waiting".to_string(),
            signal: None,
            score: None,
            metric: None,
            summary: None,
            generated_at: None,
        }
    }
}

/// Clamp a section summary to one short line for the overview grid.
pub(crate) fn short_summary(value: &str) -> String {
    let mut output = value
        .trim()
        .chars()
        .take(SUMMARY_MAX_CHARS)
        .collect::<String>();
    if value.trim().chars().count() > SUMMARY_MAX_CHARS {
        output.push('…');
    }
    output
}

pub(crate) async fn handle_get_research_overview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = ActorIdentity::new("web", &user.user_id, Option::<String>::None).ok();

    let mut cards = Vec::with_capacity(10);
    cards.push(
        super::daily_signals::overview_card(&state, "macro")
            .await
            .unwrap_or_else(|| {
                OverviewCard::waiting("daily-signal-macro", "宏观红绿灯", "领先周期判断")
            }),
    );
    cards.push(
        super::daily_signals::overview_card(&state, "ai")
            .await
            .unwrap_or_else(|| {
                OverviewCard::waiting("daily-signal-ai", "AI 红绿灯", "AI 增长可持续性")
            }),
    );
    cards.push(
        super::company_ratings::overview_card(&state)
            .await
            .unwrap_or_else(|| {
                OverviewCard::waiting("company-ratings", "公司评级", "52 家研究基线")
            }),
    );
    let is_admin = state
        .web_auth
        .is_web_admin(&user.user_id)
        .await
        .unwrap_or(false);
    if is_admin {
        cards.push(
            super::valuation_lab::overview_card(&state)
                .await
                .unwrap_or_else(|| {
                    OverviewCard::waiting("valuation-lab", "估值实验室", "三情景估值")
                }),
        );
    }
    let portfolio_news_card = match actor.as_ref() {
        Some(actor) => super::portfolio_news::overview_card(&state, actor).await,
        None => None,
    };
    cards.push(portfolio_news_card.unwrap_or_else(|| {
        OverviewCard::waiting("portfolio-news", "持仓重点新闻", "按你的持仓筛选")
    }));
    let position_card = match actor.as_ref() {
        Some(actor) => super::position_management::overview_card(&state, actor).await,
        None => None,
    };
    cards.push(position_card.unwrap_or_else(|| {
        OverviewCard::waiting("position-management", "仓位管理", "评分 × 宏观 × 新闻")
    }));
    cards.push(
        super::influencer_digest::overview_card(&state)
            .await
            .unwrap_or_else(|| {
                OverviewCard::waiting("influencer-digest", "大V速报", "观点不等于事实")
            }),
    );
    // Weekly brief must never compute live here (it can fan out to FMP for
    // the whole coverage universe); only a pre-generated snapshot qualifies.
    cards.push(
        super::weekly_brief::overview_card(&state)
            .await
            .unwrap_or_else(|| OverviewCard::waiting("weekly-brief", "周度简报", "回顾与前瞻")),
    );
    cards.push(
        super::key_event_chain::overview_card(&state)
            .await
            .unwrap_or_else(|| {
                OverviewCard::waiting("key-event-chain", "关键事件链", "第一性证据链")
            }),
    );
    cards.push(
        super::research_library::overview_card(&state, &user.user_id)
            .await
            .unwrap_or_else(|| OverviewCard::waiting("research-library", "研究文库", "你的知识源")),
    );

    let generated_at = cards
        .iter()
        .filter_map(|card| card.generated_at)
        .max()
        .map(|value| value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    // The freshness label compares against the runtime calendar day, not the
    // reader's browser clock: a snapshot is "today's" by the schedule that
    // produced it, and a reader abroad must not be told otherwise.
    let report_today = hone_core::time::local_now().format("%Y-%m-%d").to_string();
    Json(json!({
        "generated_at": generated_at,
        "report_today": report_today,
        "cards": cards,
    }))
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_card_serializes_with_null_optionals() {
        let card = OverviewCard::waiting("weekly-brief", "周度简报", "回顾与前瞻");
        let value = serde_json::to_value(&card).expect("card serializes");
        assert_eq!(value["key"], "weekly-brief");
        assert_eq!(value["status"], "waiting");
        assert!(value["report_date"].is_null());
        assert!(value["signal"].is_null());
        assert!(value["score"].is_null());
        assert!(value["metric"].is_null());
        assert!(value["summary"].is_null());
        assert!(value.get("generated_at").is_none());
    }

    #[test]
    fn summaries_are_clamped_to_sixty_chars() {
        let long = "长".repeat(80);
        let short = short_summary(&long);
        assert_eq!(short.chars().count(), 61);
        assert!(short.ends_with('…'));
        assert_eq!(short_summary(" 简短摘要 "), "简短摘要");
    }
}
