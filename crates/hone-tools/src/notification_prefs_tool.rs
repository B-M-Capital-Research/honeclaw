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
use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::{ActorIdentity, HoneError, HoneResult};
use hone_event_engine::Severity;
use hone_event_engine::prefs::{
    ALL_KIND_TAGS, FilePrefsStorage, NotificationDeliveryPatch, NotificationPrefs,
    PreferenceUpdate, PrefsProvider, QuietHours, first_invalid_kind_tag,
};
use hone_event_engine::unified_digest::DigestSlot;
use serde_json::{Value, json};
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};

pub struct NotificationPrefsTool {
    prefs_dir: PathBuf,
    actor: Option<ActorIdentity>,
    /// `get_overview` 聚合视图所需的上下文。HoneBotCore 构造时必传,
    /// 保证用户问「我的推送怎么配的」时拿到的是含 cron + unified digest 的完整表格。
    cron_jobs_dir: PathBuf,
    overview_defaults: crate::schedule_view::NotificationOverviewDefaults,
    postgres: Option<CloudPgRuntime>,
}

impl NotificationPrefsTool {
    pub fn new(
        prefs_dir: impl Into<PathBuf>,
        actor: Option<ActorIdentity>,
        cron_jobs_dir: impl Into<PathBuf>,
        overview_defaults: crate::schedule_view::NotificationOverviewDefaults,
    ) -> Self {
        Self {
            prefs_dir: prefs_dir.into(),
            actor,
            cron_jobs_dir: cron_jobs_dir.into(),
            overview_defaults,
            postgres: None,
        }
    }

    pub fn new_cloud(
        prefs_dir: impl Into<PathBuf>,
        actor: Option<ActorIdentity>,
        cron_jobs_dir: impl Into<PathBuf>,
        overview_defaults: crate::schedule_view::NotificationOverviewDefaults,
        postgres: CloudPgRuntime,
    ) -> Self {
        Self {
            prefs_dir: prefs_dir.into(),
            actor,
            cron_jobs_dir: cron_jobs_dir.into(),
            overview_defaults,
            postgres: Some(postgres),
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

    async fn cron_storage(&self) -> HoneResult<hone_memory::CronJobStorage> {
        if let Some(postgres) = self.postgres.clone() {
            return hone_memory::CronJobStorage::new_cloud(postgres)
                .await
                .map_err(|e| HoneError::Tool(format!("打开云端 cron 存储失败: {e}")));
        }
        Ok(hone_memory::CronJobStorage::new(&self.cron_jobs_dir).await)
    }
}

pub async fn load_notification_quiet_hours(
    prefs_dir: impl Into<PathBuf>,
    actor: &ActorIdentity,
) -> Option<(QuietHours, Option<String>)> {
    let storage = FilePrefsStorage::new(prefs_dir).ok()?;
    let prefs = storage.load(actor).await;
    Some((prefs.quiet_hours?, prefs.timezone))
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
    let string_values = value.as_array().ok_or_else(|| {
        HoneError::Tool("value 必须是字符串数组,例如 [\"news_critical\",\"sec_filing\"]".into())
    })?;
    let mut strings = Vec::with_capacity(string_values.len());
    for string_value in string_values {
        let tag = string_value
            .as_str()
            .ok_or_else(|| HoneError::Tool("kind tag 列表里出现非字符串元素".into()))?
            .trim()
            .to_string();
        if !tag.is_empty() {
            strings.push(tag);
        }
    }
    Ok(strings)
}

fn validate_tags(tags: &[String]) -> HoneResult<()> {
    if let Some(invalid_tag) = first_invalid_kind_tag(tags.iter().map(|tag| tag.as_str())) {
        return Err(HoneError::Tool(format!(
            "未知的 kind tag '{invalid_tag}';合法清单:{}",
            ALL_KIND_TAGS.join(", ")
        )));
    }
    Ok(())
}

fn parse_bool_flag(value: &Value, action: &str) -> HoneResult<bool> {
    match value {
        Value::Bool(flag) => Ok(*flag),
        Value::String(raw) => Ok(matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes" | "on"
        )),
        _ => Err(HoneError::Tool(format!("{action} 需要 true/false"))),
    }
}

fn optional_kind_tags(value: &Value) -> HoneResult<Option<Vec<String>>> {
    let tags = extract_string_array(value)?;
    validate_tags(&tags)?;
    Ok((!tags.is_empty()).then_some(tags))
}

fn parse_digest_slots(value: &Value) -> HoneResult<PreferenceUpdate<Vec<DigestSlot>>> {
    if value.is_null() {
        return Ok(PreferenceUpdate::Inherit);
    }
    let slot_values = value.as_array().ok_or_else(|| {
        HoneError::Tool(
            "set_digest_slots 需要槽位数组;可传 [\"19:00\",\"09:00\"] 或 \
             [{\"id\":\"premarket\",\"time\":\"08:30\",\"label\":\"盘前要闻\",\"floor_macro\":1}];\
             [] 关闭 digest,null 恢复全局时段"
                .into(),
        )
    })?;
    let mut slots: Vec<DigestSlot> = Vec::with_capacity(slot_values.len());
    for (idx, slot_value) in slot_values.iter().enumerate() {
        if let Some(slot_time) = slot_value.as_str() {
            let slot_time = slot_time.trim().to_string();
            if slot_time.is_empty() {
                continue;
            }
            slots.push(DigestSlot {
                id: format!("slot_{idx}"),
                time: slot_time,
                label: None,
                floor_macro: None,
            });
            continue;
        }

        let slot_object = slot_value.as_object().ok_or_else(|| {
            HoneError::Tool(
                "digest_slots 元素必须是 HH:MM 字符串或 {id,time,label?,floor_macro?} 对象".into(),
            )
        })?;
        let slot_time = slot_object
            .get("time")
            .and_then(Value::as_str)
            .ok_or_else(|| HoneError::Tool("digest slot 缺少 time (HH:MM)".into()))?
            .trim()
            .to_string();
        let id = match slot_object.get("id") {
            None | Some(Value::Null) => format!("slot_{idx}"),
            Some(Value::String(id)) => id.trim().to_string(),
            Some(_) => {
                return Err(HoneError::Tool("digest slot 的 id 必须是字符串".into()));
            }
        };
        let label = match slot_object.get("label") {
            None | Some(Value::Null) => None,
            Some(Value::String(label)) => Some(label.trim().to_string()),
            Some(_) => {
                return Err(HoneError::Tool(
                    "digest slot 的 label 必须是字符串或 null".into(),
                ));
            }
        };
        let floor_macro = match slot_object.get("floor_macro") {
            None | Some(Value::Null) => None,
            Some(Value::Number(number)) => {
                let value = number.as_u64().ok_or_else(|| {
                    HoneError::Tool("digest slot 的 floor_macro 必须是非负整数".into())
                })?;
                Some(u32::try_from(value).map_err(|_| {
                    HoneError::Tool("digest slot 的 floor_macro 超出 u32 范围".into())
                })?)
            }
            Some(_) => {
                return Err(HoneError::Tool(
                    "digest slot 的 floor_macro 必须是非负整数或 null".into(),
                ));
            }
        };
        slots.push(DigestSlot {
            id,
            time: slot_time,
            label,
            floor_macro,
        });
    }
    Ok(PreferenceUpdate::Set(slots))
}

fn parse_percentage_update(value: &Value, action: &str) -> HoneResult<PreferenceUpdate<f64>> {
    if value.is_null() {
        return Ok(PreferenceUpdate::Inherit);
    }
    let percentage = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(raw_text) => raw_text.trim().parse::<f64>().ok(),
        _ => None,
    }
    .ok_or_else(|| {
        HoneError::Tool(format!(
            "{action} 需要数字;传 null 可清空 actor override 并恢复继承"
        ))
    })?;
    Ok(PreferenceUpdate::Set(percentage))
}

