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
//! 3. `scraper` 解析,先丢 `<script>/<style>`、hidden inline XBRL header/resources
//!    等噪声,再按 filing 类型选择 MD&A、财务附注里的业务窗口、风险/诉讼变化
//!    或 8-K exhibit 新闻稿前段。不会把整份 filing 直接塞给 LLM。
//! 4. 拼 system + user message,调 `LlmProvider::chat`,system prompt 锚定
//!    长期主线投资者视角(跳过 GAAP 数字,抓 backlog / 资本配置 / 风险新增)。
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
//! - 不做基于上一期 filing 的 diff;Risk Factors 只能摘取本期披露出的显式变化
//!   或 "no material changes" 口径。

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use hone_llm::{LlmProvider, Message};
use regex::Regex;
use scraper::{Html, Selector};
use tracing::{debug, warn};

use crate::event::MarketEvent;

/// SEC filing 摘抄后送 LLM 的最大字符数。
///
/// OpenRouter 会按当前 key 可承担的 prompt/output 预算做预授权。2026-05-07
/// 生产日志显示同一 TEM 10-Q 全文抽取约 54k prompt tokens,而当前 key 当时只能
/// 承担约 6.7k prompt tokens。POC 用 TEM/AMD/COHR 真实 10-Q 与 TEM 8-K 验证后,
/// 改为 section-aware 摘抄:10k chars 通常落在 2k-3k prompt tokens,给 system
/// prompt 和 800 completion tokens 留余量。
pub const MAX_FILING_CHARS: usize = 10_000;

/// OpenRouter key 的可承受 prompt budget 会随 weekly limit 余额继续下降。
/// 如果默认摘抄仍触发 `Prompt tokens limit exceeded`,按同一语义选择逻辑继续
/// 压缩上下文重试,而不是把整条 enrichment 放弃。
const RETRY_FILING_CHARS: &[usize] = &[7_000, 4_500, 2_800];

/// LLM 摘要默认 system prompt —— **不要随意改**。
pub const DEFAULT_SYSTEM_PROMPT: &str = "你是一个长期主线投资者的分析助手。我会给你一份 SEC filing(10-Q 季报 / 10-K 年报 / 8-K / S-1 / DEF 14A 等)的精选摘抄,不是全文。你需要只根据摘抄用中文写一段 ~200 字核心要点摘要,**严格遵守**以下原则:\n\n1. **不要重复 GAAP 数字**(收入 X 亿、净利润 Y 亿等),这些用户从财报新闻已知道\n2. **重点抓业务驱动信号**:大客户 / 大订单 / backlog 变化、产能 / 工厂进展、产品 line 节奏(新品发布 / 量产 ramp)、地区 mix 变化、定价 / 毛利结构性变化\n3. **重点抓新增风险**:Risk Factors 章节里**本次新增或显著修改**的条目(不要把模板风险全列一遍)、监管 / 诉讼 / 供应链异常、stock-based comp 大幅波动等隐藏信号\n4. **重点抓资本配置变化**:大额回购 / 派息变化、新债 / 偿债、并购 / 资产剥离、capex 节奏\n5. 不分点,自然段。开头一句直接讲「这份 filing 最值得长期主线投资者关注的是 X」\n6. 保持克制。如果摘抄没有显示显著新信号,直接说「无显著新信号,routine」一行就够";

