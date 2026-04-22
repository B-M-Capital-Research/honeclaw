//! 持仓存储 — JSON 文件（按 actor 隔离）

use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

pub const HOLDING_HORIZON_LONG_TERM: &str = "long_term";
pub const HOLDING_HORIZON_SHORT_TERM: &str = "short_term";

/// 持仓存储管理器
pub struct PortfolioStorage {
    data_dir: PathBuf,
}

/// 持仓数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<ActorIdentity>,
    #[serde(default)]
    pub user_id: String,
    /// 支持旧字段名 `positions` 向后兼容
    #[serde(default, alias = "positions")]
    pub holdings: Vec<Holding>,
    #[serde(default)]
    pub updated_at: String,
}

/// 单个持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding {
    /// 支持旧字段名 `ticker` 向后兼容
    #[serde(alias = "ticker")]
    pub symbol: String,
    #[serde(default = "default_asset_type")]
    pub asset_type: String,
    pub shares: f64,
    /// 支持旧字段名 `cost_price` 向后兼容
    #[serde(alias = "cost_price")]
    pub avg_cost: f64,
    #[serde(default)]
    pub underlying: Option<String>,
    #[serde(default)]
    pub option_type: Option<String>,
    #[serde(default)]
    pub strike_price: Option<f64>,
    #[serde(default)]
    pub expiration_date: Option<String>,
    #[serde(default)]
    pub contract_multiplier: Option<f64>,
    #[serde(default, alias = "horizon")]
    pub holding_horizon: Option<String>,
    #[serde(default, alias = "strategy")]
    pub strategy_notes: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    /// 关注标的标记：`Some(true)` → 仅关注(无持仓,shares/avg_cost 约定为 0)；
    /// `None` / `Some(false)` → 真实持仓。
    /// 下游若要按持仓真实市值/股数聚合,应显式 `filter(|h| !h.tracking_only.unwrap_or(false))`。
    #[serde(default, skip_serializing_if = "is_false_or_none")]
    pub tracking_only: Option<bool>,
}

fn default_asset_type() -> String {
    "stock".to_string()
}

fn is_false_or_none(v: &Option<bool>) -> bool {
    !matches!(v, Some(true))
}

pub fn normalize_holding_horizon(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" => None,
        "long" | "long_term" | "long-term" | "longterm" | "长期" | "长持" | "长线" => {
            Some(HOLDING_HORIZON_LONG_TERM.to_string())
        }
        "short" | "short_term" | "short-term" | "shortterm" | "短期" | "短持" | "短线" => {
            Some(HOLDING_HORIZON_SHORT_TERM.to_string())
        }
        _ => None,
    }
}

/// 从 storage_key 字符串（`{channel}__{scope}__{user_id}`）解析 ActorIdentity。
/// storage_key 中各组成部分使用十六进制转义（非字母数字字符 → `_{hex:02x}`）。
fn actor_from_storage_key(key: &str) -> Option<ActorIdentity> {
    let parts: Vec<&str> = key.splitn(3, "__").collect();
    if parts.len() != 3 {
        return None;
    }
    let channel = decode_component(parts[0]);
    let scope_raw = decode_component(parts[1]);
    let user_id = decode_component(parts[2]);

    if channel.is_empty() || user_id.is_empty() {
        return None;
    }

    let scope = if scope_raw == "direct" {
        None
    } else {
        Some(scope_raw)
    };

    ActorIdentity::new(channel, user_id, scope).ok()
}