fn parse_timezone_update(value: &Value) -> HoneResult<PreferenceUpdate<String>> {
    if value.is_null() {
        return Ok(PreferenceUpdate::Inherit);
    }
    let raw = value.as_str().ok_or_else(|| {
        HoneError::Tool("timezone 需要 IANA 字符串,例 \"America/New_York\";null 表示继承".into())
    })?;
    let trimmed = raw.trim();
    Ok(if trimmed.is_empty() {
        PreferenceUpdate::Inherit
    } else {
        PreferenceUpdate::Set(trimmed.to_string())
    })
}

fn parse_quiet_hours(value: &Value) -> HoneResult<QuietHours> {
    let quiet_hours_object = value.as_object().ok_or_else(|| {
        HoneError::Tool(
            "set_quiet_hours 需要对象 {from, to, exempt_kinds?},例 {\"from\":\"23:00\",\"to\":\"07:00\"}"
                .into(),
        )
    })?;
    let from = quiet_hours_object
        .get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HoneError::Tool("set_quiet_hours 缺少 from (HH:MM)".into()))?
        .trim()
        .to_string();
    let to = quiet_hours_object
        .get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HoneError::Tool("set_quiet_hours 缺少 to (HH:MM)".into()))?
        .trim()
        .to_string();
    let exempt_kinds = match quiet_hours_object.get("exempt_kinds") {
        Some(v) if !v.is_null() => extract_string_array(v)?,
        _ => Vec::new(),
    };
    Ok(QuietHours {
        from,
        to,
        exempt_kinds,
    })
}

fn parse_delivery_controls(value: &Value) -> HoneResult<NotificationDeliveryPatch> {
    let controls = value.as_object().ok_or_else(|| {
        HoneError::Tool(
            "update_delivery_controls 需要对象，字段可选:timezone,digest_slots,\
             price_high_pct,price_high_pct_up,price_high_pct_down,\
             price_realert_step_pct,large_position_weight_pct,quiet_hours"
                .into(),
        )
    })?;
    const ALLOWED_FIELDS: &[&str] = &[
        "timezone",
        "digest_slots",
        "price_high_pct",
        "price_high_pct_up",
        "price_high_pct_down",
        "price_realert_step_pct",
        "large_position_weight_pct",
        "quiet_hours",
    ];
    if let Some(unknown_field) = controls
        .keys()
        .find(|field| !ALLOWED_FIELDS.contains(&field.as_str()))
    {
        return Err(HoneError::Tool(format!(
            "update_delivery_controls 不支持字段 {unknown_field:?};合法字段:{}",
            ALLOWED_FIELDS.join(",")
        )));
    }

    let timezone = controls
        .get("timezone")
        .map(parse_timezone_update)
        .transpose()?
        .unwrap_or_default();
    let digest_slots = controls
        .get("digest_slots")
        .map(parse_digest_slots)
        .transpose()?
        .unwrap_or_default();
    let price_high_pct_override = controls
        .get("price_high_pct")
        .map(|value| parse_percentage_update(value, "price_high_pct"))
        .transpose()?
        .unwrap_or_default();
    let price_high_pct_up_override = controls
        .get("price_high_pct_up")
        .map(|value| parse_percentage_update(value, "price_high_pct_up"))
        .transpose()?
        .unwrap_or_default();
    let price_high_pct_down_override = controls
        .get("price_high_pct_down")
        .map(|value| parse_percentage_update(value, "price_high_pct_down"))
        .transpose()?
        .unwrap_or_default();
    let price_realert_step_pct_override = controls
        .get("price_realert_step_pct")
        .map(|value| parse_percentage_update(value, "price_realert_step_pct"))
        .transpose()?
        .unwrap_or_default();
    let large_position_weight_pct = controls
        .get("large_position_weight_pct")
        .map(|value| parse_percentage_update(value, "large_position_weight_pct"))
        .transpose()?
        .unwrap_or_default();
    let quiet_hours = match controls.get("quiet_hours") {
        None => PreferenceUpdate::Keep,
        Some(Value::Null) => PreferenceUpdate::Inherit,
        Some(value) => PreferenceUpdate::Set(parse_quiet_hours(value)?),
    };
    Ok(NotificationDeliveryPatch {
        timezone,
        digest_slots,
        price_high_pct_override,
        price_high_pct_up_override,
        price_high_pct_down_override,
        price_realert_step_pct_override,
        large_position_weight_pct,
        quiet_hours,
    })
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
        "digest_slots": prefs.digest_slots,
        "price_high_pct_override": prefs.price_high_pct_override,
        "price_high_pct_up_override": prefs.price_high_pct_up_override,
        "price_high_pct_down_override": prefs.price_high_pct_down_override,
        "price_realert_step_pct_override": prefs.price_realert_step_pct_override,
        "large_position_weight_pct": prefs.large_position_weight_pct,
        "immediate_kinds": prefs.immediate_kinds,
        "mainline_style": prefs.mainline_style,
        "mainline_by_ticker": prefs.mainline_by_ticker,
        "quiet_hours": prefs.quiet_hours.as_ref().map(|quiet_hours| json!({
            "from": quiet_hours.from,
            "to": quiet_hours.to,
            "exempt_kinds": quiet_hours.exempt_kinds,
        })),
    })
}

