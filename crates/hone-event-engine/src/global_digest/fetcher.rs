//! 抓原文 + HTML→text。Pass 2 用得上;失败/被 403 时 fallback 到 FMP `text`。
//!
//! 设计:
//! - `extract_article_text` 是纯函数(html → 截断后的纯文本),完全可单测
//! - `ArticleFetcher::fetch` 才走网络;15s 超时、跟随重定向、伪装 UA;
//!   非 2xx 或超时一律 fallback,不 panic、不冒泡 error —— 上游 Pass 2 总是
//!   能拿到字符串(可能是原文、可能是 FMP 摘要、可能是空)
//! - 三段式 fallback:直抓 → Jina Reader(若配置 key)→ FMP `text`。Jina 用无头浏览器
//!   渲染 + 抽正文,Reuters 类反爬站点和 WSJ 试读段都能拿到。
//! - 截断到 6000 字符;长文章往往末尾是相关阅读/广告,保留头部对 LLM 判断够用

use scraper::{Html, Selector};

const FETCH_TIMEOUT_SECS: u64 = 15;
const JINA_TIMEOUT_SECS: u64 = 25;
const USER_AGENT: &str = "honeclaw-bot/0.3 (+https://github.com/)";
const JINA_BASE_URL: &str = "https://r.jina.ai/";
/// 直抓被反爬 / 付费墙挡掉是常态(Reuters / WSJ / Barron's / NYT / Bloomberg / FT /
/// Economist),为这些域名打 INFO 而非 WARN —— 真正失败的语义已经由 Jina + FMP 兜底
/// 兜了,不需要每小时刷一屏的 WARN。
const PAYWALL_DOMAINS: &[&str] = &[
    "reuters.com",
    "wsj.com",
    "barrons.com",
    "nytimes.com",
    "bloomberg.com",
    "ft.com",
    "economist.com",
];
/// 截断阈值 —— Pass 2 prompt 经济性。15 篇 × 6000 字 ≈ 90K chars ≈ 30K tokens,
/// 加 prompt 与 system 大约 100K input,仍远低于 grok-4.1-fast 的 2M context。
pub const MAX_ARTICLE_CHARS: usize = 6000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleSource {
    /// 直抓原文且非空
    Fetched,
    /// 直抓失败 / 空,Jina Reader 二次抓到正文
    JinaFallback,
    /// 直抓 + Jina 都失败,落到 FMP 摘要
    FmpFallback,
    /// FMP 摘要也空
    Empty,
}

#[derive(Debug, Clone)]
pub struct ArticleBody {
    pub url: String,
    pub text: String,
    pub source: ArticleSource,
}

pub struct ArticleFetcher {
    client: reqwest::Client,
    jina_client: reqwest::Client,
    jina_api_key: Option<String>,
}

impl ArticleFetcher {
    /// 不带 Jina key 的构造,纯直抓 + FMP fallback。测试和 admin 默认用这个。
    pub fn new() -> Self {
        Self::with_jina_api_key(None)
    }

