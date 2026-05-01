//! `SecFilingSummarizer` —— 给 SEC filing 事件挂一段 LLM 写的 ~200 字业务摘要。
//!
//! 触发位置:`SecFilingsPoller::fetch` 在产出 `MarketEvent` 后,逐个调
//! `summarizer.summarize(&event)`,把返回的文本写进 `payload.llm_summary`,
//! 由 `renderer` 在 body 里优先渲染。
//!
//! 流水(LlmSecFilingSummarizer):
//! 1. 取 `event.url`(指向 SEC.gov 上 filing 的最终 HTML)。
//! 2. `reqwest::Client` 用配置里的 `User-Agent` GET HTML —— **SEC EDGAR 强制要求
//!    UA 含联系邮箱**,否则会被限流甚至 403。
//! 3. `scraper` 解析,纯文本抽取,丢 `<script>/<style>` 等噪声。截断到 ~300k
//!    字符(实证 grok-4.1-fast 在 ~75k token 输入下稳定,留余量给 system prompt)。
//! 4. 拼 system + user message,调 `LlmProvider::chat`,system prompt 锚定
//!    thesis-investor 视角(跳过 GAAP 数字,抓 backlog / 资本配置 / 风险新增)。
//! 5. (event_id, prompt_hash) 内存缓存,同一 filing 在多 tick 间只调 1 次 LLM
//!    —— SEC poller `with_sec_recent_hours=48` 会让同一 filing 在窗口内被反复
//!    见到。EventStore 幂等不阻止 fetch 路径上的 LLM 重复调用。
//!
//! 失败模式:fetch 失败 / extract 空 / LLM 错误 / content 空 → 一律返回 None,
//! poller 拿到 None 就跳过,不写 llm_summary,renderer 自动 fallback 到现有
//! `event.summary`(filing date)body。**enrichment 永远是非阻塞 best-effort**,
//! 不会让 SecFiling 事件因 LLM 失败而失踪。
//!
//! 不做的事:
//! - 不做磁盘缓存(SEC filing 量级 ~70 条/年,内存够用)。
//! - 不做并发抓取(同 tick 1-2 条,不值得 join_all)。
//! - 不解析具体小节(MD&A/Risk Factors)—— LLM 自己抓更稳,POC 实测过。

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use hone_llm::{LlmProvider, Message};
use scraper::{Html, Selector};
use tracing::{debug, warn};

use crate::event::MarketEvent;

/// 抽取后送 LLM 的最大字符数。grok-4.1-fast 2M context 够用,但更长 input 既费
/// token 又触发更多 reasoning 噪声。POC 验证 ~300k chars(≈ 75k tokens)在 11
/// 持仓 × 五种 form 上 100% 不触发模型截断。
pub const MAX_FILING_CHARS: usize = 300_000;

/// LLM 摘要默认 system prompt —— **不要随意改**。
/// 与 POC `step3_summarize.py` 验证过的 prompt 完全一致。
pub const DEFAULT_SYSTEM_PROMPT: &str = "你是一个长期叙事派投资者的分析助手。我会给你一份 SEC filing(10-Q 季报 / 10-K 年报 / 8-K / S-1 / DEF 14A 等)的全文,你需要用中文写一段 ~200 字的核心要点摘要,**严格遵守**以下原则:\n\n1. **不要重复 GAAP 数字**(收入 X 亿、净利润 Y 亿等),这些用户从财报新闻已知道\n2. **重点抓业务驱动信号**:大客户 / 大订单 / backlog 变化、产能 / 工厂进展、产品 line 节奏(新品发布 / 量产 ramp)、地区 mix 变化、定价 / 毛利结构性变化\n3. **重点抓新增风险**:Risk Factors 章节里**本次新增或显著修改**的条目(不要把模板风险全列一遍)、监管 / 诉讼 / 供应链异常、stock-based comp 大幅波动等隐藏信号\n4. **重点抓资本配置变化**:大额回购 / 派息变化、新债 / 偿债、并购 / 资产剥离、capex 节奏\n5. 不分点,自然段。开头一句直接讲「这份 filing 最值得 long-thesis 投资者关注的是 X」\n6. 保持克制。如果这份 filing 内容平淡(routine 季报),直接说「无显著新信号,routine」一行就够";

