use std::sync::Arc;

use hone_core::ActorIdentity;
use regex::Regex;
use serde_json::{Value, json};

use crate::HoneBotCore;

const EVIDENCE_ITEM_CHAR_LIMIT: usize = 6_000;
const CONTRACT_FAILURE_MESSAGE: &str =
    "这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvestmentResponseContract {
    pub symbol_hint: String,
    pub deep_single_stock: bool,
    pub needs_outlook_evidence: bool,
}

impl InvestmentResponseContract {
    pub(crate) fn enforcement_block(&self, symbol: &str) -> String {
        if !self.deep_single_stock {
            return format!(
                "\n\n【本轮代码级证券数据门禁】\n已核验标的：{symbol}。价格、估值、财务、新闻和日期数字只能使用下方本轮证据；不得从历史对话或模型记忆补数。"
            );
        }
        format!(
            "\n\n【本轮代码级投研路由：单股深度分析，必须完整执行】\n已核验标的：{symbol}。这不是简短行情问答。最终答案必须按以下九个编号章节逐项回答，不得合并或省略：\n1. 结论\n2. 公司是什么、靠什么赚钱\n3. 护城河与竞争壁垒\n4. 行业位置与关键对手\n5. 财务质量\n6. 估值（至少两种适配方法或“倍数法 + 情景法”，写清假设）\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议（买、等、减、卖、观察之一，并给触发条件）\n先给结论并标明数据时间；明确区分本轮已核验事实、推断和动作。不得先追问持仓成本来代替完整回答。证据没有的数字明确写“本轮未核验”，不得从历史对话或模型记忆补数。"
        )
    }

    pub(crate) fn retry_block(&self, missing: &[&'static str]) -> String {
        format!(
            "\n\n【上一版草稿已被代码级完整性检查拒绝】\n缺失或不合格章节：{}。重新生成完整最终答案，严格使用九个编号章节；不得解释检查过程，不得用追问持仓成本代替动作建议。",
            missing.join("、")
        )
    }
}

pub(crate) fn contract_failure_message() -> &'static str {
    CONTRACT_FAILURE_MESSAGE
}

pub(crate) fn classify_investment_response_contract(
    input: &str,
) -> Option<InvestmentResponseContract> {
    let symbol_hint = extract_security_hint(input)?;
    let normalized = input.to_ascii_lowercase();
    let deep_single_stock = [
        "分析",
        "研究",
        "怎么看",
        "值不值得",
        "能不能买",
        "能否买",
        "起飞",
        "前景",
        "估值",
        "目标价",
        "未来",
        "财报",
        "业绩",
        "基本面",
        "bull",
        "bear",
        "case",
    ]
    .iter()
    .any(|keyword| normalized.contains(keyword))
        || Regex::new(r"(?i)\bq[1-4]\b")
            .expect("quarter regex")
            .is_match(input);
    let needs_outlook_evidence = deep_single_stock
        && [
            "起飞", "前景", "未来", "财报", "业绩", "催化", "q1", "q2", "q3", "q4",
        ]
        .iter()
        .any(|keyword| normalized.contains(keyword));
    Some(InvestmentResponseContract {
        symbol_hint,
        deep_single_stock,
        needs_outlook_evidence,
    })
}

