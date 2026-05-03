//! 投资主线自动蒸馏 —— 把用户在 agent-sandbox 里写的 100-400 行公司画像,定期
//! 蒸馏成 Pass2 personalize 期望的 1-2 句投资主线,写入 `NotificationPrefs.
//! mainline_by_ticker` 字段。同时跨 ticker 提取 `mainline_style`。
//!
//! 设计要点:
//! - **read-only on profile.md**:本模块只读用户画像,不改;写出方向只到 prefs。
//! - **per-actor**:每个有 portfolio 的 actor 跑一次,扫他的 sandbox profile 目录。
//! - **失败降级**:任一 ticker 蒸馏失败 → 跳过该 ticker(保留旧主线不变),
//!   不让一个失败影响其他 ticker。
//! - **整文喂 LLM**:不靠 section header 切分(framework 允许 agent 把用户视角
//!   合并到主线主文 / 单列 用户视角 / 日期 update log,各种写法都常见),
//!   POC 验证整文喂 grok 1-2k tokens 完全 OK。
//! - **持久化方式**:就地修改 prefs JSON 的 `mainline_by_ticker` /
//!   `mainline_style` / `last_mainline_distilled_at` 字段,
//!   curator 每次 dispatch 重读,无需 hot-reload。
//!
//! 路径约定:
//! - 画像:`{HONE_DATA_DIR}/agent-sandboxes/{channel_fs}/{scoped_user_fs_key}/
//!   company_profiles/{kebab-name}/profile.md`
//! - 输出:`{prefs_dir}/{actor_slug}.json`(同 NotificationPrefs 文件)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hone_core::ActorIdentity;
use hone_llm::{LlmProvider, Message};
use serde::{Deserialize, Serialize};

/// 一只 ticker 的画像,蒸馏前的原料。
#[derive(Debug, Clone)]
pub struct ProfileSource {
    pub ticker: String,
    pub dir_name: String,
    pub markdown: String,
}

/// 蒸馏结果,准备写回 prefs。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistilledMainlines {
    #[serde(alias = "theses")]
    pub by_ticker: HashMap<String, String>,
    #[serde(alias = "global_style")]
    pub style: Option<String>,
    pub last_distilled_at: Option<DateTime<Utc>>,
    /// 蒸馏中跳过的 ticker(LLM 失败 / 画像缺失等),便于诊断。
    pub skipped_tickers: Vec<String>,
}

/// 蒸馏抽象,生产实现走 LLM,测试可注入 stub。
#[async_trait]
pub trait MainlineDistiller: Send + Sync {
    async fn distill_mainline(&self, ticker: &str, profile_md: &str) -> anyhow::Result<String>;
    async fn distill_style(&self, all_profiles: &[ProfileSource]) -> anyhow::Result<String>;
}

/// LLM 实现 —— 默认走 grok-4.1-fast(POC 验证质量好,速度可接受)。
pub struct LlmMainlineDistiller {
    provider: Arc<dyn LlmProvider>,
    model: String,
}

impl LlmMainlineDistiller {
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
        }
    }
}

const DISTILL_PROMPT: &str = "下面是用户对 {{TICKER}} 的长期投资档案(可能 50-400 行,各 section 散布:\
正式投资主线、用户主观看法、估值偏好、风险红线、近期 update log)。\n\
\n\
把它蒸馏成 **1-2 句中文投资主线**,给一个全球新闻 digest 系统看,用来过滤\"对该用户视角下噪音\
vs 实质信号\"。\n\
\n\
要求:\n\
- 涵盖核心多空逻辑(为什么持有,关键变量是什么)\n\
- 涵盖用户独有视角(如有)—— 例如用户重视的具体催化点 / 用户明确反对的指标\n\
- 不要堆砌财务数字、不要写\"风险包括...\"的笼统结尾\n\
- 风格:像投资人在简短自述,不像 Wikipedia 摘要\n\
- 只输出主线文字,不要 markdown 标题、不要前言\n\
\n\
档案原文:\n\
---\n\
{{PROFILE}}\n\
---\n";

