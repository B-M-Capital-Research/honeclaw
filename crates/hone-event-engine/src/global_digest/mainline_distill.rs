//! 投资主线自动蒸馏 —— 把用户在 agent-sandbox 里写的 100-400 行公司画像,定期
//! 蒸馏成 Pass2 personalize 期望的 1-2 句投资主线,写入 `NotificationPrefs.
//! mainline_by_ticker` 字段。同时跨 ticker 提取 `mainline_style`。
//!
//! 设计要点:
//! - **read-only on profile.md**:本模块只读用户画像,不改;写出方向只到 prefs。
//! - **per-actor**:每个有 portfolio 的 actor 跑一次,扫他的 sandbox profile 目录。
//! - **失败降级**:任一 ticker 蒸馏失败 → 标入 skipped,继续处理其他 ticker。
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
use hone_memory::CompanyProfileStorage;
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
    /// `by_ticker` 中每条主线的来源画像内容哈希。增量蒸馏的判据,随主线一起持久化。
    #[serde(default)]
    pub source_hashes: HashMap<String, String>,
    /// 本轮 distiller 指纹。`None` = 空轮(无画像),不更新存量指纹。
    #[serde(default)]
    pub fingerprint: Option<String>,
}

/// 上一轮蒸馏的持久化状态视图,驱动增量决策。全 `None` = 无历史 ⇒ 全量蒸馏,
/// 这也保证 [`distill_for_actor`] 等无 prefs 上下文的入口行为与历史版本一致。
#[derive(Debug, Clone, Copy, Default)]
pub struct PriorMainlines<'a> {
    pub by_ticker: Option<&'a HashMap<String, String>>,
    pub source_hashes: Option<&'a HashMap<String, String>>,
    pub fingerprint: Option<&'a str>,
}

impl<'a> PriorMainlines<'a> {
    pub fn from_prefs(prefs: &'a crate::prefs::NotificationPrefs) -> Self {
        Self {
            by_ticker: prefs.mainline_by_ticker.as_ref(),
            source_hashes: prefs.mainline_source_hashes.as_ref(),
            fingerprint: prefs.mainline_distill_fingerprint.as_deref(),
        }
    }
}

/// profile.md 内容哈希。持久化在 prefs 里跨进程/跨后端比较,必须是稳定算法,
/// 不能用 `std::hash`(其输出不保证跨版本稳定)。
pub fn profile_content_hash(markdown: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(markdown.as_bytes());
    format!("{digest:x}")
}

/// 蒸馏抽象,生产实现走 LLM,测试可注入 stub。
#[async_trait]
pub trait MainlineDistiller: Send + Sync {
    async fn distill_mainline(&self, ticker: &str, profile_md: &str) -> anyhow::Result<String>;
    async fn distill_style(&self, all_profiles: &[ProfileSource]) -> anyhow::Result<String>;
    /// 输出语义指纹(模型 + prompt 模板)。指纹变化 ⇒ 增量失效、全量重蒸,
    /// 保证模型/prompt 升级能刷新存量主线。默认空串:stub 恒定 ⇒ 恒定指纹。
    fn fingerprint(&self) -> String {
        String::new()
    }
}

