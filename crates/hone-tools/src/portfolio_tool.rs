//! PortfolioTool — 持仓管理工具
//!
//! 管理用户的投资组合持仓。

use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_memory::portfolio::{Holding, Portfolio, PortfolioStorage, normalize_holding_horizon};
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

/// PortfolioTool — 持仓管理
pub struct PortfolioTool {
    data_dir: String,
    actor: ActorIdentity,
}

impl PortfolioTool {
    pub fn new(data_dir: &str, actor: ActorIdentity) -> Self {
        Self {
            data_dir: data_dir.to_string(),
            actor,
        }
    }
}

#[async_trait]
impl Tool for PortfolioTool {
    fn name(&self) -> &str {
        "portfolio"
    }

    fn description(&self) -> &str {
        "管理投资组合持仓。支持股票和期权。支持操作：view（查看持仓）、add（添加持仓）、remove（删除持仓）、update（更新持仓）。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "操作类型".to_string(),
                required: true,
                r#enum: Some(vec![
                    "view".into(),
                    "add".into(),
                    "remove".into(),
                    "update".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "ticker".to_string(),
                param_type: "string".to_string(),
                description: "股票代码或期权合约代码。若是期权，也可留空并由 underlying/expiration_date/option_type/strike_price 自动生成。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "asset_type".to_string(),
                param_type: "string".to_string(),
                description: "持仓类型，默认 stock，可选 option。".to_string(),
                required: false,
                r#enum: Some(vec!["stock".into(), "option".into()]),
                items: None,
            },
            ToolParameter {
                name: "quantity".to_string(),
                param_type: "number".to_string(),
                description: "数量。股票表示股数，期权表示合约张数。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "cost_basis".to_string(),
                param_type: "number".to_string(),
                description: "成本价。股票为每股成本，期权为每张合约权利金成本。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "underlying".to_string(),
                param_type: "string".to_string(),
                description: "期权标的代码，例如 AAPL。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "option_type".to_string(),
                param_type: "string".to_string(),
                description: "期权类型，call 或 put。".to_string(),
                required: false,
                r#enum: Some(vec!["call".into(), "put".into()]),
                items: None,
            },
            ToolParameter {
                name: "strike_price".to_string(),
                param_type: "number".to_string(),
                description: "期权行权价。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "expiration_date".to_string(),
                param_type: "string".to_string(),
                description: "期权到期日，建议 YYYY-MM-DD。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "contract_multiplier".to_string(),
                param_type: "number".to_string(),
                description: "期权合约乘数，默认 100。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "notes".to_string(),
                param_type: "string".to_string(),
                description: "备注。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "holding_horizon".to_string(),
                param_type: "string".to_string(),
                description: "持有期限倾向。建议 long_term 或 short_term，也兼容 long/short/长持/短持。".to_string(),
                required: false,
                r#enum: Some(vec!["long_term".into(), "short_term".into()]),
                items: None,
            },
            ToolParameter {
                name: "strategy_notes".to_string(),
                param_type: "string".to_string(),
                description: "特殊策略说明，例如 现金担保卖沽、财报事件驱动、核心长期仓位。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "holdings".to_string(),
                param_type: "array".to_string(),
                description: "批量写入或删除时使用。数组里的每个对象都支持 ticker/asset_type/quantity/cost_basis/underlying/option_type/strike_price/expiration_date/contract_multiplier/holding_horizon/strategy_notes/notes。".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({
                    "type": "object"
                })),
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let storage = PortfolioStorage::new(&self.data_dir);
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("view");