pub(crate) async fn append_verified_investment_evidence(
    core: &Arc<HoneBotCore>,
    actor: &ActorIdentity,
    channel_target: &str,
    allow_cron: bool,
    user_input: &str,
    runtime_input: &mut String,
) -> Result<Option<InvestmentResponseContract>, String> {
    let Some(contract) = classify_investment_response_contract(user_input) else {
        return Ok(None);
    };
    let registry = core.create_tool_registry(Some(actor), channel_target, allow_cron);
    let search = registry
        .execute_tool(
            "data_fetch",
            json!({"data_type": "search", "ticker": contract.symbol_hint}),
        )
        .await
        .map_err(|err| format!("证券实体核验失败：{err}"))?;
    let symbol = resolve_verified_symbol(&contract.symbol_hint, &search).ok_or_else(|| {
        format!(
            "当前无法稳定核验证券实体 `{}`，已停止生成可能指向错误公司的分析。",
            contract.symbol_hint
        )
    })?;
    let quote = registry
        .execute_tool(
            "data_fetch",
            json!({"data_type": "quote", "ticker": symbol}),
        )
        .await
        .map_err(|err| format!("{symbol} 行情核验失败：{err}"))?;
    if !quote_has_positive_matching_price(&quote, &symbol) {
        return Err(format!(
            "当前无法稳定核验 {symbol} 的本轮同标的有效价格，已停止生成数值性投资结论。"
        ));
    }

    let mut evidence = vec![("实体检索", search), ("最新行情", quote)];
    if contract.deep_single_stock {
        let (profile, financials, news) = tokio::join!(
            registry.execute_tool(
                "data_fetch",
                json!({"data_type": "profile", "ticker": symbol}),
            ),
            registry.execute_tool(
                "data_fetch",
                json!({"data_type": "financials", "ticker": symbol}),
            ),
            registry.execute_tool("data_fetch", json!({"data_type": "news", "ticker": symbol}),),
        );
        let financials = financials.map_err(|err| format!("{symbol} 财务核验失败：{err}"))?;
        if !has_nonempty_data(&financials) {
            return Err(format!(
                "当前无法稳定核验 {symbol} 的本轮财务数据，已停止生成完整估值结论。"
            ));
        }
        evidence.push(("公司概况", result_or_error_value(profile)));
        evidence.push(("财务数据", financials));
        evidence.push(("公司新闻", result_or_error_value(news)));
    }
    if contract.needs_outlook_evidence {
        let from = hone_core::beijing_now().date_naive();
        let to = from + chrono::Duration::days(120);
        let calendar = registry
            .execute_tool(
                "data_fetch",
                json!({
                    "data_type": "earnings_calendar",
                    "ticker": symbol,
                    "from": from.format("%Y-%m-%d").to_string(),
                    "to": to.format("%Y-%m-%d").to_string(),
                }),
            )
            .await;
        evidence.push((
            "未来 120 天财报日历（仅当前标的）",
            matching_symbol_objects(&result_or_error_value(calendar), &symbol),
        ));
    }

    runtime_input.push_str(&contract.enforcement_block(&symbol));
    runtime_input.push_str("\n\n【本轮已核验数据证据】\n");
    for (label, value) in evidence {
        runtime_input.push_str(&format!(
            "- {label}：{}\n",
            truncate_chars(&value.to_string(), EVIDENCE_ITEM_CHAR_LIMIT)
        ));
    }
    runtime_input
        .push_str("以上证据是本轮运行时注入，不得向用户暴露工具名、原始 JSON 或内部检查流程。\n");
    Ok(Some(contract))
}

pub(crate) fn missing_deep_single_stock_sections(content: &str) -> Vec<&'static str> {
    let text = content.to_ascii_lowercase();
    let mut missing = Vec::new();
    require_any(&text, &["结论"], "1. 结论", &mut missing);
    require_any(
        &text,
        &["靠什么赚钱", "商业模式", "公司是什么"],
        "2. 公司与商业模式",
        &mut missing,
    );
    require_any(
        &text,
        &["护城河", "竞争壁垒", "壁垒"],
        "3. 护城河与壁垒",
        &mut missing,
    );
    require_any(
        &text,
        &["行业位置", "关键对手", "竞争对手"],
        "4. 行业位置与对手",
        &mut missing,
    );
    require_any(
        &text,
        &["财务质量", "毛利率", "自由现金流"],
        "5. 财务质量",
        &mut missing,
    );
    require_any(&text, &["估值"], "6. 估值", &mut missing);
    if !(text.contains("bull") && text.contains("bear") && text.contains("base")) {
        missing.push("7. Bull / Bear / Base Case");
    }
    if !(text.contains("催化") && text.contains("风险") && text.contains("证伪")) {
        missing.push("8. 催化、风险与证伪");
    }
    require_any(
        &text,
        &["动作建议", "行动建议", "操作建议"],
        "9. 动作建议",
        &mut missing,
    );
    for (number, label) in [
        (1, "1. 结论"),
        (2, "2. 公司与商业模式"),
        (3, "3. 护城河与壁垒"),
        (4, "4. 行业位置与对手"),
        (5, "5. 财务质量"),
        (6, "6. 估值"),
        (7, "7. Bull / Bear / Base Case"),
        (8, "8. 催化、风险与证伪"),
        (9, "9. 动作建议"),
    ] {
        if !has_numbered_section(content, number) && !missing.contains(&label) {
            missing.push(label);
        }
    }
    if !(text.contains("北京时间") || text.contains("数据时间")) {
        missing.push("数据时间口径");
    }
    // Do not require the model to repeat the exact words “事实 / 推断”. A draft has
    // already separated the two when it labels source-backed statements as verified
    // and forward-looking statements as assumptions, estimates, or judgments.
    let has_fact_marker = ["事实", "已核验", "实际", "本轮数据"]
        .iter()
        .any(|marker| text.contains(marker));
    let has_inference_marker = ["推断", "假设", "估算", "判断", "预期", "情景"]
        .iter()
        .any(|marker| text.contains(marker));
    if !(has_fact_marker && has_inference_marker) {
        missing.push("事实 / 推断标识");
    }
    let valuation_method_count = [
        ["p/s", "ps 倍", "ps估值"].as_slice(),
        ["p/e", "pe 倍", "pe估值"].as_slice(),
        ["ev/ebitda", "ev / ebitda"].as_slice(),
        ["fcf yield", "自由现金流收益率"].as_slice(),
        ["dcf", "现金流折现"].as_slice(),
        ["sotp", "分部估值"].as_slice(),
        ["情景法", "情景分析"].as_slice(),
    ]
    .iter()
    .filter(|aliases| aliases.iter().any(|alias| text.contains(alias)))
    .count();
    if valuation_method_count < 2 {
        missing.push("至少两种估值方法");
    }
    missing
}