/// LLM 实现 —— 默认走当前 OpenRouter 可用的 grok 级模型。
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
    fn fingerprint(&self) -> String {
        profile_content_hash(&format!("{}\n{DISTILL_PROMPT}\n{STYLE_PROMPT}", self.model))
    }

    async fn distill_mainline(&self, ticker: &str, profile_md: &str) -> anyhow::Result<String> {
        let prompt = DISTILL_PROMPT
            .replace("{{TICKER}}", ticker)
            .replace("{{PROFILE}}", profile_md);
        let messages = vec![Message {
            images: Vec::new(),
            role: "user".into(),
            content: Some(prompt),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let llm_response = self
            .provider
            .chat(&messages, Some(&self.model))
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {e}"))?;
        let trimmed = llm_response.content.trim().to_string();
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
            images: Vec::new(),
            role: "user".into(),
            content: Some(prompt),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let llm_response = self
            .provider
            .chat(&messages, Some(&self.model))
            .await
            .map_err(|e| anyhow::anyhow!("LLM call failed: {e}"))?;
        let trimmed = llm_response.content.trim().to_string();
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
    let mut profiles = Vec::new();
    let entries = match std::fs::read_dir(&cp_dir) {
        Ok(entries) => entries,
        Err(_) => return profiles,
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
        for ticker in tickers {
            if let Some(filter) = &holdings_set
                && !filter.contains(&ticker)
            {
                continue;
            }
            profiles.push(ProfileSource {
                ticker,
                dir_name: dir_name.clone(),
                markdown: markdown.clone(),
            });
        }
    }
    profiles
}

/// 从 profile.md 解析出 ticker 列表(可能 ≥1)。
pub fn extract_tickers(md: &str) -> Vec<String> {
    // 1. YAML frontmatter `ticker: X` 或 `ticker: X / Y`
    for line in md.lines().take(20) {
        let trimmed_line = line.trim();
        if let Some(rest) = trimmed_line
            .strip_prefix("ticker:")
            .or_else(|| trimmed_line.strip_prefix("Ticker:"))
        {
            let raw_tickers = rest.trim().trim_matches('"').trim_matches('\'');
            return parse_ticker_list(raw_tickers);
        }
    }
    // 2. 标题里的 (TICKER) / (TICKER) — 第一行 `# Foo (TICKER)` 或 `# Foo（TICKER）`
    if let Some(first) = md.lines().find(|l| l.starts_with("# "))
        && let Some(t) = extract_paren_ticker(first)
    {
        return vec![t];
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
    if let (Some(start), Some(end)) = (line.rfind('('), line.rfind(')'))
        && end > start
    {
        let candidate = &line[start + 1..end];
        if is_plausible_ticker(candidate) {
            return Some(candidate.to_uppercase());
        }
    }
    // 全角 — U+FF08 / U+FF09(每个 3 字节 UTF-8)
    let lp = '\u{FF08}';
    let rp = '\u{FF09}';
    if let (Some(start), Some(end)) = (line.rfind(lp), line.rfind(rp))
        && end > start
    {
        let candidate = &line[start + lp.len_utf8()..end];
        if is_plausible_ticker(candidate) {
            return Some(candidate.to_uppercase());
        }
    }
    None
}

fn is_plausible_ticker(raw_ticker: &str) -> bool {
    let trimmed_ticker = raw_ticker.trim();
    !trimmed_ticker.is_empty()
        && trimmed_ticker.len() <= 6
        && trimmed_ticker
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && trimmed_ticker.chars().any(|c| c.is_ascii_alphabetic())
}

/// 蒸馏一个 actor 的所有持仓主线 + 整体风格。`holdings` 决定要蒸哪些 ticker;
/// `provider`/`model` 注入由调用方组装。
///
/// 行为:
/// 1. scan_profiles(sandbox_root, Some(holdings)) → ProfileSource 列表
/// 2. 并发蒸馏每只 ticker 的主线(任一失败 → 加入 skipped,继续)
/// 3. 用全部 profile 蒸馏一条整体风格(失败 → None,merge 时保留旧 style)
/// 4. 返回 DistilledMainlines(可序列化,调用方 merge 进 prefs JSON)
pub async fn distill_for_actor(
    distiller: &dyn MainlineDistiller,
    sandbox_root: &Path,
    holdings: &[String],
) -> DistilledMainlines {
    let profiles = scan_profiles(sandbox_root, Some(holdings));
    distill_from_profiles(distiller, profiles, holdings, PriorMainlines::default()).await
}

pub async fn distill_from_profiles(
    distiller: &dyn MainlineDistiller,
    profiles: Vec<ProfileSource>,
    holdings: &[String],
    prior: PriorMainlines<'_>,
) -> DistilledMainlines {
    if profiles.is_empty() {
        return DistilledMainlines {
            by_ticker: HashMap::new(),
            style: None,
            last_distilled_at: Some(Utc::now()),
            skipped_tickers: holdings.to_vec(),
            source_hashes: HashMap::new(),
            fingerprint: None,
        };
    }

    // 增量决策:指纹一致时,内容哈希未变且已覆盖的 ticker 沿用旧主线,不发 LLM。
    // 生产常态是"持仓有缺画像的 ticker ⇒ 每 6h 触发一次",没有这层判断时每次
    // 触发都会把全部未变化画像重蒸一遍(实测 ~2000 次/天的纯重复调用)。
    let current_fingerprint = distiller.fingerprint();
    let fingerprint_matches = prior.fingerprint == Some(current_fingerprint.as_str());
    let hashed: Vec<(ProfileSource, String)> = profiles
        .iter()
        .map(|profile| (profile.clone(), profile_content_hash(&profile.markdown)))
        .collect();

    let mut by_ticker: HashMap<String, String> = HashMap::new();
    let mut source_hashes: HashMap<String, String> = HashMap::new();
    let mut to_distill: Vec<(ProfileSource, String)> = Vec::new();
    for (profile, content_hash) in hashed {
        let reusable = fingerprint_matches
            && prior
                .source_hashes
                .and_then(|hashes| hashes.get(&profile.ticker))
                == Some(&content_hash);
        let prior_text = reusable.then(|| {
            prior
                .by_ticker
                .and_then(|map| map.get(&profile.ticker))
                .cloned()
        });
        match prior_text.flatten() {
            Some(text) => {
                by_ticker.insert(profile.ticker.clone(), text);
                source_hashes.insert(profile.ticker, content_hash);
            }
            None => to_distill.push((profile, content_hash)),
        }
    }
    let anything_changed = !to_distill.is_empty();

    // 并发蒸主线(每个独立 LLM call)
    use futures::stream::{self, StreamExt};
    let ticker_distill_results: Vec<(String, String, anyhow::Result<String>)> =
        stream::iter(to_distill)
            .map(|(profile, content_hash)| async move {
                let distill_result = distiller
                    .distill_mainline(&profile.ticker, &profile.markdown)
                    .await;
                (profile.ticker, content_hash, distill_result)
            })
            .buffer_unordered(6)
            .collect()
            .await;

    let mut skipped_tickers: Vec<String> = Vec::new();
    for (ticker, content_hash, distill_result) in ticker_distill_results {
        match distill_result {
            Ok(mainline) => {
                by_ticker.insert(ticker.clone(), mainline);
                // 失败的 ticker 不落 hash ⇒ 下轮触发时仍会重试,与历史重试语义一致。
                source_hashes.insert(ticker, content_hash);
            }
            Err(e) => {
                tracing::warn!(ticker = %ticker, "mainline distill failed: {e}");
                skipped_tickers.push(ticker);
            }
        }
    }
    // holdings 里没有 profile 的 ticker 也算 skipped
    let distilled_tickers: std::collections::HashSet<String> = by_ticker.keys().cloned().collect();
    for holding_ticker in holdings {
        let normalized_ticker = holding_ticker.to_uppercase();
        if !distilled_tickers.contains(&normalized_ticker)
            && !skipped_tickers.contains(&normalized_ticker)
        {
            skipped_tickers.push(normalized_ticker);
        }
    }

    // style 覆盖全部画像:有新蒸的、或画像集合本身变了(增删/改动)才需要重算;
    // 否则返回 None,merge 侧语义是"保留旧 style"。
    let hash_set_changed = prior
        .source_hashes
        .map(|prior_hashes| {
            prior_hashes.len() != source_hashes.len()
                || source_hashes
                    .iter()
                    .any(|(ticker, hash)| prior_hashes.get(ticker) != Some(hash))
        })
        .unwrap_or(true);
    let style = if anything_changed || hash_set_changed || !fingerprint_matches {
        match distiller.distill_style(&profiles).await {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::warn!("mainline style distill failed: {e}");
                None
            }
        }
    } else {
        None
    };

    DistilledMainlines {
        by_ticker,
        style,
        last_distilled_at: Some(Utc::now()),
        skipped_tickers,
        source_hashes,
        fingerprint: Some(current_fingerprint),
    }
}

/// 计算 actor 的 sandbox 根目录:`{base}/{channel_fs}/{scoped_user_fs_key}`。
/// `base` 通常 = `{HONE_DATA_DIR}/agent-sandboxes`。
pub fn actor_sandbox_dir(base: &Path, actor: &ActorIdentity) -> PathBuf {
    base.join(actor.channel_fs_component())
        .join(actor.scoped_user_fs_key())
}

pub async fn scan_profiles_for_actor(
    sandbox_base: &Path,
    actor: &ActorIdentity,
    holdings_filter: Option<&[String]>,
) -> Vec<ProfileSource> {
    let storage = CompanyProfileStorage::new(sandbox_base).for_actor(actor);
    let holdings_set: Option<std::collections::HashSet<String>> =
        holdings_filter.map(|hs| hs.iter().map(|h| h.to_uppercase()).collect());
    let mut profiles = Vec::new();
    for document in storage.list_profile_documents_raw().await {
        let tickers = extract_tickers(&document.markdown);
        if tickers.is_empty() {
            tracing::warn!(
                dir = %document.profile_id,
                "mainline_distill: profile.md 没找到 ticker 标识,跳过"
            );
            continue;
        }
        for ticker in tickers {
            if let Some(filter) = &holdings_set
                && !filter.contains(&ticker)
            {
                continue;
            }
            profiles.push(ProfileSource {
                ticker,
                dir_name: document.profile_id.clone(),
                markdown: document.markdown.clone(),
            });
        }
    }
    profiles
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
    let profiles = scan_profiles_for_actor(sandbox_base, actor, Some(holdings)).await;
    // prefs 只 load 一次:既做增量蒸馏的先验,也做 merge 的底稿。
    let mut prefs = prefs_storage.load(actor).await;
    let distilled_mainlines = distill_from_profiles(
        distiller,
        profiles,
        holdings,
        PriorMainlines::from_prefs(&prefs),
    )
    .await;
    apply_distilled(&mut prefs, distilled_mainlines);
    prefs_storage
        .save(actor, &prefs)
        .await
        .map_err(|e| anyhow::anyhow!("save prefs: {e}"))?;
    Ok(prefs)
}

/// 把蒸馏结果合并进 prefs(纯内存,调用方负责持久化)。
///
/// 行为:
/// - 如果 `by_ticker` 非空 → 覆盖整个 `mainline_by_ticker` 字段(系统全权管)。
/// - 如果 `by_ticker` 为空(没有可写入的新主线) → **不覆盖** 现有主线,只更新
///   distill 时间和 skipped 列表。这样用户单次画像目录被误删不会立刻丢历史主线。
/// - `style` 同样:有就覆盖,无就保留旧的。
/// - `last_mainline_distilled_at` 只在结果携带时间戳时更新;`mainline_distill_skipped`
///   每次 merge 都替换为本轮 skipped 列表。
pub fn apply_distilled(prefs: &mut crate::prefs::NotificationPrefs, distilled: DistilledMainlines) {
    if !distilled.by_ticker.is_empty() {
        prefs.mainline_by_ticker = Some(distilled.by_ticker);
        prefs.mainline_source_hashes = Some(distilled.source_hashes);
        // 指纹只随成功产物一起前进:全失败轮保留旧 map 时若写入新指纹,
        // 下一轮会把旧模型的产物误判为已刷新,升级通道就断了。
        if let Some(fingerprint) = distilled.fingerprint {
            prefs.mainline_distill_fingerprint = Some(fingerprint);
        }
    }
    if distilled.style.is_some() {
        prefs.mainline_style = distilled.style;
    }
    prefs.last_mainline_distilled_at = distilled
        .last_distilled_at
        .map(|t| t.to_rfc3339())
        .or(prefs.last_mainline_distilled_at.take());
    prefs.mainline_distill_skipped = distilled.skipped_tickers;
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
        let tickers = extract_tickers(md);
        assert!(tickers.contains(&"GOOGL".into()));
        assert!(tickers.contains(&"GOOG".into()));
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
        let profiles_dir = dir.path().join("company_profiles");
        std::fs::create_dir(&profiles_dir).unwrap();
        for (name, content) in &[
            ("micron-technology", "# MU\n\nticker: MU\n\n投资主线"),
            ("rocket-lab", "# Rocket Lab (RKLB)\n\n投资主线"),
            ("alphabet", "ticker: GOOGL / GOOG\n"),
            ("garbage-no-ticker", "Just text without ticker marker"),
        ] {
            let profile_dir = profiles_dir.join(name);
            std::fs::create_dir(&profile_dir).unwrap();
            std::fs::write(profile_dir.join("profile.md"), content).unwrap();
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
        let profiles_dir = dir.path().join("company_profiles");
        std::fs::create_dir(&profiles_dir).unwrap();
        for (name, content) in &[
            ("mu", "ticker: MU\n"),
            ("rklb", "ticker: RKLB\n"),
            ("aaoi", "ticker: AAOI\n"),
        ] {
            let profile_dir = profiles_dir.join(name);
            std::fs::create_dir(&profile_dir).unwrap();
            std::fs::write(profile_dir.join("profile.md"), content).unwrap();
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

    // Counting test distiller
    struct CountingDistiller {
        mainline_calls: AtomicUsize,
        style_calls: AtomicUsize,
        fail_for_ticker: Option<String>,
    }
    #[async_trait]
    impl MainlineDistiller for CountingDistiller {
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
        let profiles_dir = dir.path().join("company_profiles");
        std::fs::create_dir(&profiles_dir).unwrap();
        for (name, content) in &[
            ("mu", "ticker: MU\n# Micron\nlong mainline content"),
            ("rklb", "ticker: RKLB\n# Rocket Lab"),
        ] {
            let profile_dir = profiles_dir.join(name);
            std::fs::create_dir(&profile_dir).unwrap();
            std::fs::write(profile_dir.join("profile.md"), content).unwrap();
        }
        let distiller = CountingDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: None,
        };
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];
        let distilled = distill_for_actor(&distiller, dir.path(), &holdings).await;
        assert_eq!(distilled.by_ticker.len(), 2);
        assert_eq!(distilled.by_ticker["MU"], "mainline for MU");
        assert!(distilled.style.is_some());
        assert!(distilled.last_distilled_at.is_some());
        assert!(distilled.skipped_tickers.is_empty());
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 2);
        assert_eq!(distiller.style_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn distill_for_actor_skips_failing_ticker_keeps_others() {
        let dir = tempdir().unwrap();
        let profiles_dir = dir.path().join("company_profiles");
        std::fs::create_dir(&profiles_dir).unwrap();
        for (name, content) in &[("mu", "ticker: MU\n"), ("rklb", "ticker: RKLB\n")] {
            let profile_dir = profiles_dir.join(name);
            std::fs::create_dir(&profile_dir).unwrap();
            std::fs::write(profile_dir.join("profile.md"), content).unwrap();
        }
        let distiller = CountingDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: Some("MU".into()),
        };
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];
        let distilled = distill_for_actor(&distiller, dir.path(), &holdings).await;
        assert_eq!(distilled.by_ticker.len(), 1);
        assert_eq!(distilled.by_ticker["RKLB"], "mainline for RKLB");
        assert!(distilled.skipped_tickers.contains(&"MU".to_string()));
    }

    #[tokio::test]
    async fn distill_for_actor_marks_holding_without_profile_as_skipped() {
        let dir = tempdir().unwrap();
        let profiles_dir = dir.path().join("company_profiles");
        std::fs::create_dir(&profiles_dir).unwrap();
        let profile_dir = profiles_dir.join("mu");
        std::fs::create_dir(&profile_dir).unwrap();
        std::fs::write(profile_dir.join("profile.md"), "ticker: MU\n").unwrap();

        let distiller = CountingDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: None,
        };
        let holdings = vec!["MU".to_string(), "AAPL".to_string()];
        let distilled = distill_for_actor(&distiller, dir.path(), &holdings).await;
        assert_eq!(distilled.by_ticker.len(), 1);
        assert!(distilled.skipped_tickers.contains(&"AAPL".to_string()));
    }

    #[tokio::test]
    async fn distill_for_actor_empty_dir_returns_empty_result() {
        let dir = tempdir().unwrap();
        let distiller = CountingDistiller {
            mainline_calls: AtomicUsize::new(0),
            style_calls: AtomicUsize::new(0),
            fail_for_ticker: None,
        };
        let holdings = vec!["MU".to_string()];
        let distilled = distill_for_actor(&distiller, dir.path(), &holdings).await;
        assert!(distilled.by_ticker.is_empty());
        assert!(distilled.style.is_none());
        assert!(distilled.skipped_tickers.contains(&"MU".to_string()));
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 0);
        assert_eq!(distiller.style_calls.load(Ordering::SeqCst), 0);
    }

    /// 用真实 LlmMainlineDistiller 但用 capturing provider,验证 prompt 构造正确。
    struct CapturingPromptProvider {
        captured_prompt: std::sync::Mutex<Option<String>>,
    }
    #[async_trait]
    impl LlmProvider for CapturingPromptProvider {
        async fn chat(&self, messages: &[Message], _model: Option<&str>) -> HoneResult<ChatResult> {
            *self.captured_prompt.lock().unwrap() =
                messages.first().and_then(|m| m.content.clone());
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
    async fn apply_distilled_overwrites_mainlines_and_style_when_present() {
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
            ..Default::default()
        };
        let mut prefs = storage.load(&actor).await;
        apply_distilled(&mut prefs, distilled);
        storage.save(&actor, &prefs).await.unwrap();
        assert_eq!(prefs.mainline_by_ticker.as_ref().unwrap().len(), 2);
        assert_eq!(prefs.mainline_style.as_deref(), Some("style text"));
        assert!(prefs.last_mainline_distilled_at.is_some());
        assert_eq!(prefs.mainline_distill_skipped, vec!["AAPL".to_string()]);

        // 重新加载验证落盘
        let reloaded = storage.load(&actor).await;
        assert_eq!(reloaded.mainline_by_ticker.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn apply_distilled_preserves_old_mainlines_when_distilled_empty() {
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
        storage.save(&actor, &old).await.unwrap();

        // 蒸馏失败 → 空主线 + 空 style
        let distilled = DistilledMainlines {
            by_ticker: HashMap::new(),
            style: None,
            last_distilled_at: Some(Utc::now()),
            skipped_tickers: vec!["MU".into()],
            ..Default::default()
        };
        let mut prefs = storage.load(&actor).await;
        apply_distilled(&mut prefs, distilled);
        storage.save(&actor, &prefs).await.unwrap();
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
        let provider = Arc::new(CapturingPromptProvider {
            captured_prompt: std::sync::Mutex::new(None),
        });
        let distiller = LlmMainlineDistiller::new(provider.clone(), "test-model");
        let distilled_mainline = distiller
            .distill_mainline("RKLB", "long profile content")
            .await
            .unwrap();
        assert_eq!(distilled_mainline, "蒸馏出的 mainline");
        let prompt = provider.captured_prompt.lock().unwrap().clone().unwrap();
        assert!(prompt.contains("RKLB"));
        assert!(prompt.contains("long profile content"));
        assert!(!prompt.contains("{{TICKER}}")); // template var 应已替换
        assert!(!prompt.contains("{{PROFILE}}"));
    }

    /// 增量蒸馏专用 stub:计数 + 可配置指纹。
    struct IncrementalDistiller {
        mainline_calls: AtomicUsize,
        style_calls: AtomicUsize,
        fingerprint: String,
    }
    impl IncrementalDistiller {
        fn new(fingerprint: &str) -> Self {
            Self {
                mainline_calls: AtomicUsize::new(0),
                style_calls: AtomicUsize::new(0),
                fingerprint: fingerprint.to_string(),
            }
        }
    }
    #[async_trait]
    impl MainlineDistiller for IncrementalDistiller {
        async fn distill_mainline(&self, ticker: &str, _profile: &str) -> anyhow::Result<String> {
            let n = self.mainline_calls.fetch_add(1, Ordering::SeqCst);
            Ok(format!("mainline for {ticker} (call {n})"))
        }
        async fn distill_style(&self, profiles: &[ProfileSource]) -> anyhow::Result<String> {
            let n = self.style_calls.fetch_add(1, Ordering::SeqCst);
            Ok(format!("style over {} profiles (call {n})", profiles.len()))
        }
        fn fingerprint(&self) -> String {
            self.fingerprint.clone()
        }
    }

    fn write_actor_profile(
        sandbox_base: &Path,
        actor: &ActorIdentity,
        dir_name: &str,
        content: &str,
    ) {
        let dir = actor_sandbox_dir(sandbox_base, actor)
            .join("company_profiles")
            .join(dir_name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("profile.md"), content).unwrap();
    }

    fn incremental_fixture() -> (
        tempfile::TempDir,
        crate::prefs::FilePrefsStorage,
        ActorIdentity,
    ) {
        let dir = tempdir().unwrap();
        let storage = crate::prefs::FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        (dir, storage, actor)
    }

    #[tokio::test]
    async fn second_run_with_unchanged_profiles_makes_zero_llm_calls() {
        let (dir, storage, actor) = incremental_fixture();
        let sandbox = dir.path().join("sandbox");
        write_actor_profile(&sandbox, &actor, "mu", "ticker: MU\n# Micron\n看 NAND 稀缺");
        write_actor_profile(
            &sandbox,
            &actor,
            "rklb",
            "ticker: RKLB\n# Rocket Lab\n看发射节奏",
        );
        let distiller = IncrementalDistiller::new("fp-a");
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];

        let first = distill_and_persist_one(&distiller, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        assert_eq!(first.mainline_by_ticker.as_ref().unwrap().len(), 2);
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 2);
        assert_eq!(distiller.style_calls.load(Ordering::SeqCst), 1);

        let second = distill_and_persist_one(&distiller, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        // 内容未变 ⇒ 第二轮零 LLM 调用,主线与 style 原样保留
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 2);
        assert_eq!(distiller.style_calls.load(Ordering::SeqCst), 1);
        assert_eq!(second.mainline_by_ticker, first.mainline_by_ticker);
        assert_eq!(second.mainline_style, first.mainline_style);
        // 时间戳仍要前进:cron 的节流依赖它
        assert!(second.last_mainline_distilled_at.is_some());
    }

    #[tokio::test]
    async fn editing_one_profile_redistills_only_that_ticker() {
        let (dir, storage, actor) = incremental_fixture();
        let sandbox = dir.path().join("sandbox");
        write_actor_profile(&sandbox, &actor, "mu", "ticker: MU\n初版内容");
        write_actor_profile(&sandbox, &actor, "rklb", "ticker: RKLB\n初版内容");
        let distiller = IncrementalDistiller::new("fp-a");
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];

        let first = distill_and_persist_one(&distiller, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        let mu_before = first.mainline_by_ticker.as_ref().unwrap()["MU"].clone();

        write_actor_profile(&sandbox, &actor, "rklb", "ticker: RKLB\n改过的内容");
        let second = distill_and_persist_one(&distiller, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        // 恰好多 1 次主线调用(RKLB),MU 主线文本原样;画像集变了 ⇒ style 重算
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 3);
        assert_eq!(distiller.style_calls.load(Ordering::SeqCst), 2);
        assert_eq!(second.mainline_by_ticker.as_ref().unwrap()["MU"], mu_before);
        assert_ne!(
            second.mainline_by_ticker.as_ref().unwrap()["RKLB"],
            first.mainline_by_ticker.as_ref().unwrap()["RKLB"]
        );
    }

    #[tokio::test]
    async fn fingerprint_change_forces_full_redistill() {
        let (dir, storage, actor) = incremental_fixture();
        let sandbox = dir.path().join("sandbox");
        write_actor_profile(&sandbox, &actor, "mu", "ticker: MU\n内容");
        write_actor_profile(&sandbox, &actor, "rklb", "ticker: RKLB\n内容");
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];

        let distiller_v1 = IncrementalDistiller::new("fp-v1");
        distill_and_persist_one(&distiller_v1, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        assert_eq!(distiller_v1.mainline_calls.load(Ordering::SeqCst), 2);

        // 模型/prompt 升级 ⇒ 指纹变化 ⇒ 内容没变也要全量重蒸
        let distiller_v2 = IncrementalDistiller::new("fp-v2");
        distill_and_persist_one(&distiller_v2, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        assert_eq!(distiller_v2.mainline_calls.load(Ordering::SeqCst), 2);
        assert_eq!(distiller_v2.style_calls.load(Ordering::SeqCst), 1);

        // 升级完成后回到增量:同指纹再跑一轮 ⇒ 0 调用
        let distiller_v2_again = IncrementalDistiller::new("fp-v2");
        distill_and_persist_one(&distiller_v2_again, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        assert_eq!(distiller_v2_again.mainline_calls.load(Ordering::SeqCst), 0);
        assert_eq!(distiller_v2_again.style_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn uncovered_holding_does_not_trigger_redistill_of_covered_tickers() {
        // 生产常态:持仓里有永远没画像的 ticker(6h 重试永动机的根源)。
        // 已覆盖 ticker 不得因此被反复重蒸。
        let (dir, storage, actor) = incremental_fixture();
        let sandbox = dir.path().join("sandbox");
        write_actor_profile(&sandbox, &actor, "mu", "ticker: MU\n内容");
        let distiller = IncrementalDistiller::new("fp-a");
        let holdings = vec!["MU".to_string(), "AAPL".to_string()];

        for _ in 0..3 {
            let prefs = distill_and_persist_one(&distiller, &storage, &sandbox, &actor, &holdings)
                .await
                .unwrap();
            assert!(prefs.mainline_distill_skipped.contains(&"AAPL".to_string()));
        }
        // 3 轮下来 MU 只蒸了第一轮那 1 次
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 1);
        assert_eq!(distiller.style_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn legacy_prefs_without_hashes_full_distill_once_then_incremental() {
        let (dir, storage, actor) = incremental_fixture();
        let sandbox = dir.path().join("sandbox");
        write_actor_profile(&sandbox, &actor, "mu", "ticker: MU\n内容");
        // 老数据:有主线、无 source_hashes / fingerprint
        use crate::prefs::PrefsProvider;
        let mut legacy = crate::prefs::NotificationPrefs::default();
        legacy.mainline_by_ticker = Some(HashMap::from([("MU".to_string(), "旧主线".to_string())]));
        assert!(legacy.mainline_source_hashes.is_none());
        storage.save(&actor, &legacy).await.unwrap();

        let distiller = IncrementalDistiller::new("fp-a");
        let holdings = vec!["MU".to_string()];
        let first = distill_and_persist_one(&distiller, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        // 无先验哈希 ⇒ 首轮全量蒸(旧主线被替换)
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 1);
        assert_ne!(first.mainline_by_ticker.as_ref().unwrap()["MU"], "旧主线");

        distill_and_persist_one(&distiller, &storage, &sandbox, &actor, &holdings)
            .await
            .unwrap();
        // 之后进入增量
        assert_eq!(distiller.mainline_calls.load(Ordering::SeqCst), 1);
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
        let distilled: DistilledMainlines =
            serde_json::from_str(json).expect("legacy DistilledTheses JSON should load");
        assert_eq!(
            distilled.by_ticker.get("MU").map(String::as_str),
            Some("看 NAND 长期稀缺")
        );
        assert_eq!(distilled.style.as_deref(), Some("长期叙事派"));
    }
}
