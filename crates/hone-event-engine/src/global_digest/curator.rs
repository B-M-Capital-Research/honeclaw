//! 全局 digest curator —— LLM 两段式精读管道。
//!
//! Pass 1(本文件):批量打分 + cluster 聚类 + 一句话 takeaway。
//!   - prompt 用 POC 验证过的"5 分锚点 + 具体例子"版本,避免两极化
//!   - cluster id 由 LLM 一次性输出(同事件不同媒体合并),不需要算法 dedup
//!   - 输出按 cluster 取分最高,按 score 降序截 top_n
//!
//! Pass 2 baseline / personalize 在同模块的 pass2_baseline / pass2_personalize 里。
//!
//! POC 验证(见 SKILL `poc-driven-feature-design`):
//! - **必须用 grok-4.1-fast 或更强模型**。2026-04-27 复盘 POC 实测 nova-lite-v1 在
//!   42-61 候选量级下塌成 1/2/3 三档,完全没有 4/5;同时 cluster id 给得过细
//!   (Iran/Hormuz/Oil/Gold 5+ 个独立 cluster),thematic dedup 失效。
//!   grok-4.1-fast 在同 prompt 给出健康 5/4/3/2/1 分布,且自动把 11 条 Iran 主题
//!   合到一个 cluster。代价 3× ($0.001 → $0.003/run),绝对值仍 < 1¢。
//! - 174 候选下 cluster dedup 仍准确(174→124,iran-war 一次合 21)
//! - 带 audience brief 后 LLM 自动推 NVDA/Intel/TSM 是 AMD 同行
//! - Pass 1 cost ≈ $0.003 / 60 候选 / grok-4.1-fast

use std::collections::HashMap;
use std::sync::Arc;

use hone_llm::{LlmProvider, Message};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use async_trait::async_trait;

use crate::global_digest::audience::AudienceContext;
use crate::global_digest::collector::GlobalDigestCandidate;
use crate::global_digest::fetcher::ArticleBody;

/// Pass 1 输出项(带原始 candidate 引用)。
#[derive(Debug, Clone)]
pub struct RankedCandidate {
    pub candidate: GlobalDigestCandidate,
    pub pass1_score: u8,        // 1-5
    pub pass1_cluster: String,  // 跨条目去重的 cluster id
    pub pass1_takeaway: String, // 一句话精炼
}

/// Pass 2 baseline 输出项(无 thesis,跨用户共享,落 daily_report 审计用)。
#[derive(Debug, Clone)]
pub struct BaselineCuratedItem {
    pub candidate: GlobalDigestCandidate,
    pub article: ArticleBody,
    pub rank: u32,
    pub comment: String, // ≤80 字
}

/// 单条 personalize 后 pick 的归类。决定渲染时的标识(🎯 / ⚠️ / 🌍)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickCategory {
    /// 印证用户 thesis(rank 通常靠前)
    ThesisAligned,
    /// 反证 thesis(保留以警觉用户)
    ThesisCounter,
    /// 宏观底线 slot —— 即使 thesis 不关心,大盘背景必须保留至少 N 条
    MacroFloor,
}

impl PickCategory {
    fn from_str(s: &str) -> Self {
        match s {
            "thesis_counter" => PickCategory::ThesisCounter,
            "macro_floor" => PickCategory::MacroFloor,
            _ => PickCategory::ThesisAligned,
        }
    }
}

/// thesis 对该 pick 的关系标记(短评里 LLM 用,渲染时也展示)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThesisRelation {
    Aligned,
    Counter,
    Neutral,
    NotApplicable,
}

impl ThesisRelation {
    fn from_str(s: &str) -> Self {
        match s {
            "印证" | "aligned" => ThesisRelation::Aligned,
            "反证" | "counter" => ThesisRelation::Counter,
            "中立" | "neutral" => ThesisRelation::Neutral,
            _ => ThesisRelation::NotApplicable,
        }
    }
}

/// Pass 2 personalize 输出项(每用户独立)。这是真正发到 sink 的内容。
#[derive(Debug, Clone)]
pub struct PersonalizedItem {
    pub candidate: GlobalDigestCandidate,
    pub article: ArticleBody,
    pub rank: u32,
    pub comment: String, // ≤100 字,直接引用 thesis 关键词
    pub category: PickCategory,
    pub thesis_relation: ThesisRelation,
}

/// 用户投资逻辑输入。global_style + per-ticker theses。两个都为空时 personalize
/// 退化成 baseline 行为。
#[derive(Debug, Clone, Default)]
pub struct UserThesis<'a> {
    pub global_style: Option<&'a str>,
    pub theses: Option<&'a HashMap<String, String>>,
}

/// LLM 返回的 Pass 1 单项原始字段(不含 candidate)。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pass1Item {
    idx: usize,
    score: u8,
    cluster: String,
    takeaway: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Pass1Response {
    items: Vec<Pass1Item>,
}

/// LLM 返回的 Pass 2 baseline 单项。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pass2BaselineItem {
    idx: usize,
    rank: u32,
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pass2BaselineResponse {
    picks: Vec<Pass2BaselineItem>,
}

