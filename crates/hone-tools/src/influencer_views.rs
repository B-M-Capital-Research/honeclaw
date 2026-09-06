//! InfluencerViewsTool —— 把研究台「大V速报」的快照开放给对话。
//!
//! 速报 worker 每十几分钟把注册作者（Serenity/白毛、SemiAnalysis）的公开推文与
//! 文章写进数据目录 `influencer_digest/latest.json`（近 7 天）和 `history/<日期>.json`。
//! 这个工具只读这些文件，按 symbol / 关键词 / 天数过滤后返回带日期与原链的条目，
//! 让「MU 最近有什么消息」「白毛怎么看光模块」这类问题能把大V近期观点当**线索**带上。
//!
//! 它不抓网、不调模型、不写任何东西；返回的是作者观点，不是事实，也不是 HONE 结论。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::base::{Tool, ToolParameter};

const DEFAULT_DAYS: i64 = 14;
const MAX_DAYS: i64 = 30;
const DEFAULT_LIMIT: usize = 8;
const MAX_LIMIT: usize = 20;
/// Enough to carry a whole post; the panel shows the full text anyway.
const MAX_TEXT_CHARS: usize = 700;
const FOCUS_ITEMS: usize = 8;

pub struct InfluencerViewsTool {
    data_root: PathBuf,
}

impl InfluencerViewsTool {
    pub fn new(data_root: PathBuf) -> Self {
        Self { data_root }
    }

    fn digest_root(&self) -> PathBuf {
        self.data_root.join("influencer_digest")
    }
}

#[derive(Debug, Default, Deserialize)]
struct Snapshot {
    #[serde(default)]
    generated_at_local: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    refresh_interval_minutes: u32,
    #[serde(default)]
    items: Vec<Item>,
    #[serde(default)]
    disclaimer: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct Metrics {
    #[serde(default)]
    views: u64,
    #[serde(default)]
    likes: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct Item {
    id: String,
    #[serde(default)]
    author_name: String,
    #[serde(default)]
    public_handle: String,
    #[serde(default)]
    title: String,
    published_at: DateTime<Utc>,
    #[serde(default)]
    published_at_local: String,
    #[serde(default)]
    source_url: String,
    #[serde(default)]
    aggregation_source: Option<String>,
    #[serde(default)]
    post_kind: String,
    #[serde(default)]
    source_excerpt: String,
    #[serde(default)]
    source_text_cn: String,
    #[serde(default)]
    source_text_en: String,
    #[serde(default)]
    metrics: Metrics,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    stance: String,
    #[serde(default)]
    horizon: String,
    #[serde(default)]
    content_type: String,
    #[serde(default)]
    topics: Vec<String>,
    #[serde(default)]
    tickers: Vec<String>,
    #[serde(default)]
    counterpoint: String,
    #[serde(default)]
    analysis_status: String,
}

impl Item {
    fn text(&self) -> &str {
        [
            self.source_text_cn.as_str(),
            self.source_text_en.as_str(),
            self.source_excerpt.as_str(),
            self.title.as_str(),
        ]
        .into_iter()
        .map(str::trim)
        .find(|value| !value.is_empty())
        .unwrap_or_default()
    }

    fn searchable(&self) -> String {
        let mut haystack = format!(
            "{}\n{}\n{}\n{}\n{}",
            self.title, self.source_text_cn, self.source_text_en, self.source_excerpt, self.summary
        );
        for topic in &self.topics {
            haystack.push('\n');
            haystack.push_str(topic);
        }
        haystack
    }

    /// Same rule the digest applies before it exposes a ticker: a cashtag
    /// anywhere, a listed ticker, or a whole-token match for symbols of three
    /// letters and up. Two-letter symbols such as AI or BE are ordinary words.
    fn mentions_symbol(&self, symbol: &str) -> bool {
        if self
            .tickers
            .iter()
            .any(|ticker| ticker.eq_ignore_ascii_case(symbol))
        {
            return true;
        }
        let cashtag = format!("${symbol}");
        self.searchable()
            .split(|ch: char| !ch.is_ascii_alphanumeric() && !matches!(ch, '.' | '-' | '$'))
            .any(|token| {
                token.eq_ignore_ascii_case(&cashtag)
                    || (symbol.len() >= 3 && token.eq_ignore_ascii_case(symbol))
            })
    }
}

fn normalize_symbol(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('$')
        .trim()
        .to_ascii_uppercase()
}

fn truncate_chars(value: &str, max: usize) -> String {
    let mut output = value.chars().take(max).collect::<String>();
    if value.chars().count() > max {
        output.push('…');
    }
    output
}

fn read_snapshot(path: &Path) -> Option<Snapshot> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// `latest.json` plus every daily history file inside the window, merged by
/// post id with the newest file winning. History exists because the rolling
/// window is a week and a reader may ask about the last month.
fn load_items(root: &Path, since: DateTime<Utc>) -> Option<(Snapshot, Vec<Item>)> {
    let latest = read_snapshot(&root.join("latest.json"))?;
    let mut merged: HashMap<String, Item> = HashMap::new();
    let cutoff_day = since.format("%Y-%m-%d").to_string();
    if let Ok(entries) = std::fs::read_dir(root.join("history")) {
        let mut days = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "json")
                    && path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .is_some_and(|stem| stem >= cutoff_day.as_str())
            })
            .collect::<Vec<_>>();
        // Oldest first, so a later file overrides an earlier reading.
        days.sort();
        for path in days {
            if let Some(day) = read_snapshot(&path) {
                for item in day.items {
                    merged.insert(item.id.clone(), item);
                }
            }
        }
    }
    for item in &latest.items {
        merged.insert(item.id.clone(), item.clone());
    }
    let mut items = merged
        .into_values()
        .filter(|item| item.published_at >= since)
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Some((latest, items))
}

