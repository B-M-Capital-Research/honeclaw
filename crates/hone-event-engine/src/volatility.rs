//! σ-自适应价格阈值(2026-08,推送体检 item 1)。
//!
//! 固定 `price_alert_{low,high}_pct` 对高波动持仓失真:σ≈9 的标的每 1.2 个
//! 交易日就触发一次"警报",而那只是它的寻常日。本模块按标的自身 60 日日收益率
//! 标准差缩放 poller 层阈值,设计、参数锚定与回归验收标准见
//! `docs/proposals/sigma-adaptive-price-thresholds.md`。
//!
//! - σ 数据源:FMP `/v3/historical-price-full/{sym}?timeseries=…&serietype=line`,
//!   按 (symbol, ET 交易日) 缓存 —— 同日阈值绝不漂移(band id 依赖此稳定性)。
//! - σ 不可得(新股、样本不足、API 失败)→ 回退固定阈值;失败不缓存,下 tick 重试。

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::NaiveDate;
use hone_core::config::PriceSigmaThresholds;
use serde_json::Value;

use crate::fmp::FmpClient;

/// 过去 N 日 close-to-close 简单收益率(%)的样本标准差(n-1)。
/// `closes_asc` 按日期升序;样本(收益率个数)少于 `min_samples` 返回 None。
pub fn sigma_pct_from_closes(closes_asc: &[f64], min_samples: usize) -> Option<f64> {
    let returns: Vec<f64> = closes_asc
        .windows(2)
        .filter(|w| w[0].is_finite() && w[0] > 0.0 && w[1].is_finite() && w[1] > 0.0)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0)
        .collect();
    if returns.len() < min_samples.max(2) {
        return None;
    }
    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let sigma = var.sqrt();
    sigma.is_finite().then_some(sigma)
}

/// 由 σ 推有效 (low, high) 阈值;σ 缺失或未启用时回退 (base_low, base_high)。
pub fn effective_thresholds(
    cfg: &PriceSigmaThresholds,
    sigma_pct: Option<f64>,
    base_low: f64,
    base_high: f64,
) -> (f64, f64) {
    let Some(sigma) = sigma_pct.filter(|s| cfg.enabled && s.is_finite() && *s > 0.0) else {
        return (base_low, base_high);
    };
    let low = (cfg.low_mult * sigma).clamp(cfg.low_floor_pct, cfg.low_cap_pct);
    let high = (cfg.high_mult * sigma).clamp(cfg.high_floor_pct, cfg.high_cap_pct);
    // 上下限配置矛盾时保证 high ≥ low,避免出现「达到 low 即 High」的倒挂。
    (low, high.max(low))
}

/// 每标的每 ET 交易日拉一次日线、算一次 σ 并缓存。线上由 PricePoller /
/// ExtendedHoursPoller 共享一个实例。
pub struct SigmaProvider {
    client: FmpClient,
    cfg: PriceSigmaThresholds,
    /// symbol → (ET 交易日, σ%)。只缓存成功值。
    cache: Mutex<HashMap<String, (NaiveDate, f64)>>,
}

impl SigmaProvider {
    pub fn new(client: FmpClient, cfg: PriceSigmaThresholds) -> Self {
        Self {
            client,
            cfg,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn config(&self) -> &PriceSigmaThresholds {
        &self.cfg
    }

    #[cfg(test)]
    fn seed_cache(&self, symbol: &str, day: NaiveDate, sigma: f64) {
        if let Ok(mut map) = self.cache.lock() {
            map.insert(symbol.to_string(), (day, sigma));
        }
    }

    /// 该标的当日的 σ(%)。禁用时恒 None(调用方回退固定阈值)。
    pub async fn sigma_pct(&self, symbol: &str, et_today: NaiveDate) -> Option<f64> {
        if !self.cfg.enabled {
            return None;
        }
        if let Ok(map) = self.cache.lock()
            && let Some((day, sigma)) = map.get(symbol)
            && *day == et_today
        {
            return Some(*sigma);
        }

        // +5:σ 用相邻两日收盘差分,多拉几天冗余吸收停牌/节假日空洞。
        let rows = self.cfg.lookback_days.saturating_add(5).max(2);
        let path = format!("/v3/historical-price-full/{symbol}?timeseries={rows}&serietype=line");
        let raw = match self.client.get_json(&path).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(poller = "price.sigma", "σ 日线拉取 {symbol} 失败: {e:#}");
                return None;
            }
        };
        let sigma = sigma_pct_from_historical_response(&raw, self.cfg.min_samples as usize);
        if let Some(sigma) = sigma
            && let Ok(mut map) = self.cache.lock()
        {
            map.insert(symbol.to_string(), (et_today, sigma));
        }
        sigma
    }

    /// 一次性算出一组标的的有效阈值。σ 不可得的标的回退 (base_low, base_high)。
    pub async fn thresholds_for(
        &self,
        symbols: &[String],
        et_today: NaiveDate,
        base_low: f64,
        base_high: f64,
    ) -> HashMap<String, SymbolThresholds> {
        let mut out = HashMap::new();
        if !self.cfg.enabled {
            return out;
        }
        for symbol in symbols {
            let sigma = self.sigma_pct(symbol, et_today).await;
            let (low, high) = effective_thresholds(&self.cfg, sigma, base_low, base_high);
            out.insert(
                symbol.clone(),
                SymbolThresholds {
                    low_pct: low,
                    high_pct: high,
                    sigma_pct: sigma,
                },
            );
        }
        out
    }
}

/// 单标的当日生效阈值(σ 缺失时 low/high 即固定配置值,sigma_pct=None)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolThresholds {
    pub low_pct: f64,
    pub high_pct: f64,
    pub sigma_pct: Option<f64>,
}

