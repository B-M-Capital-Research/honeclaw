//! NewsClassifier — 路由层对"不确定来源"新闻做 LLM 仲裁,按 actor 自定义的
//! 重要性 prompt 决定是否升级 severity。
//!
//! 触发条件(由 router 判断):
//! - `EventKind::NewsCritical`
//! - `payload.source_class == "uncertain"`(由 `pollers::news` 写入)
//! - `payload.legal_ad_template == false`(命中律所模板的不再 LLM 仲裁)
//!
//! 输出:`Importance::Important` → router 把 severity 升到 Medium;
//! `Importance::NotImportant` → 维持原 severity(通常 Low,直接进 digest 末端
//! 或被 prefs 截掉)。
//!
//! 缓存键:`(event_id, prompt_hash)`。同一新闻面对同一 prompt 只调一次 LLM,
//! 跨 actor 复用——避免 N 个用户 × M 条新闻爆 LLM 配额。

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hone_llm::{LlmProvider, Message};

use crate::event::MarketEvent;

/// LLM 仲裁结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Importance {
    Important,
    NotImportant,
}

#[async_trait]
pub trait NewsClassifier: Send + Sync {
    /// `event` 是 NewsCritical;`importance_prompt` 是 actor 配置的重要性短语。
    /// 返回 `None` 表示分类失败(网络/解析错误等),router 应保持原 severity。
    async fn classify(&self, event: &MarketEvent, importance_prompt: &str) -> Option<Importance>;
}

/// 默认重要性 prompt——用户未配置时使用。
pub const DEFAULT_IMPORTANCE_PROMPT: &str = "公司或潜在影响公司长期逻辑和宏观叙事的重大事件";

/// 始终返回 NotImportant 的 stub,用于关闭 LLM 路径或测试。
pub struct NoopClassifier;

#[async_trait]
impl NewsClassifier for NoopClassifier {
    async fn classify(&self, _event: &MarketEvent, _prompt: &str) -> Option<Importance> {
        Some(Importance::NotImportant)
    }
}

/// OpenRouter / OpenAI 兼容 LLM 实现。
///
/// 默认走 `google/gemini-3-flash-preview`(由 `model` 字段控制)。请求 prompt
/// 强制要求一行 `yes`/`no` 输出,解析失败按 NotImportant 处理(保守降级)。
///
/// 缓存策略:
/// - L1 `(event_id, prompt_hash)`:同 URL 同 prompt → 一次 LLM。
/// - L2 `(title_norm, symbols_norm, prompt_hash)`:同标题同股票同 prompt 即使
///   URL 不同也复用判定,解决"主流大事被多源重发包装" + LLM 非确定性导致
///   同主题判定漂移的问题。L2 命中时同时回填 L1。
pub struct LlmNewsClassifier {
    provider: Arc<dyn LlmProvider>,
    model: String,
    cache: Arc<Mutex<HashMap<(String, u64), Importance>>>,
    title_cache: Arc<Mutex<HashMap<(String, String, u64), Importance>>>,
}