const STYLE_PROMPT: &str = "下面是同一个用户对 {{N}} 只持仓的长期画像主线段落集合。\n\
请蒸馏出 **该用户的整体投资风格** —— 跨 ticker 反复出现的偏好、判断框架、明确反感。\n\
\n\
要求:\n\
- 1-2 句中文,像投资人自我描述\n\
- 突出用户重复表达的偏好(如\"长期叙事派\"\"重视行业稀缺性\"\"严格区分赔率与确定性\")\n\
- 突出用户明确反感的(如\"轻视估值/技术形态/单日涨跌/分析师评级\")\n\
- 不要列具体公司名\n\
- 只输出风格文字,不要前言\n\
\n\
档案集合:\n\
{{PROFILES_BLOCK}}\n";

#[async_trait]
impl MainlineDistiller for LlmMainlineDistiller {
    async fn distill_mainline(&self, ticker: &str, profile_md: &str) -> anyhow::Result<String> {
        let prompt = DISTILL_PROMPT
            .replace("{{TICKER}}", ticker)
            .replace("{{PROFILE}}", profile_md);
        let messages = vec![Message {
            role: "user".into(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let resp = self
            .provider
            .chat(&messages, Some(&self.model))
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {e}"))?;
        let trimmed = resp.content.trim().to_string();
        if trimmed.is_empty() {
            anyhow::bail!("empty mainline output for {ticker}");
        }
        Ok(trimmed)
    }

    async fn distill_style(&self, all_profiles: &[ProfileSource]) -> anyhow::Result<String> {
        if all_profiles.is_empty() {
            anyhow::bail!("no profiles to extract style from");
        }
        // 每只画像取前 ~1500 字,避免 prompt 过长
        let block: String = all_profiles
            .iter()
            .map(|p| {
                let preview: String = p.markdown.chars().take(1500).collect();
                format!("\n## {}\n{preview}\n", p.ticker)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = STYLE_PROMPT
            .replace("{{N}}", &all_profiles.len().to_string())
            .replace("{{PROFILES_BLOCK}}", &block);
        let messages = vec![Message {
            role: "user".into(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let resp = self
            .provider
            .chat(&messages, Some(&self.model))
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {e}"))?;
        let trimmed = resp.content.trim().to_string();
        if trimmed.is_empty() {
            anyhow::bail!("empty global style output");
        }
        Ok(trimmed)
    }
}

/// 给 actor 在自己 sandbox 下扫所有画像目录,parse 出 ticker 列表 +
/// 完整 markdown 内容。
///
/// ticker 解析顺序(按可靠度):
/// 1. YAML frontmatter `ticker: GOOGL / GOOG` 字段
/// 2. 第一行标题里的 `(TICKER)` / `(TICKER)` 中文括号
/// 3. 第一行标题里 `/ TICKER` 模式
///
/// 如果 holdings_filter 非空,只返回 ticker ∈ holdings 的 profile。
pub fn scan_profiles(
    sandbox_root: &Path,
    holdings_filter: Option<&[String]>,
) -> Vec<ProfileSource> {
    let cp_dir = sandbox_root.join("company_profiles");
    if !cp_dir.is_dir() {
        return Vec::new();
    }
    let holdings_set: Option<std::collections::HashSet<String>> =
        holdings_filter.map(|hs| hs.iter().map(|h| h.to_uppercase()).collect());
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&cp_dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let profile_md = path.join("profile.md");
        if !profile_md.is_file() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let markdown = match std::fs::read_to_string(&profile_md) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let tickers = extract_tickers(&markdown);
        if tickers.is_empty() {
            tracing::warn!(
                dir = %dir_name,
                "mainline_distill: profile.md 没找到 ticker 标识,跳过"
            );
            continue;
        }
        // 一个 profile 可能含多个 ticker(GOOGL / GOOG),分别 emit
        for t in tickers {
            if let Some(filter) = &holdings_set {
                if !filter.contains(&t) {
                    continue;
                }
            }
            out.push(ProfileSource {
                ticker: t,
                dir_name: dir_name.clone(),
                markdown: markdown.clone(),
            });
        }
    }
    out
}

/// 从 profile.md 解析出 ticker 列表(可能 ≥1)。
pub fn extract_tickers(md: &str) -> Vec<String> {
    // 1. YAML frontmatter `ticker: X` 或 `ticker: X / Y`
    for line in md.lines().take(20) {
        let l = line.trim();
        if let Some(rest) = l
            .strip_prefix("ticker:")
            .or_else(|| l.strip_prefix("Ticker:"))
        {
            let raw = rest.trim().trim_matches('"').trim_matches('\'');
            return parse_ticker_list(raw);
        }
    }
    // 2. 标题里的 (TICKER) / (TICKER) — 第一行 `# Foo (TICKER)` 或 `# Foo（TICKER）`
    if let Some(first) = md.lines().find(|l| l.starts_with("# ")) {
        if let Some(t) = extract_paren_ticker(first) {
            return vec![t];
        }
    }
    Vec::new()
}

fn parse_ticker_list(raw: &str) -> Vec<String> {
    raw.split(['/', ',', ' '])
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty() && is_plausible_ticker(s))
        .collect()
}

fn extract_paren_ticker(line: &str) -> Option<String> {
    // 半角 ()  — ASCII 0x28 / 0x29
    if let (Some(start), Some(end)) = (line.rfind('('), line.rfind(')')) {
        if end > start {
            let candidate = &line[start + 1..end];
            if is_plausible_ticker(candidate) {
                return Some(candidate.to_uppercase());
            }
        }
    }
    // 全角 — U+FF08 / U+FF09(每个 3 字节 UTF-8)
    let lp = '\u{FF08}';
    let rp = '\u{FF09}';
    if let (Some(start), Some(end)) = (line.rfind(lp), line.rfind(rp)) {
        if end > start {
            let candidate = &line[start + lp.len_utf8()..end];
            if is_plausible_ticker(candidate) {
                return Some(candidate.to_uppercase());
            }
        }
    }
    None
}

fn is_plausible_ticker(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty()
        && s.len() <= 6
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && s.chars().any(|c| c.is_ascii_alphabetic())
}

/// 蒸馏一个 actor 的所有持仓主线 + 整体风格。`holdings` 决定要蒸哪些 ticker;
/// `provider`/`model` 注入由调用方组装。
///
/// 行为:
/// 1. scan_profiles(sandbox_root, Some(holdings)) → ProfileSource 列表
/// 2. 并发蒸馏每只 ticker 的主线(任一失败 → 加入 skipped,继续)
/// 3. 用全部 profile 蒸馏一条整体风格(失败 → None,不影响主线)
/// 4. 返回 DistilledMainlines(可序列化,调用方 merge 进 prefs JSON)
pub async fn distill_for_actor(
    distiller: &dyn MainlineDistiller,
    sandbox_root: &Path,
    holdings: &[String],
) -> DistilledMainlines {
    let profiles = scan_profiles(sandbox_root, Some(holdings));
    if profiles.is_empty() {
        return DistilledMainlines {
            by_ticker: HashMap::new(),
            style: None,
            last_distilled_at: Some(Utc::now()),
            skipped_tickers: holdings.to_vec(),
        };
    }

    // 并发蒸主线(每个独立 LLM call)
    use futures::stream::{self, StreamExt};
    let results: Vec<(String, anyhow::Result<String>)> = stream::iter(profiles.iter().cloned())
        .map(|p| async move {
            let r = distiller.distill_mainline(&p.ticker, &p.markdown).await;
            (p.ticker, r)
        })
        .buffer_unordered(6)
        .collect()
        .await;

    let mut by_ticker: HashMap<String, String> = HashMap::new();
    let mut skipped: Vec<String> = Vec::new();
    for (ticker, r) in results {
        match r {
            Ok(t) => {
                by_ticker.insert(ticker, t);
            }
            Err(e) => {
                tracing::warn!(ticker = %ticker, "mainline distill failed: {e}");
                skipped.push(ticker);
            }
        }
    }
    // holdings 里没有 profile 的 ticker 也算 skipped
    let covered: std::collections::HashSet<String> = by_ticker.keys().cloned().collect();
    for h in holdings {
        let h_up = h.to_uppercase();
        if !covered.contains(&h_up) && !skipped.contains(&h_up) {
            skipped.push(h_up);
        }
    }

    let style = match distiller.distill_style(&profiles).await {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!("mainline style distill failed: {e}");
            None
        }
    };

    DistilledMainlines {
        by_ticker,
        style,
        last_distilled_at: Some(Utc::now()),
        skipped_tickers: skipped,
    }
}

/// 计算 actor 的 sandbox 根目录:`{base}/{channel_fs}/{scoped_user_fs_key}`。
/// `base` 通常 = `{HONE_DATA_DIR}/agent-sandboxes`。
pub fn actor_sandbox_dir(base: &Path, actor: &ActorIdentity) -> PathBuf {
    base.join(actor.channel_fs_component())
        .join(actor.scoped_user_fs_key())
}

/// 一次性蒸馏单个 actor 并写回 prefs。封装 scan + LLM call + merge。
///
/// 适合 admin "立即跑一次" 端点 / cron job 内部循环。
pub async fn distill_and_persist_one(
    distiller: &dyn MainlineDistiller,
    prefs_storage: &dyn crate::prefs::PrefsProvider,
    sandbox_base: &Path,
    actor: &ActorIdentity,
    holdings: &[String],
) -> anyhow::Result<crate::prefs::NotificationPrefs> {
    let sandbox_root = actor_sandbox_dir(sandbox_base, actor);
    let distilled = distill_for_actor(distiller, &sandbox_root, holdings).await;
    merge_into_prefs(prefs_storage, actor, distilled)
}

/// 把蒸馏结果合并写回 actor 的 NotificationPrefs 文件。
///
/// 行为:
/// - 如果 `by_ticker` 非空 → 覆盖整个 `mainline_by_ticker` 字段(系统全权管)
/// - 如果 `by_ticker` 为空(扫不到任何画像) → **不覆盖** 现有主线,只更新 last_distilled_at
///   和 skipped 列表。这样用户单次画像目录被误删不会立刻丢历史主线。
/// - `style` 同样:有就覆盖,无就保留旧的
/// - 总是更新 `last_mainline_distilled_at` 和 `mainline_distill_skipped`
///
/// 返回写入后的 prefs 副本。
pub fn merge_into_prefs(
    prefs_storage: &dyn crate::prefs::PrefsProvider,
    actor: &ActorIdentity,
    distilled: DistilledMainlines,
) -> anyhow::Result<crate::prefs::NotificationPrefs> {
    let mut prefs = prefs_storage.load(actor);
    if !distilled.by_ticker.is_empty() {
        prefs.mainline_by_ticker = Some(distilled.by_ticker);
    }
    if distilled.style.is_some() {
        prefs.mainline_style = distilled.style;
    }
    prefs.last_mainline_distilled_at = distilled
        .last_distilled_at
        .map(|t| t.to_rfc3339())
        .or(prefs.last_mainline_distilled_at);
    prefs.mainline_distill_skipped = distilled.skipped_tickers;
    prefs_storage
        .save(actor, &prefs)
        .map_err(|e| anyhow::anyhow!("save prefs: {e}"))?;
    Ok(prefs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::{self, BoxStream};
    use hone_core::{HoneError, HoneResult};
    use hone_llm::{ChatResponse, provider::ChatResult};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    #[test]
    fn extract_tickers_from_yaml_frontmatter_single() {
        let md = "# Foo\n\nstatus: active\nticker: MU\ncompany_name: Micron\n";
        assert_eq!(extract_tickers(md), vec!["MU".to_string()]);
    }

    #[test]
    fn extract_tickers_from_yaml_frontmatter_multi() {
        let md = "# Alphabet / Google\n\nticker: GOOGL / GOOG\n";
        let t = extract_tickers(md);
        assert!(t.contains(&"GOOGL".into()));
        assert!(t.contains(&"GOOG".into()));
    }

    #[test]
    fn extract_tickers_from_paren_in_title() {
        let md = "# Rocket Lab (RKLB)\n\n## 投资主线\n";
        assert_eq!(extract_tickers(md), vec!["RKLB".to_string()]);
    }

    #[test]
    fn extract_tickers_handles_chinese_paren() {
        // 真·全角括号 (U+FF08 / U+FF09) - 这是生产里 rocket-lab / caris-life-sciences 的真实写法
        let md = "# Rocket Lab(RKLB)\n## 投资主线";
        assert_eq!(extract_tickers(md), vec!["RKLB".to_string()]);

        // mixed: 半角 + 中文括号
        let md2 = "# Caris Life Sciences(CAI)\n";
        assert_eq!(extract_tickers(md2), vec!["CAI".to_string()]);

        // 半角也兼容
        let md3 = "# Apple Inc (AAPL)\n";
        assert_eq!(extract_tickers(md3), vec!["AAPL".to_string()]);
    }

    #[test]
    fn extract_tickers_empty_when_no_marker() {
        let md = "Just a profile without any explicit ticker marker.";
        assert!(extract_tickers(md).is_empty());
    }

    #[test]
    fn extract_tickers_rejects_implausible_strings() {
        // "Annoyingly long ticker" 应被拒
        let md = "ticker: VERYLONGTICKERSTRING\n";
        assert!(extract_tickers(md).is_empty());
        let md2 = "ticker: 12345\n";
        assert!(extract_tickers(md2).is_empty()); // 全数字
    }

    #[test]
    fn scan_profiles_finds_all_in_directory() {
        let dir = tempdir().unwrap();
        let cp = dir.path().join("company_profiles");
        std::fs::create_dir(&cp).unwrap();
        for (name, content) in &[
            ("micron-technology", "# MU\n\nticker: MU\n\n投资主线"),
            ("rocket-lab", "# Rocket Lab (RKLB)\n\n投资主线"),
            ("alphabet", "ticker: GOOGL / GOOG\n"),
            ("garbage-no-ticker", "Just text without ticker marker"),
        ] {
            let sub = cp.join(name);
            std::fs::create_dir(&sub).unwrap();
            std::fs::write(sub.join("profile.md"), content).unwrap();
        }
        let profiles = scan_profiles(dir.path(), None);
        let tickers: Vec<&str> = profiles.iter().map(|p| p.ticker.as_str()).collect();
        assert!(tickers.contains(&"MU"));
        assert!(tickers.contains(&"RKLB"));
        assert!(tickers.contains(&"GOOGL"));
        assert!(tickers.contains(&"GOOG"));
        // garbage-no-ticker 应被跳过
        assert_eq!(profiles.len(), 4);
    }

    #[test]
    fn scan_profiles_filters_by_holdings() {
        let dir = tempdir().unwrap();
        let cp = dir.path().join("company_profiles");
        std::fs::create_dir(&cp).unwrap();
        for (name, content) in &[
            ("mu", "ticker: MU\n"),
            ("rklb", "ticker: RKLB\n"),
            ("aaoi", "ticker: AAOI\n"),
        ] {
            let sub = cp.join(name);
            std::fs::create_dir(&sub).unwrap();
            std::fs::write(sub.join("profile.md"), content).unwrap();
        }
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];
        let profiles = scan_profiles(dir.path(), Some(&holdings));
        assert_eq!(profiles.len(), 2);
        assert!(profiles.iter().any(|p| p.ticker == "MU"));
        assert!(profiles.iter().any(|p| p.ticker == "RKLB"));
        assert!(profiles.iter().all(|p| p.ticker != "AAOI"));
    }

    #[test]
    fn scan_profiles_returns_empty_when_no_dir() {
        let dir = tempdir().unwrap();
        // no company_profiles subdir
        let profiles = scan_profiles(dir.path(), None);
        assert!(profiles.is_empty());
    }

    // Mock distiller
    struct MockDistiller {
        mainline_calls: AtomicUsize,
        style_calls: AtomicUsize,
        fail_for_ticker: Option<String>,
    }
    #[async_trait]
    impl MainlineDistiller for MockDistiller {
        async fn distill_mainline(&self, ticker: &str, _profile: &str) -> anyhow::Result<String> {
            self.mainline_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_for_ticker.as_deref() == Some(ticker) {
                anyhow::bail!("simulated failure");
            }
            Ok(format!("mainline for {ticker}"))
        }
        async fn distill_style(&self, profiles: &[ProfileSource]) -> anyhow::Result<String> {
            self.style_calls.fetch_add(1, Ordering::SeqCst);
            Ok(format!("style covering {} tickers", profiles.len()))
        }
    }

    #[tokio::test]
    async fn distill_for_actor_happy_path() {
        let dir = tempdir().unwrap();
        let cp = dir.path().join("company_profiles");
        std::fs::create_dir(&cp).unwrap();
        for (name, content) in &[
            ("mu", "ticker: MU\n# Micron\nlong mainline content"),
            ("rklb", "ticker: RKLB\n# Rocket Lab"),
        ] {
            let sub = cp.join(name);
            std::fs::create_dir(&sub).unwrap();
            std::fs::write(sub.join("profile.md"), content).unwrap();
        }
        let mock = MockDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: None,
        };
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];
        let result = distill_for_actor(&mock, dir.path(), &holdings).await;
        assert_eq!(result.by_ticker.len(), 2);
        assert_eq!(result.by_ticker["MU"], "mainline for MU");
        assert!(result.style.is_some());
        assert!(result.last_distilled_at.is_some());
        assert!(result.skipped_tickers.is_empty());
        assert_eq!(mock.mainline_calls.load(Ordering::SeqCst), 2);
        assert_eq!(mock.style_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn distill_for_actor_skips_failing_ticker_keeps_others() {
        let dir = tempdir().unwrap();
        let cp = dir.path().join("company_profiles");
        std::fs::create_dir(&cp).unwrap();
        for (name, content) in &[("mu", "ticker: MU\n"), ("rklb", "ticker: RKLB\n")] {
            let sub = cp.join(name);
            std::fs::create_dir(&sub).unwrap();
            std::fs::write(sub.join("profile.md"), content).unwrap();
        }
        let mock = MockDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: Some("MU".into()),
        };
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];
        let result = distill_for_actor(&mock, dir.path(), &holdings).await;
        assert_eq!(result.by_ticker.len(), 1);
        assert_eq!(result.by_ticker["RKLB"], "mainline for RKLB");
        assert!(result.skipped_tickers.contains(&"MU".to_string()));
    }

    #[tokio::test]
    async fn distill_for_actor_marks_holding_without_profile_as_skipped() {
        let dir = tempdir().unwrap();
        let cp = dir.path().join("company_profiles");
        std::fs::create_dir(&cp).unwrap();
        let sub = cp.join("mu");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("profile.md"), "ticker: MU\n").unwrap();

        let mock = MockDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: None,
        };
        let holdings = vec!["MU".to_string(), "AAPL".to_string()];
        let result = distill_for_actor(&mock, dir.path(), &holdings).await;
        assert_eq!(result.by_ticker.len(), 1);
        assert!(result.skipped_tickers.contains(&"AAPL".to_string()));
    }

    #[tokio::test]
    async fn distill_for_actor_empty_dir_returns_empty_result() {
        let dir = tempdir().unwrap();
        let mock = MockDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: None,
        };
        let holdings = vec!["MU".to_string()];
        let result = distill_for_actor(&mock, dir.path(), &holdings).await;
        assert!(result.by_ticker.is_empty());
        assert!(result.style.is_none());
        assert!(result.skipped_tickers.contains(&"MU".to_string()));
        assert_eq!(mock.mainline_calls.load(Ordering::SeqCst), 0);
        assert_eq!(mock.style_calls.load(Ordering::SeqCst), 0);
    }

