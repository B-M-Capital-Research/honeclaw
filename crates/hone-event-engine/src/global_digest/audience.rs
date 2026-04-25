//! 受众持仓概览构建 —— 给 Curator Pass 1/2 的 prompt 提供 ground-truth 公司业务方向。
//!
//! POC 验证(见 SKILL `poc-driven-feature-design`):FMP `/api/v3/profile/{TICKERS}` 的
//! `description` 字段对小盘 / 新 IPO / 偏门 ticker(POET / CAI / BE / TEM / AAOI 等)
//! 都准确写明业务方向,**LLM 不需要自己猜**,直接看 brief 就能把"NVDA / Intel / TSM
//! 是 AMD 同行"、"Anthropic 是 GOOGL 战略"等推理出来。所以**不需要 admin override 层**,
//! FMP 直接够用。
//!
//! 流程:
//! 1. 从 PortfolioStorage 收集所有 direct actor 的 holdings 并集(去重)
//! 2. 对每个 ticker,从本地缓存读 profile(7 天 TTL),miss 则 batch FMP /profile
//! 3. 顺便从 portfolio 收集 holdings.notes 作为 user_notes 透传给 curator(POC 没测但
//!    plan 已确定保留——多用户角度作为额外信号给 LLM,不参与 one_liner 覆盖)
//!
//! 失败降级:
//! - FMP 失败的 ticker → `CompanyBrief { name=ticker, sector="", industry="",
//!   one_liner="(无 profile)", source: BriefSource::Empty }`,不让一条失败拖死整批
//! - 无缓存目录 / 无写权限 → 仅内存,下次还会重拉(warn 一次)

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use hone_memory::portfolio::PortfolioStorage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::fmp::FmpClient;

/// 缓存 TTL —— 业务方向变化慢,周级别足够。
const CACHE_TTL_SECS: i64 = 7 * 86400;

/// 单 ticker 的简介。Pass 1/2 prompt 把它做成"持仓概览"段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyBrief {
    pub ticker: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub one_liner: String,
    /// 该 ticker 在所有 direct actor 的 portfolio 里的 `holdings.notes`,去重后保留。
    /// 为空时不出现在 prompt 里。
    pub user_notes: Vec<String>,
    pub source: BriefSource,
}

/// 该 brief 的 one_liner 来源 —— 给 daily_report 审计时知道哪些 ticker 走的是
/// FMP 凑合版,哪些是空。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BriefSource {
    FmpDescription,
    Empty,
}

#[derive(Debug, Clone)]
pub struct AudienceContext {
    pub briefs: Vec<CompanyBrief>,
}

/// 缓存到磁盘的 profile 条目。
#[derive(Debug, Serialize, Deserialize)]
struct CachedProfile {
    fetched_at_ts: i64,
    profile: Value, // 原始 FMP profile JSON
}

pub struct AudienceBuilder<'a> {
    fmp: &'a FmpClient,
    cache_dir: PathBuf,
    portfolio_storage: &'a PortfolioStorage,
}

impl<'a> AudienceBuilder<'a> {
    pub fn new(
        fmp: &'a FmpClient,
        cache_dir: impl Into<PathBuf>,
        portfolio_storage: &'a PortfolioStorage,
    ) -> Self {
        Self {
            fmp,
            cache_dir: cache_dir.into(),
            portfolio_storage,
        }
    }

    /// 收集 direct actor 的 ticker 并集 + notes,返回 AudienceContext。
    pub async fn build(&self) -> AudienceContext {
        let (tickers, notes_by_ticker) = self.collect_tickers_and_notes();
        if tickers.is_empty() {
            return AudienceContext { briefs: Vec::new() };
        }
        // 1. 从缓存读 hit / 收集 miss
        let now_ts = Utc::now().timestamp();
        let mut from_cache: HashMap<String, Value> = HashMap::new();
        let mut to_fetch: Vec<String> = Vec::new();
        for t in &tickers {
            match self.read_cache(t, now_ts) {
                Some(p) => {
                    from_cache.insert(t.clone(), p);
                }
                None => to_fetch.push(t.clone()),
            }
        }

        // 2. batch fetch missing
        let fetched = if to_fetch.is_empty() {
            HashMap::new()
        } else {
            self.fetch_profiles(&to_fetch).await
        };
        // 写缓存(只对 fetched 成功的)
        for (t, p) in &fetched {
            self.write_cache(t, p, now_ts);
        }

        // 3. 合并 + 渲染 brief
        let mut briefs = Vec::with_capacity(tickers.len());
        for t in &tickers {
            let profile = from_cache.get(t).or_else(|| fetched.get(t));
            let notes = notes_by_ticker.get(t).cloned().unwrap_or_default();
            briefs.push(self.profile_to_brief(t, profile, notes));
        }
        AudienceContext { briefs }
    }

