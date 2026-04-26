//! NotificationPrefsTool — 终端用户在自己渠道(iMessage/TG/飞书/Discord)里
//! 用自然语言管理推送偏好的入口。
//!
//! 设计要点:
//! - 构造时注入调用方的 ActorIdentity,`execute` 里只操作"自己"这份 prefs,
//!   不暴露任何"帮别人改"的参数——权限边界硬编码在构造阶段。
//! - 落盘位置与 event-engine 同目录 (`data_dir/notif_prefs/`),保证写入后下一条
//!   事件即时生效(router/scheduler 每次 dispatch 重读)。
//! - 允许/阻止的 kind tag 必须在 `ALL_KIND_TAGS` 白名单内,非法值直接报错并附
//!   合法清单——LLM 自动纠错。

use async_trait::async_trait;
use hone_core::{ActorIdentity, HoneError, HoneResult};
use hone_event_engine::Severity;
use hone_event_engine::prefs::{
    ALL_KIND_TAGS, FilePrefsStorage, NotificationPrefs, PrefsProvider, first_invalid_kind_tag,
};
use serde_json::{Value, json};
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};

pub struct NotificationPrefsTool {
    prefs_dir: PathBuf,
    actor: Option<ActorIdentity>,
}

impl NotificationPrefsTool {
    pub fn new(prefs_dir: impl Into<PathBuf>, actor: Option<ActorIdentity>) -> Self {
        Self {
            prefs_dir: prefs_dir.into(),
            actor,
        }
    }

    fn actor(&self) -> HoneResult<&ActorIdentity> {
        self.actor
            .as_ref()
            .ok_or_else(|| HoneError::Tool("缺少 actor 身份,无法修改推送偏好".into()))
    }

    fn storage(&self) -> HoneResult<FilePrefsStorage> {
        FilePrefsStorage::new(&self.prefs_dir)
            .map_err(|e| HoneError::Tool(format!("打开 prefs 目录失败: {e}")))
    }
}

fn parse_severity(raw: &str) -> HoneResult<Severity> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "low" => Ok(Severity::Low),
        "medium" | "med" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        other => Err(HoneError::Tool(format!(
            "min_severity 必须是 low/medium/high 之一,收到 {other}"
        ))),
    }
}

fn extract_string_array(value: &Value) -> HoneResult<Vec<String>> {
    let arr = value.as_array().ok_or_else(|| {
        HoneError::Tool("value 必须是字符串数组,例如 [\"news_critical\",\"press_release\"]".into())
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| HoneError::Tool("kind tag 列表里出现非字符串元素".into()))?
            .trim()
            .to_string();
        if !s.is_empty() {
            out.push(s);
        }
    }
    Ok(out)
}

fn validate_tags(tags: &[String]) -> HoneResult<()> {
    if let Some(bad) = first_invalid_kind_tag(tags.iter().map(|s| s.as_str())) {
        return Err(HoneError::Tool(format!(
            "未知的 kind tag '{bad}';合法清单:{}",
            ALL_KIND_TAGS.join(", ")
        )));
    }
    Ok(())
}

fn validate_hhmm(s: &str) -> HoneResult<()> {
    chrono::NaiveTime::parse_from_str(s, "%H:%M")
        .map(|_| ())
        .map_err(|_| HoneError::Tool(format!("时间格式必须为 HH:MM (24h),收到 {s:?}")))
}

fn prefs_to_json(prefs: &NotificationPrefs) -> Value {
    json!({
        "enabled": prefs.enabled,
        "portfolio_only": prefs.portfolio_only,
        "min_severity": match prefs.min_severity {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
        },
        "allow_kinds": prefs.allow_kinds,
        "blocked_kinds": prefs.blocked_kinds,
        "timezone": prefs.timezone,
        "digest_windows": prefs.digest_windows,
        "price_high_pct_override": prefs.price_high_pct_override,
        "immediate_kinds": prefs.immediate_kinds,
        "global_digest_enabled": prefs.global_digest_enabled,
        "investment_global_style": prefs.investment_global_style,
        "investment_theses": prefs.investment_theses,
        "global_digest_floor_macro_picks": prefs.global_digest_floor_macro_picks,
    })
}

fn parse_bool_flag(value: &Value, action: &str) -> HoneResult<bool> {
    match value {
        Value::Bool(b) => Ok(*b),
        Value::String(s) => Ok(matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes" | "on"
        )),
        _ => Err(HoneError::Tool(format!("{action} 需要 true/false"))),
    }
}

#[async_trait]
impl Tool for NotificationPrefsTool {
    fn name(&self) -> &str {
        "notification_prefs"
    }