/// LLM 返回的 Pass 2 personalize 单项。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pass2PersonalizeItem {
    idx: usize,
    rank: u32,
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    comment: String,
    /// "thesis_aligned" / "thesis_counter" / "macro_floor"
    #[serde(default)]
    category: String,
    /// "印证" / "反证" / "中立" / "N/A" / 英文同义
    #[serde(default)]
    thesis_relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pass2PersonalizeResponse {
    picks: Vec<Pass2PersonalizeItem>,
    #[serde(default)]
    floor_satisfied: Option<bool>,
}

/// Curator —— 把 LLM provider + 两个模型名包起来,所有 Pass 共享。
pub struct Curator {
    provider: Arc<dyn LlmProvider>,
    pass1_model: String,
    /// Pass 2 模型(baseline 与 personalize 共用)。本文件不直接读它,
    /// 留给同模块的 pass2_baseline / pass2_personalize。
    pub(super) pass2_model: String,
}

impl Curator {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        pass1_model: impl Into<String>,
        pass2_model: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            pass1_model: pass1_model.into(),
            pass2_model: pass2_model.into(),
        }
    }

    /// Pass 1:对全部候选打分 + 聚类,返回按 score 降序、cluster 内只保留最高分的
    /// top_n 列表。LLM 失败 → propagate Err(scheduler 会 warn 跳过本次 run)。
    pub async fn pass1_select(
        &self,
        candidates: &[GlobalDigestCandidate],
        audience: &AudienceContext,
        top_n: usize,
    ) -> anyhow::Result<Vec<RankedCandidate>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let messages = build_pass1_messages(candidates, audience);
        let resp = self
            .provider
            .chat(&messages, Some(&self.pass1_model))
            .await
            .map_err(|e| anyhow::anyhow!("pass1 LLM call failed: {e}"))?;
        let parsed = parse_pass1_response(&resp.content)?;
        Ok(rank_and_dedupe(candidates, parsed.items, top_n))
    }

    /// Pass 2 personalize:每用户独立跑,带其 thesis + macro floor。
    ///
    /// 行为(POC 验证):
    /// - 印证 thesis 优先(即使 Pass 1 中等分)
    /// - 反证保留并标 ThesisCounter,LLM 必须在短评里点出"是否构成实质反证"
    /// - 用户视角噪音(短期估值/技术见顶/单日波动/笼统泡沫论)直接剔除
    /// - **macro_floor**:无论 thesis 怎么过滤,至少留 floor_macro 条 macro_floor 标记
    ///   候选池真没够格的就不强加(metadata `floor_satisfied=false`,只 warn 不错)
    /// - thesis 完全为空 → 退化成 baseline 行为(全部标 ThesisAligned + Neutral)
    pub async fn pass2_personalize(
        &self,
        picks_with_bodies: Vec<(RankedCandidate, ArticleBody)>,
        audience: &AudienceContext,
        thesis: UserThesis<'_>,
        floor_macro: u32,
        final_n: u32,
    ) -> anyhow::Result<Vec<PersonalizedItem>> {
        if picks_with_bodies.is_empty() {
            return Ok(Vec::new());
        }
        let messages = build_pass2_personalize_messages(
            &picks_with_bodies,
            audience,
            &thesis,
            floor_macro,
            final_n,
        );
        let resp = self
            .provider
            .chat(&messages, Some(&self.pass2_model))
            .await
            .map_err(|e| anyhow::anyhow!("pass2 personalize LLM call failed: {e}"))?;
        let parsed = parse_pass2_personalize_response(&resp.content)?;
        let macro_count = parsed
            .picks
            .iter()
            .filter(|p| PickCategory::from_str(&p.category) == PickCategory::MacroFloor)
            .count();
        if floor_macro > 0
            && macro_count < floor_macro as usize
            && parsed.floor_satisfied != Some(false)
        {
            // LLM 没声明 floor_satisfied=false 又没出够 macro_floor —— warn 一下,
            // 但不强行修补(信任 LLM 候选池真没硬料的判断)。
            tracing::warn!(
                requested_floor = floor_macro,
                actual_macro = macro_count,
                "pass2 personalize: macro_floor 数量不足且 LLM 未声明 floor_satisfied=false"
            );
        }
        Ok(map_pass2_personalize(picks_with_bodies, parsed.picks))
    }

    /// Pass 2 baseline:用强模型看全文重排,无用户 thesis。**输出落 daily_report 审计**,
    /// 不直接发给用户。所有用户共享同一份(节省成本)。
    pub async fn pass2_baseline(
        &self,
        picks_with_bodies: Vec<(RankedCandidate, ArticleBody)>,
        audience: &AudienceContext,
        final_n: u32,
    ) -> anyhow::Result<Vec<BaselineCuratedItem>> {
        if picks_with_bodies.is_empty() {
            return Ok(Vec::new());
        }
        let messages = build_pass2_baseline_messages(&picks_with_bodies, audience, final_n);
        let resp = self
            .provider
            .chat(&messages, Some(&self.pass2_model))
            .await
            .map_err(|e| anyhow::anyhow!("pass2 baseline LLM call failed: {e}"))?;
        let parsed = parse_pass2_baseline_response(&resp.content)?;
        Ok(map_pass2_baseline(picks_with_bodies, parsed.picks))
    }
}

