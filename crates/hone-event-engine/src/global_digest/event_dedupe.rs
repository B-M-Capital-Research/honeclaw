//! 事件级去重 —— 在 collector 之后、Pass1 之前,把同一事件的多源报道合并成一条
//! 代表,避免 baseline / personalize picks 被同事件不同包装挤满(参考 2026-04-26
//! 早 5 条 Hormuz 重复挤满 7 条 picks 的事故)。
//!
//! 设计:**保守 dedup,只合明显同一具体事件**(同 actor + 同行动 + 几天内)。
//! 主题相同但事件独立(如 SpaceX $57M 合同 vs Pentagon $2.3B Maven AI 都是国防
//! 太空)绝对不合。POC 验证:`x-ai/grok-4.1-fast` 二阶段 induce-then-assign 在
//! 04-22 → 04-25 四天 + 04-21 高量 236 条上稳定保守(详见 docs 或 commit msg)。
//!
//! 失败降级:LLM 调用失败 / JSON 解析失败 → 透传原候选,scheduler 不会因为
//! dedup 挂掉断了整次推送。
//!
//! 漏 idx 防护:grok 偶尔会丢条(高量日 236 条丢 5),实现层强制把缺的 idx 当
//! singleton 补回,确保每条候选都有归宿。
//!
//! 代表选取:每簇内 trusted source > 最长 summary > 最新 occurred_at。

use std::sync::Arc;

use async_trait::async_trait;
use hone_llm::{LlmProvider, Message};
use serde::Deserialize;

use crate::global_digest::collector::GlobalDigestCandidate;
use crate::pollers::news::NewsSourceClass;

/// 单簇的 LLM 输出。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ClusterRaw {
    pub(crate) id: String,
    pub(crate) items: Vec<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DedupResponse {
    pub(crate) clusters: Vec<ClusterRaw>,
}

/// 一次 dedup 的统计,scheduler audit log 用。
#[derive(Debug, Clone, Default)]
pub struct DedupeStats {
    pub input: usize,
    pub clusters: usize,
    pub multi_clusters: usize,
    pub silent_drops_recovered: usize,
    pub fell_back_to_pass_through: bool,
}

/// 一个簇的 audit 行,渲染到 daily_report 用。
#[derive(Debug, Clone)]
pub struct ClusterAudit {
    pub id: String,
    pub kept_event_id: String,
    pub merged_event_ids: Vec<String>,
}

#[async_trait]
pub trait EventDeduper: Send + Sync {
    /// 输入候选 → 输出去重后的代表列表 + 统计 + 每簇 audit。
    async fn dedupe(
        &self,
        candidates: Vec<GlobalDigestCandidate>,
    ) -> (Vec<GlobalDigestCandidate>, DedupeStats, Vec<ClusterAudit>);
}

/// 始终透传的 stub,用于禁用 dedup 或测试。
pub struct PassThroughDeduper;

#[async_trait]
impl EventDeduper for PassThroughDeduper {
    async fn dedupe(
        &self,
        candidates: Vec<GlobalDigestCandidate>,
    ) -> (Vec<GlobalDigestCandidate>, DedupeStats, Vec<ClusterAudit>) {
        let n = candidates.len();
        (
            candidates,
            DedupeStats {
                input: n,
                clusters: n,
                multi_clusters: 0,
                silent_drops_recovered: 0,
                fell_back_to_pass_through: false,
            },
            Vec::new(),
        )
    }
}

/// 走 OpenRouter / OpenAI 兼容 LLM 的实现。生产配 `x-ai/grok-4.1-fast`。
pub struct LlmEventDeduper {
    provider: Arc<dyn LlmProvider>,
    model: String,
}