impl LlmNewsClassifier {
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            title_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn prompt_hash(prompt: &str) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        prompt.hash(&mut h);
        h.finish()
    }

    /// 标题归一化:小写 + 仅保留 [a-z0-9 ] + 多空格折叠 + 截前 80 字符。
    /// 目的是让 "Apple CEO Tim Cook's 15-Year Legacy by the Numbers" 与
    /// "Apple CEO Tim Cook's 15-year legacy by the numbers" 命中同 key,
    /// 而又不会过分激进把不同标题误合并。
    fn normalize_title(title: &str) -> String {
        let mut out = String::with_capacity(title.len());
        let mut prev_space = false;
        for ch in title.to_lowercase().chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch);
                prev_space = false;
            } else if ch.is_whitespace() || ch == '-' || ch == '_' {
                if !prev_space && !out.is_empty() {
                    out.push(' ');
                    prev_space = true;
                }
            }
            // 其他字符(标点/emoji/CJK punctuation)直接丢弃
        }
        let trimmed = out.trim_end();
        trimmed.chars().take(80).collect()
    }

    /// 涉及股票 normalize:排序 + 大写 + join,把 "AAPL,MSFT" 与 "MSFT,AAPL"
    /// 视为同一组。空 → 空串,落到 title-only 桶。
    fn normalize_symbols(symbols: &[String]) -> String {
        let mut s: Vec<String> = symbols.iter().map(|x| x.to_uppercase()).collect();
        s.sort();
        s.join(",")
    }

    fn build_messages(event: &MarketEvent, importance_prompt: &str) -> Vec<Message> {
        let symbols = if event.symbols.is_empty() {
            "(无)".to_string()
        } else {
            event.symbols.join(", ")
        };
        let user = format!(
            "请按以下重要性标准判断这条新闻是否重要:\n\
             【重要性标准】{importance_prompt}\n\n\
             【新闻】\n\
             - 标题: {title}\n\
             - 涉及股票: {symbols}\n\
             - 来源: {source}\n\
             - 摘要: {summary}\n\n\
             请只输出一个英文单词: 'yes' 表示重要, 'no' 表示不重要。\n\
             不要输出其它任何字符。",
            title = event.title,
            symbols = symbols,
            source = event.source,
            summary = event.summary,
        );
        vec![
            Message {
                role: "system".into(),
                content: Some(
                    "你是金融新闻重要性判别助手。只回答 yes 或 no。\n\n\
                     **yes 只能用于硬事件**:新闻必须描述已经发生或由公司/监管/可信媒体\
                     明确报道的具体事件,且可能实质改变相关公司的收入、成本、监管状态、\
                     管理层、资本结构或长期产品路线。硬事件包括:\n\
                     - 并购/收购/分拆/IPO/上市/破产/退市/重组;\n\
                     - CEO/CFO 正式任免、离任或突发死亡;\n\
                     - SEC/FDA/反垄断/重大诉讼/召回/网络攻击/数据泄露;\n\
                     - 有明确客户/供应商/金额/多年期限的重大合同、订单、合作或产能计划;\n\
                     - 公司正式发布的重大芯片、平台、产线或技术突破。\n\n\
                     **默认 no**:如果不是上述硬事件,或者证据不清楚,回答 no。\
                     常见 no 情况:\n\
                     - 财报 preview / what to expect / 财报前观望;\n\
                     - earnings call transcript 作为独立事件由用户偏好控制,不在新闻仲裁里升重要;\n\
                     - 估值、目标价、upgrade/downgrade、buy/sell、技术形态、单日涨跌;\n\
                     - 'X stocks to buy / top picks / hot stocks' 等列表/promo;\n\
                     - 泛 AI 叙事、客户部署、agent/tool 试点、产品小功能、代码生成比例;\n\
                     - CEO 接班猜测、人物回顾或管理层评论,但没有正式任免;\n\
                     - 大盘走势评论、13F/机构持仓变化、YouTube/对比文章;\n\
                     - 标题/摘要主语不是涉及股票公司,目标股票只是被 FMP 自动关联;\n\
                     - ETF/基金推销、'Magnificent Seven' 等组合性回顾。"
                        .into(),
                ),
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

    fn parse(content: &str) -> Option<Importance> {
        let trimmed = content.trim().to_lowercase();
        // 取第一个非空行的开头几个字符,容错 LLM 多说话的情况
        let head: String = trimmed
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .chars()
            .take(8)
            .collect();
        if head.starts_with("yes") || head.starts_with("是") || head.starts_with("重要") {
            Some(Importance::Important)
        } else if head.starts_with("no") || head.starts_with("否") || head.starts_with("不重要")
        {
            Some(Importance::NotImportant)
        } else {
            None
        }
    }

    fn deterministic_fallback(event: &MarketEvent) -> Importance {
        let text = format!("{} {}", event.title, event.summary).to_lowercase();
        let important_patterns = [
            "acquisition",
            "acquire",
            "merger",
            "takeover",
            "bankruptcy",
            "bankrupt",
            "delist",
            "sec investigation",
            "sec probe",
            "fda reject",
            "fda approval",
            "recall",
            "guidance cut",
            "cuts guidance",
            "lowers guidance",
            "ceo resign",
            "cfo resign",
            "data breach",
            "cyberattack",
            "major contract",
            "partnership",
            "pact",
            "joint venture",
        ];
        if important_patterns.iter().any(|p| text.contains(p)) {
            Importance::Important
        } else {
            Importance::NotImportant
        }
    }

    fn cache_result(
        &self,
        l1_key: (String, u64),
        l2_key: (String, String, u64),
        title_norm: &str,
        importance: Importance,
    ) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(l1_key, importance);
        }
        if !title_norm.is_empty() {
            if let Ok(mut cache) = self.title_cache.lock() {
                cache.insert(l2_key, importance);
            }
        }
    }
}