    fn description(&self) -> &str {
        "管理当前用户的市场事件推送偏好(仅影响自己)。支持:get 查看当前设置、\
         enable/disable 总开关、set_min_severity 调整最低严重度 (low/medium/high)、\
         set_portfolio_only 只推持仓相关、allow_kinds 设置白名单、block_kinds 设置黑名单、\
         clear_allow/clear_block 清空对应列表、reset 恢复默认。\
         per-actor 推送节奏:set_timezone 设本人 IANA 时区(如 Asia/Shanghai、America/New_York)、\
         set_digest_windows 设本地 HH:MM 摘要时刻列表(传 [] 则关 digest)、\
         set_price_high_pct 调价格异动即时推阈值 (0<x≤50,如 3.5)、\
         set_immediate_kinds 指定哪些 kind 强制升 High 即时推。\
         全局要闻 digest:set_global_digest_enabled 开关、\
         set_macro_floor_picks 设置宏观料底线条数(0-5)。\
         **注意**:每只持仓的 thesis 与整体 investment_style 现在由系统每周自动从用户\
         自己写的公司画像(走 company_portrait skill)蒸馏,**不再支持手动通过本工具编辑**。\
         若用户问\"为什么我的 thesis 是 X / 想改 Y\",指引他更新对应公司画像即可,\
         系统会在下次蒸馏(默认 7 天周期)自动反映。\
         kind tag 必须选自:earnings_upcoming / earnings_released / news_critical / \
         press_release / price_alert / weekly52_high / weekly52_low / volume_spike / \
         dividend / split / buyback / sec_filing / analyst_grade / macro_event / \
         portfolio_pre_market / portfolio_post_market。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "操作类型".to_string(),
                required: true,
                r#enum: Some(vec![
                    "get".into(),
                    "enable".into(),
                    "disable".into(),
                    "set_min_severity".into(),
                    "set_portfolio_only".into(),
                    "allow_kinds".into(),
                    "block_kinds".into(),
                    "clear_allow".into(),
                    "clear_block".into(),
                    "set_timezone".into(),
                    "set_digest_windows".into(),
                    "set_price_high_pct".into(),
                    "set_immediate_kinds".into(),
                    "set_global_digest_enabled".into(),
                    "set_macro_floor_picks".into(),
                    "reset".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "参数值:\
                    set_min_severity 传 low/medium/high;\
                    set_portfolio_only 传 true/false;\
                    allow_kinds/block_kinds/set_immediate_kinds 传 JSON 数组 (例 [\"news_critical\"]);\
                    set_timezone 传 IANA 名 (例 \"Asia/Shanghai\");\
                    set_digest_windows 传 HH:MM 数组 (例 [\"19:00\",\"02:30\",\"09:00\"],空数组关 digest);\
                    set_price_high_pct 传数字 (0<x≤50,例 3.5);\
                    set_global_digest_enabled 传 true/false;\
                    set_macro_floor_picks 传整数 0-5 (默认 1)。\
                    get/clear_allow/clear_block/enable/disable/reset 不需要 value。"
                    .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> HoneResult<Value> {
        let actor = self.actor()?.clone();
        let storage = self.storage()?;
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HoneError::Tool("缺少 action 参数".into()))?
            .to_string();
        let value = args.get("value").cloned().unwrap_or(Value::Null);

        let mut prefs = storage.load(&actor);
        match action.as_str() {
            "get" => {
                return Ok(json!({ "status": "ok", "prefs": prefs_to_json(&prefs) }));
            }
            "enable" => {
                prefs.enabled = true;
            }
            "disable" => {
                prefs.enabled = false;
            }
            "set_min_severity" => {
                let raw = value.as_str().ok_or_else(|| {
                    HoneError::Tool("set_min_severity 需要 value (low/medium/high)".into())
                })?;
                prefs.min_severity = parse_severity(raw)?;
            }
            "set_portfolio_only" => {
                let flag = match &value {
                    Value::Bool(b) => *b,
                    Value::String(s) => {
                        matches!(
                            s.trim().to_ascii_lowercase().as_str(),
                            "true" | "1" | "yes" | "on"
                        )
                    }
                    _ => {
                        return Err(HoneError::Tool("set_portfolio_only 需要 true/false".into()));
                    }
                };
                prefs.portfolio_only = flag;
            }
            "allow_kinds" => {
                let tags = extract_string_array(&value)?;
                validate_tags(&tags)?;
                prefs.allow_kinds = if tags.is_empty() { None } else { Some(tags) };
            }
            "block_kinds" => {
                let tags = extract_string_array(&value)?;
                validate_tags(&tags)?;
                prefs.blocked_kinds = tags;
            }
            "clear_allow" => {
                prefs.allow_kinds = None;
            }
            "clear_block" => {
                prefs.blocked_kinds.clear();
            }
            "set_timezone" => {
                let raw = value.as_str().ok_or_else(|| {
                    HoneError::Tool("set_timezone 需要 IANA 字符串,例 \"Asia/Shanghai\"".into())
                })?;
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    prefs.timezone = None;
                } else {
                    use std::str::FromStr;
                    chrono_tz::Tz::from_str(trimmed).map_err(|_| {
                        HoneError::Tool(format!(
                            "未知 IANA 时区 {trimmed:?};示例:Asia/Shanghai、America/New_York、Europe/London"
                        ))
                    })?;
                    prefs.timezone = Some(trimmed.to_string());
                }
            }
            "set_digest_windows" => {
                let arr = value.as_array().ok_or_else(|| {
                    HoneError::Tool(
                        "set_digest_windows 需要 HH:MM 字符串数组,例 [\"19:00\",\"09:00\"];传 [] 关 digest".into(),
                    )
                })?;
                let mut wins: Vec<String> = Vec::with_capacity(arr.len());
                for item in arr {
                    let s = item
                        .as_str()
                        .ok_or_else(|| HoneError::Tool("digest_windows 元素必须是字符串".into()))?
                        .trim()
                        .to_string();
                    if s.is_empty() {
                        continue;
                    }
                    validate_hhmm(&s)?;
                    wins.push(s);
                }
                prefs.digest_windows = Some(wins);
            }
            "set_price_high_pct" => {
                let pct = match &value {
                    Value::Number(n) => n.as_f64(),
                    Value::String(s) => s.trim().parse::<f64>().ok(),
                    Value::Null => None,
                    _ => None,
                }
                .ok_or_else(|| {
                    HoneError::Tool(
                        "set_price_high_pct 需要数字 (0<x≤50,例 3.5);传 null 清空回到全局阈值"
                            .into(),
                    )
                })?;
                if !(pct > 0.0 && pct <= 50.0) || !pct.is_finite() {
                    return Err(HoneError::Tool(format!(
                        "price_high_pct 必须在 (0, 50] 范围,收到 {pct}"
                    )));
                }
                prefs.price_high_pct_override = Some(pct);
            }
            "set_immediate_kinds" => {
                let tags = extract_string_array(&value)?;
                validate_tags(&tags)?;
                prefs.immediate_kinds = if tags.is_empty() { None } else { Some(tags) };
            }
            "set_global_digest_enabled" => {
                prefs.global_digest_enabled = parse_bool_flag(&value, "set_global_digest_enabled")?;
            }
            "set_macro_floor_picks" => {
                let n = match &value {
                    Value::Number(n) => n.as_u64(),
                    Value::String(s) => s.trim().parse::<u64>().ok(),
                    _ => None,
                }
                .ok_or_else(|| HoneError::Tool("set_macro_floor_picks 需要整数 (0-5)".into()))?;
                if n > 5 {
                    return Err(HoneError::Tool(format!(
                        "macro_floor_picks 必须在 [0, 5] 范围,收到 {n}"
                    )));
                }
                prefs.global_digest_floor_macro_picks = n as u32;
            }
            "reset" => {
                prefs = NotificationPrefs::default();
            }
            other => {
                return Err(HoneError::Tool(format!("未知 action: {other}")));
            }
        }