impl LlmEventDeduper {
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
        }
    }

    fn build_prompt(candidates: &[GlobalDigestCandidate]) -> String {
        let cand_block: String = candidates
            .iter()
            .enumerate()
            .map(|(i, c)| format!("[{i}] {}", truncate(&c.event.title, 120)))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "把下面 {n} 条新闻分组,**目标是 event-level cluster,不是 theme-level**。\n\
             \n\
             **event-cluster 的定义**:\n\
             1. 同一具体真实事件的多篇报道(同一组 actor、同一时间窗口内的同一行动)= 同一 cluster\n\
             2. 主题或行业相同、但 actor / 行动 / 时间不同的独立事件 = 不同 cluster\n\
             3. 持续多天的同一事件链(同 actor + 同主线,例如某场地缘冲突的多侧面报道)算同一 cluster\n\
             4. 没有明显共指的 → 各自 singleton(放心 singleton,不要为了凑簇硬合)\n\
             5. cluster id 用英文 kebab-case 短语,简短贴事件本身,不要用 theme 名\n\
             \n\
             **正反例**(故意用与本批无关的虚构例子,不要把这些词当本批的 cluster):\n\
             - \"Acme 收购 Beta 股价暴涨 12%\" 和 \"Beta 创始人回应 Acme 报价\" → 同 cluster(同一收购事件多视角)\n\
             - \"Acme 收购 Beta\" 和 \"Gamma 收购 Delta\" → **不同** cluster(都是 M&A 主题但完全独立交易)\n\
             - \"FDA 召回 Drug-X 批次\" 和 \"Drug-X 上市公司股价跳水\" → 同 cluster(同一召回事件)\n\
             - \"FDA 批准 Drug-X\" 和 \"FDA 批准 Drug-Y\" → **不同** cluster(都是 FDA 但不同药)\n\
             \n\
             输出严格 JSON,无任何其它字符:\n\
             {{\"clusters\":[{{\"id\":\"some-event-id\",\"items\":[0,3,7]}},...]}}\n\
             \n\
             候选:\n\
             {cand_block}\n",
            n = candidates.len()
        )
    }
}

#[async_trait]
impl EventDeduper for LlmEventDeduper {
    async fn dedupe(
        &self,
        candidates: Vec<GlobalDigestCandidate>,
    ) -> (Vec<GlobalDigestCandidate>, DedupeStats, Vec<ClusterAudit>) {
        let input_n = candidates.len();
        if input_n == 0 {
            return (
                candidates,
                DedupeStats {
                    input: 0,
                    clusters: 0,
                    multi_clusters: 0,
                    silent_drops_recovered: 0,
                    fell_back_to_pass_through: false,
                },
                Vec::new(),
            );
        }

        let prompt = Self::build_prompt(&candidates);
        let messages = vec![Message {
            role: "user".into(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];

        let resp = match self.provider.chat(&messages, Some(&self.model)).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(model = %self.model, "event_dedupe LLM call failed: {e}; falling back to pass-through");
                return pass_through(candidates, true);
            }
        };

        let parsed: DedupResponse = match parse_dedupe_json(&resp.content) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    model = %self.model,
                    raw_prefix = %resp.content.chars().take(160).collect::<String>(),
                    "event_dedupe JSON parse failed: {e}; falling back to pass-through"
                );
                return pass_through(candidates, true);
            }
        };

        // 收集 grok 覆盖的 idx,缺的当 singleton 补回(grok 偶尔丢条)
        let mut covered: Vec<bool> = vec![false; input_n];
        let mut clusters: Vec<(String, Vec<usize>)> = Vec::with_capacity(parsed.clusters.len());
        for c in parsed.clusters {
            let valid_items: Vec<usize> = c
                .items
                .iter()
                .copied()
                .filter(|i| *i < input_n && !covered[*i])
                .collect();
            for i in &valid_items {
                covered[*i] = true;
            }
            if !valid_items.is_empty() {
                clusters.push((c.id, valid_items));
            }
        }
        let mut silent_drops = 0;
        for (i, c) in covered.iter().enumerate() {
            if !*c {
                silent_drops += 1;
                let id = format!("recovered-singleton-{i}");
                clusters.push((id, vec![i]));
            }
        }

        // 选每簇代表 + 出 audit
        let mut reps: Vec<GlobalDigestCandidate> = Vec::with_capacity(clusters.len());
        let mut audits: Vec<ClusterAudit> = Vec::with_capacity(clusters.len());
        let mut multi_count = 0;
        // 按原 idx 排序保证输出顺序稳定
        let mut sorted_clusters = clusters;
        sorted_clusters.sort_by_key(|(_, items)| *items.iter().min().unwrap_or(&0));
        for (id, items) in sorted_clusters {
            let rep_local_idx = pick_representative_idx(&candidates, &items);
            let rep = candidates[rep_local_idx].clone();
            let kept_event_id = rep.event.id.clone();
            let merged: Vec<String> = items
                .iter()
                .filter(|i| **i != rep_local_idx)
                .map(|i| candidates[*i].event.id.clone())
                .collect();
            if items.len() > 1 {
                multi_count += 1;
            }
            audits.push(ClusterAudit {
                id,
                kept_event_id,
                merged_event_ids: merged,
            });
            reps.push(rep);
        }

        (
            reps,
            DedupeStats {
                input: input_n,
                clusters: audits.len(),
                multi_clusters: multi_count,
                silent_drops_recovered: silent_drops,
                fell_back_to_pass_through: false,
            },
            audits,
        )
    }
}