    /// 带 Jina key 的构造。`Some(key)` 启用 Jina fallback 层;`None` 行为同 `new()`。
    pub fn with_jina_api_key(jina_api_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS))
            .build()
            .expect("reqwest client");
        // Jina 走无头浏览器,响应体大、稳定但慢,timeout 放宽到 25s。
        let jina_client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(JINA_TIMEOUT_SECS))
            .build()
            .expect("reqwest client (jina)");
        Self {
            client,
            jina_client,
            jina_api_key: jina_api_key.filter(|k| !k.trim().is_empty()),
        }
    }

    /// 抓 url 原文。三段式 fallback,任何失败都返回字符串 —— 上游 Pass 2 始终能往下走。
    pub async fn fetch(&self, url: &str, fmp_text_fallback: &str) -> ArticleBody {
        let fmp_fallback = || ArticleBody {
            url: url.into(),
            text: fmp_text_fallback.trim().to_string(),
            source: if fmp_text_fallback.trim().is_empty() {
                ArticleSource::Empty
            } else {
                ArticleSource::FmpFallback
            },
        };
        let paywall = is_paywall_domain(url);

        // —— 第 1 段:直抓原文 ——
        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(html) => {
                        let text = extract_article_text(&html, MAX_ARTICLE_CHARS);
                        if !text.trim().is_empty() {
                            return ArticleBody {
                                url: url.into(),
                                text,
                                source: ArticleSource::Fetched,
                            };
                        }
                        tracing::info!(url, "global_digest direct fetch empty body");
                    }
                    Err(e) => {
                        tracing::warn!(url, "global_digest direct fetch body decode failed: {e}");
                    }
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if paywall {
                    tracing::info!(
                        url,
                        status = %status,
                        "global_digest direct fetch non-2xx (paywall domain, expected)"
                    );
                } else {
                    tracing::warn!(url, status = %status, "global_digest direct fetch non-2xx");
                }
            }
            Err(e) => {
                tracing::warn!(url, "global_digest direct fetch failed: {e}");
            }
        }

        // —— 第 2 段:Jina Reader(若配置 key)——
        if let Some(key) = self.jina_api_key.as_deref() {
            if let Some(text) = self.fetch_via_jina(url, key).await {
                return ArticleBody {
                    url: url.into(),
                    text,
                    source: ArticleSource::JinaFallback,
                };
            }
        }

        // —— 第 3 段:FMP 摘要 ——
        fmp_fallback()
    }

    /// 走 `https://r.jina.ai/<url>`,带 Bearer key。任何错误返回 None。
    /// Jina 后端遇到上游 4xx/5xx 时本身仍 HTTP 200,但 body 里会有
    /// "Warning: Target URL returned error N",由 `parse_jina_markdown` 识别。
    async fn fetch_via_jina(&self, url: &str, key: &str) -> Option<String> {
        let target = format!("{JINA_BASE_URL}{url}");
        let resp = match self
            .jina_client
            .get(&target)
            .bearer_auth(key)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(url, "global_digest jina fetch failed: {e}");
                return None;
            }
        };
        let status = resp.status();
        if !status.is_success() {
            tracing::warn!(url, status = %status, "global_digest jina fetch non-2xx");
            return None;
        }
        let body = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(url, "global_digest jina body decode failed: {e}");
                return None;
            }
        };
        match parse_jina_markdown(&body, MAX_ARTICLE_CHARS) {
            Some(text) => {
                tracing::debug!(url, len = text.chars().count(), "global_digest jina ok");
                Some(text)
            }
            None => {
                tracing::info!(url, "global_digest jina returned upstream-error body");
                None
            }
        }
    }
}

impl Default for ArticleFetcher {
    fn default() -> Self {
        Self::new()
    }
}

fn is_paywall_domain(url: &str) -> bool {
    let host = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_ascii_lowercase();
    PAYWALL_DOMAINS.iter().any(|d| host == *d || host.ends_with(&format!(".{d}")))
}

/// 解析 Jina Reader 返回的 markdown。
///
/// Jina 响应固定格式:
/// ```text
/// Title: <title>
///
/// URL Source: <url>
///
/// [Published Time: <iso>]
///
/// Markdown Content:
/// # <title>
/// ... 全站 nav / 广告 / 跟踪像素 ...
/// # <article-title>          ← 真正的正文从这里开始(常常二次出现 H1)
/// ## <subheadline>
/// ... lede + 正文段 ...
/// ```
///
/// Jina 上游 4xx/5xx 时 body 里有 "Warning: Target URL returned error N",此时返回
/// `None` 让上层走 FMP fallback。
///
/// 抽取策略:
/// 1. 检测警告 sentinel,有则 None
/// 2. 截到 "Markdown Content:" 之后,丢前面元数据
/// 3. 如果出现第二个独立 H1(典型 paywall 域名 chrome 在前、正文在后),从那里开始
/// 4. 逐行清噪:跳 `![Image N](...)` 像素、纯链接列表行
/// 5. 截断到 max_chars
pub fn parse_jina_markdown(body: &str, max_chars: usize) -> Option<String> {
    if body.contains("Warning: Target URL returned error") {
        return None;
    }
    let after_anchor = body
        .find("Markdown Content:")
        .map(|i| &body[i + "Markdown Content:".len()..])
        .unwrap_or(body);

    // 找第二个 H1。如果存在,从那里截 —— 第一个常常是带 "| Reuters" / "- WSJ" 站名后缀
    // 的 chrome 标题,后面跟着大段全站导航;第二个才是干净的文章标题。
    let trimmed = skip_to_article_h1(after_anchor.trim_start());
    let cleaned = strip_jina_noise(trimmed);
    let truncated = truncate_chars(&cleaned, max_chars);
    if truncated.trim().is_empty() {
        None
    } else {
        Some(truncated)
    }
}