/// 反转 `encode_component`：将 `_{hex:02x}` 还原为原始字符。
fn decode_component(encoded: &str) -> String {
    let mut out = String::with_capacity(encoded.len());
    let bytes = encoded.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' && i + 2 < bytes.len() {
            let hi = bytes[i + 1];
            let lo = bytes[i + 2];
            if let (Some(h), Some(l)) = (hex_digit(hi), hex_digit(lo)) {
                out.push(char::from(h * 16 + l));
                i += 3;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

impl PortfolioStorage {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir).ok();
        Self { data_dir: dir }
    }

    fn actor_path(&self, actor: &ActorIdentity) -> PathBuf {
        self.data_dir
            .join(format!("portfolio_{}.json", actor.storage_key()))
    }

    /// 加载 actor 持仓
    pub fn load(&self, actor: &ActorIdentity) -> hone_core::HoneResult<Option<Portfolio>> {
        let path = self.actor_path(actor);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let mut portfolio: Portfolio = serde_json::from_str(&content)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        portfolio.actor = Some(actor.clone());
        portfolio.user_id = actor.user_id.clone();
        Ok(Some(portfolio))
    }

    /// 保存 actor 持仓
    pub fn save(&self, actor: &ActorIdentity, portfolio: &Portfolio) -> hone_core::HoneResult<()> {
        let path = self.actor_path(actor);
        let mut payload = portfolio.clone();
        payload.actor = Some(actor.clone());
        payload.user_id = actor.user_id.clone();
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn upsert_holding(
        &self,
        actor: &ActorIdentity,
        holding: Holding,
    ) -> hone_core::HoneResult<Portfolio> {
        let mut portfolio = self.load(actor)?.unwrap_or_else(|| Portfolio {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            holdings: Vec::new(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });

        if let Some(existing) = portfolio
            .holdings
            .iter_mut()
            .find(|item| item.symbol == holding.symbol && item.asset_type == holding.asset_type)
        {
            *existing = holding;
        } else {
            portfolio.holdings.push(holding);
        }

        portfolio.updated_at = chrono::Utc::now().to_rfc3339();
        self.save(actor, &portfolio)?;
        Ok(portfolio)
    }

    /// 列出所有有持仓数据的 actor（扫描目录中的 portfolio_*.json 文件）
    pub fn list_all(&self) -> Vec<(ActorIdentity, Portfolio)> {
        let entries = match std::fs::read_dir(&self.data_dir) {
            Ok(entries) => entries,
            Err(_) => return vec![],
        };

        let mut results = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if !name.starts_with("portfolio_") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut portfolio) = serde_json::from_str::<Portfolio>(&content) {
                    // actor 字段优先使用 JSON 中的值；缺失时尝试从文件名解析
                    // 文件名格式：portfolio_{channel}__{scope}__{user_id}.json
                    if portfolio.actor.is_none() {
                        let storage_key = name.trim_start_matches("portfolio_");
                        portfolio.actor = actor_from_storage_key(storage_key);
                    }
                    if let Some(actor) = portfolio.actor.clone() {
                        if portfolio.user_id.is_empty() {
                            portfolio.user_id = actor.user_id.clone();
                        }
                        results.push((actor, portfolio));
                    }
                }
            }
        }
        // 按 updated_at 降序排列（最近更新的在前）
        results.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at));
        results
    }

    /// 加入关注列表。symbol 已存在时保持现有记录不变,返回完整 Portfolio。
    pub fn upsert_watch(
        &self,
        actor: &ActorIdentity,
        symbol: &str,
        asset_type: &str,
    ) -> hone_core::HoneResult<Portfolio> {
        let mut portfolio = self.load(actor)?.unwrap_or_else(|| Portfolio {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            holdings: Vec::new(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });

        if !portfolio
            .holdings
            .iter()
            .any(|h| h.symbol == symbol && h.asset_type == asset_type)
        {
            portfolio.holdings.push(Holding {
                symbol: symbol.to_string(),
                asset_type: asset_type.to_string(),
                shares: 0.0,
                avg_cost: 0.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                tracking_only: Some(true),
            });
            portfolio.updated_at = chrono::Utc::now().to_rfc3339();
            self.save(actor, &portfolio)?;
        }

        Ok(portfolio)
    }

    /// 把关注项升级为持仓：查到对应行后清 `tracking_only`,写入 shares / avg_cost。
    /// 返回 `Ok(Some(portfolio, was_watchlist))`:
    /// - `was_watchlist=true` 表示这条确实来自关注列表,调用方可据此向用户汇报"已自动转为持仓"；
    /// - `was_watchlist=false` 表示本就是真实持仓,只是做了一次正常 upsert；
    /// - `Ok(None)` 表示该 actor 尚无任何持仓/关注记录,调用方应当走普通 upsert 路径。
    pub fn promote_to_holding(
        &self,
        actor: &ActorIdentity,
        symbol: &str,
        asset_type: &str,
        shares: f64,
        avg_cost: f64,
    ) -> hone_core::HoneResult<Option<(Portfolio, bool)>> {
        let Some(mut portfolio) = self.load(actor)? else {
            return Ok(None);
        };

        let Some(existing) = portfolio
            .holdings
            .iter_mut()
            .find(|h| h.symbol == symbol && h.asset_type == asset_type)
        else {
            return Ok(None);
        };

        let was_watchlist = existing.tracking_only.unwrap_or(false);
        existing.shares = shares;
        existing.avg_cost = avg_cost;
        existing.tracking_only = None;

        portfolio.updated_at = chrono::Utc::now().to_rfc3339();
        self.save(actor, &portfolio)?;
        Ok(Some((portfolio, was_watchlist)))
    }

    pub fn remove_holding(
        &self,
        actor: &ActorIdentity,
        symbol: &str,
    ) -> hone_core::HoneResult<Option<Portfolio>> {
        let Some(mut portfolio) = self.load(actor)? else {
            return Ok(None);
        };

        let original_len = portfolio.holdings.len();
        portfolio
            .holdings
            .retain(|holding| holding.symbol != symbol);
        if portfolio.holdings.len() == original_len {
            return Ok(None);
        }

        portfolio.updated_at = chrono::Utc::now().to_rfc3339();
        self.save(actor, &portfolio)?;
        Ok(Some(portfolio))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn actor(channel: &str, user_id: &str, channel_scope: Option<&str>) -> ActorIdentity {
        ActorIdentity::new(channel, user_id, channel_scope).expect("actor")
    }

    #[test]
    fn portfolio_storage_roundtrip() {
        let dir = make_temp_dir("hone_portfolio_storage");
        let storage = PortfolioStorage::new(&dir);
        let actor = actor("imessage", "User_test", None);

        let empty = storage.load(&actor).expect("load empty");
        assert!(empty.is_none());

        let portfolio = Portfolio {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            holdings: vec![Holding {
                symbol: "AAPL".to_string(),
                asset_type: "stock".to_string(),
                shares: 3.5,
                avg_cost: 180.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: Some(HOLDING_HORIZON_LONG_TERM.to_string()),
                strategy_notes: Some("核心仓位".to_string()),
                notes: Some("long term".to_string()),
                tracking_only: None,
            }],
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        storage.save(&actor, &portfolio).expect("save");
        let loaded = storage.load(&actor).expect("load").expect("exists");
        assert_eq!(loaded.user_id, actor.user_id);
        assert_eq!(loaded.actor, Some(actor));
        assert_eq!(loaded.holdings.len(), 1);
        assert_eq!(loaded.holdings[0].symbol, "AAPL");
        assert_eq!(loaded.holdings[0].asset_type, "stock");
        assert_eq!(loaded.holdings[0].shares, 3.5);
        assert_eq!(
            loaded.holdings[0].holding_horizon.as_deref(),
            Some(HOLDING_HORIZON_LONG_TERM)
        );
        assert_eq!(
            loaded.holdings[0].strategy_notes.as_deref(),
            Some("核心仓位")
        );
    }

    #[test]
    fn portfolio_storage_holding_crud() {
        let dir = make_temp_dir("hone_portfolio_storage_crud");
        let storage = PortfolioStorage::new(&dir);
        let actor = actor("imessage", "User_test", None);

        let portfolio = storage
            .upsert_holding(
                &actor,
                Holding {
                    symbol: "AAPL".to_string(),
                    asset_type: "stock".to_string(),
                    shares: 10.0,
                    avg_cost: 200.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: Some(HOLDING_HORIZON_LONG_TERM.to_string()),
                    strategy_notes: Some("逢跌加仓".to_string()),
                    notes: Some("long".to_string()),
                    tracking_only: None,
                },
            )
            .expect("upsert add");
        assert_eq!(portfolio.holdings.len(), 1);

        let portfolio = storage
            .upsert_holding(
                &actor,
                Holding {
                    symbol: "AAPL".to_string(),
                    asset_type: "stock".to_string(),
                    shares: 12.0,
                    avg_cost: 198.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: Some(HOLDING_HORIZON_SHORT_TERM.to_string()),
                    strategy_notes: Some("事件驱动".to_string()),
                    notes: None,
                    tracking_only: None,
                },
            )
            .expect("upsert update");
        assert_eq!(portfolio.holdings.len(), 1);
        assert_eq!(portfolio.holdings[0].shares, 12.0);
        assert_eq!(portfolio.holdings[0].avg_cost, 198.0);
        assert_eq!(
            portfolio.holdings[0].holding_horizon.as_deref(),
            Some(HOLDING_HORIZON_SHORT_TERM)
        );
        assert_eq!(
            portfolio.holdings[0].strategy_notes.as_deref(),
            Some("事件驱动")
        );

        let portfolio = storage
            .remove_holding(&actor, "AAPL")
            .expect("remove")
            .expect("portfolio");
        assert!(portfolio.holdings.is_empty());
    }

    #[test]
    fn portfolio_storage_isolated_by_actor() {
        let dir = make_temp_dir("hone_portfolio_storage_actor");
        let storage = PortfolioStorage::new(&dir);
        let left = actor("imessage", "alice", None);
        let right = actor("discord", "alice", None);

        storage
            .upsert_holding(
                &left,
                Holding {
                    symbol: "AAPL".to_string(),
                    asset_type: "stock".to_string(),
                    shares: 1.0,
                    avg_cost: 100.0,
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
            )
            .expect("left save");
        storage
            .upsert_holding(
                &right,
                Holding {
                    symbol: "TSLA".to_string(),
                    asset_type: "stock".to_string(),
                    shares: 2.0,
                    avg_cost: 200.0,
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
            )
            .expect("right save");

        let left_loaded = storage
            .load(&left)
            .expect("left load")
            .expect("left exists");
        let right_loaded = storage
            .load(&right)
            .expect("right load")
            .expect("right exists");

        assert_eq!(left_loaded.holdings[0].symbol, "AAPL");
        assert_eq!(right_loaded.holdings[0].symbol, "TSLA");
    }

    #[test]
    fn portfolio_storage_supports_option_holdings() {
        let dir = make_temp_dir("hone_portfolio_storage_option");
        let storage = PortfolioStorage::new(&dir);
        let actor = actor("imessage", "User_option", None);

        let portfolio = storage
            .upsert_holding(
                &actor,
                Holding {
                    symbol: "AAPL 2026-06-19 C 200".to_string(),
                    asset_type: "option".to_string(),
                    shares: 2.0,
                    avg_cost: 5.25,
                    underlying: Some("AAPL".to_string()),
                    option_type: Some("call".to_string()),
                    strike_price: Some(200.0),
                    expiration_date: Some("2026-06-19".to_string()),
                    contract_multiplier: Some(100.0),
                    holding_horizon: Some(HOLDING_HORIZON_SHORT_TERM.to_string()),
                    strategy_notes: Some("卖波动率".to_string()),
                    notes: Some("swing trade".to_string()),
                    tracking_only: None,
                },
            )
            .expect("upsert option");

        assert_eq!(portfolio.holdings.len(), 1);
        assert_eq!(portfolio.holdings[0].asset_type, "option");
        assert_eq!(portfolio.holdings[0].underlying.as_deref(), Some("AAPL"));
        assert_eq!(portfolio.holdings[0].option_type.as_deref(), Some("call"));
        assert_eq!(portfolio.holdings[0].strike_price, Some(200.0));
        assert_eq!(
            portfolio.holdings[0].expiration_date.as_deref(),
            Some("2026-06-19")
        );
        assert_eq!(
            portfolio.holdings[0].holding_horizon.as_deref(),
            Some(HOLDING_HORIZON_SHORT_TERM)
        );
        assert_eq!(
            portfolio.holdings[0].strategy_notes.as_deref(),
            Some("卖波动率")
        );
    }

    #[test]
    fn portfolio_storage_supports_negative_avg_cost_and_strategy_metadata() {
        let dir = make_temp_dir("hone_portfolio_storage_negative_avg_cost");
        let storage = PortfolioStorage::new(&dir);
        let actor = actor("discord", "credit_trade", None);

        let portfolio = storage
            .upsert_holding(
                &actor,
                Holding {
                    symbol: "AAPL 2026-06-19 P 180".to_string(),
                    asset_type: "option".to_string(),
                    shares: 1.0,
                    avg_cost: -2.35,
                    underlying: Some("AAPL".to_string()),
                    option_type: Some("put".to_string()),
                    strike_price: Some(180.0),
                    expiration_date: Some("2026-06-19".to_string()),
                    contract_multiplier: Some(100.0),
                    holding_horizon: Some(HOLDING_HORIZON_SHORT_TERM.to_string()),
                    strategy_notes: Some("现金担保卖沽，权利金净流入".to_string()),
                    notes: Some("credit position".to_string()),
                    tracking_only: None,
                },
            )
            .expect("upsert negative avg cost");

        assert_eq!(portfolio.holdings[0].avg_cost, -2.35);
        assert_eq!(
            portfolio.holdings[0].strategy_notes.as_deref(),
            Some("现金担保卖沽，权利金净流入")
        );
    }

    #[test]
    fn normalize_holding_horizon_accepts_common_aliases() {
        assert_eq!(
            normalize_holding_horizon("长持").as_deref(),
            Some(HOLDING_HORIZON_LONG_TERM)
        );
        assert_eq!(
            normalize_holding_horizon("short-term").as_deref(),
            Some(HOLDING_HORIZON_SHORT_TERM)
        );
        assert_eq!(normalize_holding_horizon(""), None);
        assert_eq!(normalize_holding_horizon("event-driven"), None);
    }

    #[test]
    fn holding_tracking_only_roundtrip() {
        let dir = make_temp_dir("hone_portfolio_storage_watchlist");
        let storage = PortfolioStorage::new(&dir);
        let actor = actor("imessage", "watcher", None);

        let portfolio = storage
            .upsert_watch(&actor, "NVDA", "stock")
            .expect("upsert watch");
        assert_eq!(portfolio.holdings.len(), 1);
        assert_eq!(portfolio.holdings[0].symbol, "NVDA");
        assert_eq!(portfolio.holdings[0].shares, 0.0);
        assert_eq!(portfolio.holdings[0].avg_cost, 0.0);
        assert_eq!(portfolio.holdings[0].tracking_only, Some(true));

        let loaded = storage.load(&actor).expect("load").expect("exists");
        assert_eq!(loaded.holdings[0].tracking_only, Some(true));

        let again = storage
            .upsert_watch(&actor, "NVDA", "stock")
            .expect("idempotent watch");
        assert_eq!(again.holdings.len(), 1);
    }

    #[test]
    fn legacy_json_without_tracking_only_deserializes_as_none() {
        let legacy = r#"{
            "user_id": "legacy",
            "holdings": [
                {"symbol":"AAPL","asset_type":"stock","shares":10,"avg_cost":180}
            ],
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;
        let portfolio: Portfolio = serde_json::from_str(legacy).expect("parse legacy");
        assert_eq!(portfolio.holdings.len(), 1);
        assert_eq!(portfolio.holdings[0].tracking_only, None);

        let serialized = serde_json::to_string(&portfolio).expect("re-serialize");
        assert!(!serialized.contains("tracking_only"));
    }

    #[test]
    fn upsert_watch_and_promote() {
        let dir = make_temp_dir("hone_portfolio_storage_promote");
        let storage = PortfolioStorage::new(&dir);
        let actor = actor("telegram", "promoter", None);

        storage
            .upsert_watch(&actor, "TSLA", "stock")
            .expect("watch");

        let (portfolio, was_watchlist) = storage
            .promote_to_holding(&actor, "TSLA", "stock", 50.0, 120.0)
            .expect("promote")
            .expect("record exists");
        assert!(was_watchlist);
        assert_eq!(portfolio.holdings.len(), 1);
        assert_eq!(portfolio.holdings[0].shares, 50.0);
        assert_eq!(portfolio.holdings[0].avg_cost, 120.0);
        assert_eq!(portfolio.holdings[0].tracking_only, None);

        let promote_again = storage
            .promote_to_holding(&actor, "TSLA", "stock", 60.0, 115.0)
            .expect("second promote")
            .expect("record exists");
        assert!(!promote_again.1);
        assert_eq!(promote_again.0.holdings[0].shares, 60.0);
    }
}