    fn collect_tickers_and_notes(&self) -> (Vec<String>, HashMap<String, Vec<String>>) {
        let mut tickers: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut notes_by: HashMap<String, Vec<String>> = HashMap::new();
        for (actor, portfolio) in self.portfolio_storage.list_all() {
            if !actor.is_direct() {
                continue;
            }
            for h in &portfolio.holdings {
                let sym = h.symbol.to_uppercase();
                if seen.insert(sym.clone()) {
                    tickers.push(sym.clone());
                }
                if let Some(n) = h.notes.as_deref() {
                    let n = n.trim();
                    if !n.is_empty() {
                        let entry = notes_by.entry(sym).or_default();
                        if !entry.iter().any(|existing| existing == n) {
                            entry.push(n.to_string());
                        }
                    }
                }
            }
        }
        (tickers, notes_by)
    }

    async fn fetch_profiles(&self, tickers: &[String]) -> HashMap<String, Value> {
        if !self.fmp.has_keys() {
            tracing::warn!("audience: FMP key 未配置,profile 全部空");
            return HashMap::new();
        }
        let joined = tickers.join(",");
        let path = format!("/v3/profile/{joined}");
        match self.fmp.get_json(&path).await {
            Ok(Value::Array(arr)) => arr
                .into_iter()
                .filter_map(|p| {
                    let sym = p
                        .get("symbol")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_uppercase())?;
                    Some((sym, p))
                })
                .collect(),
            Ok(other) => {
                tracing::warn!("audience: FMP /profile 非数组返回: {other:?}");
                HashMap::new()
            }
            Err(e) => {
                tracing::warn!("audience: FMP /profile 失败: {e:#}");
                HashMap::new()
            }
        }
    }

    fn cache_path(&self, ticker: &str) -> PathBuf {
        self.cache_dir
            .join(format!("{}.json", ticker.to_uppercase()))
    }

    fn read_cache(&self, ticker: &str, now_ts: i64) -> Option<Value> {
        let path = self.cache_path(ticker);
        let bytes = std::fs::read(&path).ok()?;
        let cached: CachedProfile = serde_json::from_slice(&bytes).ok()?;
        if now_ts - cached.fetched_at_ts > CACHE_TTL_SECS {
            return None;
        }
        Some(cached.profile)
    }

    fn write_cache(&self, ticker: &str, profile: &Value, now_ts: i64) {
        if std::fs::create_dir_all(&self.cache_dir).is_err() {
            return;
        }
        let cached = CachedProfile {
            fetched_at_ts: now_ts,
            profile: profile.clone(),
        };
        if let Ok(bytes) = serde_json::to_vec_pretty(&cached) {
            let _ = std::fs::write(self.cache_path(ticker), bytes);
        }
    }

    fn profile_to_brief(
        &self,
        ticker: &str,
        profile: Option<&Value>,
        user_notes: Vec<String>,
    ) -> CompanyBrief {
        let Some(p) = profile else {
            return CompanyBrief {
                ticker: ticker.to_string(),
                name: ticker.to_string(),
                sector: String::new(),
                industry: String::new(),
                one_liner: "(无 profile)".to_string(),
                user_notes,
                source: BriefSource::Empty,
            };
        };
        let name = p
            .get("companyName")
            .and_then(|v| v.as_str())
            .unwrap_or(ticker)
            .to_string();
        let sector = p
            .get("sector")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let industry = p
            .get("industry")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let desc = p.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let one_liner = extract_one_liner(desc, 200);
        let source = if one_liner == "(无 profile)" {
            BriefSource::Empty
        } else {
            BriefSource::FmpDescription
        };
        CompanyBrief {
            ticker: ticker.to_string(),
            name,
            sector,
            industry,
            one_liner,
            user_notes,
            source,
        }
    }
}