/// What the authors kept coming back to inside the window: the tickers and
/// topics the model attached, counted across posts.
fn focus(items: &[Item]) -> Value {
    fn top(values: impl Iterator<Item = String>) -> Vec<Value> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for value in values {
            *counts.entry(value).or_default() += 1;
        }
        let mut ranked = counts.into_iter().collect::<Vec<_>>();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked
            .into_iter()
            .take(FOCUS_ITEMS)
            .map(|(value, count)| json!({ "value": value, "posts": count }))
            .collect()
    }
    json!({
        "tickers": top(items.iter().flat_map(|item| item.tickers.iter().map(|t| t.to_ascii_uppercase()))),
        "topics": top(items.iter().flat_map(|item| item.topics.iter().cloned())),
    })
}

fn public_item(item: &Item) -> Value {
    let analyzed = item.analysis_status == "model_analyzed";
    json!({
        "author": item.author_name,
        "handle": item.public_handle,
        "posted_at": item.published_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "posted_at_local": item.published_at_local,
        "kind": item.post_kind,
        "text": truncate_chars(item.text(), MAX_TEXT_CHARS),
        "url": item.source_url,
        "translation_source": item.aggregation_source,
        "views": item.metrics.views,
        "likes": item.metrics.likes,
        "hone_reading": if analyzed {
            json!({
                "summary": item.summary,
                "stance": item.stance,
                "horizon": item.horizon,
                "content_type": item.content_type,
                "topics": item.topics,
                "tickers": item.tickers,
                "counterpoint": item.counterpoint,
            })
        } else {
            Value::Null
        },
    })
}

#[async_trait]
impl Tool for InfluencerViewsTool {
    fn name(&self) -> &str {
        "influencer_views"
    }