fn skip_to_article_h1(s: &str) -> &str {
    // 搜第一个 "\n# " 之后,再看后续是否还有第二个 "\n# ";若有,从第二个开始
    let bytes = s.as_bytes();
    let mut count = 0usize;
    let mut second_h1_idx: Option<usize> = None;
    let mut i = 0usize;
    while i + 2 < bytes.len() {
        let line_start = i == 0
            || bytes
                .get(i.wrapping_sub(1))
                .map(|b| *b == b'\n')
                .unwrap_or(false);
        if line_start && bytes[i] == b'#' && bytes[i + 1] == b' ' {
            count += 1;
            if count == 2 {
                second_h1_idx = Some(i);
                break;
            }
        }
        i += 1;
    }
    match second_h1_idx {
        Some(idx) => &s[idx..],
        None => s,
    }
}

fn strip_jina_noise(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank_run = 0u8;
    for line in s.lines() {
        let trimmed = line.trim();
        // 跳 `![Image N](...)` 像素行(可能整行只有图片,也可能多个图片连排)
        if is_image_only_line(trimmed) {
            continue;
        }
        if trimmed.is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                out.push('\n');
            }
            continue;
        }
        blank_run = 0;
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_string()
}

fn is_image_only_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    // 行内全是 `![alt](url)` 块的拼接,中间可能空格 —— 这种是 Jina 里典型的跟踪像素
    // 与社交分享按钮排版,完全没有正文价值。
    let mut rest = line;
    let mut saw_image = false;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if !rest.starts_with("![") {
            return false;
        }
        let close_alt = match rest.find("](") {
            Some(i) => i,
            None => return false,
        };
        let after_alt = &rest[close_alt + 2..];
        let close_paren = match after_alt.find(')') {
            Some(i) => i,
            None => return false,
        };
        rest = &after_alt[close_paren + 1..];
        saw_image = true;
    }
    saw_image
}

/// 把 HTML 抽成"主文区"纯文本。
///
/// 启发式:依次尝试 `<article>`、`<main>`、`[role=main]`,取第一个找到的子树;
/// 都没有则退到 `<body>`。在选中子树里去掉 script/style/nav/aside/footer/figure
/// 之后,按 `<p>` / `<h1-h6>` / `<li>` 收集文本,中间用空行分隔。
///
/// 截断到 `max_chars`(按 char 计,UTF-8 安全)。空白合并(连续空格/换行折叠成
/// 单一空格 / 单一空行)。
pub fn extract_article_text(html: &str, max_chars: usize) -> String {
    let doc = Html::parse_document(html);

    let candidates = [
        Selector::parse("article").unwrap(),
        Selector::parse("main").unwrap(),
        Selector::parse("[role=\"main\"]").unwrap(),
        Selector::parse("body").unwrap(),
    ];
    let root = candidates
        .iter()
        .find_map(|sel| doc.select(sel).next())
        .unwrap_or_else(|| doc.root_element());

    // scraper 没有原生"删节点"操作;改在收集时按祖先黑名单跳过。
    let drop_tags: &[&str] = &[
        "script", "style", "nav", "aside", "footer", "figure", "form",
    ];
    let p_sel = Selector::parse("p, h1, h2, h3, h4, h5, h6, li").unwrap();
    let mut chunks: Vec<String> = Vec::new();
    for el in root.select(&p_sel) {
        if has_blacklisted_ancestor(&el, drop_tags) {
            continue;
        }
        let text = el.text().collect::<String>();
        let cleaned = collapse_whitespace(&text);
        if cleaned.is_empty() {
            continue;
        }
        chunks.push(cleaned);
    }
    let joined = chunks.join("\n\n");
    truncate_chars(&joined, max_chars)
}