        match action {
            "view" => {
                let portfolio = storage.load(&self.actor)?;
                let data = match portfolio {
                    Some(p) => serde_json::to_value(&p).unwrap_or_default(),
                    None => serde_json::json!({"holdings": [], "message": "暂无持仓"}),
                };
                Ok(serde_json::json!({
                    "action": "view",
                    "portfolio": data
                }))
            }
            "add" | "update" => {
                let input_holdings = parse_holdings_from_args(&args)?;

                let mut portfolio = storage.load(&self.actor)?.unwrap_or_else(|| Portfolio {
                    actor: Some(self.actor.clone()),
                    user_id: self.actor.user_id.clone(),
                    holdings: Vec::new(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                });

                let mut processed = Vec::with_capacity(input_holdings.len());

                for holding in input_holdings {
                    if let Some(existing) = portfolio
                        .holdings
                        .iter_mut()
                        .find(|h| h.symbol == holding.symbol && h.asset_type == holding.asset_type)
                    {
                        *existing = holding.clone();
                    } else {
                        portfolio.holdings.push(holding.clone());
                    }
                    processed.push(serde_json::json!({
                        "ticker": holding.symbol,
                        "asset_type": holding.asset_type,
                        "holding_horizon": holding.holding_horizon,
                        "strategy_notes": holding.strategy_notes
                    }));
                }
                portfolio.updated_at = chrono::Utc::now().to_rfc3339();

                storage.save(&self.actor, &portfolio)?;
                let mut response = serde_json::json!({
                    "action": action,
                    "count": processed.len(),
                    "holdings": processed,
                    "success": true
                });
                if let Some(first) = response["holdings"]
                    .as_array()
                    .and_then(|items| items.first())
                    .cloned()
                {
                    response["ticker"] = first["ticker"].clone();
                    response["asset_type"] = first["asset_type"].clone();
                }
                Ok(response)
            }
            "remove" => {
                let removals = parse_removals_from_args(&args)?;

                if let Some(mut portfolio) = storage.load(&self.actor)? {
                    for removal in &removals {
                        portfolio.holdings.retain(|h| {
                            !(h.symbol == removal.symbol && h.asset_type == removal.asset_type)
                        });
                    }
                    portfolio.updated_at = chrono::Utc::now().to_rfc3339();
                    storage.save(&self.actor, &portfolio)?;
                }

                let mut response = serde_json::json!({
                    "action": "remove",
                    "count": removals.len(),
                    "holdings": removals.iter().map(|removal| serde_json::json!({
                        "ticker": removal.symbol,
                        "asset_type": removal.asset_type
                    })).collect::<Vec<_>>(),
                    "success": true
                });
                if let Some(first) = response["holdings"]
                    .as_array()
                    .and_then(|items| items.first())
                    .cloned()
                {
                    response["ticker"] = first["ticker"].clone();
                    response["asset_type"] = first["asset_type"].clone();
                }
                Ok(response)
            }
            _ => Ok(serde_json::json!({"error": format!("不支持的操作: {action}")})),
        }
    }
}

#[derive(Clone, Default)]
struct OptionMetadata {
    underlying: Option<String>,
    option_type: Option<String>,
    strike_price: Option<f64>,
    expiration_date: Option<String>,
    contract_multiplier: Option<f64>,
}

#[derive(Clone)]
struct HoldingInput {
    symbol: String,
    asset_type: String,
    quantity: f64,
    cost_basis: f64,
    holding_horizon: Option<String>,
    strategy_notes: Option<String>,
    notes: Option<String>,
    option_meta: OptionMetadata,
}

#[derive(Clone)]
struct RemovalInput {
    symbol: String,
    asset_type: String,
}

fn normalize_asset_type(asset_type: &str) -> hone_core::HoneResult<String> {
    use hone_core::HoneError;

    match asset_type.trim().to_ascii_lowercase().as_str() {
        "" | "stock" => Ok("stock".to_string()),
        "option" => Ok("option".to_string()),
        other => Err(HoneError::Tool(format!(
            "不支持的 asset_type: {other}，当前只支持 stock 或 option"
        ))),
    }
}

fn normalize_option_type(option_type: &str) -> hone_core::HoneResult<String> {
    use hone_core::HoneError;

    match option_type.trim().to_ascii_lowercase().as_str() {
        "call" | "c" => Ok("call".to_string()),
        "put" | "p" => Ok("put".to_string()),
        other => Err(HoneError::Tool(format!(
            "不支持的 option_type: {other}，当前只支持 call 或 put"
        ))),
    }
}

fn normalized_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_uppercase())
        .filter(|v| !v.is_empty())
}