    fn description(&self) -> &str {
        "读取 HONE 大V速报（研究台同款快照）：注册作者 Serenity/白毛（@aleabitoreddit，AI/半导体供应链）与 SemiAnalysis 近 7–30 天的公开推文与文章，\
        带中文翻译、发布时间、原链，以及 HONE 已做的观点整理（立场 / 事实或观点 / 反方或未证实处）。\
        用 `symbol`（如 MU、AAOI）取点名该公司的条目，用 `query`（如 HBM、光模块、CPO）取行业话题，都不传则返回窗口内最近的条目与作者关注焦点。\
        返回的是作者观点线索，不是事实、不是 HONE 结论：引用时写作者、日期与原链，独立成段，不据此给买卖建议；窗口内没有命中就如实说没有。\
        不联网、不调模型，只读本地快照。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "symbol".to_string(),
                param_type: "string".to_string(),
                description: "可选。股票代码（如 MU、NVDA），匹配作者点名 $MU 或 HONE 整理出的 ticker。"
                    .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "可选。关键词或行业话题（如 HBM、光模块、内存、Rubin），在原文、翻译与话题标签里做子串匹配。"
                    .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "days".to_string(),
                param_type: "number".to_string(),
                description: format!("可选。回看天数，默认 {DEFAULT_DAYS}，最多 {MAX_DAYS}。"),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "limit".to_string(),
                param_type: "number".to_string(),
                description: format!("可选。最多返回几条，默认 {DEFAULT_LIMIT}，最多 {MAX_LIMIT}。"),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let symbol = args
            .get("symbol")
            .and_then(Value::as_str)
            .map(normalize_symbol)
            .filter(|value| !value.is_empty());
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let days = args
            .get("days")
            .and_then(Value::as_f64)
            .map(|value| value.round() as i64)
            .unwrap_or(DEFAULT_DAYS)
            .clamp(1, MAX_DAYS);
        let limit = args
            .get("limit")
            .and_then(Value::as_f64)
            .map(|value| value.round().max(1.0) as usize)
            .unwrap_or(DEFAULT_LIMIT)
            .min(MAX_LIMIT);

        let now = Utc::now();
        let since = now - chrono::Duration::days(days);
        let Some((latest, window)) = load_items(&self.digest_root(), since) else {
            return Ok(json!({
                "success": false,
                "error": "大V速报快照不存在：速报 worker 还没有生成过数据，或来源未配置。不要用搜索摘要或记忆替代大V原话；如实告诉用户本轮没有大V观点可引用。",
            }));
        };

        let matched = window
            .iter()
            .filter(|item| {
                symbol
                    .as_deref()
                    .is_none_or(|symbol| item.mentions_symbol(symbol))
            })
            .filter(|item| {
                query
                    .as_deref()
                    .is_none_or(|query| item.searchable().to_lowercase().contains(query))
            })
            .collect::<Vec<_>>();
        let total_matched = matched.len();
        let items = matched
            .iter()
            .take(limit)
            .map(|item| public_item(item))
            .collect::<Vec<_>>();

        let note = if total_matched == 0 {
            match (&symbol, &query) {
                (Some(symbol), _) => format!(
                    "近 {days} 天窗口里没有作者点名 {symbol} 的条目。可以换 query=行业词再取一次（例如它所在环节），仍没有就如实写“大V近期没有直接提到 {symbol}”，不要拿别的公司的话顶替。"
                ),
                (None, Some(query)) => format!(
                    "近 {days} 天窗口里没有包含“{query}”的条目；换个同义词或直接看 focus 里作者在聊什么。"
                ),
                (None, None) => format!("近 {days} 天窗口里没有条目。"),
            }
        } else {
            "每条都是作者观点线索：引用时写作者、日期与原链；与本轮已核验事实冲突时以事实为准并指出；不据此给买卖或仓位建议。".to_string()
        };

        Ok(json!({
            "success": true,
            "as_of": latest.generated_at_local,
            "snapshot_status": latest.status,
            "refresh_interval_minutes": latest.refresh_interval_minutes,
            "window_days": days,
            "filter": { "symbol": symbol, "query": query },
            "matched": total_matched,
            "returned": items.len(),
            "items": items,
            "focus": focus(&window),
            "note": note,
            "disclaimer": if latest.disclaimer.is_empty() {
                "作者观点仅供信息参考，不代表 HONE 判断，不构成投资建议；请打开原文核实上下文。".to_string()
            } else {
                latest.disclaimer
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "hone-influencer-views-{name}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(dir.join("influencer_digest").join("history")).unwrap();
        dir
    }

    fn item(id: &str, days_ago: i64, text: &str, tickers: &[&str], topics: &[&str]) -> Value {
        let at = Utc::now() - chrono::Duration::days(days_ago);
        json!({
            "id": id,
            "author_id": "serenity",
            "author_name": "Serenity / 白毛",
            "public_handle": "@aleabitoreddit",
            "title": text.lines().next().unwrap_or(""),
            "published_at": at,
            "published_at_local": at.format("%m-%d %H:%M").to_string(),
            "source_url": format!("https://x.com/aleabitoreddit/status/{id}"),
            "aggregation_source": "AI产业链地图·白毛速报",
            "post_kind": "original",
            "source_excerpt": text,
            "source_text_cn": text,
            "metrics": {"views": 1200, "likes": 8},
            "summary": if tickers.is_empty() { "" } else { "作者认为供给仍紧" },
            "stance": "bullish",
            "horizon": "medium",
            "content_type": "opinion",
            "topics": topics,
            "tickers": tickers,
            "counterpoint": "未见库存数据",
            "analysis_status": if tickers.is_empty() { "source_only" } else { "model_analyzed" },
        })
    }

    fn write(dir: &Path, rel: &str, items: Vec<Value>) {
        let snapshot = json!({
            "report_date": "2026-09-06",
            "generated_at_local": "2026-09-06 12:40",
            "status": "live",
            "refresh_interval_minutes": 15,
            "items": items,
            "disclaimer": "仅供参考",
        });
        std::fs::write(
            dir.join("influencer_digest").join(rel),
            serde_json::to_vec(&snapshot).unwrap(),
        )
        .unwrap();
    }

    fn seeded(name: &str) -> PathBuf {
        let dir = temp(name);
        write(
            &dir,
            "latest.json",
            vec![
                item("s1", 1, "现在 $MU 到 $SNDK 已经大幅反弹。\n内存瓶颈并未改变！", &["MU", "SNDK"], &["内存"]),
                item("s2", 3, "CW 激光器与 InP 衬底的供需失衡还在，$AAOI 的产能仍然紧", &["AAOI"], &["光模块"]),
                item("s3", 5, "看到大家学习我的思考方式，而不是照抄具体的股票", &[], &[]),
            ],
        );
        // A month-old post only the history file still holds.
        write(
            &dir,
            &format!(
                "history/{}.json",
                (Utc::now() - chrono::Duration::days(20)).format("%Y-%m-%d")
            ),
            vec![item("h1", 20, "$NVDA Rubin 时间表没有变化", &["NVDA"], &["Rubin"])],
        );
        dir
    }

    #[tokio::test]
    async fn filters_by_symbol_and_reports_the_window() {
        let dir = seeded("symbol");
        let tool = InfluencerViewsTool::new(dir.clone());
        let out = tool.execute(json!({"symbol": "$mu"})).await.unwrap();
        assert_eq!(out["success"], true);
        assert_eq!(out["matched"], 1);
        assert_eq!(out["items"][0]["url"], "https://x.com/aleabitoreddit/status/s1");
        assert_eq!(out["items"][0]["hone_reading"]["tickers"][0], "MU");
        assert_eq!(out["refresh_interval_minutes"], 15);
        assert_eq!(out["window_days"], DEFAULT_DAYS);
        // Focus is computed over the whole window, not the filtered rows.
        let tickers = out["focus"]["tickers"].as_array().unwrap();
        assert!(tickers.iter().any(|entry| entry["value"] == "AAOI"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn query_matches_text_and_topics_and_misses_say_so() {
        let dir = seeded("query");
        let tool = InfluencerViewsTool::new(dir.clone());
        let out = tool.execute(json!({"query": "光模块"})).await.unwrap();
        assert_eq!(out["matched"], 1);
        assert_eq!(out["items"][0]["url"], "https://x.com/aleabitoreddit/status/s2");
        // A source-only row carries no reading rather than an empty one.
        let all = tool.execute(json!({})).await.unwrap();
        assert_eq!(all["matched"], 3);
        assert!(all["items"][2]["hone_reading"].is_null());
        let miss = tool.execute(json!({"symbol": "TSLA"})).await.unwrap();
        assert_eq!(miss["success"], true);
        assert_eq!(miss["matched"], 0);
        assert!(miss["note"].as_str().unwrap().contains("没有直接提到 TSLA"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn history_widens_the_window_and_short_symbols_need_a_cashtag() {
        let dir = seeded("history");
        let tool = InfluencerViewsTool::new(dir.clone());
        let week = tool.execute(json!({"symbol": "NVDA", "days": 7})).await.unwrap();
        assert_eq!(week["matched"], 0);
        let month = tool.execute(json!({"symbol": "NVDA", "days": 30})).await.unwrap();
        assert_eq!(month["matched"], 1);
        assert_eq!(month["items"][0]["url"], "https://x.com/aleabitoreddit/status/h1");
        // "CW" appears as a plain word; a two-letter symbol must be a cashtag.
        let cw = tool.execute(json!({"symbol": "CW"})).await.unwrap();
        assert_eq!(cw["matched"], 0);
        // Bounds hold.
        let capped = tool.execute(json!({"days": 400, "limit": 99})).await.unwrap();
        assert_eq!(capped["window_days"], MAX_DAYS);
        assert_eq!(capped["matched"], 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn a_missing_snapshot_is_an_explicit_gap() {
        let dir = temp("missing");
        let tool = InfluencerViewsTool::new(dir.clone());
        let out = tool.execute(json!({"symbol": "MU"})).await.unwrap();
        assert_eq!(out["success"], false);
        assert!(out["error"].as_str().unwrap().contains("快照不存在"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