    /// 用真实 LlmMainlineDistiller 但用 mock provider,验证 prompt 构造正确。
    struct CapturePromptProvider {
        last_prompt: std::sync::Mutex<Option<String>>,
    }
    #[async_trait]
    impl LlmProvider for CapturePromptProvider {
        async fn chat(&self, messages: &[Message], _model: Option<&str>) -> HoneResult<ChatResult> {
            *self.last_prompt.lock().unwrap() = messages.first().and_then(|m| m.content.clone());
            Ok(ChatResult {
                content: "蒸馏出的 mainline".into(),
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

    #[tokio::test]
    async fn merge_into_prefs_overwrites_mainlines_and_style_when_present() {
        use crate::prefs::{FilePrefsStorage, PrefsProvider};
        let dir = tempdir().unwrap();
        let storage = FilePrefsStorage::new(dir.path()).unwrap();
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();

        let mut by_ticker = HashMap::new();
        by_ticker.insert("MU".to_string(), "MU mainline text".to_string());
        by_ticker.insert("RKLB".to_string(), "RKLB mainline text".to_string());
        let distilled = DistilledMainlines {
            by_ticker,
            style: Some("style text".into()),
            last_distilled_at: Some(Utc::now()),
            skipped_tickers: vec!["AAPL".into()],
        };
        let prefs = merge_into_prefs(&storage, &actor, distilled).unwrap();
        assert_eq!(prefs.mainline_by_ticker.as_ref().unwrap().len(), 2);
        assert_eq!(prefs.mainline_style.as_deref(), Some("style text"));
        assert!(prefs.last_mainline_distilled_at.is_some());
        assert_eq!(prefs.mainline_distill_skipped, vec!["AAPL".to_string()]);

        // 重新加载验证落盘
        let reloaded = storage.load(&actor);
        assert_eq!(reloaded.mainline_by_ticker.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn merge_into_prefs_preserves_old_mainlines_when_distilled_empty() {
        use crate::prefs::PrefsProvider;
        use crate::prefs::{FilePrefsStorage, NotificationPrefs};
        let dir = tempdir().unwrap();
        let storage = FilePrefsStorage::new(dir.path()).unwrap();
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();

        // 先写一份带主线的旧 prefs
        let mut old = NotificationPrefs::default();
        let mut old_by_ticker = HashMap::new();
        old_by_ticker.insert("MU".into(), "old MU mainline".into());
        old.mainline_by_ticker = Some(old_by_ticker);
        old.mainline_style = Some("old style".into());
        storage.save(&actor, &old).unwrap();

        // 蒸馏失败 → 空主线 + 空 style
        let distilled = DistilledMainlines {
            by_ticker: HashMap::new(),
            style: None,
            last_distilled_at: Some(Utc::now()),
            skipped_tickers: vec!["MU".into()],
        };
        let prefs = merge_into_prefs(&storage, &actor, distilled).unwrap();
        // 旧主线应保留(防止误删历史)
        assert_eq!(
            prefs.mainline_by_ticker.as_ref().unwrap()["MU"],
            "old MU mainline"
        );
        assert_eq!(prefs.mainline_style.as_deref(), Some("old style"));
        // skipped 仍更新 + last_distilled_at 仍写入
        assert_eq!(prefs.mainline_distill_skipped, vec!["MU".to_string()]);
        assert!(prefs.last_mainline_distilled_at.is_some());
    }

    #[tokio::test]
    async fn llm_distiller_substitutes_ticker_and_profile_into_prompt() {
        let provider = Arc::new(CapturePromptProvider {
            last_prompt: std::sync::Mutex::new(None),
        });
        let distiller = LlmMainlineDistiller::new(provider.clone(), "test-model");
        let result = distiller
            .distill_mainline("RKLB", "long profile content")
            .await
            .unwrap();
        assert_eq!(result, "蒸馏出的 mainline");
        let prompt = provider.last_prompt.lock().unwrap().clone().unwrap();
        assert!(prompt.contains("RKLB"));
        assert!(prompt.contains("long profile content"));
        assert!(!prompt.contains("{{TICKER}}")); // template var 应已替换
        assert!(!prompt.contains("{{PROFILE}}"));
    }

    #[test]
    fn distilled_mainlines_loads_legacy_field_names_via_alias() {
        // 旧 DistilledTheses 序列化结构体可能存在内存运行时序列化(eg. test fixture / 跨进程传递);
        // serde alias 兼容旧字段名 theses / global_style 即可平滑加载。
        let json = r#"{
            "theses": {"MU": "看 NAND 长期稀缺"},
            "global_style": "长期叙事派",
            "last_distilled_at": null,
            "skipped_tickers": []
        }"#;
        let d: DistilledMainlines =
            serde_json::from_str(json).expect("legacy DistilledTheses JSON should load");
        assert_eq!(
            d.by_ticker.get("MU").map(String::as_str),
            Some("看 NAND 长期稀缺")
        );
        assert_eq!(d.style.as_deref(), Some("长期叙事派"));
    }
}