fn has_blacklisted_ancestor(el: &scraper::ElementRef, blacklist: &[&str]) -> bool {
    let mut cur = el.parent();
    while let Some(node) = cur {
        if let Some(parent_el) = scraper::ElementRef::wrap(node) {
            let name = parent_el.value().name();
            if blacklist.contains(&name) {
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

    #[test]
    fn extracts_paragraph_text_from_article_tag() {
        let html = r#"
            <html><body>
              <header>SiteName</header>
              <article>
                <h1>Big Story</h1>
                <p>First paragraph of body.</p>
                <p>Second paragraph with details.</p>
              </article>
              <footer>copyright</footer>
            </body></html>
        "#;
        let text = extract_article_text(html, 1000);
        assert!(text.contains("Big Story"));
        assert!(text.contains("First paragraph of body."));
        assert!(text.contains("Second paragraph with details."));
        assert!(
            !text.contains("SiteName"),
            "header outside article should be excluded"
        );
        assert!(
            !text.contains("copyright"),
            "footer outside article should be excluded"
        );
    }

    #[test]
    fn falls_back_to_main_when_no_article() {
        let html = r#"
            <html><body>
              <main>
                <p>Main content here.</p>
              </main>
              <aside>Sidebar noise</aside>
            </body></html>
        "#;
        let text = extract_article_text(html, 1000);
        assert!(text.contains("Main content here."));
        assert!(!text.contains("Sidebar noise"));
    }

    #[test]
    fn falls_back_to_body_when_no_article_or_main() {
        let html = r#"
            <html><body>
              <p>Bare body paragraph.</p>
            </body></html>
        "#;
        let text = extract_article_text(html, 1000);
        assert_eq!(text, "Bare body paragraph.");
    }

    #[test]
    fn drops_script_style_nav_aside_footer_inside_article() {
        let html = r#"
            <html><body>
              <article>
                <p>Keep this.</p>
                <script>var x = 1;</script>
                <style>p { color: red; }</style>
                <nav><p>Navigation link</p></nav>
                <aside><p>Aside text</p></aside>
                <footer><p>Footer text</p></footer>
                <figure><p>Caption</p></figure>
                <p>Keep this too.</p>
              </article>
            </body></html>
        "#;
        let text = extract_article_text(html, 1000);
        assert!(text.contains("Keep this."));
        assert!(text.contains("Keep this too."));
        for noise in [
            "var x",
            "Navigation link",
            "Aside text",
            "Footer text",
            "Caption",
        ] {
            assert!(!text.contains(noise), "should drop {noise}, got: {text}");
        }
    }

    #[test]
    fn collapses_whitespace_within_paragraphs() {
        let html = "<article><p>Multi\n\n\tline   spaced</p></article>";
        let text = extract_article_text(html, 1000);
        assert_eq!(text, "Multi line spaced");
    }

    #[test]
    fn truncates_long_text_with_ellipsis() {
        let p = "abc ".repeat(2000); // ~8000 chars
        let html = format!("<article><p>{p}</p></article>");
        let text = extract_article_text(&html, 100);
        assert_eq!(text.chars().count(), 101); // 100 chars + ellipsis
        assert!(text.ends_with('…'));
    }

    #[test]
    fn empty_html_returns_empty() {
        let text = extract_article_text("<html><body></body></html>", 1000);
        assert!(text.is_empty());
    }

    #[test]
    fn handles_li_and_headers() {
        let html = r#"
            <article>
              <h2>Section title</h2>
              <ul>
                <li>Bullet one</li>
                <li>Bullet two</li>
              </ul>
            </article>
        "#;
        let text = extract_article_text(html, 1000);
        assert!(text.contains("Section title"));
        assert!(text.contains("Bullet one"));
        assert!(text.contains("Bullet two"));
    }

    // ----- Jina Reader 解析 -----

    #[test]
    fn parse_jina_strips_metadata_and_keeps_body() {
        let body = "Title: Moderna tops revenue estimates\n\
            URL Source: https://www.reuters.com/x\n\
            Published Time: 2026-05-01T10:32:33.255Z\n\
            \n\
            Markdown Content:\n\
            # Moderna tops revenue estimates | Reuters\n\
            [nav A](https://example.com/a)\n\
            [nav B](https://example.com/b)\n\
            \n\
            # Moderna tops revenue estimates\n\
            ## International sales outpace US\n\
            \n\
            International revenue came in at $311 million.\n\
            \"Our story has become a more balanced story,\" CFO said.\n";
        let text = parse_jina_markdown(body, 6000).expect("body present");
        assert!(text.contains("International revenue came in at $311 million."));
        assert!(text.contains("CFO said"));
        // 第二个 H1 之前的 Reuters chrome 标题应被跳过
        assert!(!text.contains("| Reuters"));
        // 元数据头不应混进正文
        assert!(!text.contains("URL Source:"));
        assert!(!text.contains("Published Time:"));
    }

    #[test]
    fn parse_jina_returns_none_on_upstream_error_warning() {
        let body = "Title: nytimes.com\n\
            URL Source: https://www.nytimes.com/x\n\
            \n\
            Warning: Target URL returned error 403: Forbidden\n\
            Warning: This page maybe requiring CAPTCHA, please make sure you are authorized to access this page.\n\
            \n\
            Markdown Content:\n\n";
        assert!(parse_jina_markdown(body, 6000).is_none());
    }

    #[test]
    fn parse_jina_drops_image_only_lines() {
        let body = "Markdown Content:\n\
            # Story\n\
            \n\
            ![Image 1](https://t.co/pixel?a=1)![Image 2](https://t.co/pixel?b=2)\n\
            \n\
            Real paragraph one.\n\
            ![Image 3](https://example.com/banner.png)\n\
            Real paragraph two.\n";
        let text = parse_jina_markdown(body, 6000).expect("body present");
        assert!(text.contains("Real paragraph one."));
        assert!(text.contains("Real paragraph two."));
        assert!(!text.contains("![Image"));
        assert!(!text.contains("t.co/pixel"));
    }

    #[test]
    fn parse_jina_handles_single_h1_without_skipping_body() {
        // 没有重复 H1 的情况 —— 直接保留全部
        let body = "Markdown Content:\n\
            # Single Headline\n\
            Body lede paragraph here.\n";
        let text = parse_jina_markdown(body, 6000).expect("body present");
        assert!(text.contains("Single Headline"));
        assert!(text.contains("Body lede paragraph here."));
    }

    #[test]
    fn parse_jina_truncates_long_body() {
        let huge = "x".repeat(10_000);
        let body = format!("Markdown Content:\n# Title\n{huge}");
        let text = parse_jina_markdown(&body, 200).expect("body present");
        assert_eq!(text.chars().count(), 201);
        assert!(text.ends_with('…'));
    }

    #[test]
    fn parse_jina_returns_none_when_body_empty_after_clean() {
        let body = "Markdown Content:\n\
            ![Image 1](https://t.co/pixel?a=1)\n\
            ![Image 2](https://t.co/pixel?b=2)\n";
        assert!(parse_jina_markdown(body, 6000).is_none());
    }

    #[test]
    fn paywall_domain_detection() {
        for u in [
            "https://www.reuters.com/foo",
            "https://reuters.com/foo",
            "https://www.wsj.com/x",
            "https://www.barrons.com/x",
            "https://www.nytimes.com/x",
            "https://www.bloomberg.com/x",
            "https://www.ft.com/x",
            "https://www.economist.com/x",
        ] {
            assert!(is_paywall_domain(u), "{u} should be paywall");
        }
        for u in [
            "https://www.cnbc.com/x",
            "https://seekingalpha.com/x",
            "https://example.com/wsj.com",
        ] {
            assert!(!is_paywall_domain(u), "{u} should NOT be paywall");
        }
    }

    #[test]
    fn fetcher_constructors_normalize_empty_key_to_none() {
        let f1 = ArticleFetcher::with_jina_api_key(None);
        assert!(f1.jina_api_key.is_none());
        let f2 = ArticleFetcher::with_jina_api_key(Some("".into()));
        assert!(f2.jina_api_key.is_none());
        let f3 = ArticleFetcher::with_jina_api_key(Some("   ".into()));
        assert!(f3.jina_api_key.is_none());
        let f4 = ArticleFetcher::with_jina_api_key(Some("jina_xyz".into()));
        assert_eq!(f4.jina_api_key.as_deref(), Some("jina_xyz"));
    }

    #[test]
    fn is_image_only_line_recognizes_pixel_chains() {
        assert!(is_image_only_line("![Image 1](https://t.co/x)"));
        assert!(is_image_only_line(
            "![Image 1](https://t.co/x)![Image 2](https://t.co/y)"
        ));
        assert!(is_image_only_line(
            "![Image 1](https://t.co/x) ![Image 2](https://t.co/y)"
        ));
        assert!(!is_image_only_line(
            "Real text ![Image 1](https://t.co/x)"
        ));
        assert!(!is_image_only_line(""));
        assert!(!is_image_only_line("Just plain text"));
        assert!(!is_image_only_line("![incomplete"));
    }
}