/// SEC filing 摘要器。给 `MarketEvent`(必须是 `EventKind::SecFiling`)产出
/// 一段 ~200 字中文 thesis-investor 摘要。
#[async_trait]
pub trait SecFilingSummarizer: Send + Sync {
    async fn summarize(&self, event: &MarketEvent) -> Option<String>;
}

/// 始终返回 None 的 stub,用于关闭 enrichment 通道或单测注入。
pub struct NoopSecFilingSummarizer;

#[async_trait]
impl SecFilingSummarizer for NoopSecFilingSummarizer {
    async fn summarize(&self, _event: &MarketEvent) -> Option<String> {
        None
    }
}

/// 真实 LLM 实现。
pub struct LlmSecFilingSummarizer {
    provider: Arc<dyn LlmProvider>,
    model: String,
    max_summary_tokens: u32,
    user_agent: String,
    http: reqwest::Client,
    cache: Arc<Mutex<HashMap<(String, u64), String>>>,
    system_prompt: String,
}

impl LlmSecFilingSummarizer {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        model: impl Into<String>,
        max_summary_tokens: u32,
        user_agent: impl Into<String>,
    ) -> Self {
        let user_agent = user_agent.into();
        // 30s timeout 对 EDGAR HTML(通常 1-5MB)绰绰有余;遇到偶尔慢请求也不
        // 把整个 SecFilingsPoller tick 卡死。
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(user_agent.clone())
            .build()
            .expect("reqwest client should build");
        Self {
            provider,
            model: model.into(),
            max_summary_tokens,
            user_agent,
            http,
            cache: Arc::new(Mutex::new(HashMap::new())),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    /// 仅在测试或定制路径覆盖默认 prompt。生产路径默认走 `DEFAULT_SYSTEM_PROMPT`。
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    fn cache_key(&self, event: &MarketEvent) -> (String, u64) {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.system_prompt.hash(&mut h);
        self.model.hash(&mut h);
        (event.id.clone(), h.finish())
    }

    async fn fetch_filing_html(&self, url: &str) -> Option<String> {
        let resp = match self.http.get(url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!(url = %url, "SEC filing fetch failed: {e:#}");
                return None;
            }
        };
        if !resp.status().is_success() {
            warn!(
                url = %url,
                status = %resp.status(),
                ua = %self.user_agent,
                "SEC filing non-2xx — 检查 User-Agent 是否含联系邮箱"
            );
            return None;
        }
        match resp.text().await {
            Ok(t) => Some(t),
            Err(e) => {
                warn!(url = %url, "SEC filing body read failed: {e:#}");
                None
            }
        }
    }
}

#[async_trait]
impl SecFilingSummarizer for LlmSecFilingSummarizer {
    async fn summarize(&self, event: &MarketEvent) -> Option<String> {
        // 必须是 SecFiling 才有意义
        let form = match &event.kind {
            crate::event::EventKind::SecFiling { form } => form.clone(),
            _ => return None,
        };
        let url = event.url.as_deref()?;
        if url.is_empty() {
            return None;
        }
        let ticker = event.symbols.first().cloned().unwrap_or_default();

        let key = self.cache_key(event);
        if let Some(hit) = self.cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
            debug!(event_id = %event.id, "sec_enrichment cache hit");
            return Some(hit);
        }

        let html = self.fetch_filing_html(url).await?;
        let text = extract_filing_text(&html, MAX_FILING_CHARS);
        if text.trim().is_empty() {
            warn!(event_id = %event.id, "sec_enrichment: extracted text empty after parsing");
            return None;
        }

