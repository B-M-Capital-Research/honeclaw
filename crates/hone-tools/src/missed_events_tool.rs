//! `MissedEventsTool` —— 让用户通过自然语言或 `/missed` 斜杠命令查询「digest
//! 没推给我但我可能想看的事件」。
//!
//! 从 event-engine 的 `delivery_log` 里捞同 actor 最近的 omitted/capped/cooled_down/
//! filtered 行。这些都是被 router 或 digest 主动筛掉的事件,**不是 bug 也不是丢失**,
//! 而是 noise/数量上限/冷却/用户偏好造成的有意识取舍——本工具让用户回看决策。
//!
//! 设计原则:
//! - 构造时绑定 `actor` —— 不允许查别人的;
//! - `events_db_path` 直接指向 sqlite 文件,每次 `execute` 打开一次。EventStore
//!   open 是 idempotent + 快的(<1ms),不开常驻连接是为了简单 + 避免 tool 持有
//!   跨进程 lock(BotCore 不持有 Arc<EventStore>);
//! - 返回结构化 JSON 列表,LLM 自行渲染中文文案给用户。

use async_trait::async_trait;
use hone_core::{ActorIdentity, HoneError, HoneResult};
use hone_event_engine::store::EventStore;
use serde_json::{Value, json};
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};

pub struct MissedEventsTool {
    events_db_path: PathBuf,
    actor: Option<ActorIdentity>,
}

impl MissedEventsTool {
    pub fn new(events_db_path: impl Into<PathBuf>, actor: Option<ActorIdentity>) -> Self {
        Self {
            events_db_path: events_db_path.into(),
            actor,
        }
    }

    fn actor_key(&self) -> HoneResult<String> {
        let actor = self
            .actor
            .as_ref()
            .ok_or_else(|| HoneError::Tool("缺少 actor 身份,无法查询本人的未展示事件".into()))?;
        Ok(format!(
            "{}::{}::{}",
            actor.channel,
            actor.channel_scope.clone().unwrap_or_default(),
            actor.user_id
        ))
    }
}

/// status → 「人话」原因。LLM 复述给用户时不要直接用 status 英文 token,
/// 要用这边的中文标签让用户直观看懂为什么没收到。
fn status_label(status: &str) -> &str {
    match status {
        "omitted" => "被 curation 砍掉(噪音/重复/低质)",
        "capped" => "已超过当日 High 推送上限",
        "cooled_down" => "同 ticker 推送冷却中",
        "price_capped" => "同 symbol+方向 当日价格推送上限",
        "price_cooled_down" => "同 symbol+方向 价格推送冷却中",
        "filtered" => "命中你的推送偏好被过滤",
        other => other,
    }
}

#[async_trait]
impl Tool for MissedEventsTool {
    fn name(&self) -> &str {
        "missed_events"
    }

    fn description(&self) -> &str {
        "查询当前用户最近一段时间内被 digest/router 主动筛掉、没有推送的市场事件。\
         典型来源:每批数量上限截断、同 ticker/symbol 推送冷却、同主题 jaccard \
         去重、opinion_blog/PR-wire 噪音过滤、用户推送偏好过滤。\
         适用场景:用户问「最近有什么我没看到的」「digest 漏推了什么」「/missed」。\
         返回每条事件的标题、symbol、kind、来源以及被砍的具体原因。\
         默认看过去 24 小时;`since_hours` 可调到最大 168(7 天)。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "since_hours".to_string(),
                param_type: "number".to_string(),
                description: "从现在向前回看多少小时,默认 24,最大 168(7 天)。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "limit".to_string(),
                param_type: "number".to_string(),
                description: "最多返回多少条,默认 30,最大 200。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> HoneResult<Value> {
        let actor_key = self.actor_key()?;
        let since_hours = args
            .get("since_hours")
            .and_then(|v| v.as_f64())
            .map(|h| h.clamp(1.0, 168.0))
            .unwrap_or(24.0);
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n.clamp(1, 200))
            .unwrap_or(30) as usize;

        let store = EventStore::open(&self.events_db_path).map_err(|e| {
            HoneError::Tool(format!(
                "打开 event store 失败({}): {e}",
                self.events_db_path.display()
            ))
        })?;
        let since = chrono::Utc::now()
            - chrono::Duration::milliseconds((since_hours * 3600.0 * 1000.0) as i64);
        let rows = store
            .list_missed_digest_items_since(&actor_key, since)
            .map_err(|e| HoneError::Tool(format!("查询 delivery_log 失败: {e}")))?;

        let truncated = rows.len() > limit;
        let items: Vec<Value> = rows
            .into_iter()
            .take(limit)
            .map(|(ev, status)| {
                json!({
                    "id": ev.id,
                    "kind": ev.kind,
                    "severity": match ev.severity {
                        hone_event_engine::Severity::High => "high",
                        hone_event_engine::Severity::Medium => "medium",
                        hone_event_engine::Severity::Low => "low",
                    },
                    "symbols": ev.symbols,
                    "title": ev.title,
                    "source": ev.source,
                    "url": ev.url,
                    "occurred_at": ev.occurred_at.to_rfc3339(),
                    "status": status,
                    "reason": status_label(&status),
                })
            })
            .collect();

        Ok(json!({
            "status": "ok",
            "actor": actor_key,
            "since_hours": since_hours,
            "count": items.len(),
            "truncated": truncated,
            "items": items,
        }))
    }
}