/// SEC filing 摘要器。给 `MarketEvent`(必须是 `EventKind::SecFiling`)产出
/// 一段 ~200 字中文长期主线投资者摘要。
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
        let mut result = None;
        let budgets = filing_context_char_budgets();
        for (idx, max_context_chars) in budgets.iter().copied().enumerate() {
            let context = extract_filing_llm_context(&html, &form, &ticker, max_context_chars);
            if context.trim().is_empty() {
                warn!(event_id = %event.id, "sec_enrichment: extracted text empty after parsing");
                return None;
            }

            let messages = build_summary_messages(&self.system_prompt, &ticker, &form, &context);

            // 注:LlmProvider trait 当前没暴露 per-call max_tokens 旋钮。输出 cap 已由
            // hone-web-api 为 SEC enrichment 注入专用 provider 解决;这里处理 input
            // prompt budget,遇到 OpenRouter 明确的 prompt-token 402 时缩小摘抄重试。
            let _ = self.max_summary_tokens;
            match self.provider.chat(&messages, Some(&self.model)).await {
                Ok(r) => {
                    result = Some(r);
                    break;
                }
                Err(e) => {
                    let err = e.to_string();
                    let can_retry = is_prompt_token_budget_error(&err) && idx + 1 < budgets.len();
                    if can_retry {
                        warn!(
                            event_id = %event.id,
                            model = %self.model,
                            context_chars = context.chars().count(),
                            next_context_max_chars = budgets[idx + 1],
                            "sec_enrichment prompt budget exceeded; retrying with smaller filing excerpts"
                        );
                        continue;
                    }
                    warn!(
                        event_id = %event.id,
                        model = %self.model,
                        context_chars = context.chars().count(),
                        "sec_enrichment LLM call failed: {e}"
                    );
                    return None;
                }
            }
        }
        let result = result?;
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

fn filing_context_char_budgets() -> Vec<usize> {
    let mut budgets = vec![MAX_FILING_CHARS];
    for budget in RETRY_FILING_CHARS {
        if *budget < MAX_FILING_CHARS && !budgets.contains(budget) {
            budgets.push(*budget);
        }
    }
    budgets
}