        let user_msg = format!(
            "以下是 {ticker} 的 {form} 文件全文,请按 system prompt 规则输出摘要:\n\n{text}"
        );
        let messages = vec![
            Message {
                role: "system".into(),
                content: Some(self.system_prompt.clone()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            Message {
                role: "user".into(),
                content: Some(user_msg),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];

        // 注:LlmProvider trait 当前没暴露 max_tokens 旋钮 —— OpenRouterProvider 内部
        // 用其默认。max_summary_tokens 字段保留在配置里供后续扩展(若要换 provider
        // 或者拓展 trait 增加 generation params 时使用),目前是 dead config knob。
        // 不删字段是为了 config 兼容性 —— 用户设过 max_summary_tokens=2000 想改默认
        // 的时候,trait 升级后可以直接生效。
        let _ = self.max_summary_tokens;
        let result = match self.provider.chat(&messages, Some(&self.model)).await {
            Ok(r) => r,
            Err(e) => {
                warn!(
                    event_id = %event.id,
                    model = %self.model,
                    "sec_enrichment LLM call failed: {e}"
                );
                return None;
            }
        };
        let summary = result.content.trim().to_string();
        if summary.is_empty() {
            warn!(event_id = %event.id, "sec_enrichment LLM returned empty content");
            return None;
        }

        if let Ok(mut c) = self.cache.lock() {
            c.insert(key, summary.clone());
        }
        Some(summary)
    }
}

/// 从 SEC EDGAR HTML 抽取纯文本。
///
/// SEC filings 没有 `<article>` 这种语义标签,大量内容在 `<table>` 里;
/// `global_digest::fetcher::extract_article_text` 选 `article/main/role=main/body`
/// 在 SEC HTML 上会丢掉表格内容。这里改成"全文遍历 + 黑名单跳过 script/style"
/// 的最小方案,跟 POC python `HTMLParser` 行为对齐(POC 验证产出的纯文本喂
/// grok-4.1-fast 后摘要质量稳定)。
pub fn extract_filing_text(html: &str, max_chars: usize) -> String {
    let doc = Html::parse_document(html);
    let drop_tags: &[&str] = &["script", "style", "noscript", "head"];
    let body_sel = Selector::parse("body").unwrap();
    let root = doc
        .select(&body_sel)
        .next()
        .unwrap_or_else(|| doc.root_element());

    // 遍历所有元素,把直接文本子节点收集起来;祖先黑名单跳过。
    let all_sel = Selector::parse("*").unwrap();
    let mut chunks: Vec<String> = Vec::new();
    for el in root.select(&all_sel) {
        let name = el.value().name();
        if drop_tags.contains(&name) {
            continue;
        }
        if has_blacklisted_ancestor(&el, drop_tags) {
            continue;
        }
        // 只取直接文本子节点,避免双重计数(子元素的文本会再次被父元素 select 时
        // 通过 .text() 抓到)。
        let mut local = String::new();
        for node in el.children() {
            if let Some(t) = node.value().as_text() {
                let s = t.trim();
                if !s.is_empty() {
                    if !local.is_empty() {
                        local.push(' ');
                    }
                    local.push_str(s);
                }
            }
        }
        if !local.is_empty() {
            chunks.push(collapse_whitespace(&local));
        }
    }
    let joined = chunks.join(" ");
    truncate_chars(&joined, max_chars)
}

fn has_blacklisted_ancestor(el: &scraper::ElementRef, blacklist: &[&str]) -> bool {
    let mut cur = el.parent();
    while let Some(node) = cur {
        if let Some(parent_el) = scraper::ElementRef::wrap(node) {
            if blacklist.contains(&parent_el.value().name()) {
                return true;
            }
            cur = parent_el.parent();
        } else {
            break;
        }
    }
    false
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = true;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use chrono::Utc;
    use serde_json::json;

    fn make_filing_event(id: &str, url: Option<&str>, form: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::SecFiling { form: form.into() },
            severity: Severity::Medium,
            symbols: vec!["TSLA".into()],
            occurred_at: Utc::now(),
            title: "TSLA filed 10-Q".into(),
            summary: "2026-04-20".into(),
            url: url.map(|s| s.to_string()),
            source: "fmp.sec_filings".into(),
            payload: json!({}),
        }
    }

    #[test]
    fn extract_filing_text_skips_script_and_style() {
        let html = r#"<html><body>
            <p>Real paragraph one.</p>
            <script>console.log("noise")</script>
            <p>Real paragraph two.</p>
            <style>.x{color:red}</style>
            <table><tr><td>Cell A</td><td>Cell B</td></tr></table>
        </body></html>"#;
        let text = extract_filing_text(html, 10_000);
        assert!(text.contains("Real paragraph one."));
        assert!(text.contains("Real paragraph two."));
        assert!(text.contains("Cell A"));
        assert!(text.contains("Cell B"));
        assert!(!text.contains("console.log"));
        assert!(!text.contains("color:red"));
    }

    #[test]
    fn extract_filing_text_truncates_to_max_chars() {
        let big = "X".repeat(2_000);
        let html = format!("<html><body><p>{big}</p></body></html>");
        let text = extract_filing_text(&html, 100);
        // 100 chars + ellipsis
        assert!(text.chars().count() <= 101);
        assert!(text.ends_with('…'));
    }

    #[test]
    fn extract_filing_text_empty_on_garbage() {
        let html = "<html><body></body></html>";
        let text = extract_filing_text(html, 1000);
        assert!(text.is_empty());
    }

    #[tokio::test]
    async fn noop_summarizer_returns_none() {
        let s = NoopSecFilingSummarizer;
        let ev = make_filing_event("sec:TSLA:abc", Some("https://sec.gov/x.htm"), "10-Q");
        assert!(s.summarize(&ev).await.is_none());
    }

    /// 模拟 LlmProvider —— 直接 echo 一个固定 summary,用来验证 cache + LLM 调用路径
    /// 但不发真实 HTTP 给 SEC。注意:`LlmSecFilingSummarizer.summarize` 会先尝试 fetch
    /// SEC URL,本测试用一个**不存在的 URL** 让 fetch 失败 → 路径返回 None。所以本
    /// 测试只用来证明 "fetch 失败时 summarize 返回 None,不调 LLM"。
    struct PanicProvider;
    #[async_trait::async_trait]
    impl LlmProvider for PanicProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::provider::ChatResult> {
            panic!("PanicProvider should never be called when SEC fetch fails");
        }
        async fn chat_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[serde_json::Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::ChatResponse> {
            unreachable!()
        }
        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> futures::stream::BoxStream<'a, hone_core::HoneResult<String>> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn llm_summarizer_returns_none_when_url_fetch_fails() {
        let provider: Arc<dyn LlmProvider> = Arc::new(PanicProvider);
        let s = LlmSecFilingSummarizer::new(
            provider,
            "x-ai/grok-4.1-fast",
            800,
            "test-ua test@example.com",
        );
        // 真实但 unroutable 的 URL —— reqwest 应该 timeout / error 出来,触发 None 路径
        // 而非走到 LLM call。
        let ev = make_filing_event(
            "sec:TSLA:offline",
            Some("http://127.0.0.1:1/never-listening"),
            "10-Q",
        );
        // 触发 fetch 失败路径
        assert!(s.summarize(&ev).await.is_none());
    }

    #[tokio::test]
    async fn llm_summarizer_returns_none_for_non_secfiling_event() {
        let provider: Arc<dyn LlmProvider> = Arc::new(PanicProvider);
        let s = LlmSecFilingSummarizer::new(
            provider,
            "x-ai/grok-4.1-fast",
            800,
            "test-ua test@example.com",
        );
        let mut ev = make_filing_event("x", Some("https://x"), "10-Q");
        ev.kind = EventKind::Split; // 不是 SecFiling
        assert!(s.summarize(&ev).await.is_none());
    }

    #[tokio::test]
    async fn llm_summarizer_returns_none_for_empty_url() {
        let provider: Arc<dyn LlmProvider> = Arc::new(PanicProvider);
        let s = LlmSecFilingSummarizer::new(
            provider,
            "x-ai/grok-4.1-fast",
            800,
            "test-ua test@example.com",
        );
        let ev = make_filing_event("x", None, "10-Q");
        assert!(s.summarize(&ev).await.is_none());
    }
}