fn build_pass2_baseline_messages(
    picks_with_bodies: &[(RankedCandidate, ArticleBody)],
    audience: &AudienceContext,
    final_n: u32,
) -> Vec<Message> {
    let n = picks_with_bodies.len();
    let system = format!(
        "你是金融新闻精读助手。Pass 1 已经初筛出 {n} 篇候选,你看到了原文(部分付费墙抓不到,fallback 到 RSS / FMP 摘要)。

请:
1. 重新评估每条是否真值得放进\"今日全球要闻\",按全球投资者关注度从高到低排序
2. 为每条最终入选的写 ≤80 字短评:这件事意味着什么、对受众持仓有何潜在影响
3. 至多保留 {final_n} 条;Pass1 高分但原文显示 retrospective/explainer/讣告/无新增信息可剔除

判断标准:优先持仓相关、其次大盘宏观、再次行业级硬事件。

严格输出 JSON,无其它字符:
{{\"picks\":[{{\"idx\":0,\"rank\":1,\"title\":\"原标题\",\"url\":\"原URL\",\"comment\":\"≤80字\"}}]}}
按 rank 1..N 排序。"
    );

    let briefs_block = render_briefs_block(audience);
    let cand_block: String = picks_with_bodies
        .iter()
        .enumerate()
        .map(|(i, (rc, body))| {
            let symbols = if rc.candidate.event.symbols.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", rc.candidate.event.symbols.join(","))
            };
            let body_preview: String = body.text.chars().take(5000).collect();
            format!(
                "=== [{i}] {} ===\nsource: {} | symbols: {symbols}\nPass1 score: {} | cluster: {}\nURL: {}\n原文({:?}, {}c):\n{body_preview}",
                rc.candidate.event.title,
                rc.candidate.event.source,
                rc.pass1_score,
                rc.pass1_cluster,
                rc.candidate.event.url.as_deref().unwrap_or(""),
                body.source,
                body.text.chars().count(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let user = format!("## 受众持仓概览\n{briefs_block}\n\n## 候选全文\n{cand_block}");

    vec![
        Message {
            role: "system".into(),
            content: Some(system),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".into(),
            content: Some(user),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

fn parse_pass2_baseline_response(content: &str) -> anyhow::Result<Pass2BaselineResponse> {
    let cleaned = strip_json_fence(content);
    serde_json::from_str(&cleaned).map_err(|e| {
        anyhow::anyhow!(
            "pass2 baseline JSON parse: {e}; raw_prefix={}",
            &cleaned.chars().take(200).collect::<String>()
        )
    })
}

fn map_pass2_baseline(
    picks_with_bodies: Vec<(RankedCandidate, ArticleBody)>,
    items: Vec<Pass2BaselineItem>,
) -> Vec<BaselineCuratedItem> {
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        if it.idx >= picks_with_bodies.len() {
            continue;
        }
        let (rc, body) = picks_with_bodies[it.idx].clone();
        out.push(BaselineCuratedItem {
            candidate: rc.candidate,
            article: body,
            rank: it.rank,
            comment: it.comment,
        });
    }
    out.sort_by_key(|x| x.rank);
    out
}

fn build_pass2_personalize_messages(
    picks_with_bodies: &[(RankedCandidate, ArticleBody)],
    audience: &AudienceContext,
    thesis: &UserThesis<'_>,
    floor_macro: u32,
    final_n: u32,
) -> Vec<Message> {
    let n = picks_with_bodies.len();
    let system = format!(
        "你是金融新闻精读助手。Pass 1 已经初筛出 {n} 篇候选,你看到了原文(部分付费墙抓不到)。

**关键 1**:用户有明确投资风格和个股逻辑(下方\"用户投资逻辑\"段)。请按用户视角:
- 印证用户叙事的事件优先(即使 Pass1 中等分),标 category=\"thesis_aligned\"
- 反证事件保留(用户需知道叙事是否被证伪),标 category=\"thesis_counter\",短评必须点出\"是否构成实质反证\"
- 用户视角下的噪音(短期估值警告 / 技术见顶 / 单日涨跌评论 / 笼统泡沫论)直接剔除

**关键 2:宏观底线 (floor)**:无论用户 thesis 怎么过滤,至少保留 **{floor_macro} 条**宏观/地缘/油价/联储/政策/重大监管类硬料,标 category=\"macro_floor\"。
- 这类事件不是噪音 —— 是大盘背景,可能波及所有持仓
- 例:Hormuz 海峡危机、Iran-US 摩擦、Fed 政策、油价急涨急跌、关税/制裁立法、CPI/就业意外
- 候选池里**根本没有**够格的宏观硬料(都是公司新闻),可以不强加,但要在输出 metadata 里 `floor_satisfied=false`

短评 ≤100 字,直接引用用户叙事关键词;宏观条目用\"对持仓的潜在波及\"角度写。至多 {final_n} 条。

严格输出 JSON,无其它字符:
{{
  \"picks\":[{{\"idx\":0,\"rank\":1,\"title\":\"原标题\",\"url\":\"原URL\",\"comment\":\"≤100字\",\"category\":\"thesis_aligned/thesis_counter/macro_floor\",\"thesis_relation\":\"印证/中立/反证/N/A\"}}],
  \"floor_satisfied\": true
}}"
    );

    let briefs_block = render_briefs_block(audience);
    let cand_block: String = picks_with_bodies
        .iter()
        .enumerate()
        .map(|(i, (rc, body))| {
            let symbols = if rc.candidate.event.symbols.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", rc.candidate.event.symbols.join(","))
            };
            let body_preview: String = body.text.chars().take(5000).collect();
            format!(
                "=== [{i}] {} ===\nsource: {} | symbols: {symbols}\nPass1 score: {} | cluster: {}\nURL: {}\n原文({:?}, {}c):\n{body_preview}",
                rc.candidate.event.title,
                rc.candidate.event.source,
                rc.pass1_score,
                rc.pass1_cluster,
                rc.candidate.event.url.as_deref().unwrap_or(""),
                body.source,
                body.text.chars().count(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let thesis_block = render_thesis_block(thesis);

    let user = format!(
        "## 受众持仓概览\n{briefs_block}\n\n## 用户投资逻辑\n{thesis_block}\n\n## 候选全文\n{cand_block}"
    );

    vec![
        Message {
            role: "system".into(),
            content: Some(system),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".into(),
            content: Some(user),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

fn render_thesis_block(thesis: &UserThesis<'_>) -> String {
    let mut lines = Vec::new();
    if let Some(style) = thesis.global_style {
        lines.push(format!("### 全局风格\n{style}"));
    }
    if let Some(theses) = thesis.theses {
        if !theses.is_empty() {
            lines.push("### 个股投资逻辑".to_string());
            // 按 ticker 排序保证 prompt 稳定
            let mut entries: Vec<_> = theses.iter().collect();
            entries.sort_by_key(|(k, _)| k.as_str());
            for (sym, txt) in entries {
                lines.push(format!("- **{sym}**:{txt}"));
            }
        }
    }
    if lines.is_empty() {
        // 完全无 thesis → 给 LLM 一个明确指示退化到 baseline 风格
        return "(用户未配置 thesis,按 baseline 排序即可。category 全部标 \"thesis_aligned\",thesis_relation 标 \"中立\"。)".to_string();
    }
    lines.join("\n\n")
}

fn parse_pass2_personalize_response(content: &str) -> anyhow::Result<Pass2PersonalizeResponse> {
    let cleaned = strip_json_fence(content);
    serde_json::from_str(&cleaned).map_err(|e| {
        anyhow::anyhow!(
            "pass2 personalize JSON parse: {e}; raw_prefix={}",
            &cleaned.chars().take(200).collect::<String>()
        )
    })
}

fn map_pass2_personalize(
    picks_with_bodies: Vec<(RankedCandidate, ArticleBody)>,
    items: Vec<Pass2PersonalizeItem>,
) -> Vec<PersonalizedItem> {
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        if it.idx >= picks_with_bodies.len() {
            continue;
        }
        let (rc, body) = picks_with_bodies[it.idx].clone();
        out.push(PersonalizedItem {
            candidate: rc.candidate,
            article: body,
            rank: it.rank,
            comment: it.comment,
            category: PickCategory::from_str(&it.category),
            thesis_relation: ThesisRelation::from_str(&it.thesis_relation),
        });
    }
    out.sort_by_key(|x| x.rank);
    out
}

/// 渲染 audience briefs 段(给 Pass 1/2 prompt 共用)。
fn render_briefs_block(audience: &AudienceContext) -> String {
    audience
        .briefs
        .iter()
        .map(|b| {
            let mut line = format!(
                "- {} — {} ({}{}{})\n  业务: {}",
                b.ticker,
                b.name,
                b.sector,
                if b.sector.is_empty() || b.industry.is_empty() {
                    ""
                } else {
                    " / "
                },
                b.industry,
                b.one_liner,
            );
            if !b.user_notes.is_empty() {
                line.push_str("\n  用户备注: ");
                line.push_str(&b.user_notes.join(" | "));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 构造 Pass 1 messages。POC 验证的 prompt(中文 system + audience brief + 候选块)。
fn build_pass1_messages(
    candidates: &[GlobalDigestCandidate],
    audience: &AudienceContext,
) -> Vec<Message> {
    let system = "你是金融新闻精炼助手。把候选打分,要充分利用中间档,不要把所有非持仓相关的都丢到 1 分。

打分锚点(每档都要有真实事件参照,避免两极化):
5 分 — 直接关于受众持仓里的公司,或它们紧密同行/上下游(基于持仓业务方向判断同行)
       例:CAI brief 是 \"AI 分子诊断\" → DHR/EXAS/GH/NTRA 等诊断同行打 5
4 分 — 大盘宏观或全美经济指标:联储/CPI/就业/油价大波动/地缘冲突直接命中美股/重大监管立法
       例:Fed 加息、ISM 制造业 PMI、Iran-US 摩擦升级
3 分 — 行业/全球商业级重要事件,即使不命中持仓:
       a) 全球 top 50 科技/能源/医药/汽车公司的硬事件(高管变动/重大并购/破产/重大召回/SEC 处罚)
       b) 大型跨国 M&A(交易额 > $10B 或被多家主流媒体报道)
       c) 全美级别监管/政策事件
2 分 — 中型企业事件、海外区域事件、explainer/CEO 评论
1 分 — 杂讯:列表/promo/估值评论/澄清文/已了结诉讼回顾/单日涨跌/讣告

cluster id 用英文短词,同事件不同媒体一定要合并(merger/recall/lawsuit 等大事件常被多家媒体报)。

严格输出 JSON,无其它字符:
{\"items\":[{\"idx\":0,\"score\":4,\"cluster\":\"openai-musk-trial\",\"takeaway\":\"中文一句话\"}]}";

    let briefs_block = render_briefs_block(audience);

    let cand_block: String = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let symbols = if c.event.symbols.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", c.event.symbols.join(","))
            };
            let text_preview: String = c.fmp_text.chars().take(160).collect();
            format!(
                "[{i}] title={} | source={} | symbols={symbols} | text={text_preview}",
                c.event.title, c.event.source
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let user = format!(
        "## 受众持仓概览\n{briefs_block}\n\n## 候选({n} 篇)\n{cand_block}\n\n请输出 JSON,items 数组要覆盖全部 {n} 条候选。",
        n = candidates.len()
    );

    vec![
        Message {
            role: "system".into(),
            content: Some(system.into()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".into(),
            content: Some(user),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

/// 容错 JSON 解析:strip 三种 markdown fence,允许前后多余 prose。
pub(super) fn parse_pass1_response(content: &str) -> anyhow::Result<Pass1Response> {
    let cleaned = strip_json_fence(content);
    serde_json::from_str(&cleaned).map_err(|e| {
        anyhow::anyhow!(
            "pass1 JSON parse: {e}; raw_prefix={}",
            &cleaned.chars().take(200).collect::<String>()
        )
    })
}

pub(super) fn strip_json_fence(s: &str) -> String {
    let s = s.trim();
    // 形如 ```json ... ``` 或 ``` ... ```
    if let Some(rest) = s.strip_prefix("```") {
        let rest = rest.trim_start_matches("json").trim_start_matches('\n');
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim().to_string();
        }
        return rest.trim().to_string();
    }
    // 找第一个 `{` 之后到最后一个 `}`,截出 JSON 主体(LLM 可能在前后加 prose)
    if let (Some(start), Some(end)) = (s.find('{'), s.rfind('}')) {
        if end > start {
            return s[start..=end].to_string();
        }
    }
    s.to_string()
}

/// cluster dedup + top_n 截断。
fn rank_and_dedupe(
    candidates: &[GlobalDigestCandidate],
    items: Vec<Pass1Item>,
    top_n: usize,
) -> Vec<RankedCandidate> {
    use std::collections::HashMap;
    // 同 cluster 内只保留最高分 item
    let mut by_cluster: HashMap<String, Pass1Item> = HashMap::new();
    for it in items {
        if it.idx >= candidates.len() {
            // 越界 idx 直接跳过(保护 LLM 编造 idx)
            continue;
        }
        let cluster_key = if it.cluster.is_empty() {
            // 空 cluster 退化成 idx-唯一,保留为独立条目
            format!("__noclust__{}", it.idx)
        } else {
            it.cluster.clone()
        };
        by_cluster
            .entry(cluster_key)
            .and_modify(|exist| {
                if it.score > exist.score {
                    *exist = it.clone();
                }
            })
            .or_insert(it);
    }
    let mut deduped: Vec<Pass1Item> = by_cluster.into_values().collect();
    // 按 score 降序;同分按 idx 升序保稳定
    deduped.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.idx.cmp(&b.idx)));
    deduped
        .into_iter()
        .take(top_n)
        .map(|it| RankedCandidate {
            candidate: candidates[it.idx].clone(),
            pass1_score: it.score,
            pass1_cluster: it.cluster,
            pass1_takeaway: it.takeaway,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::global_digest::audience::{BriefSource, CompanyBrief};
    use crate::global_digest::collector::GlobalDigestCandidate;
    use crate::pollers::news::NewsSourceClass;
    use chrono::Utc;
    use futures::stream::{self, BoxStream};
    use hone_core::{HoneError, HoneResult};
    use hone_llm::{ChatResponse, provider::ChatResult};
    use std::sync::Mutex;

    fn cand(id: &str, title: &str) -> GlobalDigestCandidate {
        GlobalDigestCandidate {
            event: MarketEvent {
                id: id.into(),
                kind: EventKind::NewsCritical,
                severity: Severity::High,
                symbols: vec!["AAPL".into()],
                occurred_at: Utc::now(),
                title: title.into(),
                summary: "summary".into(),
                url: Some(format!("https://x/{id}")),
                source: "fmp.stock_news:reuters.com".into(),
                payload: serde_json::json!({}),
            },
            source_class: NewsSourceClass::Trusted,
            fmp_text: format!("body of {id}"),
            site: "reuters.com".into(),
        }
    }

    fn audience() -> AudienceContext {
        AudienceContext {
            briefs: vec![CompanyBrief {
                ticker: "AAPL".into(),
                name: "Apple Inc.".into(),
                sector: "Technology".into(),
                industry: "Consumer Electronics".into(),
                one_liner: "designs phones, computers, services".into(),
                user_notes: vec![],
                source: BriefSource::FmpDescription,
            }],
        }
    }

    /// Mock LLM:返回固定 content,记录调用次数。
    struct MockProvider {
        content: String,
        calls: Mutex<usize>,
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn chat(&self, _m: &[Message], _model: Option<&str>) -> HoneResult<ChatResult> {
            *self.calls.lock().unwrap() += 1;
            Ok(ChatResult {
                content: self.content.clone(),
                usage: None,
            })
        }
        async fn chat_with_tools(
            &self,
            _: &[Message],
            _: &[serde_json::Value],
            _: Option<&str>,
        ) -> HoneResult<ChatResponse> {
            Err(HoneError::Llm("not used".into()))
        }
        fn chat_stream<'a>(
            &'a self,
            _: &'a [Message],
            _: Option<&'a str>,
        ) -> BoxStream<'a, HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    fn make_curator(content: &str) -> (Curator, Arc<MockProvider>) {
        let mock = Arc::new(MockProvider {
            content: content.into(),
            calls: Mutex::new(0),
        });
        let curator = Curator::new(mock.clone(), "p1-model", "p2-model");
        (curator, mock)
    }

    #[test]
    fn strip_json_fence_handles_bare_json() {
        assert_eq!(strip_json_fence(r#"{"a":1}"#), r#"{"a":1}"#);
    }

    #[test]
    fn strip_json_fence_strips_markdown_code_block() {
        let s = "```json\n{\"a\":1}\n```";
        assert_eq!(strip_json_fence(s), r#"{"a":1}"#);
    }

    #[test]
    fn strip_json_fence_strips_unlabeled_fence() {
        let s = "```\n{\"a\":2}\n```";
        assert_eq!(strip_json_fence(s), r#"{"a":2}"#);
    }

    #[test]
    fn strip_json_fence_extracts_object_from_prose() {
        let s = "Sure! Here is the JSON:\n\n{\"a\":3}\n\nLet me know if you need more.";
        assert_eq!(strip_json_fence(s), r#"{"a":3}"#);
    }

    #[test]
    fn parse_pass1_handles_valid_response() {
        let resp = r#"{"items":[{"idx":0,"score":4,"cluster":"foo","takeaway":"hello"}]}"#;
        let p = parse_pass1_response(resp).unwrap();
        assert_eq!(p.items.len(), 1);
        assert_eq!(p.items[0].idx, 0);
        assert_eq!(p.items[0].score, 4);
    }

    #[test]
    fn parse_pass1_handles_fenced_response() {
        let resp = "```json\n{\"items\":[{\"idx\":1,\"score\":5,\"cluster\":\"x\",\"takeaway\":\"y\"}]}\n```";
        let p = parse_pass1_response(resp).unwrap();
        assert_eq!(p.items[0].idx, 1);
    }

    #[test]
    fn rank_and_dedupe_keeps_highest_score_per_cluster() {
        let cands = vec![
            cand("a", "Story A"),
            cand("b", "Story B"),
            cand("c", "Story C"),
        ];
        let items = vec![
            Pass1Item {
                idx: 0,
                score: 3,
                cluster: "merger".into(),
                takeaway: "low".into(),
            },
            Pass1Item {
                idx: 1,
                score: 5,
                cluster: "merger".into(),
                takeaway: "high".into(),
            },
            Pass1Item {
                idx: 2,
                score: 4,
                cluster: "other".into(),
                takeaway: "mid".into(),
            },
        ];
        let out = rank_and_dedupe(&cands, items, 10);
        assert_eq!(out.len(), 2, "merger cluster 应被 dedupe 成 1 条");
        assert_eq!(out[0].pass1_score, 5);
        assert_eq!(out[0].pass1_takeaway, "high");
        assert_eq!(out[1].pass1_score, 4);
    }

    #[test]
    fn rank_and_dedupe_truncates_to_top_n() {
        let cands: Vec<_> = (0..5).map(|i| cand(&format!("e{i}"), "T")).collect();
        let items: Vec<_> = (0..5)
            .map(|i| Pass1Item {
                idx: i,
                score: (5 - i) as u8,
                cluster: format!("c{i}"),
                takeaway: "t".into(),
            })
            .collect();
        let out = rank_and_dedupe(&cands, items, 3);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].pass1_score, 5);
        assert_eq!(out[1].pass1_score, 4);
        assert_eq!(out[2].pass1_score, 3);
    }

    #[test]
    fn rank_and_dedupe_skips_out_of_range_idx() {
        let cands = vec![cand("a", "T")];
        let items = vec![
            Pass1Item {
                idx: 0,
                score: 4,
                cluster: "c".into(),
                takeaway: "ok".into(),
            },
            Pass1Item {
                idx: 99,
                score: 5,
                cluster: "x".into(),
                takeaway: "fake".into(),
            },
        ];
        let out = rank_and_dedupe(&cands, items, 10);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].pass1_takeaway, "ok");
    }

    #[test]
    fn rank_and_dedupe_treats_empty_cluster_as_unique() {
        let cands = vec![cand("a", "T1"), cand("b", "T2")];
        let items = vec![
            Pass1Item {
                idx: 0,
                score: 3,
                cluster: "".into(),
                takeaway: "u1".into(),
            },
            Pass1Item {
                idx: 1,
                score: 4,
                cluster: "".into(),
                takeaway: "u2".into(),
            },
        ];
        let out = rank_and_dedupe(&cands, items, 10);
        assert_eq!(out.len(), 2, "空 cluster 不应该被合并");
    }

    #[tokio::test]
    async fn pass1_select_calls_llm_and_returns_ranked() {
        let json = r#"{"items":[{"idx":0,"score":5,"cluster":"x","takeaway":"hot"}]}"#;
        let (curator, mock) = make_curator(json);
        let cands = vec![cand("a", "Big news")];
        let out = curator.pass1_select(&cands, &audience(), 10).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].pass1_score, 5);
        assert_eq!(*mock.calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn pass1_select_empty_candidates_skips_llm() {
        let (curator, mock) = make_curator("");
        let out = curator.pass1_select(&[], &audience(), 10).await.unwrap();
        assert!(out.is_empty());
        assert_eq!(*mock.calls.lock().unwrap(), 0);
    }

    fn body(text: &str) -> ArticleBody {
        ArticleBody {
            url: "https://x/y".into(),
            text: text.into(),
            source: crate::global_digest::ArticleSource::Fetched,
        }
    }

    #[tokio::test]
    async fn pass2_baseline_calls_llm_and_maps_picks() {
        let json = r#"{"picks":[
            {"idx":1,"rank":1,"title":"Story B","url":"https://x/b","comment":"nice"},
            {"idx":0,"rank":2,"title":"Story A","url":"https://x/a","comment":"ok"}
        ]}"#;
        let (curator, _mock) = make_curator(json);
        let cands = vec![cand("a", "Story A"), cand("b", "Story B")];
        let picks: Vec<(RankedCandidate, ArticleBody)> = cands
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                (
                    RankedCandidate {
                        candidate: c,
                        pass1_score: 5,
                        pass1_cluster: format!("c{i}"),
                        pass1_takeaway: "t".into(),
                    },
                    body("article body"),
                )
            })
            .collect();
        let out = curator.pass2_baseline(picks, &audience(), 8).await.unwrap();
        assert_eq!(out.len(), 2);
        // 按 rank 升序
        assert_eq!(out[0].rank, 1);
        assert_eq!(out[0].candidate.event.id, "b");
        assert_eq!(out[1].rank, 2);
        assert_eq!(out[1].candidate.event.id, "a");
    }

    #[tokio::test]
    async fn pass2_baseline_skips_invalid_idx() {
        let json = r#"{"picks":[
            {"idx":99,"rank":1,"title":"fake","url":"x","comment":"c"},
            {"idx":0,"rank":2,"title":"real","url":"x","comment":"r"}
        ]}"#;
        let (curator, _) = make_curator(json);
        let cands = vec![cand("a", "T")];
        let picks: Vec<_> = cands
            .into_iter()
            .map(|c| {
                (
                    RankedCandidate {
                        candidate: c,
                        pass1_score: 5,
                        pass1_cluster: "x".into(),
                        pass1_takeaway: "".into(),
                    },
                    body("b"),
                )
            })
            .collect();
        let out = curator.pass2_baseline(picks, &audience(), 8).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].candidate.event.id, "a");
    }

    #[tokio::test]
    async fn pass2_baseline_empty_picks_skips_llm() {
        let (curator, mock) = make_curator("");
        let out = curator
            .pass2_baseline(vec![], &audience(), 8)
            .await
            .unwrap();
        assert!(out.is_empty());
        assert_eq!(*mock.calls.lock().unwrap(), 0);
    }

    fn picks() -> Vec<(RankedCandidate, ArticleBody)> {
        vec![
            (
                RankedCandidate {
                    candidate: cand("a", "GOOGL Anthropic $40B"),
                    pass1_score: 5,
                    pass1_cluster: "google-anthropic".into(),
                    pass1_takeaway: "google invests".into(),
                },
                body("Google commits up to $40 billion in Anthropic..."),
            ),
            (
                RankedCandidate {
                    candidate: cand("b", "Semi rally 见顶警告"),
                    pass1_score: 5,
                    pass1_cluster: "semi-rally".into(),
                    pass1_takeaway: "warning of overheat".into(),
                },
                body("Unprecedented semi rally triggers warnings..."),
            ),
            (
                RankedCandidate {
                    candidate: cand("c", "Macron Hormuz strait"),
                    pass1_score: 4,
                    pass1_cluster: "hormuz".into(),
                    pass1_takeaway: "macron diplomacy".into(),
                },
                body("Macron reaffirms efforts to reopen Strait of Hormuz..."),
            ),
        ]
    }

    #[tokio::test]
    async fn pass2_personalize_categorizes_picks() {
        // LLM 输出:GOOGL 印证、半导体见顶被剔除、Hormuz 作为 macro_floor
        let json = r#"{"picks":[
            {"idx":0,"rank":1,"title":"GOOGL Anthropic","url":"u","comment":"印证 Gemini 飞轮","category":"thesis_aligned","thesis_relation":"印证"},
            {"idx":2,"rank":2,"title":"Hormuz","url":"u","comment":"波及电力叙事","category":"macro_floor","thesis_relation":"N/A"}
        ],"floor_satisfied":true}"#;
        let (curator, _) = make_curator(json);
        let mut theses_map = HashMap::new();
        theses_map.insert("GOOGL".into(), "看 Gemini 生态飞轮".into());
        let thesis = UserThesis {
            global_style: Some("长期叙事派"),
            theses: Some(&theses_map),
        };
        let out = curator
            .pass2_personalize(picks(), &audience(), thesis, 1, 8)
            .await
            .unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].rank, 1);
        assert_eq!(out[0].category, PickCategory::ThesisAligned);
        assert_eq!(out[0].thesis_relation, ThesisRelation::Aligned);
        assert_eq!(out[1].rank, 2);
        assert_eq!(out[1].category, PickCategory::MacroFloor);
        assert_eq!(out[1].thesis_relation, ThesisRelation::NotApplicable);
    }

    #[tokio::test]
    async fn pass2_personalize_empty_thesis_works_like_baseline() {
        let json = r#"{"picks":[
            {"idx":0,"rank":1,"title":"T","url":"u","comment":"c","category":"thesis_aligned","thesis_relation":"中立"}
        ]}"#;
        let (curator, _) = make_curator(json);
        let thesis = UserThesis::default(); // 全 None
        let out = curator
            .pass2_personalize(picks(), &audience(), thesis, 0, 8)
            .await
            .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].thesis_relation, ThesisRelation::Neutral);
    }

    #[tokio::test]
    async fn pass2_personalize_handles_unknown_category_string() {
        // LLM 返回的 category 字符串未在 enum 里 → fallback 到 ThesisAligned
        let json = r#"{"picks":[
            {"idx":0,"rank":1,"title":"T","url":"u","comment":"c","category":"weird_value","thesis_relation":"???"}
        ]}"#;
        let (curator, _) = make_curator(json);
        let out = curator
            .pass2_personalize(picks(), &audience(), UserThesis::default(), 0, 8)
            .await
            .unwrap();
        assert_eq!(out[0].category, PickCategory::ThesisAligned);
        assert_eq!(out[0].thesis_relation, ThesisRelation::NotApplicable);
    }

    #[tokio::test]
    async fn pass2_personalize_skips_invalid_idx() {
        let json = r#"{"picks":[
            {"idx":99,"rank":1,"title":"x","url":"u","comment":"c","category":"thesis_aligned","thesis_relation":"中立"},
            {"idx":1,"rank":2,"title":"y","url":"u","comment":"c","category":"thesis_aligned","thesis_relation":"中立"}
        ]}"#;
        let (curator, _) = make_curator(json);
        let out = curator
            .pass2_personalize(picks(), &audience(), UserThesis::default(), 0, 8)
            .await
            .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].candidate.event.id, "b");
    }

    #[test]
    fn render_thesis_block_empty_returns_baseline_hint() {
        let block = render_thesis_block(&UserThesis::default());
        assert!(block.contains("baseline"));
        assert!(block.contains("thesis_aligned"));
    }

    #[test]
    fn render_thesis_block_includes_global_style_and_per_ticker() {
        let mut m = HashMap::new();
        m.insert("MU".into(), "看 NAND 稀缺".into());
        m.insert("AAPL".into(), "看回购".into());
        let block = render_thesis_block(&UserThesis {
            global_style: Some("长期叙事派"),
            theses: Some(&m),
        });
        assert!(block.contains("长期叙事派"));
        assert!(block.contains("MU"));
        assert!(block.contains("看 NAND"));
        assert!(block.contains("AAPL"));
        // ticker 按字母序
        let mu_pos = block.find("MU").unwrap();
        let aapl_pos = block.find("AAPL").unwrap();
        assert!(aapl_pos < mu_pos, "expect AAPL before MU (alphabetical)");
    }

    #[tokio::test]
    async fn pass1_select_propagates_llm_error() {
        struct FailProvider;
        #[async_trait]
        impl LlmProvider for FailProvider {
            async fn chat(&self, _: &[Message], _: Option<&str>) -> HoneResult<ChatResult> {
                Err(HoneError::Llm("provider down".into()))
            }
            async fn chat_with_tools(
                &self,
                _: &[Message],
                _: &[serde_json::Value],
                _: Option<&str>,
            ) -> HoneResult<ChatResponse> {
                Err(HoneError::Llm("nu".into()))
            }
            fn chat_stream<'a>(
                &'a self,
                _: &'a [Message],
                _: Option<&'a str>,
            ) -> BoxStream<'a, HoneResult<String>> {
                Box::pin(stream::empty())
            }
        }
        let curator = Curator::new(Arc::new(FailProvider), "p1", "p2");
        let cands = vec![cand("a", "T")];
        let err = curator
            .pass1_select(&cands, &audience(), 10)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("pass1 LLM call failed"));
    }
}