fn apply_delivery_patch(
    prefs: &mut NotificationPrefs,
    patch: NotificationDeliveryPatch,
) -> HoneResult<()> {
    prefs
        .apply_delivery_patch(patch)
        .map_err(|error| HoneError::Tool(error.to_string()))
}

#[async_trait]
impl Tool for NotificationPrefsTool {
    fn name(&self) -> &str {
        "notification_prefs"
    }

    fn description(&self) -> &str {
        "管理当前用户的市场事件推送偏好(仅影响自己)。支持:get 查看当前设置、\
         enable/disable 事件推送总开关、disable_all 取消当前用户的所有自动提醒\
         （关闭事件即时/摘要推送，并删除全部定时任务和心跳任务）、\
         set_min_severity 调整最低严重度 (low/medium/high)、\
         set_portfolio_only 只推持仓相关、allow_kinds 设置白名单、block_kinds 设置黑名单、\
         clear_allow/clear_block 清空对应列表、reset 恢复默认。\
         per-actor 推送节奏:set_timezone 设本人 IANA 时区(如 America/New_York、Europe/London)、\
         set_digest_slots 设 digest 触发槽位(支持旧 HH:MM 数组或 \
         {id,time,label?,floor_macro?} 对象数组;[] 关 digest;null 恢复全局)、\
         set_price_high_pct / set_price_high_pct_up / set_price_high_pct_down \
         调通用、上涨、下跌价格异动即时推阈值 (0<x≤50),\
         set_price_realert_step_pct 调首次命中后的重复提醒最小前进步长 (0<x≤50),\
         set_large_position_weight_pct 调大仓位权重边界 (0<x≤100)。\
         对应 inherit_* action 可单项清空 actor override、恢复系统默认。\
         多项一起修改时优先用 update_delivery_controls，一次传对象并原子校验/保存。\
         set_immediate_kinds 指定哪些 kind 强制升 High 即时推。\
         **概览类问题**(用户问\"我的推送怎么配的\"/\"推送日程\"/\"都什么时候推什么\"/\"quiet 设了没\"等):\
         调 get_overview 拿到拍平后的全部推送时刻 + 最终生效价格阶梯 + quiet_hours；\
         价格阶梯会解析系统候选档、继承来源、普通/大仓位首次阈值、重复步长、示例、每日 High 上限，\
         并明确普通同标的冷却不限制盘中价格阶梯。返回里有 display_text \
         字段已经按调用方所在渠道(Discord 用代码块表 / Telegram 用 <pre> / Feishu+iMessage 用列表)\
         渲染好,**直接整段 relay 给用户**,不要 dump 原始 prefs JSON,也不要把 display_text 拆开重写。\
         勿扰时段(quiet_hours):set_quiet_hours 传 {from:\"23:00\", to:\"07:00\", exempt_kinds?:[...]} \
         在区间内 hold 一切 immediate 推送 + 跳过 digest 触发,到 to 时刻把 hold 住的事件 + \
         buffer 累积的 Medium/Low 合并成一条早间合集发出;过保鲜期事件直接 drop \
         (PriceAlert 2h, Weekly52 8h, Social 12h, 其它事实性事件不过期)。\
         exempt_kinds 命中的 kind 即使在 quiet 内仍立即推(例如想财报夜里也响:[\"earnings_released\"])。\
         clear_quiet_hours 关掉勿扰。\
         **注意**:每只持仓的 thesis 与整体 investment_style 现在由系统从用户\
         自己写的公司画像(走 company_portrait skill)按需蒸馏,**不再支持手动通过本工具编辑**。\
         若用户问\"为什么我的 thesis 是 X / 想改 Y\",指引他更新对应公司画像即可,\
         新建画像/新增持仓后通常在下一次小时级检查里尝试更新;覆盖完整后约每 7 天刷新一次。\
         用户明确说“取消所有自动提醒 / 关闭所有自动推送 / 以后不要自动通知”时，\
         必须直接调用 disable_all；不要只调用 disable，也不要再让用户逐项确认。\
         kind tag 必须选自:earnings_upcoming / earnings_released / earnings_call_transcript / \
         news_critical / price_alert / weekly52_high / weekly52_low / dividend / split / \
         sec_filing / analyst_grade / macro_event / social_post。"
    }