        storage
            .save(&actor, &prefs)
            .map_err(|e| HoneError::Tool(format!("保存 prefs 失败: {e}")))?;
        Ok(json!({ "status": "ok", "prefs": prefs_to_json(&prefs) }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn mk(dir: &std::path::Path) -> NotificationPrefsTool {
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        NotificationPrefsTool::new(dir.to_path_buf(), Some(actor))
    }

    #[tokio::test]
    async fn get_returns_default_when_file_absent() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["enabled"], json!(true));
        assert_eq!(out["prefs"]["min_severity"], json!("low"));
    }

    #[tokio::test]
    async fn disable_then_get_shows_enabled_false() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        let _ = tool.execute(json!({"action":"disable"})).await.unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["enabled"], json!(false));
    }

    #[tokio::test]
    async fn allow_kinds_rejects_unknown_tag() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        let err = tool
            .execute(json!({"action":"allow_kinds","value":["not_a_tag"]}))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("未知的 kind tag")),
            other => panic!("unexpected err {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_min_severity_writes_json_roundtrip() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({"action":"set_min_severity","value":"high"}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["min_severity"], json!("high"));
    }

    #[tokio::test]
    async fn allow_and_block_kinds_persisted() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({
            "action": "allow_kinds",
            "value": ["earnings_released", "sec_filing"]
        }))
        .await
        .unwrap();
        tool.execute(json!({
            "action": "block_kinds",
            "value": ["press_release"]
        }))
        .await
        .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(
            out["prefs"]["allow_kinds"],
            json!(["earnings_released", "sec_filing"])
        );
        assert_eq!(out["prefs"]["blocked_kinds"], json!(["press_release"]));
    }

    #[tokio::test]
    async fn reset_restores_defaults() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({"action":"disable"})).await.unwrap();
        tool.execute(json!({"action":"reset"})).await.unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["enabled"], json!(true));
        assert_eq!(out["prefs"]["portfolio_only"], json!(false));
        assert_eq!(out["prefs"]["allow_kinds"], json!(null));
    }

    #[tokio::test]
    async fn set_portfolio_only_accepts_bool_and_string() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({"action":"set_portfolio_only","value":true}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["portfolio_only"], json!(true));

        tool.execute(json!({"action":"set_portfolio_only","value":"false"}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["portfolio_only"], json!(false));
    }

    #[tokio::test]
    async fn set_timezone_validates_iana_and_persists() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({"action":"set_timezone","value":"America/New_York"}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["timezone"], json!("America/New_York"));

        let err = tool
            .execute(json!({"action":"set_timezone","value":"Mars/Olympus"}))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("未知 IANA 时区"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }

        // 空字符串等价清空
        tool.execute(json!({"action":"set_timezone","value":""}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["timezone"], json!(null));
    }

    #[tokio::test]
    async fn set_digest_windows_round_trips_and_validates_format() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({
            "action": "set_digest_windows",
            "value": ["19:00", "02:30", "09:00"]
        }))
        .await
        .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(
            out["prefs"]["digest_windows"],
            json!(["19:00", "02:30", "09:00"])
        );

        // 非法格式被拒
        let err = tool
            .execute(json!({"action":"set_digest_windows","value":["25:99"]}))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("HH:MM"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }

        // 空数组允许 = 关 digest
        tool.execute(json!({"action":"set_digest_windows","value":[]}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["digest_windows"], json!([]));
    }

    #[tokio::test]
    async fn set_price_high_pct_enforces_range() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({"action":"set_price_high_pct","value":3.5}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["price_high_pct_override"], json!(3.5));

        // 0 与负数被拒
        let err = tool
            .execute(json!({"action":"set_price_high_pct","value":0}))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("(0, 50]"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }
        let err = tool
            .execute(json!({"action":"set_price_high_pct","value":99}))
            .await
            .unwrap_err();
        assert!(matches!(err, HoneError::Tool(_)));

        // 字符串数字也接受
        tool.execute(json!({"action":"set_price_high_pct","value":"4.2"}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["price_high_pct_override"], json!(4.2));
    }

    #[tokio::test]
    async fn set_immediate_kinds_validates_and_clears_on_empty() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({
            "action": "set_immediate_kinds",
            "value": ["weekly52_high", "analyst_grade"]
        }))
        .await
        .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(
            out["prefs"]["immediate_kinds"],
            json!(["weekly52_high", "analyst_grade"])
        );

        let err = tool
            .execute(json!({"action":"set_immediate_kinds","value":["bogus_kind"]}))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("未知的 kind tag"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }

        // 空数组等价 None(== 不强升)
        tool.execute(json!({"action":"set_immediate_kinds","value":[]}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["immediate_kinds"], json!(null));
    }

    #[tokio::test]
    async fn set_global_digest_enabled_round_trips() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        // 默认为 true
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["global_digest_enabled"], json!(true));

        tool.execute(json!({"action":"set_global_digest_enabled","value":false}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["global_digest_enabled"], json!(false));

        // 字符串形式也接受
        tool.execute(json!({"action":"set_global_digest_enabled","value":"true"}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["global_digest_enabled"], json!(true));
    }

    #[tokio::test]
    async fn set_macro_floor_picks_validates_range() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({"action":"set_macro_floor_picks","value":3}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["global_digest_floor_macro_picks"], json!(3));

        // 0 合法(关闭 floor)
        tool.execute(json!({"action":"set_macro_floor_picks","value":0}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["global_digest_floor_macro_picks"], json!(0));

        // 超过 5 拒绝
        let err = tool
            .execute(json!({"action":"set_macro_floor_picks","value":99}))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("[0, 5]"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }

        // 字符串数字也接受
        tool.execute(json!({"action":"set_macro_floor_picks","value":"2"}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["global_digest_floor_macro_picks"], json!(2));
    }

    #[tokio::test]
    async fn missing_actor_is_rejected() {
        let dir = tempdir().unwrap();
        let tool = NotificationPrefsTool::new(dir.path().to_path_buf(), None);
        let err = tool.execute(json!({"action":"get"})).await.unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("actor 身份")),
            other => panic!("unexpected err {other:?}"),
        }
    }
}