fn pass_through(
    candidates: Vec<GlobalDigestCandidate>,
    failed: bool,
) -> (Vec<GlobalDigestCandidate>, DedupeStats, Vec<ClusterAudit>) {
    let n = candidates.len();
    (
        candidates,
        DedupeStats {
            input: n,
            clusters: n,
            multi_clusters: 0,
            silent_drops_recovered: 0,
            fell_back_to_pass_through: failed,
        },
        Vec::new(),
    )
}

/// 簇代表选取:trusted > 最长 summary > 最新 occurred_at。返回原 candidates 里的 idx。
fn pick_representative_idx(candidates: &[GlobalDigestCandidate], items: &[usize]) -> usize {
    *items
        .iter()
        .max_by(|a, b| {
            let ca = &candidates[**a];
            let cb = &candidates[**b];
            let trust_a = matches!(ca.source_class, NewsSourceClass::Trusted) as u8;
            let trust_b = matches!(cb.source_class, NewsSourceClass::Trusted) as u8;
            // 注意:这里用 Greater = a 更优;max_by 返回最大者
            (
                trust_a,
                ca.event.summary.len(),
                ca.event.occurred_at.timestamp(),
            )
                .cmp(&(
                    trust_b,
                    cb.event.summary.len(),
                    cb.event.occurred_at.timestamp(),
                ))
        })
        .unwrap_or(&items[0])
}

/// 容错:剥 markdown fence + 允许前后零散 prose,只要内部能 parse 就 OK。
pub(crate) fn parse_dedupe_json(content: &str) -> anyhow::Result<DedupResponse> {
    let cleaned = strip_fence(content);
    serde_json::from_str(&cleaned).map_err(|e| anyhow::anyhow!("parse: {e}"))
}

fn strip_fence(s: &str) -> String {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("```") {
        let rest = rest.trim_start_matches("json").trim_start_matches('\n');
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim().to_string();
        }
    }
    // 找 JSON 主体的起止 brace
    if let (Some(start), Some(end)) = (s.find('{'), s.rfind('}')) {
        if end > start {
            return s[start..=end].to_string();
        }
    }
    s.to_string()
}