fn build_summary_messages(
    system_prompt: &str,
    ticker: &str,
    form: &str,
    context: &str,
) -> Vec<Message> {
    let user_msg = format!(
        "以下是 {ticker} 的 {form} 文件精选摘抄,请按 system prompt 规则输出摘要:\n\n{context}"
    );
    vec![
        Message {
            role: "system".into(),
            content: Some(system_prompt.to_string()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".into(),
            content: Some(user_msg),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

fn is_prompt_token_budget_error(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("prompt tokens limit exceeded")
        || (lower.contains("prompt")
            && lower.contains("token")
            && lower.contains("exceeded")
            && lower.contains("402"))
}

/// 从 SEC EDGAR HTML 抽取送给 LLM 的精选摘抄。
///
/// SEC 10-Q/10-K 的信号并不平均分布在全文里。POC 在 TEM/AMD/COHR 真实 10-Q
/// 上确认:有用内容主要在 MD&A、部分财务附注、Risk Factors/Legal Proceedings,
/// 而目录、普通财务表、exhibit index 与 hidden inline XBRL header/resources 会
/// 消耗大量 prompt budget。这里先做 deterministic 摘抄,再交给 LLM 总结。
pub fn extract_filing_llm_context(
    html: &str,
    form: &str,
    ticker: &str,
    max_chars: usize,
) -> String {
    let blocks = extract_filing_blocks(html);
    if blocks.is_empty() || max_chars == 0 {
        return String::new();
    }

    let form_upper = form.to_ascii_uppercase();
    if form_upper.contains("10-Q") || form_upper.contains("10-K") {
        build_periodic_filing_context(&blocks, form, ticker, max_chars)
    } else if form_upper.contains("8-K") {
        build_front_loaded_filing_context(
            &blocks,
            form,
            ticker,
            max_chars,
            "8-K / exhibit narrative",
        )
    } else {
        build_generic_filing_context(&blocks, form, ticker, max_chars)
    }
}

/// 从 SEC EDGAR HTML 抽取纯文本。
///
/// SEC filings 没有 `<article>` 这种语义标签,大量内容在 `<table>` 里;
/// `global_digest::fetcher::extract_article_text` 选 `article/main/role=main/body`
/// 在 SEC HTML 上会丢掉表格内容。这里保留表格直接文本,但跳过不可见 XBRL/HTML
/// 噪声。
pub fn extract_filing_text(html: &str, max_chars: usize) -> String {
    truncate_chars(&extract_filing_blocks(html).join(" "), max_chars)
}

fn extract_filing_blocks(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);
    let drop_tags: &[&str] = &[
        "script",
        "style",
        "noscript",
        "head",
        "title",
        "meta",
        "link",
        "ix:hidden",
        "ix:header",
        "ix:references",
        "ix:resources",
    ];
    let body_sel = Selector::parse("body").unwrap();
    let root = doc
        .select(&body_sel)
        .next()
        .unwrap_or_else(|| doc.root_element());

    // 遍历所有元素,把直接文本子节点收集起来;祖先黑名单跳过。
    let all_sel = Selector::parse("*").unwrap();
    let mut chunks: Vec<String> = Vec::new();
    for el in root.select(&all_sel) {
        if should_skip_element(&el, drop_tags) || has_blacklisted_ancestor(&el, drop_tags) {
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
    dedupe_adjacent_blocks(chunks)
}

#[derive(Debug, Clone)]
struct FilingItemSpan {
    start: usize,
    end: usize,
    label: String,
    title: String,
    raw_heading: String,
    chars: usize,
}

fn build_periodic_filing_context(
    blocks: &[String],
    form: &str,
    ticker: &str,
    max_chars: usize,
) -> String {
    let mut out = String::new();
    push_context_section(
        &mut out,
        "metadata",
        &format!(
            "{ticker} {form}; selected SEC filing excerpts, not full filing. Extraction prioritizes MD&A/recent developments, capital allocation, strategic customers or agreements, acquisitions, debt, legal/regulatory/risk changes. Routine GAAP tables, table of contents, exhibit indexes, and inline XBRL hidden data are omitted."
        ),
        max_chars,
    );

    let spans = item_spans(blocks);
    if let Some(mdna) = largest_item_span(&spans, "2") {
        let body = signal_excerpt(blocks, mdna, 7_000);
        push_context_section(
            &mut out,
            "Item 2 MD&A high-signal excerpts",
            &body,
            max_chars,
        );
    }

    let cross = keyword_windows(blocks, 6_500);
    push_context_section(
        &mut out,
        "Cross-filing strategic/capital/risk windows",
        &cross,
        max_chars,
    );

    if let Some(risk) = largest_item_span(&spans, "1A") {
        let body = risk_excerpt(blocks, risk, 3_500);
        push_context_section(&mut out, "Item 1A Risk Factors excerpt", &body, max_chars);
    }

    if let Some(legal) = find_item_span_by_title(&spans, "1", "(?i)legal")
        && legal.chars <= 8_000
    {
        let body = join_blocks_under_cap(&blocks[legal.start..legal.end], 2_200);
        push_context_section(
            &mut out,
            "Part II legal proceedings excerpt",
            &body,
            max_chars,
        );
    }

    if out.trim().is_empty() {
        build_generic_filing_context(blocks, form, ticker, max_chars)
    } else {
        truncate_chars(out.trim(), max_chars)
    }
}

fn build_front_loaded_filing_context(
    blocks: &[String],
    form: &str,
    ticker: &str,
    max_chars: usize,
    body_label: &str,
) -> String {
    let mut out = String::new();
    push_context_section(
        &mut out,
        "metadata",
        &format!(
            "{ticker} {form}; selected SEC filing excerpts, not full filing. For 8-K, the useful exhibit or press-release narrative is usually front-loaded; tail financial tables and exhibit metadata are omitted when the budget is reached."
        ),
        max_chars,
    );
    let remaining = max_chars.saturating_sub(char_len(&out)).saturating_sub(64);
    let body = join_blocks_under_cap(blocks, remaining);
    push_context_section(&mut out, body_label, &body, max_chars);
    truncate_chars(out.trim(), max_chars)
}

fn build_generic_filing_context(
    blocks: &[String],
    form: &str,
    ticker: &str,
    max_chars: usize,
) -> String {
    let mut out = String::new();
    push_context_section(
        &mut out,
        "metadata",
        &format!(
            "{ticker} {form}; selected SEC filing excerpts, not full filing. Extraction keeps the front narrative plus windows around business, capital allocation, risk, and legal keywords."
        ),
        max_chars,
    );
    let front_cap = (max_chars / 2).clamp(1_500, 7_000);
    push_context_section(
        &mut out,
        "front narrative",
        &join_blocks_under_cap(blocks, front_cap),
        max_chars,
    );
    push_context_section(
        &mut out,
        "business/capital/risk windows",
        &keyword_windows(blocks, max_chars / 2),
        max_chars,
    );
    truncate_chars(out.trim(), max_chars)
}

fn item_spans(blocks: &[String]) -> Vec<FilingItemSpan> {
    let item_re =
        Regex::new(r"(?i)^(?:part\s+[ivxlcdm]+\s*)?item\s+([0-9]+[a-z]?)\.?\s*(.{0,160})$")
            .expect("valid SEC item regex");
    let mut heads: Vec<(usize, String, String, String)> = Vec::new();
    for (idx, block) in blocks.iter().enumerate() {
        if let Some(caps) = item_re.captures(block.trim()) {
            let label = caps
                .get(1)
                .map(|m| m.as_str().to_ascii_uppercase())
                .unwrap_or_default();
            let title = caps
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            heads.push((idx, label, title, block.clone()));
        }
    }

    let mut spans = Vec::new();
    for (pos, (start, label, title, raw_heading)) in heads.iter().enumerate() {
        let end = heads
            .get(pos + 1)
            .map(|(idx, _, _, _)| *idx)
            .unwrap_or(blocks.len());
        let chars = blocks[*start..end]
            .iter()
            .map(|b| char_len(b) + 1)
            .sum::<usize>();
        spans.push(FilingItemSpan {
            start: *start,
            end,
            label: label.clone(),
            title: title.clone(),
            raw_heading: raw_heading.clone(),
            chars,
        });
    }
    spans
}

fn largest_item_span<'a>(spans: &'a [FilingItemSpan], label: &str) -> Option<&'a FilingItemSpan> {
    spans
        .iter()
        .filter(|span| span.label.eq_ignore_ascii_case(label))
        .max_by_key(|span| span.chars)
}

fn find_item_span_by_title<'a>(
    spans: &'a [FilingItemSpan],
    label: &str,
    pattern: &str,
) -> Option<&'a FilingItemSpan> {
    let re = Regex::new(pattern).expect("valid title regex");
    spans.iter().find(|span| {
        span.label.eq_ignore_ascii_case(label)
            && (re.is_match(&span.title) || re.is_match(&span.raw_heading))
    })
}

fn signal_excerpt(blocks: &[String], span: &FilingItemSpan, cap: usize) -> String {
    let mut out = String::new();
    let leading = join_blocks_under_cap(&blocks[span.start..span.end], 1_200);
    push_excerpt_chunk(&mut out, &leading, cap);

    let windows = windows_in_range(
        blocks,
        span.start,
        span.end,
        1,
        cap.saturating_sub(char_len(&out)),
    );
    push_excerpt_chunk(&mut out, &windows, cap);
    out
}

fn risk_excerpt(blocks: &[String], span: &FilingItemSpan, cap: usize) -> String {
    let no_material_re = Regex::new(
        r"(?i)no material changes|not materially changed|there have been no material changes",
    )
    .expect("valid no-material risk regex");
    let preview_end = span.end.min(span.start + 40);
    for block in &blocks[span.start..preview_end] {
        if no_material_re.is_match(block) {
            return block.clone();
        }
    }
    join_blocks_under_cap(&blocks[span.start..span.end], cap)
}

fn keyword_windows(blocks: &[String], cap: usize) -> String {
    windows_in_range(blocks, 0, blocks.len(), 1, cap)
}

fn windows_in_range(
    blocks: &[String],
    start: usize,
    end: usize,
    radius: usize,
    cap: usize,
) -> String {
    if cap == 0 || start >= end {
        return String::new();
    }
    let signal_re = signal_regex();
    let heading_re = heading_regex();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for (idx, block) in blocks.iter().enumerate().take(end).skip(start) {
        if signal_re.is_match(block) || heading_re.is_match(block) {
            let a = idx.saturating_sub(radius).max(start);
            let b = (idx + radius + 2).min(end);
            if let Some(last) = ranges.last_mut()
                && a <= last.1
            {
                last.1 = last.1.max(b);
                continue;
            }
            ranges.push((a, b));
        }
    }

    let mut out = String::new();
    let mut seen: Vec<String> = Vec::new();
    for (a, b) in ranges {
        let chunk = blocks[a..b].join("\n");
        let signature = normalized_signature(&chunk);
        if seen.iter().any(|s| s == &signature) {
            continue;
        }
        seen.push(signature);
        if !out.is_empty() {
            push_excerpt_chunk(&mut out, "---", cap);
        }
        push_excerpt_chunk(&mut out, &chunk, cap);
        if char_len(&out) >= cap.saturating_sub(200) {
            break;
        }
    }
    out
}

fn signal_regex() -> Regex {
    Regex::new(
        r"(?i)\b(acquisition|acquired|merger|divest|disposition|strategic agreement|collaboration|master services agreement|commercialization|reference laboratory|minimum commitment|purchase commitment|private placement|securities purchase agreement|warrant|purchase milestone|capacity rights|gpu purchase|backlog|manufacturing capacity|supply chain|pricing|margin structure|credit facility|convertible|repurchase|dividend|capital expenditure|liquidity|restructuring|impairment|litigation|investigation|regulatory|subpoena|cybersecurity|subsequent events|commitments?|contingencies|stock-based compensation|reportable segments?|segment realign|export controls?|tariffs?)\b",
    )
    .expect("valid filing signal regex")
}

fn heading_regex() -> Regex {
    Regex::new(
        r"(?i)^(overview|recent developments|industry conditions|agreements with|liquidity and capital resources|research and development|new products|risk factors|legal proceedings|commitments|contingencies|segment reporting|macroeconomic conditions)",
    )
    .expect("valid filing heading regex")
}

fn join_blocks_under_cap(blocks: &[String], cap: usize) -> String {
    let mut out = String::new();
    for block in blocks {
        if block.trim().is_empty() {
            continue;
        }
        let addition = if out.is_empty() {
            block.clone()
        } else {
            format!("\n{block}")
        };
        if char_len(&out) + char_len(&addition) > cap {
            break;
        }
        out.push_str(&addition);
    }
    out
}

fn push_context_section(out: &mut String, label: &str, body: &str, max_chars: usize) {
    let body = body.trim();
    if body.is_empty() {
        return;
    }
    let prefix = if out.is_empty() { "" } else { "\n\n" };
    let section = format!("{prefix}## {label}\n{body}");
    let remaining = max_chars.saturating_sub(char_len(out));
    if remaining < 64 {
        return;
    }
    if char_len(&section) <= remaining {
        out.push_str(&section);
    } else {
        out.push_str(&truncate_chars(&section, remaining));
    }
}

fn push_excerpt_chunk(out: &mut String, chunk: &str, cap: usize) {
    let chunk = chunk.trim();
    if chunk.is_empty() || char_len(out) >= cap {
        return;
    }
    let prefix = if out.is_empty() { "" } else { "\n" };
    let addition = format!("{prefix}{chunk}");
    let remaining = cap.saturating_sub(char_len(out));
    if char_len(&addition) <= remaining {
        out.push_str(&addition);
    } else {
        out.push_str(&truncate_chars(&addition, remaining));
    }
}

fn normalized_signature(s: &str) -> String {
    let re = Regex::new(r"[^a-z0-9]+").expect("valid signature regex");
    let lower = s.to_ascii_lowercase();
    let compact = re.replace_all(&lower, " ");
    truncate_chars(compact.trim(), 220)
}

fn dedupe_adjacent_blocks(blocks: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut prev = String::new();
    for block in blocks {
        let block = collapse_whitespace(&block);
        if block.is_empty() || block == prev {
            continue;
        }
        prev = block.clone();
        out.push(block);
    }
    out
}

fn should_skip_element(el: &scraper::ElementRef, blacklist: &[&str]) -> bool {
    let name = el.value().name();
    blacklist.contains(&name)
        || is_hidden_style(el.value().attr("style"))
        || el.value().attr("hidden").is_some()
        || el
            .value()
            .attr("aria-hidden")
            .is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

fn has_blacklisted_ancestor(el: &scraper::ElementRef, blacklist: &[&str]) -> bool {
    let mut cur = el.parent();
    while let Some(node) = cur {
        if let Some(parent_el) = scraper::ElementRef::wrap(node) {
            if should_skip_element(&parent_el, blacklist) {
                return true;
            }
            cur = parent_el.parent();
        } else {
            break;
        }
    }
    false
}

fn is_hidden_style(style: Option<&str>) -> bool {
    let Some(style) = style else {
        return false;
    };
    let normalized = style.replace(' ', "").to_ascii_lowercase();
    normalized.contains("display:none") || normalized.contains("visibility:hidden")
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

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn truncate_chars(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
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

    #[test]
    fn extract_filing_text_skips_hidden_inline_xbrl_noise() {
        let html = r#"<html><body>
            <div style="display:none"><ix:header>Hidden context noise</ix:header></div>
            <p>Visible operating narrative.</p>
            <ix:hidden>Hidden fact noise</ix:hidden>
        </body></html>"#;
        let text = extract_filing_text(html, 10_000);
        assert!(text.contains("Visible operating narrative."));
        assert!(!text.contains("Hidden context noise"));
        assert!(!text.contains("Hidden fact noise"));
    }

    #[test]
    fn extract_filing_llm_context_prioritizes_10q_business_sections() {
        let html = r#"<html><body>
            <h2>Item 1. Financial Statements</h2>
            <p>Routine GAAP table row 1.</p>
            <p>Routine GAAP table row 2.</p>
            <h2>Item 2. Management's Discussion and Analysis of Financial Condition and Results of Operations</h2>
            <p>Overview</p>
            <p>We entered a strategic agreement with a large customer to expand manufacturing capacity.</p>
            <p>We increased capital expenditures for the new facility.</p>
            <h2>Item 1A. Risk Factors</h2>
            <p>There have been no material changes to risk factors.</p>
            <h2>Item 6. Exhibits</h2>
            <p>Exhibit index noise should not be selected.</p>
        </body></html>"#;
        let context = extract_filing_llm_context(html, "10-Q", "TEST", 1_800);
        assert!(context.contains("selected SEC filing excerpts"));
        assert!(context.contains("strategic agreement"));
        assert!(context.contains("capital expenditures"));
        assert!(context.contains("no material changes"));
        assert!(!context.contains("Exhibit index noise"));
        assert!(context.chars().count() <= 1_800);
    }

    #[test]
    fn extract_filing_llm_context_front_loads_8k_exhibit() {
        let mut html = String::from(
            r#"<html><body>
            <p>Exhibit 99.1</p>
            <p>Company reports quarter results and raises guidance.</p>
            <p>Recent Operational Highlights include a strategic collaboration and capacity expansion.</p>"#,
        );
        for _ in 0..30 {
            html.push_str(
                "<p>Tail financial table that can be omitted when budget is reached.</p>",
            );
        }
        html.push_str("</body></html>");

        let context = extract_filing_llm_context(&html, "8-K", "TEST", 900);
        assert!(context.contains("raises guidance"));
        assert!(context.contains("Recent Operational Highlights"));
        assert!(context.chars().count() <= 900);
    }

    #[test]
    fn filing_context_budgets_retry_with_stricter_excerpts() {
        assert_eq!(
            filing_context_char_budgets(),
            vec![10_000, 7_000, 4_500, 2_800]
        );
    }

    #[test]
    fn prompt_token_budget_error_is_retryable() {
        let err = "LLM 错误: upstream HTTP 402: Prompt tokens limit exceeded: 5198 > 3256";
        assert!(is_prompt_token_budget_error(err));
        assert!(!is_prompt_token_budget_error(
            "upstream HTTP 402: insufficient credits"
        ));
        assert!(!is_prompt_token_budget_error(
            "maximum context length exceeded"
        ));
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
        let s =
            LlmSecFilingSummarizer::new(provider, "x-ai/grok-4.3", 800, "test-ua test@example.com");
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
        let s =
            LlmSecFilingSummarizer::new(provider, "x-ai/grok-4.3", 800, "test-ua test@example.com");
        let mut ev = make_filing_event("x", Some("https://x"), "10-Q");
        ev.kind = EventKind::Split; // 不是 SecFiling
        assert!(s.summarize(&ev).await.is_none());
    }

    #[tokio::test]
    async fn llm_summarizer_returns_none_for_empty_url() {
        let provider: Arc<dyn LlmProvider> = Arc::new(PanicProvider);
        let s =
            LlmSecFilingSummarizer::new(provider, "x-ai/grok-4.3", 800, "test-ua test@example.com");
        let ev = make_filing_event("x", None, "10-Q");
        assert!(s.summarize(&ev).await.is_none());
    }
}
