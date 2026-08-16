//! Actor-scoped earnings research continuity.
//!
//! The shared earnings-quality review owns public facts. This module runs as a
//! post-delivery background step for A-tier tracked profiles: it compares those
//! facts with one actor's explicit thesis and append-only research ledger, then
//! records status changes without rewriting the thesis itself.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_llm::{LlmProvider, Message};
use hone_memory::{
    AppendEventInput, AppendResearchEventInput, CompanyProfileDocument, CompanyProfileStorage,
    CoverageTier, ResearchItemKind, ResearchItemStatus, ResearchLedgerItem, ResearchLedgerUpdate,
    research_item_id,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::earnings_document::{
    earnings_research_material_kind, earnings_research_object_key_for_event,
};
use crate::event::{EventKind, MarketEvent};

pub const DEFAULT_EARNINGS_CONTINUITY_SYSTEM_PROMPT: &str = r#"你是专业股票研究团队的季度连续性审计员。输入包含：本次已核验的财报事实、某一用户明确保存的公司画像，以及此前仍在跟踪的问题和管理层承诺。

你的职责不是重新总结财报，而是逐项回答“旧判断这次得到什么新证据”。只能使用输入材料，不得补充模型记忆或外部事实，不得替用户修改投资主线、估值假设、覆盖等级或证伪条件。

硬规则：
1. existing_items 中每一个项目都必须出现在 existing_item_updates。没有直接证据时保持原状态，并写“本季材料未回答”，不能让旧问题静默消失。
2. 只有输入明确回答时才能标 answered / confirmed；只回答一部分用 partially_answered；出现直接反证用 contradicted；到期且材料明确不再适用才用 expired。
   - open_question：完全回答才可用 answered + resolution_basis=answered；部分回答用 partially_answered + partial_answer；不得用 confirmed。
   - management_commitment：仅当承诺事项已经实际发生/交付且证据明确时，才可用 confirmed + fulfilled；只是重申计划、仍 on track、维持指引或尚未到期时必须保持 open + reaffirmed。部分兑现用 partially_answered + partially_fulfilled；明确撤回或未兑现用 contradicted + missed_or_withdrawn；不得用 answered。
   - 任何非 open 状态都必须有 evidence；状态与 resolution_basis 不匹配时系统会保持原状态。
3. new_commitments 只能记录管理层明确作出的、未来可核验的承诺或量化指引，不能把模型推断、愿景或一般性措辞写成承诺。
4. new_questions 必须会影响投资主线或下一次决策，而且说明预期验证材料或时间；不要生成“继续关注宏观环境”一类泛化问题。
5. thesis_effect 只是供用户确认的本季建议，不得声称已经修改主线。只要 Saved profile sections 中“投资主线”不是“待补充”占位文字，就视为已有有效用户主线，必须在 strengthen / unchanged / watch / weaken 中选择；市场共识或“预期基线”缺失不等于投资主线缺失。只有“投资主线”本身缺失或仍是占位文字时才可用 insufficient_baseline。
6. 事实、判断和未知必须分开；金额单位保持输入原样。
7. 这是结构化账本，不是长篇报告：每个 assessment_zh 最多 60 个汉字，不复述问题原文；evidence 最多 1 条且最多 60 个汉字。严格遵守输入中的 new_questions_limit 和 new_commitments_limit，限额为 0 时输出空数组。

输出单个 JSON object，不要 Markdown：
{
  "thesis_effect": "strengthen|unchanged|watch|weaken|insufficient_baseline",
  "thesis_reason_zh": "为什么，最多两句",
  "existing_item_updates": [
    {"item_id":"原 ID", "status":"open|partially_answered|answered|confirmed|contradicted|expired", "resolution_basis":"none|reaffirmed|partial_answer|answered|partially_fulfilled|fulfilled|missed_or_withdrawn|superseded", "assessment_zh":"最多60字的本季核对结论", "evidence":["最多1条、最多60字的输入内证据"]}
  ],
  "new_questions": [
    {"statement":"可在未来核验的问题", "due_at":"预计季度/日期/材料", "reason_zh":"为何影响主线"}
  ],
  "new_commitments": [
    {"statement":"管理层明确承诺", "due_at":"预计验证期", "reason_zh":"本次原文事实依据"}
  ],
  "next_actions": ["最多3个具体研究动作"]
}"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExistingResearchItemUpdate {
    pub item_id: String,
    pub status: String,
    #[serde(default)]
    pub resolution_basis: String,
    #[serde(default)]
    pub assessment_zh: String,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewResearchItem {
    pub statement: String,
    #[serde(default)]
    pub due_at: String,
    #[serde(default)]
    pub reason_zh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EarningsContinuityReview {
    pub thesis_effect: String,
    #[serde(default)]
    pub thesis_reason_zh: String,
    #[serde(default)]
    pub existing_item_updates: Vec<ExistingResearchItemUpdate>,
    #[serde(default)]
    pub new_questions: Vec<NewResearchItem>,
    #[serde(default)]
    pub new_commitments: Vec<NewResearchItem>,
    #[serde(default)]
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EarningsContinuityOutcome {
    pub profile_id: String,
    pub research_object_key: String,
    pub thesis_effect: String,
    pub recorded_event_id: String,
    pub checked_existing_items: usize,
    pub created_questions: usize,
    pub created_commitments: usize,
    pub active_questions_after: usize,
    pub active_commitments_after: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EarningsResearchMaterialOutcome {
    pub profile_id: String,
    pub research_object_key: String,
    pub material_kind: String,
    pub recorded_event_id: String,
}

#[async_trait]
pub trait EarningsContinuityReconciler: Send + Sync {
    /// 只有 actor 自己启用的 A 级画像才进入付费连续性对账队列。
    async fn should_schedule(&self, _actor: &ActorIdentity, _event: &MarketEvent) -> bool {
        true
    }

    /// transcript / 10-Q(10-K) 作为同一季度研究对象的追加材料归档。
    /// 这里不调用模型，也不修改研究账本状态；默认实现保持测试替身兼容。
    async fn record_material(
        &self,
        _actor: &ActorIdentity,
        _event: &MarketEvent,
    ) -> Option<EarningsResearchMaterialOutcome> {
        None
    }

    async fn reconcile(
        &self,
        actor: &ActorIdentity,
        event: &MarketEvent,
    ) -> Option<EarningsContinuityOutcome>;
}

pub struct LlmEarningsContinuityReconciler {
    provider: Arc<dyn LlmProvider>,
    model: String,
    storage: Arc<CompanyProfileStorage>,
    inflight: Mutex<HashSet<String>>,
}

impl LlmEarningsContinuityReconciler {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        model: impl Into<String>,
        storage: CompanyProfileStorage,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            storage: Arc::new(storage),
            inflight: Mutex::new(HashSet::new()),
        }
    }

    async fn load_profile(
        &self,
        actor: &ActorIdentity,
        event: &MarketEvent,
    ) -> Option<CompanyProfileDocument> {
        let symbol = event.symbols.first()?.trim();
        if symbol.is_empty() {
            return None;
        }
        let actor_storage = self.storage.for_actor(actor);
        let profile_id = actor_storage.find_profile_id(None, Some(symbol)).await?;
        let profile = actor_storage
            .get_profile(&profile_id)
            .await
            .ok()
            .flatten()?;
        profile.metadata.tracking.enabled.then_some(profile)
    }

    async fn load_target_profile(
        &self,
        actor: &ActorIdentity,
        event: &MarketEvent,
    ) -> Option<CompanyProfileDocument> {
        if continuity_review_stage(event).is_none() {
            return None;
        }
        let profile = self.load_profile(actor, event).await?;
        (profile.metadata.tracking.enabled
            && matches!(profile.metadata.tracking.coverage_tier, CoverageTier::A))
        .then_some(profile)
    }
}

#[async_trait]
impl EarningsContinuityReconciler for LlmEarningsContinuityReconciler {
    async fn should_schedule(&self, actor: &ActorIdentity, event: &MarketEvent) -> bool {
        self.load_target_profile(actor, event).await.is_some()
    }

    async fn record_material(
        &self,
        actor: &ActorIdentity,
        event: &MarketEvent,
    ) -> Option<EarningsResearchMaterialOutcome> {
        let material_kind = earnings_research_material_kind(event)?;
        if material_kind == "earnings_release" {
            return None;
        }
        let research_object_key = earnings_research_object_key_for_event(event)?;
        let profile = self.load_profile(actor, event).await?;
        if matches!(profile.metadata.tracking.coverage_tier, CoverageTier::C) {
            return None;
        }
        let material_label = match material_kind {
            "earnings_call_transcript" => "财报电话会纪要",
            "formal_filing" => "正式季报",
            _ => "财报补充材料",
        };
        let symbol = event.symbols.first().cloned().unwrap_or_default();
        let follow_up = match material_kind {
            "earnings_call_transcript" => {
                "核对管理层问答是否回答未决问题，并区分正式承诺与一般性表述。"
            }
            "formal_filing" => "核对会计口径、现金流、分部披露与新闻稿是否一致。",
            _ => "与本季已核验财报事实交叉核对。",
        };
        let actor_storage = self.storage.for_actor(actor);
        let stored = actor_storage
            .append_research_event(
                &profile.profile_id,
                AppendResearchEventInput {
                    event: AppendEventInput {
                        title: format!("{symbol} {material_label}归档"),
                        event_type: format!("earnings_material_{material_kind}"),
                        occurred_at: event.occurred_at.to_rfc3339(),
                        mainline_impact: "new_evidence".to_string(),
                        changed_sections: vec!["研究材料".to_string()],
                        refs: event.url.clone().into_iter().collect(),
                        what_happened: if event.summary.trim().is_empty() {
                            event.title.clone()
                        } else {
                            format!("{}\n\n{}", event.title, event.summary)
                        },
                        why_it_matters: format!(
                            "该{material_label}已归入同一季度研究对象，当前状态为待交叉核验；归档本身不代表材料结论已被确认。"
                        ),
                        mainline_effect:
                            "新增证据材料，不自动加强、削弱或改写用户确认的投资主线。".to_string(),
                        evidence: format!("来源：{}；事件：{}", event.source, event.id),
                        research_log:
                            "系统自动归档材料引用；未调用 LLM，未改变问题或承诺状态。".to_string(),
                        follow_up: follow_up.to_string(),
                    },
                    research_object_key: Some(research_object_key.clone()),
                    research_updates: Vec::new(),
                },
            )
            .await
            .map_err(|error| {
                tracing::warn!(
                    actor = %actor_key(actor),
                    event_id = %event.id,
                    material_kind,
                    "earnings research material write failed: {error}"
                );
            })
            .ok()
            .flatten()?;
        Some(EarningsResearchMaterialOutcome {
            profile_id: profile.profile_id,
            research_object_key,
            material_kind: material_kind.to_string(),
            recorded_event_id: stored.id,
        })
    }

    async fn reconcile(
        &self,
        actor: &ActorIdentity,
        event: &MarketEvent,
    ) -> Option<EarningsContinuityOutcome> {
        let profile = self.load_target_profile(actor, event).await?;
        let research_object_key = earnings_research_object_key(event);
        let stage = continuity_review_stage(event)?;
        if let Some(existing) = existing_outcome(&profile, &research_object_key, stage) {
            return Some(existing);
        }

        let inflight_key = format!(
            "{}::{}::{}::{}::{}",
            actor.channel,
            actor.channel_scope.clone().unwrap_or_default(),
            actor.user_id,
            research_object_key,
            stage
        );
        {
            let mut inflight = self.inflight.lock().ok()?;
            if !inflight.insert(inflight_key.clone()) {
                return None;
            }
        }

        let outcome = self
            .reconcile_claimed(actor, event, profile, research_object_key)
            .await;
        if let Ok(mut inflight) = self.inflight.lock() {
            inflight.remove(&inflight_key);
        }
        outcome
    }
}

impl LlmEarningsContinuityReconciler {
    async fn reconcile_claimed(
        &self,
        actor: &ActorIdentity,
        event: &MarketEvent,
        profile: CompanyProfileDocument,
        research_object_key: String,
    ) -> Option<EarningsContinuityOutcome> {
        let ledger = profile.research_ledger();
        let active_items = ledger
            .items
            .iter()
            .filter(|item| item.status.is_active())
            .take(14)
            .cloned()
            .collect::<Vec<_>>();
        let messages = build_continuity_messages(&profile, &active_items, event);
        let response = match self.provider.chat(&messages, Some(&self.model)).await {
            Ok(response) => response,
            Err(error) => {
                tracing::warn!(
                    actor = %actor_key(actor),
                    event_id = %event.id,
                    model = %self.model,
                    degraded = true,
                    "earnings continuity review failed: {error}"
                );
                return None;
            }
        };
        let review = match parse_continuity_review(&response.content) {
            Some(review) => review,
            None => {
                tracing::warn!(
                    actor = %actor_key(actor),
                    event_id = %event.id,
                    model = %self.model,
                    content_prefix = %response.content.chars().take(160).collect::<String>(),
                    degraded = true,
                    "earnings continuity review returned invalid JSON"
                );
                return None;
            }
        };
        let (
            thesis_effect,
            thesis_reason,
            next_actions,
            updates,
            created_questions,
            created_commitments,
        ) = normalize_review(&profile, &active_items, review)?;

        let actor_storage = self.storage.for_actor(actor);
        let stored = actor_storage
            .append_research_event(
                &profile.profile_id,
                AppendResearchEventInput {
                    event: build_profile_event(
                        event,
                        &thesis_effect,
                        &thesis_reason,
                        &next_actions,
                        &updates,
                        &self.model,
                    ),
                    research_object_key: Some(research_object_key.clone()),
                    research_updates: updates.clone(),
                },
            )
            .await
            .map_err(|error| {
                tracing::warn!(
                    actor = %actor_key(actor),
                    event_id = %event.id,
                    degraded = true,
                    "earnings continuity ledger write failed: {error}"
                );
            })
            .ok()
            .flatten()?;
        let refreshed = actor_storage
            .get_profile(&profile.profile_id)
            .await
            .ok()
            .flatten()?;
        let refreshed_ledger = refreshed.research_ledger();
        Some(EarningsContinuityOutcome {
            profile_id: profile.profile_id,
            research_object_key,
            thesis_effect,
            recorded_event_id: stored.id,
            checked_existing_items: active_items.len(),
            created_questions,
            created_commitments,
            active_questions_after: refreshed_ledger.active_questions().count(),
            active_commitments_after: refreshed_ledger.active_commitments().count(),
        })
    }
}

fn build_continuity_messages(
    profile: &CompanyProfileDocument,
    active_items: &[ResearchLedgerItem],
    event: &MarketEvent,
) -> Vec<Message> {
    let sections = [
        "投资主线",
        "关键经营指标",
        "预期基线",
        "风险台账",
        "未决问题",
        "管理层承诺台账",
    ]
    .into_iter()
    .filter_map(|title| {
        profile
            .section(title)
            .map(|content| format!("## {title}\n{}", truncate_chars(&content, 1_500)))
    })
    .collect::<Vec<_>>()
    .join("\n\n");
    // 只把做本季判断所需的当前状态交给模型。历史证据仍完整保存在 append-only
    // ledger；把每季累积的全部 evidence 再塞回 prompt 会导致输入和输出同时膨胀。
    let existing_items = active_items
        .iter()
        .map(|item| {
            serde_json::json!({
                "item_id": item.item_id,
                "kind": item.kind,
                "statement": truncate_chars(&item.statement, 360),
                "status": item.status,
                "due_at": item.due_at,
                "latest_assessment": truncate_chars(&item.latest_assessment, 160),
            })
        })
        .collect::<Vec<_>>();
    let existing_items =
        serde_json::to_string(&existing_items).unwrap_or_else(|_| "[]".to_string());
    let active_questions = active_items
        .iter()
        .filter(|item| item.kind == ResearchItemKind::OpenQuestion)
        .count();
    let active_commitments = active_items
        .iter()
        .filter(|item| item.kind == ResearchItemKind::ManagementCommitment)
        .count();
    let new_questions_limit = 8usize.saturating_sub(active_questions).min(2);
    let new_commitments_limit = 6usize.saturating_sub(active_commitments).min(2);
    let stage = continuity_review_stage(event).unwrap_or("earnings_release");
    let (material_label, review_label, review) = if stage == "earnings_transcript" {
        (
            "verified earnings-call transcript review",
            "Transcript review JSON",
            event
                .payload
                .get("earnings_transcript_review")
                .cloned()
                .unwrap_or(Value::Null),
        )
    } else {
        (
            "verified earnings event",
            "Quality review JSON",
            event
                .payload
                .get("earnings_quality_review")
                .cloned()
                .unwrap_or(Value::Null),
        )
    };
    let user = format!(
        "Company: {} ({})\nCoverage tier: {}\nReview stage: {}\nnew_questions_limit: {}\nnew_commitments_limit: {}\n\nSaved profile sections:\n{}\n\nexisting_items:\n{}\n\nCurrent {}:\nTitle: {}\nOccurred at: {}\nSummary:\n{}\n{}:\n{}\nSource URL: {}",
        profile.metadata.company_name,
        profile.metadata.stock_code,
        profile.metadata.tracking.coverage_tier.as_str(),
        stage,
        new_questions_limit,
        new_commitments_limit,
        sections,
        existing_items,
        material_label,
        event.title,
        event.occurred_at.to_rfc3339(),
        event.summary,
        review_label,
        review,
        event.url.as_deref().unwrap_or("unavailable"),
    );
    vec![
        Message {
            role: "system".to_string(),
            content: Some(DEFAULT_EARNINGS_CONTINUITY_SYSTEM_PROMPT.to_string()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".to_string(),
            content: Some(user),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

fn normalize_review(
    profile: &CompanyProfileDocument,
    active_items: &[ResearchLedgerItem],
    review: EarningsContinuityReview,
) -> Option<(
    String,
    String,
    Vec<String>,
    Vec<ResearchLedgerUpdate>,
    usize,
    usize,
)> {
    let thesis_has_baseline = profile
        .section("投资主线")
        .map(|value| !value.trim().is_empty() && !value.trim_start().starts_with("待补充"))
        .unwrap_or(false);
    let thesis_effect = if thesis_has_baseline {
        match review.thesis_effect.trim() {
            "strengthen" | "unchanged" | "watch" | "weaken" => {
                review.thesis_effect.trim().to_string()
            }
            "insufficient_baseline" => return None,
            _ => return None,
        }
    } else {
        "insufficient_baseline".to_string()
    };

    let active_by_id = active_items
        .iter()
        .map(|item| (item.item_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let proposed = review
        .existing_item_updates
        .iter()
        .filter_map(|update| {
            active_by_id
                .contains_key(update.item_id.trim())
                .then_some((update.item_id.trim(), update))
        })
        .collect::<HashMap<_, _>>();
    let mut updates = Vec::new();
    for item in active_items {
        let (status, assessment, evidence) = match proposed.get(item.item_id.as_str()) {
            Some(update) => {
                let evidence = clean_strings(&update.evidence, 1, 240);
                (
                    validated_existing_status(item, update, !evidence.is_empty()),
                    truncate_chars(update.assessment_zh.trim(), 240),
                    evidence,
                )
            }
            None => (
                item.status.clone(),
                "本季材料未提供足以改变状态的直接证据。".to_string(),
                Vec::new(),
            ),
        };
        updates.push(ResearchLedgerUpdate {
            item_id: item.item_id.clone(),
            kind: item.kind.clone(),
            statement: String::new(),
            status,
            assessment,
            due_at: None,
            evidence,
        });
    }

    let mut known_ids = profile
        .research_ledger()
        .items
        .into_iter()
        .map(|item| item.item_id)
        .collect::<HashSet<_>>();
    let current_active_questions = active_items
        .iter()
        .filter(|item| item.kind == ResearchItemKind::OpenQuestion)
        .count();
    let question_capacity = 8usize.saturating_sub(current_active_questions).min(2);
    let mut created_questions = 0;
    for item in review.new_questions.iter().take(question_capacity) {
        if let Some(update) = new_update(ResearchItemKind::OpenQuestion, item, &mut known_ids) {
            updates.push(update);
            created_questions += 1;
        }
    }
    let current_active_commitments = active_items
        .iter()
        .filter(|item| item.kind == ResearchItemKind::ManagementCommitment)
        .count();
    let commitment_capacity = 6usize.saturating_sub(current_active_commitments).min(2);
    let mut created_commitments = 0;
    for item in review.new_commitments.iter().take(commitment_capacity) {
        if let Some(update) =
            new_update(ResearchItemKind::ManagementCommitment, item, &mut known_ids)
        {
            updates.push(update);
            created_commitments += 1;
        }
    }

    Some((
        thesis_effect,
        truncate_chars(review.thesis_reason_zh.trim(), 1_000),
        clean_strings(&review.next_actions, 3, 600),
        updates,
        created_questions,
        created_commitments,
    ))
}

fn validated_existing_status(
    item: &ResearchLedgerItem,
    update: &ExistingResearchItemUpdate,
    has_evidence: bool,
) -> ResearchItemStatus {
    let proposed = parse_research_status(&update.status).unwrap_or_else(|| item.status.clone());
    let basis = update.resolution_basis.trim().to_ascii_lowercase();
    if proposed == ResearchItemStatus::Open {
        return item.status.clone();
    }
    if !has_evidence {
        return item.status.clone();
    }
    match item.kind {
        ResearchItemKind::OpenQuestion => match (proposed, basis.as_str()) {
            (ResearchItemStatus::PartiallyAnswered, "partial_answer") => {
                ResearchItemStatus::PartiallyAnswered
            }
            (ResearchItemStatus::Answered, "answered") => ResearchItemStatus::Answered,
            (ResearchItemStatus::Expired, "superseded") => ResearchItemStatus::Expired,
            _ => item.status.clone(),
        },
        ResearchItemKind::ManagementCommitment => match (proposed, basis.as_str()) {
            (ResearchItemStatus::PartiallyAnswered, "partially_fulfilled") => {
                ResearchItemStatus::PartiallyAnswered
            }
            (ResearchItemStatus::Confirmed, "fulfilled") => ResearchItemStatus::Confirmed,
            (ResearchItemStatus::Contradicted, "missed_or_withdrawn") => {
                ResearchItemStatus::Contradicted
            }
            (ResearchItemStatus::Expired, "superseded") => ResearchItemStatus::Expired,
            _ => item.status.clone(),
        },
    }
}

fn new_update(
    kind: ResearchItemKind,
    item: &NewResearchItem,
    known_ids: &mut HashSet<String>,
) -> Option<ResearchLedgerUpdate> {
    let statement = truncate_chars(item.statement.trim(), 900);
    if statement.is_empty() {
        return None;
    }
    let item_id = research_item_id(&kind, &statement);
    if !known_ids.insert(item_id.clone()) {
        return None;
    }
    Some(ResearchLedgerUpdate {
        item_id,
        kind,
        statement,
        status: ResearchItemStatus::Open,
        assessment: truncate_chars(item.reason_zh.trim(), 1_000),
        due_at: (!item.due_at.trim().is_empty()).then(|| truncate_chars(item.due_at.trim(), 120)),
        evidence: Vec::new(),
    })
}

fn build_profile_event(
    event: &MarketEvent,
    thesis_effect: &str,
    thesis_reason: &str,
    next_actions: &[String],
    updates: &[ResearchLedgerUpdate],
    model: &str,
) -> AppendEventInput {
    let stage = continuity_review_stage(event).unwrap_or("earnings_release");
    let (event_type, title_label, evidence_pointer, source_label) =
        if stage == "earnings_transcript" {
            (
                "earnings_transcript_reconciliation",
                "电话会连续性复核",
                "/earnings_transcript_review/prepared_findings",
                "结构化电话会卡",
            )
        } else {
            (
                "earnings_reconciliation",
                "财报连续性复核",
                "/earnings_quality_review/evidence",
                "结构化财报卡",
            )
        };
    let evidence = event
        .payload
        .pointer(evidence_pointer)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    value.as_str().map(str::to_string).or_else(|| {
                        value
                            .get("finding_zh")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                })
                .take(3)
                .map(|value| format!("- {}", value.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let mut follow_up_items = next_actions
        .iter()
        .map(|action| format!("- {action}"))
        .collect::<Vec<_>>();
    follow_up_items.extend(
        updates
            .iter()
            .filter(|update| update.status.is_active())
            .take(6)
            .map(|update| {
                if update.statement.trim().is_empty() {
                    format!("- `{}`：{}", update.item_id, update.assessment)
                } else {
                    format!("- {}", update.statement)
                }
            })
            .collect::<Vec<_>>(),
    );
    let follow_up = follow_up_items.join("\n");
    AppendEventInput {
        title: format!(
            "{} {}",
            event.symbols.first().cloned().unwrap_or_default(),
            title_label
        ),
        event_type: event_type.to_string(),
        occurred_at: event.occurred_at.to_rfc3339(),
        mainline_impact: thesis_effect.to_string(),
        changed_sections: vec![
            "投资主线".to_string(),
            "未决问题".to_string(),
            "管理层承诺台账".to_string(),
        ],
        refs: event.url.clone().into_iter().collect(),
        what_happened: event.summary.clone(),
        why_it_matters: if thesis_reason.trim().is_empty() {
            format!("本轮只提出主线影响建议 `{thesis_effect}`，不自动修改用户确认的投资主线。")
        } else {
            format!(
                "{}\n\n本轮只提出主线影响建议 `{thesis_effect}`，不自动修改用户确认的投资主线。",
                thesis_reason.trim()
            )
        },
        mainline_effect: format!("建议状态：{thesis_effect}；等待用户在决策记录中确认。"),
        evidence,
        research_log: format!(
            "后台连续性复核模型：{model}；仅使用本次{source_label}和当前 actor 画像。"
        ),
        follow_up,
    }
}

fn existing_outcome(
    profile: &CompanyProfileDocument,
    research_object_key: &str,
    stage: &str,
) -> Option<EarningsContinuityOutcome> {
    let event_type = if stage == "earnings_transcript" {
        "earnings_transcript_reconciliation"
    } else {
        "earnings_reconciliation"
    };
    let event = profile.events.iter().find(|event| {
        event.metadata.event_type == event_type
            && event.metadata.research_object_key.as_deref() == Some(research_object_key)
    })?;
    let ledger = profile.research_ledger();
    Some(EarningsContinuityOutcome {
        profile_id: profile.profile_id.clone(),
        research_object_key: research_object_key.to_string(),
        thesis_effect: event.metadata.mainline_impact.clone(),
        recorded_event_id: event.id.clone(),
        checked_existing_items: event
            .metadata
            .research_updates
            .iter()
            .filter(|update| update.statement.trim().is_empty())
            .count(),
        created_questions: event
            .metadata
            .research_updates
            .iter()
            .filter(|update| {
                update.kind == ResearchItemKind::OpenQuestion && !update.statement.trim().is_empty()
            })
            .count(),
        created_commitments: event
            .metadata
            .research_updates
            .iter()
            .filter(|update| {
                update.kind == ResearchItemKind::ManagementCommitment
                    && !update.statement.trim().is_empty()
            })
            .count(),
        active_questions_after: ledger.active_questions().count(),
        active_commitments_after: ledger.active_commitments().count(),
    })
}

fn earnings_research_object_key(event: &MarketEvent) -> String {
    earnings_research_object_key_for_event(event).unwrap_or_else(|| event.id.clone())
}

fn parse_continuity_review(content: &str) -> Option<EarningsContinuityReview> {
    let trimmed = content.trim();
    let candidate = if trimmed.starts_with("```") {
        trimmed
            .lines()
            .skip(1)
            .take_while(|line| !line.trim_start().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        trimmed[start..=end].to_string()
    } else {
        trimmed.to_string()
    };
    serde_json::from_str(&candidate).ok()
}

fn parse_research_status(value: &str) -> Option<ResearchItemStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "open" | "still_open" | "unanswered" => Some(ResearchItemStatus::Open),
        "partially_answered" | "partial" | "partially_confirmed" => {
            Some(ResearchItemStatus::PartiallyAnswered)
        }
        "answered" | "resolved" | "fully_answered" => Some(ResearchItemStatus::Answered),
        "confirmed" | "fulfilled" => Some(ResearchItemStatus::Confirmed),
        "contradicted" | "not_met" | "broken" => Some(ResearchItemStatus::Contradicted),
        "expired" | "no_longer_applicable" => Some(ResearchItemStatus::Expired),
        _ => None,
    }
}

fn clean_strings(values: &[String], max_items: usize, max_chars: usize) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .take(max_items)
        .map(|value| truncate_chars(value, max_chars))
        .collect()
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let head = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

pub(crate) fn continuity_review_stage(event: &MarketEvent) -> Option<&'static str> {
    match event.kind {
        EventKind::EarningsReleased
            if event
                .payload
                .get("earnings_quality_review_applied")
                .and_then(Value::as_bool)
                == Some(true) =>
        {
            Some("earnings_release")
        }
        EventKind::EarningsCallTranscript
            if event
                .payload
                .get("earnings_transcript_review_applied")
                .and_then(Value::as_bool)
                == Some(true) =>
        {
            Some("earnings_transcript")
        }
        _ => None,
    }
}

fn actor_key(actor: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        actor.channel,
        actor.channel_scope.clone().unwrap_or_default(),
        actor.user_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashSet};

    use chrono::{NaiveDate, TimeZone, Utc};
    use futures::stream::{self, BoxStream};
    use hone_core::{HoneError, HoneResult};
    use hone_llm::ChatResponse;
    use hone_llm::provider::ChatResult;
    use hone_memory::{CreateProfileInput, IndustryTemplate, TrackingConfig};
    use serde_json::json;
    use tempfile::tempdir;

    struct StaticProvider {
        response: String,
        calls: Mutex<usize>,
    }

    #[async_trait]
    impl LlmProvider for StaticProvider {
        async fn chat(&self, _: &[Message], _: Option<&str>) -> HoneResult<ChatResult> {
            *self.calls.lock().unwrap() += 1;
            Ok(ChatResult {
                content: self.response.clone(),
                usage: None,
            })
        }

        async fn chat_with_tools(
            &self,
            _: &[Message],
            _: &[Value],
            _: Option<&str>,
        ) -> HoneResult<ChatResponse> {
            Err(HoneError::Llm("not used".to_string()))
        }

        fn chat_stream<'a>(
            &'a self,
            _: &'a [Message],
            _: Option<&'a str>,
        ) -> BoxStream<'a, HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    fn actor() -> ActorIdentity {
        ActorIdentity::new("discord", "pro", None::<&str>).unwrap()
    }

    #[tokio::test]
    async fn institutional_continuity_fixture_covers_six_archetypes_and_four_quarters() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/event_engine/earnings_continuity_baseline_2026-08-06.json"
        ))
        .expect("valid fixture JSON");
        assert_eq!(fixture["version"], "2026-08-06");
        let companies = fixture["companies"].as_array().expect("companies array");
        assert_eq!(companies.len(), 6);

        let expected_types = HashSet::from([
            "cyclical_semiconductor",
            "profitable_growth",
            "loss_to_profit_growth",
            "loss_growth_saas",
            "mature_cash_flow",
            "cyclical_memory",
        ]);
        let mut actual_types = HashSet::new();
        let mut symbols = HashSet::new();
        let mut urls = HashSet::new();
        let mut total_events = 0;
        for company in companies {
            let symbol = company["symbol"].as_str().expect("symbol");
            assert!(symbols.insert(symbol), "duplicate symbol {symbol}");
            actual_types.insert(company["company_type"].as_str().expect("company type"));
            assert!(!company["thesis"].as_str().unwrap_or_default().is_empty());
            assert!(
                company["focus_metrics"]
                    .as_array()
                    .is_some_and(|items| items.len() >= 3)
            );
            assert_eq!(company["seed_questions"].as_array().map(Vec::len), Some(2));

            let events = company["events"].as_array().expect("events array");
            assert_eq!(events.len(), 4, "{symbol} must cover four quarters");
            let mut previous_date = None;
            for event in events {
                let date = NaiveDate::parse_from_str(
                    event["date"].as_str().expect("event date"),
                    "%Y-%m-%d",
                )
                .expect("ISO event date");
                assert!(previous_date.is_none_or(|previous| date > previous));
                previous_date = Some(date);
                assert!(!event["period"].as_str().unwrap_or_default().is_empty());
                let url = event["url"].as_str().expect("event URL");
                assert!(
                    url.starts_with("https://www.sec.gov/Archives/edgar/data/"),
                    "fixture must use first-party SEC evidence: {url}"
                );
                assert!(urls.insert(url), "duplicate source URL {url}");
                total_events += 1;
            }
        }
        assert_eq!(actual_types, expected_types);
        assert_eq!(total_events, 24);
    }

    fn event() -> MarketEvent {
        MarketEvent {
            id: "earnings:SNDK:2026-q2".to_string(),
            kind: EventKind::EarningsReleased,
            severity: crate::event::Severity::High,
            symbols: vec!["SNDK".to_string()],
            occurred_at: Utc.with_ymd_and_hms(2026, 8, 5, 20, 9, 6).unwrap(),
            title: "数据中心强劲，消费端承压".to_string(),
            summary: "结论：数据中心收入增长；反向项：消费端环比下降。".to_string(),
            url: Some("https://www.sec.gov/sndk-q2.htm".to_string()),
            source: "test".to_string(),
            payload: json!({
                "earnings_quality_review_applied": true,
                "hone_earnings_release_document_key": "sec:sndk:q2",
                "earnings_quality_review": {
                    "evidence": ["数据中心收入增长", "毛利率改善"],
                    "risks": ["消费端下滑"],
                    "follow_ups": ["核验 hyperscaler 采用"]
                }
            }),
        }
    }

    fn ledger_item(kind: ResearchItemKind) -> ResearchLedgerItem {
        ResearchLedgerItem {
            item_id: "item-1".into(),
            kind,
            statement: "在2026年发布新产品".into(),
            status: ResearchItemStatus::Open,
            first_seen_at: "2026-01-01T00:00:00Z".into(),
            last_reviewed_at: "2026-01-01T00:00:00Z".into(),
            due_at: Some("2026".into()),
            latest_assessment: String::new(),
            evidence: vec![],
            latest_event_id: "seed".into(),
            update_count: 1,
        }
    }

    #[tokio::test]
    async fn reaffirmed_commitment_cannot_be_closed_as_confirmed() {
        let item = ledger_item(ResearchItemKind::ManagementCommitment);
        let update = ExistingResearchItemUpdate {
            item_id: item.item_id.clone(),
            status: "confirmed".into(),
            resolution_basis: "reaffirmed".into(),
            assessment_zh: "管理层仍称按计划推进。".into(),
            evidence: vec!["仍计划在2026年发布".into()],
        };
        assert_eq!(
            validated_existing_status(&item, &update, true),
            ResearchItemStatus::Open
        );
    }

    #[tokio::test]
    async fn only_evidenced_fulfillment_closes_management_commitment() {
        let item = ledger_item(ResearchItemKind::ManagementCommitment);
        let mut update = ExistingResearchItemUpdate {
            item_id: item.item_id.clone(),
            status: "confirmed".into(),
            resolution_basis: "fulfilled".into(),
            assessment_zh: "产品已正式发布。".into(),
            evidence: vec!["本季已发布并开始出货".into()],
        };
        assert_eq!(
            validated_existing_status(&item, &update, true),
            ResearchItemStatus::Confirmed
        );
        assert_eq!(
            validated_existing_status(&item, &update, false),
            ResearchItemStatus::Open
        );
        update.status = "answered".into();
        assert_eq!(
            validated_existing_status(&item, &update, true),
            ResearchItemStatus::Open
        );
    }

    #[tokio::test]
    async fn question_and_commitment_resolution_vocabularies_do_not_cross() {
        let item = ledger_item(ResearchItemKind::OpenQuestion);
        let update = ExistingResearchItemUpdate {
            item_id: item.item_id.clone(),
            status: "confirmed".into(),
            resolution_basis: "fulfilled".into(),
            assessment_zh: "不适用的问题状态。".into(),
            evidence: vec!["有证据但状态类型错误".into()],
        };
        assert_eq!(
            validated_existing_status(&item, &update, true),
            ResearchItemStatus::Open
        );
    }

    #[tokio::test]
    async fn open_without_new_resolution_never_erases_partial_progress() {
        let mut item = ledger_item(ResearchItemKind::OpenQuestion);
        item.status = ResearchItemStatus::PartiallyAnswered;
        let update = ExistingResearchItemUpdate {
            item_id: item.item_id.clone(),
            status: "open".into(),
            resolution_basis: "none".into(),
            assessment_zh: "本季没有新增回答。".into(),
            evidence: vec![],
        };
        assert_eq!(
            validated_existing_status(&item, &update, false),
            ResearchItemStatus::PartiallyAnswered
        );
    }

    async fn tracked_profile(storage: &CompanyProfileStorage) -> CompanyProfileDocument {
        let scoped = storage.for_actor(&actor());
        let mut sections = BTreeMap::new();
        sections.insert(
            "投资主线".to_string(),
            "企业级 SSD 客户采用与 NAND 供给纪律共同驱动盈利质量。".to_string(),
        );
        scoped
            .create_profile(CreateProfileInput {
                company_name: "SanDisk".to_string(),
                stock_code: Some("SNDK".to_string()),
                sector: None,
                aliases: vec![],
                industry_template: IndustryTemplate::SemiconductorHardware,
                tracking: Some(TrackingConfig {
                    enabled: true,
                    coverage_tier: CoverageTier::A,
                    ..TrackingConfig::default()
                }),
                initial_sections: sections,
            })
            .await
            .unwrap()
            .0
    }

    #[tokio::test]
    async fn reconciliation_records_auditable_items_and_is_restart_idempotent() {
        let dir = tempdir().unwrap();
        let storage = CompanyProfileStorage::new(dir.path());
        let profile = tracked_profile(&storage).await;
        let provider = Arc::new(StaticProvider {
            response: json!({
                "thesis_effect": "watch",
                "thesis_reason_zh": "企业级需求改善，但消费端仍构成反向证据。",
                "existing_item_updates": [],
                "new_questions": [{
                    "statement": "hyperscaler 采用能否转化为持续收入贡献？",
                    "due_at": "FY27 Q1",
                    "reason_zh": "决定数据中心增长持续性"
                }],
                "new_commitments": [{
                    "statement": "管理层将在下一季度继续维持供给纪律。",
                    "due_at": "next quarter",
                    "reason_zh": "本季明确前瞻表述"
                }],
                "next_actions": ["电话会核验客户集中度"]
            })
            .to_string(),
            calls: Mutex::new(0),
        });
        let reconciler =
            LlmEarningsContinuityReconciler::new(provider.clone(), "x-ai/grok-4.5", storage);

        let first = reconciler
            .reconcile(&actor(), &event())
            .await
            .expect("first outcome");
        assert_eq!(first.profile_id, profile.profile_id);
        assert_eq!(first.created_questions, 1);
        assert_eq!(first.created_commitments, 1);
        assert_eq!(first.active_questions_after, 1);
        assert_eq!(first.active_commitments_after, 1);

        let second = reconciler
            .reconcile(&actor(), &event())
            .await
            .expect("existing outcome");
        assert_eq!(second.recorded_event_id, first.recorded_event_id);
        assert_eq!(*provider.calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn reviewed_transcript_gets_a_distinct_same_quarter_reconciliation() {
        let dir = tempdir().unwrap();
        let storage = CompanyProfileStorage::new(dir.path());
        let profile = tracked_profile(&storage).await;
        let provider = Arc::new(StaticProvider {
            response: json!({
                "thesis_effect": "strengthen",
                "thesis_reason_zh": "电话会直接回答了企业级订单持续性。",
                "existing_item_updates": [],
                "new_questions": [],
                "new_commitments": [],
                "next_actions": ["下季复核订单转化"]
            })
            .to_string(),
            calls: Mutex::new(0),
        });
        let reconciler = LlmEarningsContinuityReconciler::new(
            provider.clone(),
            "x-ai/grok-4.5",
            CompanyProfileStorage::new(dir.path()),
        );

        let release = reconciler
            .reconcile(&actor(), &event())
            .await
            .expect("release reconciliation");
        let mut transcript = event();
        transcript.id = "transcript:SNDK:2026-q2:reviewed".into();
        transcript.kind = EventKind::EarningsCallTranscript;
        transcript.title = "电话会：较此前更有信心".into();
        transcript.summary = "分析师问答：企业订单能见度延伸（直接回答）".into();
        transcript.url = Some("https://ir.example/sndk-q2-transcript.pdf".into());
        transcript.payload = json!({
            "hone_earnings_research_object_key": "sec:sndk:q2",
            "earnings_transcript_review_applied": true,
            "earnings_transcript_review": {
                "source_scope": "prepared_and_qa",
                "management_tone": "more_confident",
                "prepared_findings": [],
                "qa_findings": [{
                    "topic": "订单",
                    "answer_quality": "direct",
                    "answer_zh": "企业订单能见度延伸"
                }]
            }
        });
        assert!(reconciler.should_schedule(&actor(), &transcript).await);
        let transcript_outcome = reconciler
            .reconcile(&actor(), &transcript)
            .await
            .expect("transcript reconciliation");
        assert_ne!(
            transcript_outcome.recorded_event_id,
            release.recorded_event_id
        );
        assert_eq!(*provider.calls.lock().unwrap(), 2);

        let repeated = reconciler
            .reconcile(&actor(), &transcript)
            .await
            .expect("idempotent transcript reconciliation");
        assert_eq!(
            repeated.recorded_event_id,
            transcript_outcome.recorded_event_id
        );
        assert_eq!(*provider.calls.lock().unwrap(), 2);
        let refreshed = storage
            .for_actor(&actor())
            .get_profile(&profile.profile_id)
            .await
            .unwrap()
            .unwrap();
        assert!(refreshed.events.iter().any(|event| {
            event.metadata.event_type == "earnings_transcript_reconciliation"
                && event.metadata.research_object_key.as_deref() == Some("sec:sndk:q2")
        }));
    }

    #[tokio::test]
    async fn transcript_is_appended_to_the_same_quarter_without_spending_model_tokens() {
        let dir = tempdir().unwrap();
        let storage = CompanyProfileStorage::new(dir.path());
        let profile = tracked_profile(&storage).await;
        let provider = Arc::new(StaticProvider {
            response: "not used".to_string(),
            calls: Mutex::new(0),
        });
        let reconciler = LlmEarningsContinuityReconciler::new(
            provider.clone(),
            "x-ai/grok-4.5",
            CompanyProfileStorage::new(dir.path()),
        );
        let mut transcript = event();
        transcript.id = "transcript:SNDK:2026-q2".to_string();
        transcript.kind = EventKind::EarningsCallTranscript;
        transcript.title = "SNDK Q2 earnings call transcript".to_string();
        transcript.summary = "Prepared remarks and Q&A are available.".to_string();
        transcript.url = Some("https://example.com/sndk-q2-transcript".to_string());
        transcript.payload = json!({
            "hone_earnings_research_object_key": "sec:sndk:q2"
        });

        let first = reconciler
            .record_material(&actor(), &transcript)
            .await
            .expect("material recorded");
        let second = reconciler
            .record_material(&actor(), &transcript)
            .await
            .expect("idempotent existing material");
        assert_eq!(first.research_object_key, "sec:sndk:q2");
        assert_eq!(first.recorded_event_id, second.recorded_event_id);
        assert_eq!(*provider.calls.lock().unwrap(), 0);

        let refreshed = storage
            .for_actor(&actor())
            .get_profile(&profile.profile_id)
            .await
            .unwrap()
            .unwrap();
        let material = refreshed
            .events
            .iter()
            .find(|event| event.metadata.event_type == "earnings_material_earnings_call_transcript")
            .expect("transcript event");
        assert_eq!(
            material.metadata.research_object_key.as_deref(),
            Some("sec:sndk:q2")
        );
        assert!(material.markdown.contains("待交叉核验"));
        assert!(material.markdown.contains("不自动加强、削弱或改写"));
    }

    #[tokio::test]
    async fn material_archival_follows_a_b_not_c_coverage_depth() {
        let dir = tempdir().unwrap();
        let storage = CompanyProfileStorage::new(dir.path());
        let profile = tracked_profile(&storage).await;
        let scoped = storage.for_actor(&actor());
        scoped
            .set_tracking(
                &profile.profile_id,
                TrackingConfig {
                    enabled: true,
                    coverage_tier: CoverageTier::B,
                    ..TrackingConfig::default()
                },
            )
            .await
            .unwrap();
        let provider = Arc::new(StaticProvider {
            response: "not used".to_string(),
            calls: Mutex::new(0),
        });
        let reconciler = LlmEarningsContinuityReconciler::new(
            provider.clone(),
            "x-ai/grok-4.5",
            CompanyProfileStorage::new(dir.path()),
        );
        let mut transcript = event();
        transcript.kind = EventKind::EarningsCallTranscript;
        transcript.id = "transcript-b".to_string();
        transcript.payload = json!({
            "hone_earnings_research_object_key": "sec:sndk:q2"
        });
        assert!(
            reconciler
                .record_material(&actor(), &transcript)
                .await
                .is_some()
        );

        scoped
            .set_tracking(
                &profile.profile_id,
                TrackingConfig {
                    enabled: true,
                    coverage_tier: CoverageTier::C,
                    ..TrackingConfig::default()
                },
            )
            .await
            .unwrap();
        transcript.id = "transcript-c".to_string();
        transcript.occurred_at += chrono::Duration::days(1);
        assert!(
            reconciler
                .record_material(&actor(), &transcript)
                .await
                .is_none()
        );
        assert_eq!(*provider.calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn omitted_existing_items_are_carried_forward_as_open() {
        let dir = tempdir().unwrap();
        let storage = CompanyProfileStorage::new(dir.path());
        let mut profile = tracked_profile(&storage).await;
        let item = ResearchLedgerItem {
            item_id: "open_question-old".to_string(),
            kind: ResearchItemKind::OpenQuestion,
            statement: "旧问题是否得到回答？".to_string(),
            status: ResearchItemStatus::Open,
            first_seen_at: "2026-01-01".to_string(),
            last_reviewed_at: "2026-01-01".to_string(),
            due_at: None,
            latest_assessment: String::new(),
            evidence: vec![],
            latest_event_id: "q1".to_string(),
            update_count: 1,
        };
        let review = EarningsContinuityReview {
            thesis_effect: "watch".to_string(),
            thesis_reason_zh: String::new(),
            existing_item_updates: vec![ExistingResearchItemUpdate {
                item_id: "open_question-old".to_string(),
                status: "unexpected_provider_status".to_string(),
                resolution_basis: "none".to_string(),
                assessment_zh: "状态字段异常，但评估内容仍可保留。".to_string(),
                evidence: vec![],
            }],
            new_questions: vec![],
            new_commitments: vec![],
            next_actions: vec![],
        };
        let (_, _, _, updates, _, _) = normalize_review(&profile, &[item], review).unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].status, ResearchItemStatus::Open);
        assert!(updates[0].assessment.contains("状态字段异常"));
        profile.events.clear();
    }
}