/// FMP historical-price-full 返回 `{"historical":[{date,close},…]}`,日期降序。
fn sigma_pct_from_historical_response(raw: &Value, min_samples: usize) -> Option<f64> {
    let rows = raw.get("historical")?.as_array()?;
    let mut dated: Vec<(&str, f64)> = rows
        .iter()
        .filter_map(|r| {
            Some((
                r.get("date")?.as_str()?,
                r.get("close")?.as_f64().filter(|c| *c > 0.0)?,
            ))
        })
        .collect();
    dated.sort_by(|a, b| a.0.cmp(b.0));
    let closes: Vec<f64> = dated.into_iter().map(|(_, c)| c).collect();
    sigma_pct_from_closes(&closes, min_samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> PriceSigmaThresholds {
        PriceSigmaThresholds::default()
    }

    #[test]
    fn sigma_matches_hand_computed_sample_stdev() {
        // 收益率: +10%, -10%, +10% → mean=10/3, var=((6.67)^2+(-13.33)^2+(6.67)^2)/2
        let closes = [100.0, 110.0, 99.0, 108.9];
        let sigma = sigma_pct_from_closes(&closes, 2).unwrap();
        assert!((sigma - 11.547).abs() < 0.01, "sigma={sigma}");
    }

    #[test]
    fn insufficient_samples_returns_none() {
        assert_eq!(sigma_pct_from_closes(&[100.0, 105.0], 5), None);
        assert_eq!(sigma_pct_from_closes(&[], 1), None);
        assert_eq!(sigma_pct_from_closes(&[100.0], 1), None);
    }

    #[test]
    fn non_positive_and_nan_closes_are_skipped() {
        let closes = [100.0, 0.0, f64::NAN, 100.0, 110.0, 99.0, 108.9];
        // 0.0/NaN 打断窗口:有效收益率对是 100→110→99→108.9 的 3 个
        let sigma = sigma_pct_from_closes(&closes, 3).unwrap();
        assert!((sigma - 11.547).abs() < 0.01, "sigma={sigma}");
    }

    #[test]
    fn effective_thresholds_scale_and_clamp() {
        let c = cfg();
        // 典型市场股 σ=1.4 → 与固定默认基本重合
        let (low, high) = effective_thresholds(&c, Some(1.4), 2.5, 6.0);
        assert!((low - 2.45).abs() < 1e-9 && (high - 5.0).abs() < 1e-9);
        // 高波动 σ=8.9 → 双双触顶 8/12
        assert_eq!(effective_thresholds(&c, Some(8.9), 2.5, 6.0), (8.0, 12.0));
        // 极低波动 σ=0.8 → 触底 2/5
        assert_eq!(effective_thresholds(&c, Some(0.8), 2.5, 6.0), (2.0, 5.0));
    }

    #[test]
    fn missing_sigma_or_disabled_falls_back_to_base() {
        let c = cfg();
        assert_eq!(effective_thresholds(&c, None, 2.5, 6.0), (2.5, 6.0));
        let mut off = cfg();
        off.enabled = false;
        assert_eq!(effective_thresholds(&off, Some(9.0), 2.5, 6.0), (2.5, 6.0));
        assert_eq!(
            effective_thresholds(&c, Some(f64::NAN), 2.5, 6.0),
            (2.5, 6.0)
        );
    }

    #[test]
    fn contradictory_config_never_inverts_low_above_high() {
        let mut c = cfg();
        c.high_cap_pct = 6.0;
        c.low_cap_pct = 7.0;
        let (low, high) = effective_thresholds(&c, Some(9.0), 2.5, 6.0);
        assert!(high >= low, "low={low} high={high}");
    }

    /// A9:σ 按 (symbol, ET 交易日) 缓存 —— 同日重复取值不再发请求(阈值不漂移),
    /// 跨日不复用旧 σ(此处数据源不可达 → 回退 None → 固定阈值)。
    #[tokio::test]
    async fn same_day_sigma_is_cache_stable_and_new_day_invalidates() {
        let fmp_config = hone_core::config::FmpConfig {
            api_key: "test".into(),
            api_keys: vec![],
            // 不可达地址:任何真实网络请求都会立刻失败
            base_url: "http://127.0.0.1:1".into(),
            timeout: 1,
        };
        let provider = SigmaProvider::new(FmpClient::from_config(&fmp_config), cfg());
        let day1 = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2026, 8, 17).unwrap();
        provider.seed_cache("SNDK", day1, 8.9);

        assert_eq!(provider.sigma_pct("SNDK", day1).await, Some(8.9));
        assert_eq!(provider.sigma_pct("SNDK", day1).await, Some(8.9));
        // 新交易日:缓存失效,拉取失败 → None(调用方回退固定阈值)
        assert_eq!(provider.sigma_pct("SNDK", day2).await, None);
        // 禁用时恒 None,连缓存都不读
        let mut off = cfg();
        off.enabled = false;
        let disabled = SigmaProvider::new(FmpClient::from_config(&fmp_config), off);
        disabled.seed_cache("SNDK", day1, 8.9);
        assert_eq!(disabled.sigma_pct("SNDK", day1).await, None);
    }

    #[test]
    fn historical_response_parses_descending_rows() {
        let raw = serde_json::json!({
            "symbol": "X",
            "historical": [
                {"date": "2026-08-14", "close": 108.9},
                {"date": "2026-08-13", "close": 99.0},
                {"date": "2026-08-12", "close": 110.0},
                {"date": "2026-08-11", "close": 100.0}
            ]
        });
        let sigma = sigma_pct_from_historical_response(&raw, 3).unwrap();
        assert!((sigma - 11.547).abs() < 0.01, "sigma={sigma}");
    }
}