fn has_numbered_section(content: &str, number: u8) -> bool {
    Regex::new(&format!(
        r"(?m)^\s*(?:#{{1,6}}\s*)?(?:\*\*)?\s*{number}\s*[.、)]"
    ))
    .expect("numbered section regex")
    .is_match(content)
}

fn require_any(
    content: &str,
    keywords: &[&str],
    label: &'static str,
    missing: &mut Vec<&'static str>,
) {
    if !keywords.iter().any(|keyword| content.contains(keyword)) {
        missing.push(label);
    }
}

fn extract_security_hint(input: &str) -> Option<String> {
    let regex = Regex::new(r"(?i)(?:\$)?[a-z][a-z0-9.-]{1,9}").expect("ticker regex");
    let denied = [
        "Q1", "Q2", "Q3", "Q4", "AI", "ARR", "PE", "PS", "EV", "EBIT", "EBITDA", "EPS", "FCF",
        "DCF", "ETF", "ROE", "ROIC", "YOY", "QOQ", "CAGR", "CASE", "BULL", "BEAR", "THE", "AND",
        "CAN", "BUY", "SELL", "STOCK",
    ];
    let candidates = regex
        .find_iter(input)
        .filter_map(|matched| {
            let raw = matched.as_str();
            let candidate = raw.trim_start_matches('$').to_ascii_uppercase();
            if assignment_key_should_be_ignored(input, matched.start(), matched.end())
                || assignment_value_should_be_ignored(
                    input,
                    matched.start(),
                    matched.end(),
                    &candidate,
                )
            {
                return None;
            }
            (!denied.contains(&candidate.as_str())).then_some((raw, candidate))
        })
        .collect::<Vec<_>>();
    if let Some((_, candidate)) = candidates.iter().find(|(raw, _)| raw.starts_with('$')) {
        return Some(candidate.clone());
    }
    if let Some((_, candidate)) = candidates.iter().find(|(raw, candidate)| {
        let bare = raw.trim_start_matches('$');
        bare == bare.to_ascii_uppercase() && (2..=6).contains(&candidate.len())
    }) {
        return Some(candidate.clone());
    }
    if candidates
        .iter()
        .any(|(_, candidate)| candidate == "NEBIUS")
    {
        return Some("NBIS".to_string());
    }
    let contains_cjk = input
        .chars()
        .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character));
    contains_cjk.then_some(())?;
    candidates
        .iter()
        .find(|(_, candidate)| (2..=6).contains(&candidate.len()))
        .map(|(_, candidate)| candidate.clone())
}

fn assignment_key_should_be_ignored(input: &str, start: usize, end: usize) -> bool {
    assignment_context(input, start, end).is_some_and(|context| context.is_key)
}