/// 从 description 抽 one-liner:截 max_chars 字符,优先在最近的句号处断,
/// 否则硬截。空字符串 / 仅空白 → "(无 profile)"。
pub fn extract_one_liner(desc: &str, max_chars: usize) -> String {
    let trimmed = desc.trim();
    if trimmed.is_empty() {
        return "(无 profile)".to_string();
    }
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= max_chars {
        return trimmed.to_string();
    }
    let mut cut: String = chars.iter().take(max_chars).collect();
    // 找最靠后的英文/中文句号
    let last_dot = cut.rfind(". ").or_else(|| cut.rfind('。'));
    if let Some(idx) = last_dot {
        if idx > max_chars / 3 {
            // 至少要保留前 1/3,否则不如硬截
            cut.truncate(idx + 1);
            return cut.trim().to_string();
        }
    }
    cut.push('…');
    cut
}

/// SystemTime → 秒(unused 但 _SystemTime 留给后续测试)。
#[allow(dead_code)]
fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 工具:DateTime<Utc> → 秒。
#[allow(dead_code)]
fn dt_secs(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_one_liner_returns_full_text_when_short() {
        let desc = "Apple designs phones.";
        let r = extract_one_liner(desc, 100);
        assert_eq!(r, "Apple designs phones.");
    }

    #[test]
    fn extract_one_liner_truncates_at_period_when_long() {
        let desc = "Apple designs and sells phones, computers, tablets, watches, and services. \
                    The company also operates a chip business. Other ventures include AR/VR.";
        let r = extract_one_liner(desc, 80);
        // 应该在某个句号断开
        assert!(r.ends_with('.') || r.ends_with('…'), "got: {r}");
        assert!(r.chars().count() <= 100);
    }

    #[test]
    fn extract_one_liner_handles_empty_desc() {
        assert_eq!(extract_one_liner("", 200), "(无 profile)");
        assert_eq!(extract_one_liner("   ", 200), "(无 profile)");
    }

    #[test]
    fn extract_one_liner_truncates_with_ellipsis_when_no_period() {
        let desc = "a".repeat(300);
        let r = extract_one_liner(&desc, 50);
        assert!(r.ends_with('…'));
        assert_eq!(r.chars().count(), 51);
    }

    #[test]
    fn profile_to_brief_falls_back_when_no_profile() {
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path().join("portfolios"));
        let cfg = hone_core::config::FmpConfig::default();
        let fmp = FmpClient::from_config(&cfg);
        let b = AudienceBuilder::new(&fmp, dir.path().join("cache"), &storage);
        let brief = b.profile_to_brief("UNKNOWN", None, vec![]);
        assert_eq!(brief.ticker, "UNKNOWN");
        assert_eq!(brief.name, "UNKNOWN");
        assert_eq!(brief.one_liner, "(无 profile)");
        assert_eq!(brief.source, BriefSource::Empty);
    }

    #[test]
    fn profile_to_brief_extracts_fmp_fields() {
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path().join("portfolios"));
        let cfg = hone_core::config::FmpConfig::default();
        let fmp = FmpClient::from_config(&cfg);
        let b = AudienceBuilder::new(&fmp, dir.path().join("cache"), &storage);
        let profile = json!({
            "symbol": "CAI",
            "companyName": "Caris Life Sciences, Inc.",
            "sector": "Healthcare",
            "industry": "Biotechnology",
            "description": "Caris Life Sciences, an AI TechBio company, provides molecular profiling services. Focused on precision oncology."
        });
        let brief = b.profile_to_brief("CAI", Some(&profile), vec!["看 ctDNA 渗透率".into()]);
        assert_eq!(brief.name, "Caris Life Sciences, Inc.");
        assert_eq!(brief.sector, "Healthcare");
        assert_eq!(brief.industry, "Biotechnology");
        assert!(brief.one_liner.contains("AI TechBio"));
        assert_eq!(brief.user_notes, vec!["看 ctDNA 渗透率"]);
        assert_eq!(brief.source, BriefSource::FmpDescription);
    }

    #[test]
    fn collect_tickers_dedupes_across_actors_and_collects_notes() {
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path().join("portfolios"));
        // actor A: AAPL + AMD
        let actor_a = hone_core::ActorIdentity::new("telegram", "111", None::<&str>).unwrap();
        let portfolio_a = hone_memory::portfolio::Portfolio {
            actor: Some(actor_a.clone()),
            user_id: "111".into(),
            holdings: vec![
                hone_memory::portfolio::Holding {
                    symbol: "AAPL".into(),
                    asset_type: "stock".into(),
                    shares: 100.0,
                    avg_cost: 200.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: Some("看现金流 + 回购".into()),
                    tracking_only: None,
                },
                hone_memory::portfolio::Holding {
                    symbol: "AMD".into(),
                    asset_type: "stock".into(),
                    shares: 50.0,
                    avg_cost: 150.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: None,
                },
            ],
            updated_at: Utc::now().to_rfc3339(),
        };
        storage.save(&actor_a, &portfolio_a).unwrap();
        // actor B: AAPL again (重复) + GOOGL,AAPL 带不同 notes
        let actor_b = hone_core::ActorIdentity::new("telegram", "222", None::<&str>).unwrap();
        let portfolio_b = hone_memory::portfolio::Portfolio {
            actor: Some(actor_b.clone()),
            user_id: "222".into(),
            holdings: vec![
                hone_memory::portfolio::Holding {
                    symbol: "AAPL".into(),
                    asset_type: "stock".into(),
                    shares: 50.0,
                    avg_cost: 180.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: Some("我看的是 Services 增速".into()),
                    tracking_only: None,
                },
                hone_memory::portfolio::Holding {
                    symbol: "GOOGL".into(),
                    asset_type: "stock".into(),
                    shares: 20.0,
                    avg_cost: 250.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: None,
                },
            ],
            updated_at: Utc::now().to_rfc3339(),
        };
        storage.save(&actor_b, &portfolio_b).unwrap();

        let cfg = hone_core::config::FmpConfig::default();
        let fmp = FmpClient::from_config(&cfg);
        let b = AudienceBuilder::new(&fmp, dir.path().join("cache"), &storage);
        let (tickers, notes) = b.collect_tickers_and_notes();
        // dedupe AAPL, 顺序保留(A 先,所以 AAPL/AMD/GOOGL)
        assert_eq!(tickers.len(), 3);
        assert!(tickers.contains(&"AAPL".to_string()));
        assert!(tickers.contains(&"AMD".to_string()));
        assert!(tickers.contains(&"GOOGL".to_string()));
        // AAPL 应该收集 2 条 notes
        let aapl_notes = notes.get("AAPL").unwrap();
        assert_eq!(aapl_notes.len(), 2);
        assert!(aapl_notes.iter().any(|n| n.contains("回购")));
        assert!(aapl_notes.iter().any(|n| n.contains("Services")));
    }

    #[test]
    fn cache_roundtrip_within_ttl() {
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path().join("portfolios"));
        let cfg = hone_core::config::FmpConfig::default();
        let fmp = FmpClient::from_config(&cfg);
        let b = AudienceBuilder::new(&fmp, dir.path().join("cache"), &storage);
        let now_ts = Utc::now().timestamp();
        let profile = json!({"symbol": "AAPL", "companyName": "Apple Inc."});
        b.write_cache("AAPL", &profile, now_ts);
        let read = b.read_cache("AAPL", now_ts).unwrap();
        assert_eq!(
            read.get("companyName").and_then(|v| v.as_str()),
            Some("Apple Inc.")
        );
    }

    #[test]
    fn cache_expires_after_ttl() {
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path().join("portfolios"));
        let cfg = hone_core::config::FmpConfig::default();
        let fmp = FmpClient::from_config(&cfg);
        let b = AudienceBuilder::new(&fmp, dir.path().join("cache"), &storage);
        let old_ts = Utc::now().timestamp() - CACHE_TTL_SECS - 100;
        let profile = json!({"symbol": "AAPL"});
        b.write_cache("AAPL", &profile, old_ts);
        let now_ts = Utc::now().timestamp();
        assert!(b.read_cache("AAPL", now_ts).is_none());
    }

    #[tokio::test]
    async fn build_with_empty_portfolio_returns_empty_briefs() {
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path().join("portfolios"));
        let cfg = hone_core::config::FmpConfig::default();
        let fmp = FmpClient::from_config(&cfg);
        let b = AudienceBuilder::new(&fmp, dir.path().join("cache"), &storage);
        let ctx = b.build().await;
        assert!(ctx.briefs.is_empty());
    }
}