fn normalized_date(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn normalized_notes(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_holdings_from_args(args: &Value) -> hone_core::HoneResult<Vec<Holding>> {
    let inputs = if let Some(items) = args.get("holdings").and_then(|value| value.as_array()) {
        let mut parsed = Vec::with_capacity(items.len());
        for item in items {
            parsed.push(parse_single_holding_input(item)?);
        }
        parsed
    } else {
        vec![parse_single_holding_input(args)?]
    };

    Ok(inputs
        .into_iter()
        .map(|input| Holding {
            symbol: input.symbol,
            asset_type: input.asset_type,
            shares: input.quantity,
            avg_cost: input.cost_basis,
            underlying: input.option_meta.underlying,
            option_type: input.option_meta.option_type,
            strike_price: input.option_meta.strike_price,
            expiration_date: input.option_meta.expiration_date,
            contract_multiplier: input.option_meta.contract_multiplier,
            holding_horizon: input.holding_horizon,
            strategy_notes: input.strategy_notes,
            notes: input.notes,
        })
        .collect())
}

fn parse_single_holding_input(value: &Value) -> hone_core::HoneResult<HoldingInput> {
    let asset_type = normalize_asset_type(
        value
            .get("asset_type")
            .and_then(|v| v.as_str())
            .unwrap_or("stock"),
    )?;
    let symbol = resolve_symbol(value, &asset_type)?;
    let quantity = value
        .get("quantity")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| value.get("shares").and_then(|v| v.as_f64()).unwrap_or(0.0));
    let cost_basis = value
        .get("cost_basis")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| {
            value
                .get("avg_cost")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        });
    let holding_horizon = value
        .get("holding_horizon")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("horizon").and_then(|v| v.as_str()))
        .and_then(normalize_holding_horizon);
    let strategy_notes = normalized_notes(value.get("strategy_notes"))
        .or_else(|| normalized_notes(value.get("strategy")));
    let notes = normalized_notes(value.get("notes"));
    let option_meta = option_metadata(value, &asset_type)?;

    Ok(HoldingInput {
        symbol,
        asset_type,
        quantity,
        cost_basis,
        holding_horizon,
        strategy_notes,
        notes,
        option_meta,
    })
}

fn parse_removals_from_args(args: &Value) -> hone_core::HoneResult<Vec<RemovalInput>> {
    if let Some(items) = args.get("holdings").and_then(|value| value.as_array()) {
        let mut parsed = Vec::with_capacity(items.len());
        for item in items {
            parsed.push(parse_single_removal(item)?);
        }
        Ok(parsed)
    } else {
        Ok(vec![parse_single_removal(args)?])
    }
}

fn parse_single_removal(value: &Value) -> hone_core::HoneResult<RemovalInput> {
    let asset_type = normalize_asset_type(
        value
            .get("asset_type")
            .and_then(|v| v.as_str())
            .unwrap_or("stock"),
    )?;
    let symbol = resolve_symbol(value, &asset_type)?;
    Ok(RemovalInput { symbol, asset_type })
}

fn resolve_symbol(args: &Value, asset_type: &str) -> hone_core::HoneResult<String> {
    use hone_core::HoneError;

    if let Some(ticker) = normalized_string(args.get("ticker")) {
        return Ok(ticker);
    }

    if asset_type != "option" {
        return Err(HoneError::Tool("缺少 ticker 参数".into()));
    }

    let underlying = normalized_string(args.get("underlying"))
        .ok_or_else(|| HoneError::Tool("期权持仓缺少 underlying 参数".into()))?;
    let expiration_date = normalized_date(args.get("expiration_date"))
        .ok_or_else(|| HoneError::Tool("期权持仓缺少 expiration_date 参数".into()))?;
    let option_type = normalize_option_type(
        args.get("option_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HoneError::Tool("期权持仓缺少 option_type 参数".into()))?,
    )?;
    let strike_price = args
        .get("strike_price")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| HoneError::Tool("期权持仓缺少 strike_price 参数".into()))?;

    Ok(format!(
        "{} {} {} {}",
        underlying,
        expiration_date,
        option_type
            .to_ascii_uppercase()
            .chars()
            .next()
            .unwrap_or('C'),
        trim_trailing_zero(strike_price)
    ))
}

fn option_metadata(args: &Value, asset_type: &str) -> hone_core::HoneResult<OptionMetadata> {
    if asset_type != "option" {
        return Ok(OptionMetadata::default());
    }

    let option_type = args
        .get("option_type")
        .and_then(|v| v.as_str())
        .map(normalize_option_type)
        .transpose()?;

    Ok(OptionMetadata {
        underlying: normalized_string(args.get("underlying")),
        option_type,
        strike_price: args.get("strike_price").and_then(|v| v.as_f64()),
        expiration_date: normalized_date(args.get("expiration_date")),
        contract_multiplier: args
            .get("contract_multiplier")
            .and_then(|v| v.as_f64())
            .or(Some(100.0)),
    })
}