fn assignment_value_should_be_ignored(
    input: &str,
    start: usize,
    end: usize,
    candidate: &str,
) -> bool {
    let Some(context) = assignment_context(input, start, end) else {
        return false;
    };
    if !context.is_value {
        return false;
    }
    let value = context.value.to_ascii_uppercase();
    let schedule_value_tokens = [
        "DAILY",
        "WEEKLY",
        "MONTHLY",
        "TRADING_DAY",
        "TRADING-WEEK",
        "TRADING-MONTH",
        "WEEKDAY",
        "WEEKDAYS",
        "HOURLY",
    ];
    schedule_value_tokens.contains(&value.as_str())
        || (context.key.eq_ignore_ascii_case("repeat")
            && value
                .split(|ch: char| !ch.is_ascii_alphanumeric())
                .any(|token| !token.is_empty() && token.eq_ignore_ascii_case(candidate)))
}

struct AssignmentContext<'a> {
    key: &'a str,
    value: &'a str,
    is_key: bool,
    is_value: bool,
}

fn assignment_context(input: &str, start: usize, end: usize) -> Option<AssignmentContext<'_>> {
    let segment_start = input[..start]
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace())
        .map_or(0, |(idx, ch)| idx + ch.len_utf8());
    let segment_end = input[end..]
        .char_indices()
        .find(|(_, ch)| ch.is_whitespace())
        .map_or(input.len(), |(idx, _)| end + idx);
    let segment = &input[segment_start..segment_end];
    let equals_offset = segment.find('=')?;
    let key = segment[..equals_offset]
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-');
    let value = segment[equals_offset + 1..]
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-');
    if key.is_empty() || value.is_empty() {
        return None;
    }
    let relative_start = start.saturating_sub(segment_start);
    let relative_end = end.saturating_sub(segment_start);
    Some(AssignmentContext {
        key,
        value,
        is_key: relative_end <= equals_offset,
        is_value: relative_start > equals_offset,
    })
}

fn resolve_verified_symbol(hint: &str, search: &Value) -> Option<String> {
    if value_has_error(search) {
        return None;
    }
    let mut symbols = Vec::new();
    collect_string_fields(
        search.get("data").unwrap_or(&Value::Null),
        &["symbol", "ticker"],
        &mut symbols,
    );
    symbols
        .iter()
        .find(|symbol| symbol.eq_ignore_ascii_case(hint))
        .or_else(|| symbols.first())
        .map(|symbol| symbol.to_ascii_uppercase())
}

fn quote_has_positive_matching_price(value: &Value, symbol: &str) -> bool {
    match value {
        Value::Object(map) => {
            let symbol_ok = map
                .get("symbol")
                .or_else(|| map.get("ticker"))
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(symbol));
            let price_ok = map
                .get("price")
                .and_then(Value::as_f64)
                .is_some_and(|price| price.is_finite() && price > 0.0);
            (symbol_ok && price_ok)
                || map
                    .values()
                    .any(|child| quote_has_positive_matching_price(child, symbol))
        }
        Value::Array(items) => items
            .iter()
            .any(|child| quote_has_positive_matching_price(child, symbol)),
        _ => false,
    }
}

fn has_nonempty_data(value: &Value) -> bool {
    !value_has_error(value)
        && value.get("data").is_some_and(|data| match data {
            Value::Array(items) => !items.is_empty(),
            Value::Object(map) => !map.is_empty(),
            _ => !data.is_null(),
        })
}

fn value_has_error(value: &Value) -> bool {
    value
        .get("error")
        .is_some_and(|error| !error.is_null() && error.as_str() != Some(""))
}

fn result_or_error_value(result: hone_core::HoneResult<Value>) -> Value {
    result.unwrap_or_else(|err| json!({"error": err.to_string()}))
}

fn collect_string_fields(value: &Value, keys: &[&str], output: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(text) = map.get(*key).and_then(Value::as_str) {
                    output.push(text.to_string());
                }
            }
            for child in map.values() {
                collect_string_fields(child, keys, output);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_string_fields(child, keys, output);
            }
        }
        _ => {}
    }
}

fn matching_symbol_objects(value: &Value, symbol: &str) -> Value {
    let mut output = Vec::new();
    collect_matching_symbol_objects(value.get("data").unwrap_or(value), symbol, &mut output);
    Value::Array(output)
}

