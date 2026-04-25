//! 抓原文 + HTML→text。Pass 2 用得上;失败/被 403 时 fallback 到 FMP `text`。
//!
//! 设计:
//! - `extract_article_text` 是纯函数(html → 截断后的纯文本),完全可单测
//! - `ArticleFetcher::fetch` 才走网络;15s 超时、跟随重定向、伪装 UA;
//!   非 2xx 或超时一律 fallback,不 panic、不冒泡 error —— 上游 Pass 2 总是
//!   能拿到字符串(可能是原文、可能是 FMP 摘要、可能是空)
//! - 截断到 6000 字符;长文章往往末尾是相关阅读/广告,保留头部对 LLM 判断够用

use scraper::{Html, Selector};

const FETCH_TIMEOUT_SECS: u64 = 15;
const USER_AGENT: &str = "honeclaw-bot/0.3 (+https://github.com/)";
/// 截断阈值 —— Pass 2 prompt 经济性。15 篇 × 6000 字 ≈ 90K chars ≈ 30K tokens,
/// 加 prompt 与 system 大约 100K input,仍远低于 grok-4.1-fast 的 2M context。
pub const MAX_ARTICLE_CHARS: usize = 6000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleSource {
    /// 抓到原文且非空
    Fetched,
    /// 抓取失败/无内容,回落 FMP 摘要
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
}

impl ArticleFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS))
                .build()
                .expect("reqwest client"),
        }
    }

    /// 抓 url 原文。任何失败都返回 fallback 文本(FmpFallback / Empty),
    /// 调用方不需要处理 Result —— Pass 2 始终能往下走。
    pub async fn fetch(&self, url: &str, fmp_text_fallback: &str) -> ArticleBody {
        let fallback = || ArticleBody {
            url: url.into(),
            text: fmp_text_fallback.trim().to_string(),
            source: if fmp_text_fallback.trim().is_empty() {
                ArticleSource::Empty
            } else {
                ArticleSource::FmpFallback
            },
        };

        let resp = match self.client.get(url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(url, "global_digest fetch failed: {e}");
                return fallback();
            }
        };
        if !resp.status().is_success() {
            tracing::warn!(url, status = %resp.status(), "global_digest fetch non-2xx");
            return fallback();
        }
        let html = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(url, "global_digest fetch body decode failed: {e}");
                return fallback();
            }
        };
        let text = extract_article_text(&html, MAX_ARTICLE_CHARS);
        if text.trim().is_empty() {
            tracing::info!(
                url,
                "global_digest fetch produced empty text, using FMP fallback"
            );
            return fallback();
        }
        ArticleBody {
            url: url.into(),
            text,
            source: ArticleSource::Fetched,
        }
    }
}

impl Default for ArticleFetcher {
    fn default() -> Self {
        Self::new()
    }
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
}