fn truncate(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use chrono::{TimeZone, Utc};
    use futures::stream::{self, BoxStream};
    use hone_core::{HoneError, HoneResult};
    use hone_llm::{ChatResponse, provider::ChatResult};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn ev(id: &str, title: &str, summary: &str, occurred_offset_secs: i64) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Medium,
            symbols: vec![],
            occurred_at: Utc
                .timestamp_opt(1_700_000_000 + occurred_offset_secs, 0)
                .unwrap(),
            title: title.into(),
            summary: summary.into(),
            url: Some(format!("https://x/{id}")),
            source: format!("rss:{id}"),
            payload: serde_json::Value::Null,
        }
    }

    fn cand(
        id: &str,
        title: &str,
        summary: &str,
        source_class: NewsSourceClass,
        occurred_offset_secs: i64,
    ) -> GlobalDigestCandidate {
        GlobalDigestCandidate {
            event: ev(id, title, summary, occurred_offset_secs),
            source_class,
            fmp_text: summary.into(),
            site: "x".into(),
        }
    }

    /// 给定固定 JSON 响应的 mock provider。
    struct FixedResponseProvider {
        response: String,
        calls: AtomicUsize,
        last_prompt: Mutex<Option<String>>,
    }

    #[async_trait]
    impl LlmProvider for FixedResponseProvider {
        async fn chat(&self, messages: &[Message], _model: Option<&str>) -> HoneResult<ChatResult> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(content) = messages.first().and_then(|m| m.content.clone()) {
                *self.last_prompt.lock().unwrap() = Some(content);
            }
            Ok(ChatResult {
                content: self.response.clone(),
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

    struct FailingProvider;

    #[async_trait]
    impl LlmProvider for FailingProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> HoneResult<ChatResult> {
            Err(HoneError::Llm("simulated failure".into()))
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

    #[tokio::test]
    async fn empty_input_short_circuits() {
        let provider = Arc::new(FixedResponseProvider {
            response: "{}".into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider.clone(), "test-model");
        let (out, stats, audits) = dedup.dedupe(vec![]).await;
        assert!(out.is_empty());
        assert_eq!(stats.input, 0);
        assert!(audits.is_empty());
        assert_eq!(provider.calls.load(Ordering::SeqCst), 0, "空输入不应调 LLM");
    }

    #[tokio::test]
    async fn typical_three_clusters_with_kept_singleton() {
        // 输入: 3 条 hormuz + 2 条 microsoft buyout + 1 条 NASA 单独
        let input = vec![
            cand(
                "a1",
                "Trump Hormuz Blockade Crisis",
                "long summary about blockade",
                NewsSourceClass::Trusted,
                0,
            ),
            cand(
                "a2",
                "Strait of Hormuz Empty",
                "shorter",
                NewsSourceClass::Trusted,
                100,
            ),
            cand(
                "a3",
                "Hormuz Oil Shock",
                "the longest summary about oil shock and demand collapse details",
                NewsSourceClass::Trusted,
                50,
            ),
            cand(
                "b1",
                "Microsoft Buyouts 7%",
                "buyout details",
                NewsSourceClass::Trusted,
                200,
            ),
            cand(
                "b2",
                "Microsoft voluntary employee buyout",
                "more details about MS layoffs and offers",
                NewsSourceClass::Trusted,
                250,
            ),
            cand(
                "c1",
                "NASA reserves Mars payload",
                "single nasa story",
                NewsSourceClass::Trusted,
                300,
            ),
        ];
        let response = r#"{"clusters":[
            {"id":"hormuz-crisis","items":[0,1,2]},
            {"id":"microsoft-buyout","items":[3,4]},
            {"id":"nasa-mars","items":[5]}
        ]}"#;
        let provider = Arc::new(FixedResponseProvider {
            response: response.into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider.clone(), "test-model");
        let (out, stats, audits) = dedup.dedupe(input).await;

        assert_eq!(stats.input, 6);
        assert_eq!(stats.clusters, 3);
        assert_eq!(stats.multi_clusters, 2);
        assert_eq!(stats.silent_drops_recovered, 0);
        assert!(!stats.fell_back_to_pass_through);

        // 每簇代表:hormuz 应选 a3(summary 最长),microsoft 应选 b2,nasa 选 c1
        let kept_ids: Vec<&str> = out.iter().map(|c| c.event.id.as_str()).collect();
        assert!(
            kept_ids.contains(&"a3"),
            "hormuz 簇代表应是 a3(最长 summary),实际 {kept_ids:?}"
        );
        assert!(kept_ids.contains(&"b2"), "microsoft 簇代表应是 b2");
        assert!(kept_ids.contains(&"c1"));
        assert_eq!(out.len(), 3);

        let hormuz_audit = audits.iter().find(|a| a.id == "hormuz-crisis").unwrap();
        assert_eq!(hormuz_audit.kept_event_id, "a3");
        assert_eq!(hormuz_audit.merged_event_ids.len(), 2);
    }

    #[tokio::test]
    async fn silent_dropped_idx_recovered_as_singleton() {
        // grok 输出只覆盖 0 和 1,故意漏掉 2
        let input = vec![
            cand("a", "first", "x", NewsSourceClass::Trusted, 0),
            cand("b", "second", "y", NewsSourceClass::Trusted, 100),
            cand(
                "c",
                "third dropped silently",
                "z",
                NewsSourceClass::Trusted,
                200,
            ),
        ];
        let response = r#"{"clusters":[{"id":"only-cluster","items":[0,1]}]}"#;
        let provider = Arc::new(FixedResponseProvider {
            response: response.into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider, "test-model");
        let (out, stats, audits) = dedup.dedupe(input).await;

        assert_eq!(stats.input, 3);
        assert_eq!(stats.silent_drops_recovered, 1);
        assert_eq!(out.len(), 2, "只有 2 簇:[a,b] 一簇 + c 救回的 singleton");
        // c 必须出现在输出里(救回)
        let kept_ids: Vec<&str> = out.iter().map(|c| c.event.id.as_str()).collect();
        assert!(
            kept_ids.contains(&"c"),
            "c 应被救回 singleton,实际 {kept_ids:?}"
        );

        let recovered = audits
            .iter()
            .find(|a| a.id.starts_with("recovered-singleton-"))
            .expect("应有 recovered-singleton 审计");
        assert_eq!(recovered.kept_event_id, "c");
    }

    #[tokio::test]
    async fn duplicate_idx_in_multiple_clusters_only_first_wins() {
        // grok 把 idx 0 同时塞进两个簇 —— 第一个簇赢,第二个去掉 0
        let input = vec![
            cand("a", "first", "x", NewsSourceClass::Trusted, 0),
            cand("b", "second", "y", NewsSourceClass::Trusted, 100),
        ];
        let response = r#"{"clusters":[{"id":"first","items":[0,1]},{"id":"second","items":[0]}]}"#;
        let provider = Arc::new(FixedResponseProvider {
            response: response.into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider, "test-model");
        let (out, stats, _audits) = dedup.dedupe(input).await;
        // 第二个簇应空被丢弃,不留 ghost cluster
        assert_eq!(stats.clusters, 1);
        assert_eq!(out.len(), 1);
    }

    #[tokio::test]
    async fn invalid_idx_skipped_not_panic() {
        let input = vec![cand("a", "only", "x", NewsSourceClass::Trusted, 0)];
        let response = r#"{"clusters":[{"id":"bad","items":[0,99,42]}]}"#;
        let provider = Arc::new(FixedResponseProvider {
            response: response.into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider, "test-model");
        let (out, stats, _) = dedup.dedupe(input).await;
        assert_eq!(stats.clusters, 1);
        assert_eq!(out.len(), 1);
    }

    #[tokio::test]
    async fn llm_failure_falls_back_to_pass_through() {
        let input = vec![
            cand("a", "x", "y", NewsSourceClass::Trusted, 0),
            cand("b", "z", "w", NewsSourceClass::Trusted, 100),
        ];
        let provider = Arc::new(FailingProvider);
        let dedup = LlmEventDeduper::new(provider, "test-model");
        let (out, stats, audits) = dedup.dedupe(input).await;
        assert_eq!(out.len(), 2, "降级应原样输出全部候选");
        assert!(stats.fell_back_to_pass_through);
        assert!(audits.is_empty());
    }

    #[tokio::test]
    async fn malformed_json_falls_back_to_pass_through() {
        let input = vec![cand("a", "x", "y", NewsSourceClass::Trusted, 0)];
        let provider = Arc::new(FixedResponseProvider {
            response: "this is not json at all".into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider, "test-model");
        let (out, stats, _) = dedup.dedupe(input).await;
        assert_eq!(out.len(), 1);
        assert!(stats.fell_back_to_pass_through);
    }

    #[tokio::test]
    async fn markdown_fence_response_parsed() {
        let input = vec![
            cand("a", "x", "y", NewsSourceClass::Trusted, 0),
            cand("b", "z", "w", NewsSourceClass::Trusted, 100),
        ];
        let response = "```json\n{\"clusters\":[{\"id\":\"c\",\"items\":[0,1]}]}\n```";
        let provider = Arc::new(FixedResponseProvider {
            response: response.into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider, "test-model");
        let (out, stats, _) = dedup.dedupe(input).await;
        assert_eq!(out.len(), 1);
        assert!(!stats.fell_back_to_pass_through);
    }

    #[tokio::test]
    async fn trusted_source_preferred_over_uncertain_in_representative() {
        // 同簇里一条 trusted、一条 uncertain → trusted 当代表
        let input = vec![
            cand(
                "u1",
                "uncertain blog post about hormuz",
                "longer summary uncertain blog version",
                NewsSourceClass::Uncertain,
                0,
            ),
            cand(
                "t1",
                "trusted reuters about hormuz",
                "shorter trusted",
                NewsSourceClass::Trusted,
                100,
            ),
        ];
        let response = r#"{"clusters":[{"id":"hormuz","items":[0,1]}]}"#;
        let provider = Arc::new(FixedResponseProvider {
            response: response.into(),
            calls: AtomicUsize::new(0),
            last_prompt: Mutex::new(None),
        });
        let dedup = LlmEventDeduper::new(provider, "test-model");
        let (out, _, _) = dedup.dedupe(input).await;
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].event.id, "t1",
            "trusted 应优先于 uncertain 即使 summary 更短"
        );
    }

    #[tokio::test]
    async fn pass_through_deduper_keeps_everything() {
        let input = vec![
            cand("a", "x", "y", NewsSourceClass::Trusted, 0),
            cand("b", "z", "w", NewsSourceClass::Trusted, 100),
        ];
        let dedup = PassThroughDeduper;
        let (out, stats, audits) = dedup.dedupe(input).await;
        assert_eq!(out.len(), 2);
        assert_eq!(stats.input, 2);
        assert_eq!(stats.clusters, 2);
        assert_eq!(stats.multi_clusters, 0);
        assert!(audits.is_empty());
    }

    #[test]
    fn parse_dedupe_json_handles_prose_wrapping() {
        let raw = "Sure, here is the JSON:\n{\"clusters\":[{\"id\":\"x\",\"items\":[0]}]}\nThanks!";
        let parsed = parse_dedupe_json(raw).unwrap();
        assert_eq!(parsed.clusters.len(), 1);
    }
}