fn collect_matching_symbol_objects(value: &Value, symbol: &str, output: &mut Vec<Value>) {
    if output.len() >= 8 {
        return;
    }
    match value {
        Value::Object(map) => {
            if map
                .get("symbol")
                .or_else(|| map.get("ticker"))
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(symbol))
            {
                output.push(value.clone());
                return;
            }
            for child in map.values() {
                collect_matching_symbol_objects(child, symbol, output);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_matching_symbol_objects(child, symbol, output);
            }
        }
        _ => {}
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        format!(
            "{}…",
            value
                .chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_investment_response_contract, extract_security_hint,
        missing_deep_single_stock_sections, quote_has_positive_matching_price,
        resolve_verified_symbol,
    };
    use serde_json::json;

    #[test]
    fn nbis_q3_question_requires_full_single_stock_contract() {
        let contract = classify_investment_response_contract("我想了解Q3的时候nbis能不能起飞")
            .expect("contract");
        assert_eq!(contract.symbol_hint, "NBIS");
        assert!(contract.deep_single_stock);
        assert!(contract.needs_outlook_evidence);
        assert!(contract.enforcement_block("NBIS").contains("九个编号章节"));
    }

    #[test]
    fn quote_only_question_does_not_require_nine_sections() {
        assert!(
            !classify_investment_response_contract("NBIS现在多少钱")
                .expect("contract")
                .deep_single_stock
        );
    }

    #[test]
    fn english_question_prefers_explicit_uppercase_ticker() {
        let contract =
            classify_investment_response_contract("Can NBIS take off in Q3?").expect("contract");
        assert_eq!(contract.symbol_hint, "NBIS");
        assert!(contract.deep_single_stock);
    }

    #[test]
    fn repeat_assignment_is_not_treated_as_security_hint() {
        assert_eq!(
            extract_security_hint("18:00 美股盘前 X 英文帖 repeat=daily"),
            None
        );
    }

    #[test]
    fn metric_tokens_are_not_treated_as_security_hint() {
        assert_eq!(
            extract_security_hint("A股港股收盘后跨市场复盘，估值使用 EV/EBITDA"),
            None
        );
    }

    #[test]
    fn real_ticker_still_wins_over_repeat_assignment_noise() {
        let contract =
            classify_investment_response_contract("repeat=daily，帮我分析 NBIS 下一季财报和估值")
                .expect("contract");
        assert_eq!(contract.symbol_hint, "NBIS");
        assert!(contract.deep_single_stock);
    }

    #[test]
    fn incomplete_nbis_reply_is_rejected() {
        let missing = missing_deep_single_stock_sections(
            "结论：Q3可能起飞。Bull Case 看增长，Bear Case 看竞争。你成本多少？",
        );
        assert!(missing.contains(&"2. 公司与商业模式"));
        assert!(missing.contains(&"5. 财务质量"));
        assert!(missing.contains(&"9. 动作建议"));
    }

    #[test]
    fn complete_nine_part_reply_passes() {
        let content = "数据时间：北京时间 2026-07-15。事实与推断分开。\n1. 结论\n2. 公司是什么、靠什么赚钱\n3. 护城河与竞争壁垒\n4. 行业位置与关键对手\n5. 财务质量与自由现金流\n6. 估值：P/S + 情景法\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议";
        assert!(missing_deep_single_stock_sections(content).is_empty());
    }

    #[test]
    fn verified_facts_and_labeled_assumptions_count_as_separated() {
        let content = "数据时间：北京时间 2026-07-15。\n1. 结论\n2. 公司是什么、靠什么赚钱\n3. 护城河与竞争壁垒\n4. 行业位置与关键对手\n5. 财务质量：以下为本轮已核验数据\n6. 估值：P/S + EV/EBITDA；假设与估算如下\n7. Bull / Bear / Base Case\n8. 催化剂、风险点、证伪条件\n9. 动作建议";
        assert!(missing_deep_single_stock_sections(content).is_empty());
    }

    #[test]
    fn search_and_quote_must_match_symbol() {
        let search = json!({"data": [{"symbol": "NBIS", "name": "Nebius Group"}]});
        assert_eq!(
            resolve_verified_symbol("NBIS", &search).as_deref(),
            Some("NBIS")
        );
        assert!(
            resolve_verified_symbol(
                "NBIS",
                &json!({"data_type": "search", "ticker": "NBIS", "data": []})
            )
            .is_none()
        );
        assert!(quote_has_positive_matching_price(
            &json!({"data": [{"symbol": "NBIS", "price": 194.09}]}),
            "NBIS"
        ));
        assert!(!quote_has_positive_matching_price(
            &json!({"data": [{"symbol": "MBIS", "price": 15.0}]}),
            "NBIS"
        ));
    }
}