fn trim_trailing_zero(value: f64) -> String {
    let rendered = format!("{value:.4}");
    rendered
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_memory::portfolio::{HOLDING_HORIZON_LONG_TERM, HOLDING_HORIZON_SHORT_TERM};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn portfolio_crud_flow() {
        let data_dir = make_temp_dir("hone_portfolio_tool");
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).expect("actor");
        let tool = PortfolioTool::new(&data_dir, actor);

        let view_empty = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view empty");
        assert_eq!(view_empty["action"], "view");
        assert_eq!(view_empty["portfolio"]["message"], "暂无持仓");

        let add_resp = tool
            .execute(serde_json::json!({
                "action":"add",
                "ticker":"AAPL",
                "asset_type":"stock",
                "quantity":10.0,
                "cost_basis":200.5,
                "holding_horizon":"long_term",
                "strategy_notes":"核心长期仓位"
            }))
            .await
            .expect("add");
        assert_eq!(add_resp["success"], true);

        let view_after_add = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view after add");
        let holdings = view_after_add["portfolio"]["holdings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0]["symbol"], "AAPL");
        assert_eq!(holdings[0]["asset_type"], "stock");
        assert_eq!(holdings[0]["shares"], 10.0);
        assert_eq!(holdings[0]["holding_horizon"], HOLDING_HORIZON_LONG_TERM);
        assert_eq!(holdings[0]["strategy_notes"], "核心长期仓位");

        let update_resp = tool
            .execute(serde_json::json!({
                "action":"update",
                "ticker":"AAPL",
                "asset_type":"stock",
                "quantity":12.0,
                "cost_basis":198.0,
                "holding_horizon":"短持",
                "strategy_notes":"财报前事件驱动"
            }))
            .await
            .expect("update");
        assert_eq!(update_resp["success"], true);

        let view_after_update = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view after update");
        let holdings = view_after_update["portfolio"]["holdings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0]["shares"], 12.0);
        assert_eq!(holdings[0]["avg_cost"], 198.0);
        assert_eq!(holdings[0]["holding_horizon"], HOLDING_HORIZON_SHORT_TERM);
        assert_eq!(holdings[0]["strategy_notes"], "财报前事件驱动");

        let remove_resp = tool
            .execute(serde_json::json!({
                "action":"remove",
                "ticker":"AAPL",
                "asset_type":"stock"
            }))
            .await
            .expect("remove");
        assert_eq!(remove_resp["success"], true);

        let view_after_remove = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view after remove");
        let holdings = view_after_remove["portfolio"]["holdings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(holdings.is_empty());
    }

    #[tokio::test]
    async fn portfolio_supports_option_contracts() {
        let data_dir = make_temp_dir("hone_portfolio_tool_option");
        let actor = ActorIdentity::new("imessage", "u_option", None::<String>).expect("actor");
        let tool = PortfolioTool::new(&data_dir, actor);

        let add_resp = tool
            .execute(serde_json::json!({
                "action":"add",
                "asset_type":"option",
                "underlying":"aapl",
                "expiration_date":"2026-06-19",
                "option_type":"call",
                "strike_price":200.0,
                "quantity":2.0,
                "cost_basis":5.25,
                "holding_horizon":"short_term",
                "strategy_notes":"波动率交易"
            }))
            .await
            .expect("add option");
        assert_eq!(add_resp["success"], true);
        assert_eq!(add_resp["ticker"], "AAPL 2026-06-19 C 200");

        let view_resp = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view option");
        let holdings = view_resp["portfolio"]["holdings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0]["asset_type"], "option");
        assert_eq!(holdings[0]["underlying"], "AAPL");
        assert_eq!(holdings[0]["option_type"], "call");
        assert_eq!(holdings[0]["strike_price"], 200.0);
        assert_eq!(holdings[0]["contract_multiplier"], 100.0);
        assert_eq!(holdings[0]["holding_horizon"], HOLDING_HORIZON_SHORT_TERM);
        assert_eq!(holdings[0]["strategy_notes"], "波动率交易");

        let remove_resp = tool
            .execute(serde_json::json!({
                "action":"remove",
                "asset_type":"option",
                "ticker":"AAPL 2026-06-19 C 200"
            }))
            .await
            .expect("remove option");
        assert_eq!(remove_resp["success"], true);
    }

    #[tokio::test]
    async fn portfolio_supports_batch_add() {
        let data_dir = make_temp_dir("hone_portfolio_tool_batch");
        let actor = ActorIdentity::new("imessage", "u_batch", None::<String>).expect("actor");
        let tool = PortfolioTool::new(&data_dir, actor);

        let add_resp = tool
            .execute(serde_json::json!({
                "action":"add",
                "holdings":[
                    {
                        "ticker":"AAPL",
                        "asset_type":"stock",
                        "quantity":10.0,
                        "cost_basis":180.0,
                        "holding_horizon":"long",
                        "strategy_notes":"分批建仓"
                    },
                    {
                        "asset_type":"option",
                        "underlying":"tsla",
                        "expiration_date":"2026-09-18",
                        "option_type":"put",
                        "strike_price":280.0,
                        "quantity":3.0,
                        "cost_basis":8.4,
                        "holding_horizon":"short",
                        "strategy_notes":"保护性对冲"
                    }
                ]
            }))
            .await
            .expect("batch add");

        assert_eq!(add_resp["success"], true);
        assert_eq!(add_resp["count"], 2);

        let view_resp = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view batch");
        let holdings = view_resp["portfolio"]["holdings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(holdings.len(), 2);
        assert!(holdings.iter().any(|holding| holding["symbol"] == "AAPL"));
        assert!(
            holdings
                .iter()
                .any(|holding| holding["symbol"] == "TSLA 2026-09-18 P 280")
        );
        assert!(holdings.iter().any(|holding| {
            holding["symbol"] == "AAPL"
                && holding["holding_horizon"] == HOLDING_HORIZON_LONG_TERM
                && holding["strategy_notes"] == "分批建仓"
        }));
    }

    #[tokio::test]
    async fn portfolio_supports_batch_remove() {
        let data_dir = make_temp_dir("hone_portfolio_tool_batch_remove");
        let actor =
            ActorIdentity::new("imessage", "u_batch_remove", None::<String>).expect("actor");
        let tool = PortfolioTool::new(&data_dir, actor);

        tool.execute(serde_json::json!({
            "action":"add",
            "holdings":[
                {
                    "ticker":"AAPL",
                    "quantity":10.0,
                    "cost_basis":180.0
                },
                {
                    "asset_type":"option",
                    "underlying":"AAPL",
                    "expiration_date":"2026-06-19",
                    "option_type":"call",
                    "strike_price":200.0,
                    "quantity":2.0,
                    "cost_basis":5.25
                }
            ]
        }))
        .await
        .expect("seed holdings");

        let remove_resp = tool
            .execute(serde_json::json!({
                "action":"remove",
                "holdings":[
                    {"ticker":"AAPL","asset_type":"stock"},
                    {"ticker":"AAPL 2026-06-19 C 200","asset_type":"option"}
                ]
            }))
            .await
            .expect("batch remove");
        assert_eq!(remove_resp["success"], true);
        assert_eq!(remove_resp["count"], 2);

        let view_resp = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view after batch remove");
        let holdings = view_resp["portfolio"]["holdings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(holdings.is_empty());
    }

    #[tokio::test]
    async fn portfolio_supports_negative_cost_basis_and_strategy_fields() {
        let data_dir = make_temp_dir("hone_portfolio_tool_negative_cost");
        let actor =
            ActorIdentity::new("imessage", "u_negative_cost", None::<String>).expect("actor");
        let tool = PortfolioTool::new(&data_dir, actor);

        tool.execute(serde_json::json!({
            "action":"add",
            "asset_type":"option",
            "underlying":"AAPL",
            "expiration_date":"2026-06-19",
            "option_type":"put",
            "strike_price":180.0,
            "quantity":1.0,
            "cost_basis":-2.35,
            "holding_horizon":"短线",
            "strategy_notes":"现金担保卖沽"
        }))
        .await
        .expect("add negative cost");

        let view_resp = tool
            .execute(serde_json::json!({"action":"view"}))
            .await
            .expect("view negative cost");
        let holdings = view_resp["portfolio"]["holdings"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(holdings.len(), 1);
        assert_eq!(holdings[0]["avg_cost"], -2.35);
        assert_eq!(holdings[0]["holding_horizon"], HOLDING_HORIZON_SHORT_TERM);
        assert_eq!(holdings[0]["strategy_notes"], "现金担保卖沽");
    }
}
