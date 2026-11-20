//! Bounded V1 snapshot validation at the persistence boundary. The browser
//! model owns trading calculations; the server rejects malformed snapshots and
//! inconsistent recorded arithmetic rather than persisting an unreadable book.
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde_json::{Map, Value};

type Check<T = ()> = Result<T, String>;
const MAX_NUMBER: f64 = 9_007_199_254_740_991.0;
const REVIEW_KEYS: [&str; 4] = ["thesis", "value", "risk", "evidence"];

pub(super) fn validate(value: &Value) -> Check {
    safe_tree(value, 0)?;
    let root = object(value)?;
    let allowed = [
        "schemaVersion",
        "settings",
        "holdings",
        "events",
        "plans",
        "journal",
        "checks",
        "cashFlows",
        "corporateActions",
    ];
    if root.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("快照包含不支持的顶层字段；身份由服务器登录会话确定".into());
    }
    if value["schemaVersion"].as_u64() != Some(1) {
        return Err("只支持 schemaVersion 1 的交易记录".into());
    }
    object(&value["settings"])?;
    let settings = &value["settings"];
    choice(
        settings,
        "currency",
        &["USD", "CNY", "HKD", "EUR", "GBP", "JPY"],
    )?;
    for key in ["cash", "lifeCash"] {
        number(settings, key, MAX_NUMBER)?;
    }
    for key in [
        "budget",
        "singleLimit",
        "themeLimit",
        "targetGrowth",
        "targetDefense",
        "targetCash",
        "buffer",
        "maxMonthly",
        "growthShock",
        "defenseShock",
        "cashShock",
    ] {
        number(settings, key, 100.0)?;
    }
    if (n(settings, "targetGrowth") + n(settings, "targetDefense") + n(settings, "targetCash")
        - 100.0)
        .abs()
        > 0.001
    {
        return Err("成长、防守和现金目标应合计 100%".into());
    }
    choice(
        settings,
        "market",
        &[
            "positive", "neutral", "negative", "unknown", "cool", "hot", "risk", "cold", "warm",
        ],
    )?;
    text(settings, "marketNote", false)?;
    let holdings = list(root, "holdings", false)?;
    let events = list(root, "events", false)?;
    let plans = list(root, "plans", false)?;
    let journal = list(root, "journal", false)?;
    let cash_flows = list(root, "cashFlows", true)?;
    let actions = list(root, "corporateActions", true)?;
    let by_holding: HashMap<&str, &Value> = holdings.iter().map(|h| (s(h, "id"), h)).collect();
    let by_plan: HashMap<&str, &Value> = plans.iter().map(|p| (s(p, "id"), p)).collect();
    let mut symbols = HashSet::new();
    for h in holdings {
        for key in [
            "symbol",
            "name",
            "sector",
            "theme",
            "thesis",
            "invalidation",
            "evidence",
        ] {
            text(h, key, key == "symbol")?;
        }
        if !symbols.insert(s(h, "symbol").trim().to_uppercase()) {
            return Err("标的代码不能重复".into());
        }
        choice(h, "bucket", &["core", "flex", "defense"])?;
        for key in ["quantity", "cost", "price", "low", "base", "high"] {
            number(h, key, MAX_NUMBER)?;
        }
        if n(h, "quantity") > 0.0 && n(h, "price") <= 0.0 {
            return Err("持有标的必须有正数价格".into());
        }
        if !(n(h, "quantity") * n(h, "price")).is_finite() {
            return Err("持仓市值超出范围".into());
        }
        if n(h, "low") > n(h, "base") || n(h, "base") > n(h, "high") {
            return Err("情景价值应满足悲观 ≤ 基准 ≤ 乐观".into());
        }
        for key in ["scarcity", "differentiation"] {
            choice(h, key, &["stronger", "stable", "weaker", "unknown"])?;
        }
        choice(
            h,
            "valuation",
            &["attractive", "fair", "expensive", "unknown"],
        )?;
        for key in ["priceDate", "reviewDate", "nextReview"] {
            date(h, key, true, false)?;
        }
        if let Some(archived) = h.get("archived") {
            let archived = archived.as_bool().ok_or("归档状态应为布尔值")?;
            if archived
                && (n(h, "quantity") > 0.0
                    || plans.iter().any(|p| {
                        s(p, "holdingId") == s(h, "id")
                            && ["draft", "reviewed"].contains(&s(p, "status"))
                    }))
            {
                return Err("有持仓或活跃计划的标的不能归档".into());
            }
        }
    }
    for e in events {
        for key in ["symbol", "type", "title", "source", "note"] {
            text(e, key, key == "title")?;
        }
        date(e, "date", false, false)?;
        choice(e, "verification", &["pending", "verified", "refuted"])?;
        choice(e, "impact", &["positive", "neutral", "negative"])?;
        boolean(&e["reviewed"])?;
        if e.get("followUp").is_some() {
            date(e, "followUp", true, false)?;
        }
        if e.get("updatedAt").is_some() {
            timestamp(e, "updatedAt")?;
        }
    }
    let mut source_ids = HashSet::new();
    let mut ledger_ids = HashSet::new();
    for p in plans {
        if !ledger_ids.insert(s(p, "id")) {
            return Err("账本记录 ID 不能重复".into());
        }
        for key in ["holdingId", "symbol", "rationale"] {
            text(p, key, key != "rationale")?;
        }
        holding_reference(p, &by_holding, false)?;
        date(p, "date", false, false)?;
        choice(p, "side", &["buy", "sell"])?;
        choice(p, "reason", &["thesis", "valuation", "risk", "opportunity"])?;
        choice(p, "status", &["draft", "reviewed", "executed", "cancelled"])?;
        positive(p, "quantity")?;
        positive(p, "price")?;
        number(p, "fee", MAX_NUMBER)?;
        object(&p["checklist"])?;
        for key in REVIEW_KEYS {
            boolean(&p["checklist"][key])?;
        }
        if p.get("reviewedAt").is_some() {
            timestamp(p, "reviewedAt")?;
        }
        if let Some(fingerprint) = p.get("reviewFingerprint") {
            let fingerprint = fingerprint.as_str().ok_or("审查快照格式无效")?;
            if !fingerprint.starts_with("review-v1-") || fingerprint.len() > 100 {
                return Err("审查快照格式无效".into());
            }
        }
        if let Some(source) = p.get("sourcePlanId") {
            let source = source.as_str().ok_or("剩余计划来源无效")?;
            let original = by_plan.get(source).ok_or("剩余计划来源不存在")?;
            if source == s(p, "id")
                || s(original, "status") != "executed"
                || s(original, "holdingId") != s(p, "holdingId")
                || s(original, "side") != s(p, "side")
                || !source_ids.insert(source)
            {
                return Err("剩余计划来源无效或重复".into());
            }
        }
        if s(p, "status") == "executed" {
            timestamp(p, "executedAt")?;
            let x = &p["execution"];
            object(x)?;
            date(x, "date", false, true)?;
            for key in [
                "price",
                "quantity",
                "fee",
                "amount",
                "costBasis",
                "impactPct",
                "cashBefore",
                "cashAfter",
                "quantityBefore",
                "quantityAfter",
            ] {
                number(x, key, MAX_NUMBER)?;
            }
            positive(x, "price")?;
            positive(x, "quantity")?;
            for key in ["cashFlow", "realizedPnl"] {
                signed_number(x, key)?;
            }
            for key in ["growthPctBefore", "growthPctAfter"] {
                if x.get(key).is_some() {
                    number(x, key, 100.0)?;
                }
            }
            let buy = s(p, "side") == "buy";
            let direction = if buy { 1.0 } else { -1.0 };
            if !near(n(x, "amount"), n(x, "price") * n(x, "quantity"))
                || !near(n(x, "cashAfter"), n(x, "cashBefore") + n(x, "cashFlow"))
                || !near(
                    n(x, "quantityAfter"),
                    n(x, "quantityBefore") + direction * n(x, "quantity"),
                )
                || !near(n(x, "cashFlow"), -direction * n(x, "amount") - n(x, "fee"))
            {
                return Err("成交金额、现金或数量快照不一致".into());
            }
            let realized = if buy {
                0.0
            } else {
                n(x, "amount") - n(x, "fee") - n(x, "costBasis")
            };
            if !near(n(x, "realizedPnl"), realized)
                || (buy && !near(n(x, "costBasis"), n(x, "amount") + n(x, "fee")))
            {
                return Err("成交盈亏或成本快照不一致".into());
            }
            if x.get("remainingQuantity").is_some() {
                number(x, "remainingQuantity", MAX_NUMBER)?;
                if !near(
                    n(x, "remainingQuantity"),
                    (n(p, "quantity") - n(x, "quantity")).max(0.0),
                ) {
                    return Err("剩余成交数量不一致".into());
                }
            }
        }
    }
    for f in cash_flows {
        if !ledger_ids.insert(s(f, "id")) {
            return Err("账本记录 ID 不能重复".into());
        }
        choice(f, "type", &["deposit", "withdrawal", "dividend", "fee"])?;
        date(f, "date", false, true)?;
        timestamp(f, "recordedAt")?;
        positive(f, "amount")?;
        for key in ["cashBefore", "cashAfter"] {
            number(f, key, MAX_NUMBER)?;
        }
        for key in ["holdingId", "symbol", "note"] {
            text(f, key, key == "note")?;
        }
        holding_reference(f, &by_holding, true)?;
        let direction = if ["withdrawal", "fee"].contains(&s(f, "type")) {
            -1.0
        } else {
            1.0
        };
        if !near(
            n(f, "cashAfter"),
            n(f, "cashBefore") + direction * n(f, "amount"),
        ) {
            return Err("资金流水快照不一致".into());
        }
    }
    for a in actions {
        if !ledger_ids.insert(s(a, "id")) {
            return Err("账本记录 ID 不能重复".into());
        }
        choice(a, "type", &["split"])?;
        date(a, "date", false, true)?;
        timestamp(a, "recordedAt")?;
        for key in ["holdingId", "symbol", "note"] {
            text(a, key, true)?;
        }
        holding_reference(a, &by_holding, false)?;
        positive(a, "ratio")?;
        if n(a, "ratio") == 1.0 {
            return Err("拆并股比例不能为 1".into());
        }
        let before = &a["before"];
        let after = &a["after"];
        for snapshot in [before, after] {
            object(snapshot)?;
            for key in ["quantity", "cost", "price", "low", "base", "high"] {
                number(snapshot, key, MAX_NUMBER)?;
            }
            date(snapshot, "priceDate", true, false)?;
        }
        positive(before, "quantity")?;
        if !near(n(after, "quantity"), n(before, "quantity") * n(a, "ratio"))
            || ["cost", "price", "low", "base", "high"]
                .iter()
                .any(|key| !near(n(after, key), n(before, key) / n(a, "ratio")))
        {
            return Err("拆并股快照与比例不一致".into());
        }
        for key in ["cost", "price"] {
            if !near(
                n(before, "quantity") * n(before, key),
                n(after, "quantity") * n(after, key),
            ) {
                return Err("拆并股改变了总成本或总市值".into());
            }
        }
        let cancelled = a["cancelledPlanIds"]
            .as_array()
            .ok_or("缺少拆并股关联计划")?;
        if cancelled
            .iter()
            .any(|id| id.as_str().is_none_or(|id| !by_plan.contains_key(id)))
        {
            return Err("拆并股关联计划不存在".into());
        }
    }
    for j in journal {
        date(j, "date", false, false)?;
        choice(j, "discipline", &["yes", "no", "none"])?;
        text(j, "lesson", false)?;
        text(j, "next", false)?;
    }
    for (day, checks) in object(&value["checks"])? {
        valid_date(day, false, false)?;
        for checked in object(checks)?.values() {
            boolean(checked)?;
        }
    }
    Ok(())
}