#[async_trait]
impl NewsClassifier for LlmNewsClassifier {
    async fn classify(&self, event: &MarketEvent, importance_prompt: &str) -> Option<Importance> {
        let p_hash = Self::prompt_hash(importance_prompt);
        let l1_key = (event.id.clone(), p_hash);

        // L1 命中
        if let Ok(cache) = self.cache.lock() {
            if let Some(hit) = cache.get(&l1_key).copied() {
                return Some(hit);
            }
        }

        // L2 命中(同标题 + 同 symbols + 同 prompt)
        let title_norm = Self::normalize_title(&event.title);
        let symbols_norm = Self::normalize_symbols(&event.symbols);
        let l2_key = (title_norm.clone(), symbols_norm.clone(), p_hash);
        if !title_norm.is_empty() {
            if let Ok(cache) = self.title_cache.lock() {
                if let Some(hit) = cache.get(&l2_key).copied() {
                    // 回填 L1
                    if let Ok(mut l1) = self.cache.lock() {
                        l1.insert(l1_key, hit);
                    }
                    return Some(hit);
                }
            }
        }

        let messages = Self::build_messages(event, importance_prompt);
        let result = self.provider.chat(&messages, Some(&self.model)).await;
        match result {
            Ok(resp) => {
                let importance = match Self::parse(&resp.content) {
                    Some(v) => v,
                    None => {
                        let fallback = Self::deterministic_fallback(event);
                        tracing::warn!(
                            event_id = %event.id,
                            classifier_unavailable = true,
                            reason = "unparseable_response",
                            fallback = ?fallback,
                            "news LLM classifier fallback used"
                        );
                        fallback
                    }
                };
                self.cache_result(l1_key, l2_key, &title_norm, importance);
                Some(importance)
            }
            Err(e) => {
                let fallback = Self::deterministic_fallback(event);
                tracing::warn!(
                    event_id = %event.id,
                    classifier_unavailable = true,
                    reason = "provider_error",
                    fallback = ?fallback,
                    "news LLM classifier call failed, fallback used: {e}"
                );
                self.cache_result(l1_key, l2_key, &title_norm, fallback);
                Some(fallback)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, Severity};
    use chrono::Utc;
    use futures::stream::{self, BoxStream};
    use hone_core::{HoneError, HoneResult};
    use hone_llm::{ChatResponse, FunctionCall, ToolCall};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn ev() -> MarketEvent {
        MarketEvent {
            id: "news:test:1".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "Apple announces partnership".into(),
            summary: "Apple Inc partnered with...".into(),
            url: Some("https://example.com/x".into()),
            source: "fmp.stock_news:smallblog.io".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn parse_yes_variants() {
        assert_eq!(LlmNewsClassifier::parse("yes"), Some(Importance::Important));
        assert_eq!(
            LlmNewsClassifier::parse("Yes\n"),
            Some(Importance::Important)
        );
        assert_eq!(
            LlmNewsClassifier::parse("YES — this matters"),
            Some(Importance::Important)
        );
        assert_eq!(LlmNewsClassifier::parse("是"), Some(Importance::Important));
        assert_eq!(
            LlmNewsClassifier::parse("重要"),
            Some(Importance::Important)
        );
    }

    #[test]
    fn parse_no_and_garbage_defaults_to_fallback() {
        assert_eq!(
            LlmNewsClassifier::parse("no"),
            Some(Importance::NotImportant)
        );
        assert_eq!(
            LlmNewsClassifier::parse("nope, not important"),
            Some(Importance::NotImportant)
        );
        assert_eq!(LlmNewsClassifier::parse(""), None);
        assert_eq!(LlmNewsClassifier::parse("I don't know"), None);
    }

    /// Mock LLM that returns a fixed response, counting calls so we can verify caching.
    struct MockProvider {
        fixed_response: String,
        calls: AtomicUsize,
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> HoneResult<hone_llm::provider::ChatResult> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(hone_llm::provider::ChatResult {
                content: self.fixed_response.clone(),
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

    struct FailingProvider {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl LlmProvider for FailingProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> HoneResult<hone_llm::provider::ChatResult> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(HoneError::Llm("mock provider unavailable".into()))
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

    // Silence dead-code warnings on mock fields not touched in every test.
    #[allow(dead_code)]
    fn _force_use(t: &(ToolCall, FunctionCall)) {
        let _ = (&t.0, &t.1);
    }

    #[tokio::test]
    async fn classifier_returns_important_on_yes() {
        let mock = Arc::new(MockProvider {
            fixed_response: "yes".into(),
            calls: AtomicUsize::new(0),
        });
        let c = LlmNewsClassifier::new(mock.clone(), "google/gemini-3-flash-preview");
        let r = c.classify(&ev(), DEFAULT_IMPORTANCE_PROMPT).await;
        assert_eq!(r, Some(Importance::Important));
        assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn classifier_caches_result_per_event_and_prompt() {
        let mock = Arc::new(MockProvider {
            fixed_response: "no".into(),
            calls: AtomicUsize::new(0),
        });
        let c = LlmNewsClassifier::new(mock.clone(), "google/gemini-3-flash-preview");
        // 同 event + 同 prompt 重复 3 次 → 仅一次 LLM call
        for _ in 0..3 {
            let r = c.classify(&ev(), DEFAULT_IMPORTANCE_PROMPT).await;
            assert_eq!(r, Some(Importance::NotImportant));
        }
        assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
        // 换 prompt → 再调一次
        let _ = c.classify(&ev(), "完全不同的标准").await;
        assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn classifier_fallback_marks_obvious_major_event_important_on_provider_error() {
        let mock = Arc::new(FailingProvider {
            calls: AtomicUsize::new(0),
        });
        let c = LlmNewsClassifier::new(mock.clone(), "google/gemini-3-flash-preview");
        let mut event = ev();
        event.title = "ACME announces $5 billion acquisition of RivalCo".into();
        event.summary = "The transaction reshapes ACME's long-term product strategy.".into();
        let r = c.classify(&event, DEFAULT_IMPORTANCE_PROMPT).await;
        assert_eq!(r, Some(Importance::Important));
        assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn classifier_fallback_keeps_generic_opinion_not_important() {
        let mock = Arc::new(MockProvider {
            fixed_response: "maybe".into(),
            calls: AtomicUsize::new(0),
        });
        let c = LlmNewsClassifier::new(mock.clone(), "google/gemini-3-flash-preview");
        let mut event = ev();
        event.title = "Is Apple stock a buy after recent rally?".into();
        event.summary = "Analysts debate valuation and technical momentum.".into();
        let r = c.classify(&event, DEFAULT_IMPORTANCE_PROMPT).await;
        assert_eq!(r, Some(Importance::NotImportant));
        assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn l2_title_cache_dedupes_across_event_ids() {
        // 同标题、同股票、同 prompt,但 event_id 不同(不同 URL)→ 仅一次 LLM。
        let mock = Arc::new(MockProvider {
            fixed_response: "yes".into(),
            calls: AtomicUsize::new(0),
        });
        let c = LlmNewsClassifier::new(mock.clone(), "google/gemini-3-flash-preview");
        let mut e1 = ev();
        e1.id = "news:https://siteA.com/path1".into();
        let mut e2 = ev();
        e2.id = "news:https://siteB.com/path2".into();
        // 标题大小写/标点轻微差异也应命中归一化
        e2.title = "APPLE announces partnership!!".into();
        e1.title = "Apple announces partnership".into();

        assert_eq!(
            c.classify(&e1, DEFAULT_IMPORTANCE_PROMPT).await,
            Some(Importance::Important)
        );
        assert_eq!(
            c.classify(&e2, DEFAULT_IMPORTANCE_PROMPT).await,
            Some(Importance::Important)
        );
        // 两次 classify,只调一次 LLM(L2 命中)
        assert_eq!(mock.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn l2_does_not_collapse_different_symbols() {
        // 同标题但 symbols 不同(被关联到不同股票)→ 必须分别 LLM
        let mock = Arc::new(MockProvider {
            fixed_response: "no".into(),
            calls: AtomicUsize::new(0),
        });
        let c = LlmNewsClassifier::new(mock.clone(), "google/gemini-3-flash-preview");
        let mut e1 = ev();
        e1.id = "news:1".into();
        e1.symbols = vec!["AAPL".into()];
        let mut e2 = ev();
        e2.id = "news:2".into();
        e2.symbols = vec!["MSFT".into()];

        let _ = c.classify(&e1, DEFAULT_IMPORTANCE_PROMPT).await;
        let _ = c.classify(&e2, DEFAULT_IMPORTANCE_PROMPT).await;
        assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn normalize_title_collapses_punct_and_case() {
        let a = LlmNewsClassifier::normalize_title("Apple's CEO Tim Cook's 15-Year Legacy!");
        let b = LlmNewsClassifier::normalize_title("APPLE'S CEO TIM COOK's 15 year legacy");
        assert_eq!(a, b);
        assert!(a.contains("apple"));
    }

    #[tokio::test]
    async fn noop_classifier_always_not_important() {
        let c = NoopClassifier;
        let r = c.classify(&ev(), DEFAULT_IMPORTANCE_PROMPT).await;
        assert_eq!(r, Some(Importance::NotImportant));
    }
}