    fn input_schema(&self) -> Value {
        let parameters = self.parameters();
        let action = &parameters[0];
        let value = &parameters[1];
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": action.description,
                    "enum": action.r#enum,
                },
                "value": {
                    "description": value.description,
                    "anyOf": [
                        {"type": "string"},
                        {"type": "number"},
                        {"type": "boolean"},
                        {"type": "array"},
                        {"type": "object"},
                        {"type": "null"}
                    ]
                }
            },
            "required": ["action"]
        })
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
                    "disable_all".into(),
                    "set_min_severity".into(),
                    "set_portfolio_only".into(),
                    "allow_kinds".into(),
                    "block_kinds".into(),
                    "clear_allow".into(),
                    "clear_block".into(),
                    "set_timezone".into(),
                    "inherit_timezone".into(),
                    "set_digest_slots".into(),
                    "inherit_digest_slots".into(),
                    "set_price_high_pct".into(),
                    "inherit_price_high_pct".into(),
                    "set_price_high_pct_up".into(),
                    "inherit_price_high_pct_up".into(),
                    "set_price_high_pct_down".into(),
                    "inherit_price_high_pct_down".into(),
                    "set_price_realert_step_pct".into(),
                    "inherit_price_realert_step_pct".into(),
                    "set_large_position_weight_pct".into(),
                    "inherit_large_position_weight_pct".into(),
                    "update_delivery_controls".into(),
                    "set_immediate_kinds".into(),
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
                    set_timezone 传 IANA 名 (例 \"America/New_York\");\
                    set_digest_slots 可传 HH:MM 数组，或对象数组 \
                    [{\"id\":\"premarket\",\"time\":\"08:30\",\"label\":\"盘前要闻\",\"floor_macro\":1}];\
                    空数组关 digest，null 或 inherit_digest_slots 恢复全局时段;\
                    set_price_high_pct/set_price_high_pct_up/set_price_high_pct_down/\
                    set_price_realert_step_pct 传数字 (0<x≤50);\
                    set_large_position_weight_pct 传数字 (0<x≤100);\
                    inherit_timezone/inherit_price_high_pct/inherit_price_high_pct_up/\
                    inherit_price_high_pct_down/inherit_price_realert_step_pct/\
                    inherit_large_position_weight_pct 不需要 value;\
                    update_delivery_controls 传对象，可同时包含 timezone,digest_slots,\
                    price_high_pct,price_high_pct_up,price_high_pct_down,price_realert_step_pct,\
                    large_position_weight_pct,quiet_hours；字段为 null 表示单项继承，\
                    整个对象一次校验后原子保存;\
                    set_quiet_hours 传 JSON 对象 {\"from\":\"HH:MM\", \"to\":\"HH:MM\", \"exempt_kinds\":[\"earnings_released\", ...]} (exempt_kinds 可省);\
                    clear_quiet_hours 不需要 value。\
                    get/clear_allow/clear_block/enable/disable/disable_all/reset 不需要 value。"
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

        let mut prefs = storage.load(&actor).await;
        match action.as_str() {
            "get" => {
                return Ok(json!({ "status": "ok", "prefs": prefs_to_json(&prefs) }));
            }
            "get_overview" => {
                // 拿全部推送时刻拍平视图:unified digest slots / cron / 即时推 / quiet_hours。
                // 构造时已强制注入 cron_jobs_dir + digest_defaults,这里直接组装。
                // 渲染按 actor.channel 选格式:Discord/Telegram 用 monospace 代码块表,
                // Feishu/iMessage 用项目符号列表(后两者不支持 markdown/HTML)。
                let overview = if let Some(postgres) = self.postgres.clone() {
                    let cron_storage = hone_memory::CronJobStorage::new_cloud(postgres)
                        .await
                        .map_err(|e| HoneError::Tool(format!("打开云端 cron 存储失败: {e}")))?;
                    crate::schedule_view::build_overview_with_cron_jobs(
                        &self.prefs_dir,
                        cron_storage.list_jobs(&actor).await,
                        &actor,
                        &self.overview_defaults,
                    )
                    .await
                } else {
                    crate::schedule_view::build_overview(
                        &self.prefs_dir,
                        &self.cron_jobs_dir,
                        &actor,
                        &self.overview_defaults,
                        chrono::Utc::now(),
                    )
                    .await
                }
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
            "disable_all" => {
                let removed_jobs = self.cron_storage().await?.remove_all_jobs(&actor).await?;
                prefs.enabled = false;
                prefs.digest_slots = Some(Vec::new());
                storage
                    .save(&actor, &prefs)
                    .await
                    .map_err(|e| HoneError::Tool(format!("保存 prefs 失败: {e}")))?;
                return Ok(json!({
                    "status": "ok",
                    "action": "disable_all",
                    "prefs": prefs_to_json(&prefs),
                    "removed_count": removed_jobs.len(),
                    "removed_jobs": removed_jobs,
                    "remaining_count": 0,
                }));
            }
            "set_min_severity" => {
                let raw = value.as_str().ok_or_else(|| {
                    HoneError::Tool("set_min_severity 需要 value (low/medium/high)".into())
                })?;
                prefs.min_severity = parse_severity(raw)?;
            }
            "set_portfolio_only" => {
                prefs.portfolio_only = parse_bool_flag(&value, "set_portfolio_only")?;
            }
            "allow_kinds" => {
                prefs.allow_kinds = optional_kind_tags(&value)?;
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
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        timezone: parse_timezone_update(&value)?,
                        ..Default::default()
                    },
                )?;
            }
            "inherit_timezone" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        timezone: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
            }
            "set_digest_slots" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        digest_slots: parse_digest_slots(&value)?,
                        ..Default::default()
                    },
                )?;
            }
            "inherit_digest_slots" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        digest_slots: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
            }
            "set_price_high_pct" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_high_pct_override: parse_percentage_update(&value, &action)?,
                        ..Default::default()
                    },
                )?;
            }
            "inherit_price_high_pct" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_high_pct_override: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
            }
            "set_price_high_pct_up" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_high_pct_up_override: parse_percentage_update(&value, &action)?,
                        ..Default::default()
                    },
                )?;
            }
            "inherit_price_high_pct_up" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_high_pct_up_override: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
            }
            "set_price_high_pct_down" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_high_pct_down_override: parse_percentage_update(&value, &action)?,
                        ..Default::default()
                    },
                )?;
            }
            "inherit_price_high_pct_down" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_high_pct_down_override: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
            }
            "set_price_realert_step_pct" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_realert_step_pct_override: parse_percentage_update(&value, &action)?,
                        ..Default::default()
                    },
                )?;
            }
            "inherit_price_realert_step_pct" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        price_realert_step_pct_override: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
            }
            "set_large_position_weight_pct" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        large_position_weight_pct: parse_percentage_update(&value, &action)?,
                        ..Default::default()
                    },
                )?;
            }
            "inherit_large_position_weight_pct" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        large_position_weight_pct: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
            }
            "update_delivery_controls" => {
                apply_delivery_patch(&mut prefs, parse_delivery_controls(&value)?)?;
            }
            "set_immediate_kinds" => {
                prefs.immediate_kinds = optional_kind_tags(&value)?;
            }
            "set_quiet_hours" => {
                let candidate = parse_quiet_hours(&value)?;
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        quiet_hours: PreferenceUpdate::Set(candidate),
                        ..Default::default()
                    },
                )?;
            }
            "clear_quiet_hours" => {
                apply_delivery_patch(
                    &mut prefs,
                    NotificationDeliveryPatch {
                        quiet_hours: PreferenceUpdate::Inherit,
                        ..Default::default()
                    },
                )?;
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
            .await
            .map_err(|e| HoneError::Tool(format!("保存 prefs 失败: {e}")))?;
        Ok(json!({ "status": "ok", "prefs": prefs_to_json(&prefs) }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn digest_defaults_fixture() -> crate::schedule_view::NotificationOverviewDefaults {
        crate::schedule_view::NotificationOverviewDefaults {
            slots: vec![
                crate::schedule_view::DigestDefaultSlot {
                    time: "08:30".into(),
                    label: Some("盘前摘要".into()),
                },
                crate::schedule_view::DigestDefaultSlot {
                    time: "09:00".into(),
                    label: Some("晨间摘要".into()),
                },
            ],
            price_alert: hone_event_engine::prefs::PriceAlertPolicyDefaults::default(),
            event_engine_enabled: true,
            globally_disabled_kinds: Vec::new(),
            high_severity_daily_cap: 8,
            same_symbol_cooldown_minutes: 60,
        }
    }

    fn make_tool(prefs_dir: &std::path::Path) -> NotificationPrefsTool {
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        let cron_dir = prefs_dir.join("__test_cron__");
        std::fs::create_dir_all(&cron_dir).unwrap();
        NotificationPrefsTool::new(
            prefs_dir.to_path_buf(),
            Some(actor),
            cron_dir,
            digest_defaults_fixture(),
        )
    }

    #[tokio::test]
    async fn get_returns_default_when_file_absent() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["enabled"], json!(true));
        assert_eq!(response["prefs"]["min_severity"], json!("low"));
    }

    #[test]
    fn tool_schema_declares_the_real_value_union() {
        let dir = tempdir().unwrap();
        let schema = make_tool(dir.path()).to_openai_schema();
        let value_types = schema["function"]["parameters"]["properties"]["value"]["anyOf"]
            .as_array()
            .expect("value anyOf");
        assert_eq!(value_types.len(), 6);
        assert!(value_types.iter().any(|entry| entry["type"] == "array"));
        assert!(value_types.iter().any(|entry| entry["type"] == "object"));
        assert!(value_types.iter().any(|entry| entry["type"] == "number"));
        assert_eq!(
            schema["function"]["parameters"]["required"],
            json!(["action"])
        );
    }

    #[tokio::test]
    async fn disable_then_get_shows_enabled_false() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        let _ = tool.execute(json!({"action":"disable"})).await.unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["enabled"], json!(false));
    }

    #[tokio::test]
    async fn disable_all_stops_event_pushes_and_removes_only_current_actor_jobs() {
        let dir = tempdir().unwrap();
        let prefs_dir = dir.path().join("prefs");
        let cron_dir = dir.path().join("cron");
        let actor = ActorIdentity::new("feishu", "u1", None::<&str>).unwrap();
        let other_actor = ActorIdentity::new("feishu", "u2", None::<&str>).unwrap();
        let tool = NotificationPrefsTool::new(
            &prefs_dir,
            Some(actor.clone()),
            &cron_dir,
            digest_defaults_fixture(),
        );
        let cron_storage = hone_memory::CronJobStorage::new(&cron_dir).await;
        for (name, repeat) in [("daily report", "daily"), ("price heartbeat", "heartbeat")] {
            let response = cron_storage
                .add_job(
                    &actor,
                    name,
                    Some(9),
                    Some(0),
                    repeat,
                    "task",
                    &actor.user_id,
                    None,
                    None,
                    None,
                    true,
                    None,
                    false,
                )
                .await;
            assert_eq!(response["success"], true);
        }
        let other_response = cron_storage
            .add_job(
                &other_actor,
                "other actor report",
                Some(10),
                Some(0),
                "daily",
                "task",
                &other_actor.user_id,
                None,
                None,
                None,
                true,
                None,
                false,
            )
            .await;
        assert_eq!(other_response["success"], true);

        let response = tool
            .execute(json!({"action": "disable_all"}))
            .await
            .expect("disable all automatic reminders");
        assert_eq!(response["status"], "ok");
        assert_eq!(response["action"], "disable_all");
        assert_eq!(response["removed_count"], 2);
        assert_eq!(response["prefs"]["enabled"], false);
        assert_eq!(response["prefs"]["digest_slots"], json!([]));
        assert!(cron_storage.list_jobs(&actor).await.is_empty());
        assert_eq!(cron_storage.list_jobs(&other_actor).await.len(), 1);

        let repeated = tool
            .execute(json!({"action": "disable_all"}))
            .await
            .expect("disable all remains idempotent");
        assert_eq!(repeated["status"], "ok");
        assert_eq!(repeated["removed_count"], 0);
    }

    #[tokio::test]
    async fn allow_kinds_rejects_unknown_tag() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
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
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"set_min_severity","value":"high"}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["min_severity"], json!("high"));
    }

    #[tokio::test]
    async fn allow_and_block_kinds_persisted() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "allow_kinds",
            "value": ["earnings_released", "sec_filing"]
        }))
        .await
        .unwrap();
        tool.execute(json!({
            "action": "block_kinds",
            "value": ["social_post"]
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(
            response["prefs"]["allow_kinds"],
            json!(["earnings_released", "sec_filing"])
        );
        assert_eq!(response["prefs"]["blocked_kinds"], json!(["social_post"]));
    }

    #[tokio::test]
    async fn reset_restores_defaults() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"disable"})).await.unwrap();
        tool.execute(json!({"action":"reset"})).await.unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["enabled"], json!(true));
        assert_eq!(response["prefs"]["portfolio_only"], json!(false));
        assert_eq!(response["prefs"]["allow_kinds"], json!(null));
    }

    #[tokio::test]
    async fn set_portfolio_only_accepts_bool_and_string() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"set_portfolio_only","value":true}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["portfolio_only"], json!(true));

        tool.execute(json!({"action":"set_portfolio_only","value":"false"}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["portfolio_only"], json!(false));
    }

    #[tokio::test]
    async fn set_timezone_validates_iana_and_persists() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"set_timezone","value":"America/New_York"}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["timezone"], json!("America/New_York"));

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
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["timezone"], json!(null));
    }

    #[tokio::test]
    async fn set_digest_slots_round_trips_and_validates_format() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_digest_slots",
            "value": ["19:00", "02:30", "09:00"]
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        let times: Vec<String> = response["prefs"]["digest_slots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|slot_value| slot_value["time"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(times, vec!["19:00", "02:30", "09:00"]);

        // 非法格式被拒
        let err = tool
            .execute(json!({"action":"set_digest_slots","value":["25:99"]}))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => assert!(msg.contains("HH:MM"), "msg={msg}"),
            other => panic!("unexpected err {other:?}"),
        }

        // 空数组允许 = 关 digest
        tool.execute(json!({"action":"set_digest_slots","value":[]}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["digest_slots"], json!([]));
    }

    #[tokio::test]
    async fn set_digest_slots_preserves_structured_names_and_floor() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_digest_slots",
            "value": [
                {
                    "id": "postmarket",
                    "time": "07:30",
                    "label": "盘后要闻",
                    "floor_macro": 2
                },
                {
                    "id": "premarket",
                    "time": "21:00",
                    "label": "盘前要闻"
                }
            ]
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(
            response["prefs"]["digest_slots"],
            json!([
                {
                    "id": "postmarket",
                    "time": "07:30",
                    "label": "盘后要闻",
                    "floor_macro": 2
                },
                {
                    "id": "premarket",
                    "time": "21:00",
                    "label": "盘前要闻"
                }
            ])
        );
    }

    #[tokio::test]
    async fn digest_slots_can_inherit_without_conflating_explicit_disable() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"set_digest_slots","value":[]}))
            .await
            .unwrap();
        assert_eq!(
            tool.execute(json!({"action":"get"})).await.unwrap()["prefs"]["digest_slots"],
            json!([])
        );

        tool.execute(json!({"action":"inherit_digest_slots"}))
            .await
            .unwrap();
        assert_eq!(
            tool.execute(json!({"action":"get"})).await.unwrap()["prefs"]["digest_slots"],
            Value::Null
        );
    }

    #[tokio::test]
    async fn duplicate_structured_digest_slot_is_rejected_without_persisting() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        let error = tool
            .execute(json!({
                "action": "set_digest_slots",
                "value": [
                    {"id":"market","time":"07:30"},
                    {"id":"market","time":"21:00"}
                ]
            }))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("重复 id"));
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["digest_slots"], Value::Null);
    }

    #[tokio::test]
    async fn set_price_high_pct_enforces_range() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"set_price_high_pct","value":3.5}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["price_high_pct_override"], json!(3.5));

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
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["price_high_pct_override"], json!(4.2));
    }

    #[tokio::test]
    async fn directional_and_large_position_thresholds_set_and_inherit() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"set_price_high_pct_up","value":6.0}))
            .await
            .unwrap();
        tool.execute(json!({"action":"set_price_high_pct_down","value":"5.0"}))
            .await
            .unwrap();
        tool.execute(json!({"action":"set_large_position_weight_pct","value":20}))
            .await
            .unwrap();
        tool.execute(json!({"action":"set_price_realert_step_pct","value":4}))
            .await
            .unwrap();

        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["price_high_pct_up_override"], json!(6.0));
        assert_eq!(
            response["prefs"]["price_high_pct_down_override"],
            json!(5.0)
        );
        assert_eq!(response["prefs"]["large_position_weight_pct"], json!(20.0));
        assert_eq!(
            response["prefs"]["price_realert_step_pct_override"],
            json!(4.0)
        );

        tool.execute(json!({"action":"inherit_price_high_pct_up"}))
            .await
            .unwrap();
        tool.execute(json!({
            "action":"set_price_high_pct_down",
            "value": null
        }))
        .await
        .unwrap();
        tool.execute(json!({"action":"inherit_large_position_weight_pct"}))
            .await
            .unwrap();
        tool.execute(json!({"action":"inherit_price_realert_step_pct"}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["price_high_pct_up_override"], Value::Null);
        assert_eq!(
            response["prefs"]["price_high_pct_down_override"],
            Value::Null
        );
        assert_eq!(response["prefs"]["large_position_weight_pct"], Value::Null);
        assert_eq!(
            response["prefs"]["price_realert_step_pct_override"],
            Value::Null
        );
    }

    #[tokio::test]
    async fn invalid_directional_threshold_does_not_persist() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({"action":"set_price_high_pct_up","value":6.0}))
            .await
            .unwrap();
        let error = tool
            .execute(json!({"action":"set_price_high_pct_up","value":80.0}))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("(0, 50]"));
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["price_high_pct_up_override"], json!(6.0));
    }

    #[tokio::test]
    async fn update_delivery_controls_applies_cross_field_transition_atomically() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": {"from":"00:00","to":"08:00"}
        }))
        .await
        .unwrap();
        tool.execute(json!({
            "action": "set_digest_slots",
            "value": [{"id":"old","time":"09:00"}]
        }))
        .await
        .unwrap();

        // 单独先改任一字段都会和旧值冲突；复合补丁按最终状态整体校验。
        tool.execute(json!({
            "action": "update_delivery_controls",
            "value": {
                "timezone": "Asia/Shanghai",
                "digest_slots": [
                    {"id":"postmarket","time":"07:00","label":"盘后要闻","floor_macro":2}
                ],
                "quiet_hours": {"from":"08:00","to":"10:00"},
                "price_high_pct": 6,
                "price_high_pct_up": 7,
                "price_high_pct_down": 5,
                "price_realert_step_pct": 4,
                "large_position_weight_pct": 20
            }
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["timezone"], json!("Asia/Shanghai"));
        assert_eq!(
            response["prefs"]["digest_slots"][0]["id"],
            json!("postmarket")
        );
        assert_eq!(response["prefs"]["quiet_hours"]["from"], json!("08:00"));
        assert_eq!(response["prefs"]["price_high_pct_override"], json!(6.0));
        assert_eq!(response["prefs"]["price_high_pct_up_override"], json!(7.0));
        assert_eq!(
            response["prefs"]["price_high_pct_down_override"],
            json!(5.0)
        );
        assert_eq!(response["prefs"]["large_position_weight_pct"], json!(20.0));
        assert_eq!(
            response["prefs"]["price_realert_step_pct_override"],
            json!(4.0)
        );

        let error = tool
            .execute(json!({
                "action": "update_delivery_controls",
                "value": {
                    "timezone": "America/New_York",
                    "price_high_pct_up": 99
                }
            }))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("(0, 50]"));
        let unchanged = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(unchanged["prefs"]["timezone"], json!("Asia/Shanghai"));
        assert_eq!(unchanged["prefs"]["price_high_pct_up_override"], json!(7.0));
    }

    #[tokio::test]
    async fn update_delivery_controls_rejects_unknown_fields() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        let error = tool
            .execute(json!({
                "action": "update_delivery_controls",
                "value": {"news_importance_prompt":"do not expose this"}
            }))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("不支持字段"));
        assert!(error.to_string().contains("news_importance_prompt"));
    }

    #[tokio::test]
    async fn set_immediate_kinds_validates_and_clears_on_empty() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_immediate_kinds",
            "value": ["weekly52_high", "analyst_grade"]
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(
            response["prefs"]["immediate_kinds"],
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
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["immediate_kinds"], json!(null));
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
            digest_defaults_fixture(),
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
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "23:00", "to": "07:00", "exempt_kinds": ["earnings_released"] },
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["quiet_hours"]["from"], json!("23:00"));
        assert_eq!(response["prefs"]["quiet_hours"]["to"], json!("07:00"));
        assert_eq!(
            response["prefs"]["quiet_hours"]["exempt_kinds"],
            json!(["earnings_released"])
        );
    }

    #[tokio::test]
    async fn set_quiet_hours_without_exempt_defaults_to_empty() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "22:30", "to": "06:30" },
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["quiet_hours"]["exempt_kinds"], json!([]));
    }

    #[tokio::test]
    async fn set_quiet_hours_validates_hhmm() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
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
        let tool = make_tool(dir.path());
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
        let tool = make_tool(dir.path());
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
        // make_tool() 用的是 telegram actor → display_text 应是 <pre> 包的等宽块
        let tool = make_tool(dir.path());
        let response = tool
            .execute(json!({"action":"get_overview"}))
            .await
            .unwrap();
        assert_eq!(response["status"], json!("ok"));
        let display_text = response["display_text"].as_str().expect("display_text");
        assert!(display_text.contains("你的推送日程"));
        assert!(display_text.contains("时刻"));
        // telegram → 走 <pre>
        assert!(display_text.contains("<pre>"));
        // 不应再出现 markdown table 字符
        assert!(!display_text.contains("| --- |"));
        assert_eq!(response["render_format"], json!("TelegramHtml"));
        let entries = response["overview"]["schedule"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn get_overview_explains_the_same_eight_four_ladder_used_by_router() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "update_delivery_controls",
            "value": {
                "price_high_pct": 8,
                "price_realert_step_pct": 4
            }
        }))
        .await
        .unwrap();

        let response = tool
            .execute(json!({"action":"get_overview"}))
            .await
            .unwrap();
        let policy = &response["overview"]["immediate"]["effective_price_alert_policy"];
        assert_eq!(policy["up"]["first_direct_pct"], json!(8.0));
        assert_eq!(policy["down"]["first_direct_pct"], json!(8.0));
        assert_eq!(policy["repeat_step_pct"], json!(4.0));
        assert_eq!(policy["repeat_step_source"], json!("actor_common"));
        assert_eq!(
            response["overview"]["immediate"]["high_severity_daily_cap"],
            json!(8)
        );
        assert_eq!(
            response["overview"]["immediate"]["same_symbol_cooldown_minutes"],
            json!(60)
        );
        let display_text = response["display_text"].as_str().unwrap();
        for expected in [
            "价格阶梯（最终生效）",
            "上涨首次 +8%",
            "+8% / +12% / +16%",
            "下跌首次 -8%",
            "-8% / -12% / -16%",
            "重复步长来源：用户通用设置",
            "每日 High 上限：每个事件类别最多 8 条",
            "价格阶梯也受此上限约束",
            "普通同标的 High 冷却：60 分钟",
            "盘中价格阶梯豁免该冷却",
        ] {
            assert!(
                display_text.contains(expected),
                "missing {expected:?} in {display_text}"
            );
        }
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
            digest_defaults_fixture(),
        );
        let response = tool
            .execute(json!({"action":"get_overview"}))
            .await
            .unwrap();
        let display_text = response["display_text"].as_str().unwrap();
        assert!(
            display_text.contains("```"),
            "discord 应用代码块: {display_text}"
        );
        assert!(!display_text.contains("<pre>"));
        assert_eq!(response["render_format"], json!("DiscordMarkdown"));
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
            digest_defaults_fixture(),
        );
        let response = tool
            .execute(json!({"action":"get_overview"}))
            .await
            .unwrap();
        let display_text = response["display_text"].as_str().unwrap();
        assert!(!display_text.contains("```"));
        assert!(!display_text.contains("<pre>"));
        assert!(
            display_text.contains("• "),
            "imessage 应该是项目符号列表: {display_text}"
        );
        assert_eq!(response["render_format"], json!("Plain"));
    }

    #[tokio::test]
    async fn clear_quiet_hours_removes_field() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "23:00", "to": "07:00" },
        }))
        .await
        .unwrap();
        tool.execute(json!({"action":"clear_quiet_hours"}))
            .await
            .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["quiet_hours"], json!(null));
    }

    #[tokio::test]
    async fn set_digest_slots_rejects_slot_inside_existing_quiet() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "00:00", "to": "08:00" },
        }))
        .await
        .unwrap();
        let err = tool
            .execute(json!({
                "action": "set_digest_slots",
                "value": ["02:30", "09:00"]
            }))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => {
                assert!(msg.contains("02:30"), "msg={msg}");
                assert!(msg.contains("quiet_hours"), "msg={msg}");
            }
            other => panic!("unexpected err {other:?}"),
        }
        // 落盘的 slots 应保持未变(default 即 None)
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["digest_slots"], json!(null));
    }

    #[tokio::test]
    async fn set_digest_slots_outside_quiet_succeeds() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "00:00", "to": "08:00" },
        }))
        .await
        .unwrap();
        tool.execute(json!({
            "action": "set_digest_slots",
            "value": ["09:00", "19:00"]
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        let times: Vec<String> = response["prefs"]["digest_slots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|slot_value| slot_value["time"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(times, vec!["09:00", "19:00"]);
    }

    #[tokio::test]
    async fn set_quiet_hours_rejects_when_existing_slot_falls_in() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_digest_slots",
            "value": ["02:30", "09:00"]
        }))
        .await
        .unwrap();
        let err = tool
            .execute(json!({
                "action": "set_quiet_hours",
                "value": { "from": "00:00", "to": "08:00" },
            }))
            .await
            .unwrap_err();
        match err {
            HoneError::Tool(msg) => {
                assert!(msg.contains("吞掉"), "msg={msg}");
                assert!(msg.contains("02:30"), "msg={msg}");
            }
            other => panic!("unexpected err {other:?}"),
        }
        // quiet 没落盘
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["quiet_hours"], json!(null));
    }

    #[tokio::test]
    async fn set_quiet_hours_safe_when_no_slot_overlap() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_digest_slots",
            "value": ["09:00", "19:00"]
        }))
        .await
        .unwrap();
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "23:00", "to": "07:00" },
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(response["prefs"]["quiet_hours"]["from"], json!("23:00"));
    }

    #[tokio::test]
    async fn digest_slot_at_quiet_end_boundary_is_valid() {
        let dir = tempdir().unwrap();
        let tool = make_tool(dir.path());
        tool.execute(json!({
            "action": "set_quiet_hours",
            "value": { "from": "23:00", "to": "07:30" },
        }))
        .await
        .unwrap();
        tool.execute(json!({
            "action": "set_digest_slots",
            "value": [{"id":"postmarket","time":"07:30","label":"盘后要闻"}]
        }))
        .await
        .unwrap();
        let response = tool.execute(json!({"action":"get"})).await.unwrap();
        assert_eq!(
            response["prefs"]["digest_slots"][0]["label"],
            json!("盘后要闻")
        );
    }
}