fn object(value: &Value) -> Check<&Map<String, Value>> {
    value.as_object().ok_or_else(|| "缺少必要的对象字段".into())
}
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn n(v: &Value, key: &str) -> f64 {
    v[key].as_f64().unwrap_or(f64::NAN)
}
fn text(v: &Value, key: &str, required: bool) -> Check {
    let text = v[key].as_str().ok_or_else(|| format!("{key} 应为文本"))?;
    if text.encode_utf16().count() > 20_000 || (required && text.trim().is_empty()) {
        return Err(format!("{key} 文本为空或过长"));
    }
    Ok(())
}
fn signed_number(v: &Value, key: &str) -> Check<f64> {
    let value = v[key]
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| format!("{key} 应为有限数值"))?;
    Ok(value)
}
fn number(v: &Value, key: &str, max: f64) -> Check<f64> {
    let value = signed_number(v, key)?;
    if value < 0.0 || value > max {
        return Err(format!("{key} 数值超出允许范围"));
    }
    Ok(value)
}
fn positive(v: &Value, key: &str) -> Check {
    if number(v, key, MAX_NUMBER)? <= 0.0 {
        return Err(format!("{key} 必须大于零"));
    }
    Ok(())
}
fn choice(v: &Value, key: &str, choices: &[&str]) -> Check {
    if !choices.contains(&s(v, key)) {
        return Err(format!("{key} 选项无效"));
    }
    Ok(())
}
fn boolean(v: &Value) -> Check {
    if !v.is_boolean() {
        return Err("检查状态应为布尔值".into());
    }
    Ok(())
}
fn date(v: &Value, key: &str, empty: bool, past: bool) -> Check {
    let value = v[key].as_str().ok_or_else(|| format!("{key} 缺少日期"))?;
    valid_date(value, empty, past)
}
fn valid_date(value: &str, empty: bool, past: bool) -> Check {
    if empty && value.is_empty() {
        return Ok(());
    }
    let parsed =
        NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "日期无效".to_string())?;
    if parsed.format("%Y-%m-%d").to_string() != value {
        return Err("日期必须为 YYYY-MM-DD".into());
    }
    let today = (Utc::now() + Duration::hours(8)).date_naive();
    if past && parsed > today {
        return Err("记账日期不能在未来".into());
    }
    Ok(())
}
fn timestamp(v: &Value, key: &str) -> Check {
    DateTime::parse_from_rfc3339(s(v, key)).map_err(|_| format!("{key} 时间无效"))?;
    Ok(())
}
fn near(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-8 + 1e-10 * a.abs().max(b.abs())
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn list<'a>(root: &'a Map<String, Value>, key: &str, optional: bool) -> Check<&'a [Value]> {
    if optional && !root.contains_key(key) {
        return Ok(&[]);
    }
    let values = root
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{key} 应为数组"))?;
    if values.len() > 10_000 {
        return Err(format!("{key} 超过 10000 条限制"));
    }
    let mut ids = HashSet::new();
    for value in values {
        object(value)?;
        let id = s(value, "id");
        if !valid_id(id) || !ids.insert(id) {
            return Err(format!("{key} ID无效或重复"));
        }
    }
    Ok(values)
}
fn holding_reference(v: &Value, holdings: &HashMap<&str, &Value>, optional: bool) -> Check {
    let id = s(v, "holdingId");
    if optional && id.is_empty() {
        return Ok(());
    }
    if holdings
        .get(id)
        .is_none_or(|h| s(h, "symbol") != s(v, "symbol"))
    {
        return Err("引用的标的不存在或代码不一致".into());
    }
    Ok(())
}
fn safe_tree(value: &Value, depth: usize) -> Check {
    if depth > 20 {
        return Err("数据嵌套超过 20 层".into());
    }
    match value {
        Value::Object(map) => {
            if map.len() > 10_000 {
                return Err("对象字段过多".into());
            }
            for (key, child) in map {
                if ["__proto__", "constructor", "prototype"].contains(&key.as_str()) {
                    return Err("数据包含禁止字段".into());
                }
                safe_tree(child, depth + 1)?;
            }
        }
        Value::Array(values) => {
            if values.len() > 10_000 {
                return Err("数组超过 10000 条".into());
            }
            for child in values {
                safe_tree(child, depth + 1)?;
            }
        }
        Value::String(text) if text.encode_utf16().count() > 20_000 => {
            return Err("文本超过 20000 字符".into());
        }
        Value::Number(number) if number.as_f64().is_none_or(|n| !n.is_finite()) => {
            return Err("无效数值".into());
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn empty() -> Value {
        json!({"schemaVersion":1,"settings":{"currency":"USD","cash":0,"lifeCash":0,"budget":30,"singleLimit":20,"themeLimit":60,"targetGrowth":60,"targetDefense":30,"targetCash":10,"buffer":5,"maxMonthly":5,"growthShock":50,"defenseShock":15,"cashShock":0,"market":"neutral","marketNote":""},"holdings":[],"events":[],"plans":[],"journal":[],"checks":{}})
    }

    #[test]
    fn accepts_legacy_v1_and_rejects_identity_and_malformed_books() {
        let valid = empty();
        assert!(validate(&valid).is_ok());
        let mut state = valid.clone();
        state["cashFlows"] = json!([]);
        state["corporateActions"] = json!([]);
        assert!(validate(&state).is_ok());
        state["userId"] = json!("victim");
        assert!(validate(&state).is_err());
        let mut state = valid.clone();
        state["settings"]["cash"] = json!(-1);
        assert!(validate(&state).is_err());
        let mut state = valid.clone();
        state["settings"]["targetCash"] = json!(0);
        assert!(validate(&state).is_err());
        let mut state = valid.clone();
        state["checks"] = json!({"2026-02-30":{"check":true}});
        assert!(validate(&state).is_err());
        let mut state = valid.clone();
        state["checks"] = json!({"2026-02-28":{"check":"yes"}});
        assert!(validate(&state).is_err());
        let mut state = valid;
        state["checks"] = json!({"__proto__":{}});
        assert!(validate(&state).is_err());
    }

    #[test]
    fn rejects_inconsistent_cash_snapshot_and_future_record() {
        let mut state = empty();
        state["cashFlows"] = json!([{"id":"f1","date":"2026-01-01","type":"deposit","amount":100,"cashBefore":0,"cashAfter":100,"note":"测试","holdingId":"","symbol":"","recordedAt":"2026-01-01T00:00:00Z"}]);
        assert!(validate(&state).is_ok());
        state["cashFlows"][0]["cashAfter"] = json!(200);
        assert!(validate(&state).is_err());
        state["cashFlows"][0]["cashAfter"] = json!(100);
        state["cashFlows"][0]["date"] = json!("2999-01-01");
        assert!(validate(&state).is_err());
    }

    #[test]
    fn accepts_v11_model_fixture_with_partial_fill_remainder_and_split() {
        // Generated through the shipped JavaScript model using synthetic data.
        let state: Value = serde_json::from_str(include_str!("fixtures/v1_ledger.json")).unwrap();
        assert!(validate(&state).is_ok(), "{:?}", validate(&state));
        let mut invalid = state.clone();
        invalid["plans"][0]["execution"]["cashAfter"] = json!(1);
        assert!(validate(&invalid).is_err());
        let mut invalid = state.clone();
        invalid["corporateActions"][0]["after"]["quantity"] = json!(1);
        assert!(validate(&invalid).is_err());
        let mut invalid = state;
        invalid["holdings"][0]["archived"] = json!(true);
        assert!(validate(&invalid).is_err());
    }
}
