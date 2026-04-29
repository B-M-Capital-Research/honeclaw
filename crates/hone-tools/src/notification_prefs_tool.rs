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
    ALL_KIND_TAGS, FilePrefsStorage, NotificationPrefs, PrefsProvider, QuietHours,
    first_invalid_kind_tag,
};
use serde_json::{Value, json};
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};

pub struct NotificationPrefsTool {
    prefs_dir: PathBuf,
    actor: Option<ActorIdentity>,
    /// `get_overview` 聚合视图所需的上下文。HoneBotCore 构造时必传,
    /// 保证用户问「我的推送怎么配的」时拿到的是含 cron + 全局 digest 的完整表格。
    cron_jobs_dir: PathBuf,
    global_digest: crate::schedule_view::GlobalDigestSlice,
    portfolio_defaults: crate::schedule_view::PortfolioDigestDefaults,
}

impl NotificationPrefsTool {
    pub fn new(
        prefs_dir: impl Into<PathBuf>,
        actor: Option<ActorIdentity>,
        cron_jobs_dir: impl Into<PathBuf>,
        global_digest: crate::schedule_view::GlobalDigestSlice,
        portfolio_defaults: crate::schedule_view::PortfolioDigestDefaults,
    ) -> Self {
        Self {
            prefs_dir: prefs_dir.into(),
            actor,
            cron_jobs_dir: cron_jobs_dir.into(),
            global_digest,
            portfolio_defaults,
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
        "quiet_hours": prefs.quiet_hours.as_ref().map(|qh| json!({
            "from": qh.from,
            "to": qh.to,
            "exempt_kinds": qh.exempt_kinds,
        })),
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
         **概览类问题**(用户问\"我的推送怎么配的\"/\"推送日程\"/\"都什么时候推什么\"/\"quiet 设了没\"等):\
         调 get_overview 拿到拍平后的全部推送时刻 + 即时推配置 + quiet_hours,返回里有 display_text \
         字段已经按调用方所在渠道(Discord 用代码块表 / Telegram 用 <pre> / Feishu+iMessage 用列表)\
         渲染好,**直接整段 relay 给用户**,不要 dump 原始 prefs JSON,也不要把 display_text 拆开重写。\
         勿扰时段(quiet_hours):set_quiet_hours 传 {from:\"23:00\", to:\"07:00\", exempt_kinds?:[...]} \
         在区间内 hold 一切 immediate 推送 + 跳过 digest 触发,到 to 时刻把 hold 住的事件 + \
         buffer 累积的 Medium/Low 合并成一条早间合集发出;过保鲜期事件直接 drop \
         (PriceAlert/VolumeSpike 2h, Weekly52 8h, Social 12h, 其它事实性事件不过期)。\
         exempt_kinds 命中的 kind 即使在 quiet 内仍立即推(例如想财报夜里也响:[\"earnings_released\"])。\
         clear_quiet_hours 关掉勿扰。\
         **注意**:每只持仓的 thesis 与整体 investment_style 现在由系统每周自动从用户\
         自己写的公司画像(走 company_portrait skill)蒸馏,**不再支持手动通过本工具编辑**。\
         若用户问\"为什么我的 thesis 是 X / 想改 Y\",指引他更新对应公司画像即可,\
         系统会在下次蒸馏(默认 7 天周期)自动反映。\
         kind tag 必须选自:earnings_upcoming / earnings_released / news_critical / \
         press_release / price_alert / weekly52_high / weekly52_low / volume_spike / \
         dividend / split / buyback / sec_filing / analyst_grade / macro_event / \
         social_post。"
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
                    "set_quiet_hours".into(),
                    "clear_quiet_hours".into(),
                    "get_overview".into(),
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
                    set_macro_floor_picks 传整数 0-5 (默认 1);\
                    set_quiet_hours 传 JSON 对象 {\"from\":\"HH:MM\", \"to\":\"HH:MM\", \"exempt_kinds\":[\"earnings_released\", ...]} (exempt_kinds 可省);\
                    clear_quiet_hours 不需要 value。\
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
            "get_overview" => {
                // 拿全部推送时刻拍平视图:持仓 digest / 全局 digest / cron / 即时推 / quiet_hours。
                // 构造时已强制注入 cron_jobs_dir + global_digest + portfolio_defaults,这里直接组装。
                // 渲染按 actor.channel 选格式:Discord/Telegram 用 monospace 代码块表,
                // Feishu/iMessage 用项目符号列表(后两者不支持 markdown/HTML)。
                let overview = crate::schedule_view::build_overview(
                    &self.prefs_dir,
                    &self.cron_jobs_dir,
                    &actor,
                    &self.global_digest,
                    &self.portfolio_defaults,
                    chrono::Utc::now(),
                )
                .map_err(|e| HoneError::Tool(format!("聚合推送日程失败: {e}")))?;
                let fmt = crate::schedule_view::channel_render_format(&actor.channel);
                let display_text = crate::schedule_view::render_overview(&overview, fmt);
                return Ok(json!({
                    "status": "ok",
                    "overview": serde_json::to_value(&overview).unwrap_or(Value::Null),
                    "display_text": display_text,
                    "render_format": format!("{fmt:?}"),
                }));
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
            "set_quiet_hours" => {
                let obj = value.as_object().ok_or_else(|| {
                    HoneError::Tool(
                        "set_quiet_hours 需要对象 {from, to, exempt_kinds?},例 {\"from\":\"23:00\",\"to\":\"07:00\"}"
                            .into(),
                    )
                })?;
                let from = obj
                    .get("from")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HoneError::Tool("set_quiet_hours 缺少 from (HH:MM)".into()))?
                    .trim()
                    .to_string();
                let to = obj
                    .get("to")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HoneError::Tool("set_quiet_hours 缺少 to (HH:MM)".into()))?
                    .trim()
                    .to_string();
                validate_hhmm(&from)?;
                validate_hhmm(&to)?;
                if from == to {
                    return Err(HoneError::Tool(
                        "set_quiet_hours 的 from 与 to 不能相等(空区间);若想全天静音请用 disable"
                            .into(),
                    ));
                }
                let exempt_kinds: Vec<String> = match obj.get("exempt_kinds") {
                    Some(v) if !v.is_null() => extract_string_array(v)?,
                    _ => Vec::new(),
                };
                validate_tags(&exempt_kinds)?;
                prefs.quiet_hours = Some(QuietHours {
                    from,
                    to,
                    exempt_kinds,
                });
            }
            "clear_quiet_hours" => {
                prefs.quiet_hours = None;
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
        let cron_dir = dir.join("__test_cron__");
        std::fs::create_dir_all(&cron_dir).unwrap();
        NotificationPrefsTool::new(
            dir.to_path_buf(),
            Some(actor),
            cron_dir,
            crate::schedule_view::GlobalDigestSlice {
                enabled: true,
                timezone: "Asia/Shanghai".into(),
                schedules: vec!["07:30".into(), "21:00".into()],
            },
            crate::schedule_view::PortfolioDigestDefaults {
                pre_market: "08:30".into(),
                post_market: "09:00".into(),
            },
        )
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
        let cron_dir = dir.path().join("__test_cron__");
        std::fs::create_dir_all(&cron_dir).unwrap();
        let tool = NotificationPrefsTool::new(
            dir.path().to_path_buf(),
            None,
            cron_dir,
            crate::schedule_view::GlobalDigestSlice {
                enabled: false,
                timezone: "Asia/Shanghai".into(),
                schedules: vec![],
            },
            crate::schedule_view::PortfolioDigestDefaults {
                pre_market: "08:30".into(),
                post_market: "09:00".into(),
            },
        );
        let err = tool.execute(json!({"action":"get"})).await.unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("actor 身份")),
            other => panic!("unexpected err {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_quiet_hours_round_trips() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "23:00", "to": "07:00", "exempt_kinds": ["earnings_released"] },
        }))
        .await
        .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["quiet_hours"]["from"], json!("23:00"));
        assert_eq!(out["prefs"]["quiet_hours"]["to"], json!("07:00"));
        assert_eq!(
            out["prefs"]["quiet_hours"]["exempt_kinds"],
            json!(["earnings_released"])
        );
    }

    #[tokio::test]
    async fn set_quiet_hours_without_exempt_defaults_to_empty() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "22:30", "to": "06:30" },
        }))
        .await
        .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["quiet_hours"]["exempt_kinds"], json!([]));
    }

    #[tokio::test]
    async fn set_quiet_hours_validates_hhmm() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        let err = tool
            .execute(json!({
                "action": "set_quiet_hours",
                "value": { "from": "25:00", "to": "07:00" },
            }))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("HH:MM"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_quiet_hours_rejects_equal_from_to() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        let err = tool
            .execute(json!({
                "action": "set_quiet_hours",
                "value": { "from": "07:00", "to": "07:00" },
            }))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("空区间"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_quiet_hours_rejects_invalid_kind() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        let err = tool
            .execute(json!({
                "action": "set_quiet_hours",
                "value": { "from": "23:00", "to": "07:00", "exempt_kinds": ["not_a_real_kind"] },
            }))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("未知") || msg.contains("kind")),
            other => panic!("unexpected err {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_overview_returns_display_text_and_overview() {
        let dir = tempdir().unwrap();
        // mk() 用的是 telegram actor → display_text 应是 <pre> 包的等宽块
        let tool = mk(dir.path());
        let out = tool
            .execute(json!({"action":"get_overview"}))
            .await
            .unwrap();
        assert_eq!(out["status"], json!("ok"));
        let txt = out["display_text"].as_str().expect("display_text");
        assert!(txt.contains("你的推送日程"));
        assert!(txt.contains("时刻"));
        // telegram → 走 <pre>
        assert!(txt.contains("<pre>"));
        // 不应再出现 markdown table 字符
        assert!(!txt.contains("| --- |"));
        assert_eq!(out["render_format"], json!("TelegramHtml"));
        let entries = out["overview"]["schedule"].as_array().unwrap();
        assert_eq!(entries.len(), 4);
    }

    #[tokio::test]
    async fn get_overview_for_discord_actor_uses_codeblock() {
        let dir = tempdir().unwrap();
        let actor = ActorIdentity::new("discord", "u1", None::<&str>).unwrap();
        let cron_dir = dir.path().join("cron");
        std::fs::create_dir_all(&cron_dir).unwrap();
        let tool = NotificationPrefsTool::new(
            dir.path().to_path_buf(),
            Some(actor),
            cron_dir,
            crate::schedule_view::GlobalDigestSlice {
                enabled: true,
                timezone: "Asia/Shanghai".into(),
                schedules: vec!["07:30".into(), "21:00".into()],
            },
            crate::schedule_view::PortfolioDigestDefaults {
                pre_market: "08:30".into(),
                post_market: "09:00".into(),
            },
        );
        let out = tool
            .execute(json!({"action":"get_overview"}))
            .await
            .unwrap();
        let txt = out["display_text"].as_str().unwrap();
        assert!(txt.contains("```"), "discord 应用代码块: {txt}");
        assert!(!txt.contains("<pre>"));
        assert_eq!(out["render_format"], json!("DiscordMarkdown"));
    }

    #[tokio::test]
    async fn get_overview_for_imessage_uses_plain_list() {
        let dir = tempdir().unwrap();
        let actor = ActorIdentity::new("imessage", "u1", None::<&str>).unwrap();
        let cron_dir = dir.path().join("cron");
        std::fs::create_dir_all(&cron_dir).unwrap();
        let tool = NotificationPrefsTool::new(
            dir.path().to_path_buf(),
            Some(actor),
            cron_dir,
            crate::schedule_view::GlobalDigestSlice {
                enabled: true,
                timezone: "Asia/Shanghai".into(),
                schedules: vec!["07:30".into(), "21:00".into()],
            },
            crate::schedule_view::PortfolioDigestDefaults {
                pre_market: "08:30".into(),
                post_market: "09:00".into(),
            },
        );
        let out = tool
            .execute(json!({"action":"get_overview"}))
            .await
            .unwrap();
        let txt = out["display_text"].as_str().unwrap();
        assert!(!txt.contains("```"));
        assert!(!txt.contains("<pre>"));
        assert!(txt.contains("• "), "imessage 应该是项目符号列表: {txt}");
        assert_eq!(out["render_format"], json!("Plain"));
    }

    #[tokio::test]
    async fn clear_quiet_hours_removes_field() {
        let dir = tempdir().unwrap();
        let tool = mk(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "23:00", "to": "07:00" },
        }))
        .await
        .unwrap();
        tool.execute(json!({"action":"clear_quiet_hours"}))
            .await
            .unwrap();
        let out = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(out["prefs"]["quiet_hours"], json!(null));
    }
}
