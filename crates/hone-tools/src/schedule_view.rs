//! `schedule_view` —— per-actor "我的推送日程"聚合视图。
//!
//! 一个共享的 build_overview 函数，把散落在 4 个地方的推送时间拍平成一张表：
//! - 持仓 digest 时刻 (per-actor `digest_windows` 优先,缺省全局 pre/post-market)
//! - 全局要闻 digest 时刻 (admin 配置,仅在 prefs.global_digest_enabled=true 时纳入)
//! - 自定义 cron jobs (含 bypass_quiet_hours 标记 + would_be_skipped_by_quiet)
//! - 即时推阈值 (kind 黑/白名单 + price_high_pct + min_severity)
//! - quiet_hours 区间
//!
//! 同一份后端逻辑给 NL `notification_prefs.get_overview` 工具和 admin 后台
//! `/api/admin/schedule` 共用,确保用户从 chat 看到的表跟 admin 后台一致。

use chrono::{DateTime, FixedOffset, NaiveTime, TimeZone, Timelike, Utc};
use hone_core::ActorIdentity;
use hone_core::quiet::QuietHours;
use hone_event_engine::Severity;
use hone_event_engine::prefs::{FilePrefsStorage, PrefsProvider};
use hone_event_engine::renderer::RenderFormat;
use hone_memory::CronJobStorage;
use hone_memory::cron_job::CronJob;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleOverview {
    pub actor: String,
    pub timezone: String,
    pub quiet_hours: Option<QuietHoursView>,
    /// 拍平后的全部时刻条目，按 time_local (HH:MM) 升序排序
    pub schedule: Vec<ScheduleEntry>,
    pub immediate: ImmediateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHoursView {
    pub from: String,
    pub to: String,
    pub exempt_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    /// 本地时刻 `"HH:MM"`，按 actor.timezone 解释
    pub time_local: String,
    pub source: ScheduleSource,
    /// 显示给用户的内容简述（"盘前持仓事件汇总"/"今日全球要闻"/cron 名称）
    pub content_hint: String,
    /// 频率标签：daily / workday / trading_day / weekly Mon / heartbeat / once
    pub frequency: String,
    /// 仅 cron job 有
    pub job_id: Option<String>,
    /// 时刻落在 quiet_hours 区间内 → 会被静音吞（cron 还得看 bypass_quiet_hours）
    pub will_be_held_by_quiet: bool,
    /// cron 任务是否豁免 quiet_hours
    pub bypass_quiet_hours: bool,
    /// 给 LLM 的「怎么改」提示
    pub edit_hint: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleSource {
    PortfolioDigest,
    GlobalDigest,
    CronJob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmediateConfig {
    pub enabled: bool,
    pub min_severity: String,
    pub portfolio_only: bool,
    pub price_high_pct: Option<f64>,
    pub allow_kinds: Option<Vec<String>>,
    pub blocked_kinds: Vec<String>,
    pub immediate_kinds: Option<Vec<String>>,
    /// quiet_hours 期间豁免的 kind tag —— 即使在静音区间也立即推
    pub exempt_in_quiet: Vec<String>,
}

/// 全局 digest 配置（从 `event_engine.global_digest` 节读取，传入即可）。
#[derive(Debug, Clone)]
pub struct GlobalDigestSlice {
    pub enabled: bool,
    /// admin 配置的 IANA 时区（用于解释 schedules）
    pub timezone: String,
    /// "HH:MM" 列表，例 ["07:30", "21:00"]
    pub schedules: Vec<String>,
}

/// 持仓 digest 全局默认（从 `event_engine.digest` 节读取）。
#[derive(Debug, Clone)]
pub struct PortfolioDigestDefaults {
    pub pre_market: String,
    pub post_market: String,
}

/// 主入口：聚合一名 actor 的全部推送时刻视图。
pub fn build_overview(
    prefs_dir: &Path,
    cron_jobs_dir: &Path,
    actor: &ActorIdentity,
    global_digest: &GlobalDigestSlice,
    portfolio_defaults: &PortfolioDigestDefaults,
    now: DateTime<Utc>,
) -> anyhow::Result<ScheduleOverview> {
    let prefs_storage = FilePrefsStorage::new(prefs_dir)?;
    let prefs = prefs_storage.load(actor);
    let cron_storage = CronJobStorage::new(cron_jobs_dir);
    let jobs = cron_storage.list_jobs(actor);

    let actor_key = format!(
        "{}::{}::{}",
        actor.channel,
        actor.channel_scope.clone().unwrap_or_default(),
        actor.user_id
    );
    let timezone = prefs
        .timezone
        .clone()
        .unwrap_or_else(|| "Asia/Shanghai".to_string());

    let mut schedule: Vec<ScheduleEntry> = Vec::new();

    // 1. 持仓 digest
    let portfolio_windows: Vec<String> = match prefs.digest_windows.as_deref() {
        Some(v) if v.is_empty() => Vec::new(), // 用户主动关
        Some(v) => v.to_vec(),
        None => vec![
            portfolio_defaults.pre_market.clone(),
            portfolio_defaults.post_market.clone(),
        ],
    };
    for window in &portfolio_windows {
        schedule.push(ScheduleEntry {
            time_local: window.clone(),
            source: ScheduleSource::PortfolioDigest,
            content_hint: "持仓相关事件汇总".to_string(),
            frequency: "daily".to_string(),
            job_id: None,
            will_be_held_by_quiet: time_in_quiet(window, prefs.quiet_hours.as_ref()),
            bypass_quiet_hours: false,
            edit_hint: "notification_prefs(action=\"set_digest_windows\", value=[\"08:30\",\"19:00\"])"
                .to_string(),
        });
    }

    // 2. 全局要闻 digest（仅当用户未关闭 + admin 开启）
    if prefs.global_digest_enabled && global_digest.enabled {
        for window in &global_digest.schedules {
            let local_time = convert_to_actor_tz(window, &global_digest.timezone, &timezone, now)
                .unwrap_or_else(|| window.clone());
            schedule.push(ScheduleEntry {
                time_local: local_time.clone(),
                source: ScheduleSource::GlobalDigest,
                content_hint: "今日全球要闻".to_string(),
                frequency: "daily".to_string(),
                job_id: None,
                will_be_held_by_quiet: time_in_quiet(&local_time, prefs.quiet_hours.as_ref()),
                bypass_quiet_hours: false,
                edit_hint: "notification_prefs(action=\"set_global_digest_enabled\", value=false) 关掉; 时刻由管理员维护"
                    .to_string(),
            });
        }
    }

    // 3. 自定义 cron jobs
    for job in jobs.iter().filter(|j| j.enabled) {
        let time_local = format!("{:02}:{:02}", job.schedule.hour, job.schedule.minute);
        let frequency = describe_cron_frequency(job);
        let in_quiet = time_in_quiet(&time_local, prefs.quiet_hours.as_ref());
        schedule.push(ScheduleEntry {
            time_local,
            source: ScheduleSource::CronJob,
            content_hint: job.name.clone(),
            frequency,
            job_id: Some(job.id.clone()),
            will_be_held_by_quiet: in_quiet && !job.bypass_quiet_hours,
            bypass_quiet_hours: job.bypass_quiet_hours,
            edit_hint: format!(
                "cron_job(action=\"update\", job_id=\"{}\", hour=8, minute=30) 改时间; bypass_quiet_hours=true 让本任务豁免静音",
                job.id
            ),
        });
    }

    // 按 time_local 升序排
    schedule.sort_by(|a, b| a.time_local.cmp(&b.time_local));

    let immediate = ImmediateConfig {
        enabled: prefs.enabled,
        min_severity: severity_str(&prefs.min_severity),
        portfolio_only: prefs.portfolio_only,
        price_high_pct: prefs.price_high_pct_override,
        allow_kinds: prefs.allow_kinds.clone(),
        blocked_kinds: prefs.blocked_kinds.clone(),
        immediate_kinds: prefs.immediate_kinds.clone(),
        exempt_in_quiet: prefs
            .quiet_hours
            .as_ref()
            .map(|qh| qh.exempt_kinds.clone())
            .unwrap_or_default(),
    };

    Ok(ScheduleOverview {
        actor: actor_key,
        timezone,
        quiet_hours: prefs.quiet_hours.map(|qh| QuietHoursView {
            from: qh.from,
            to: qh.to,
            exempt_kinds: qh.exempt_kinds,
        }),
        schedule,
        immediate,
    })
}

/// 把概览渲染成具体渠道能正确显示的文本。**LLM 应直接 relay 输出**。
///
/// 各渠道的实际能力（不是 RenderFormat 字面意思）：
/// - Discord: 支持 `**bold**` / `\`code\`` / `\`\`\`block\`\`\``，**不支持 markdown 表格**。
///   → 用 monospace 代码块 + display-width 对齐模拟表格
/// - Telegram: 支持 `<b>` / `<pre>` HTML，**不支持表格**。→ `<pre>` 包等宽对齐
/// - Feishu: bot 文本消息**不渲染** markdown / HTML。→ 干净的项目符号列表
/// - iMessage: 同 Feishu,纯文本。→ 项目符号列表
pub fn render_overview(overview: &ScheduleOverview, fmt: RenderFormat) -> String {
    match fmt {
        RenderFormat::DiscordMarkdown => render_with_codeblock(overview, "```\n", "\n```"),
        RenderFormat::TelegramHtml => render_with_codeblock(overview, "<pre>\n", "\n</pre>"),
        RenderFormat::Plain | RenderFormat::FeishuPost => render_as_list(overview),
    }
}

/// 按 actor.channel 字段推断 RenderFormat,NL 工具按调用方所在渠道用。
pub fn channel_render_format(channel: &str) -> RenderFormat {
    match channel.to_ascii_lowercase().as_str() {
        "discord" => RenderFormat::DiscordMarkdown,
        "telegram" => RenderFormat::TelegramHtml,
        "feishu" => RenderFormat::FeishuPost,
        _ => RenderFormat::Plain,
    }
}

/// Discord / Telegram:用代码块包一张 monospace 表。CJK / emoji 显示宽度按 2 算。
fn render_with_codeblock(overview: &ScheduleOverview, open: &str, close: &str) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    write_header(&mut out, overview);

    if overview.schedule.is_empty() {
        let _ = writeln!(out, "（当前没有任何定时推送，所有事件走即时推）");
    } else {
        // 表格:时刻 / 类型 / 内容 / 频率 / 状态(emoji)
        let headers = ["时刻", "类型", "内容", "频率", "状态"];
        let mut rows: Vec<[String; 5]> = Vec::new();
        for e in &overview.schedule {
            let kind = source_label(e.source);
            let active = if e.will_be_held_by_quiet {
                "🌙 静音吞"
            } else if e.bypass_quiet_hours {
                "✅ 强发"
            } else {
                "✅"
            };
            rows.push([
                e.time_local.clone(),
                kind.to_string(),
                e.content_hint.clone(),
                e.frequency.clone(),
                active.to_string(),
            ]);
        }
        // 计算每列 display-width
        let mut widths = [0usize; 5];
        for (i, h) in headers.iter().enumerate() {
            widths[i] = display_width(h);
        }
        for row in &rows {
            for i in 0..5 {
                widths[i] = widths[i].max(display_width(&row[i]));
            }
        }
        // 表头 + 分隔(用 ─, 单字符宽)
        out.push_str(open);
        for (i, h) in headers.iter().enumerate() {
            out.push_str(&pad_to(h, widths[i]));
            if i + 1 < 5 {
                out.push_str("  ");
            }
        }
        out.push('\n');
        for (i, w) in widths.iter().enumerate() {
            out.push_str(&"─".repeat(*w));
            if i + 1 < 5 {
                out.push_str("  ");
            }
        }
        out.push('\n');
        for row in &rows {
            for i in 0..5 {
                out.push_str(&pad_to(&row[i], widths[i]));
                if i + 1 < 5 {
                    out.push_str("  ");
                }
            }
            out.push('\n');
        }
        // 去掉最后那个 \n,让 close 紧贴
        if out.ends_with('\n') {
            out.pop();
        }
        out.push_str(close);
        out.push('\n');
    }

    write_immediate_section(&mut out, overview);
    out
}

/// Feishu / iMessage:纯文本项目符号列表,不依赖 monospace。
fn render_as_list(overview: &ScheduleOverview) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    write_header(&mut out, overview);

    if overview.schedule.is_empty() {
        let _ = writeln!(out, "（当前没有任何定时推送，所有事件走即时推）");
    } else {
        let _ = writeln!(out, "定时推送：");
        for e in &overview.schedule {
            let kind = source_label(e.source);
            let active = if e.will_be_held_by_quiet {
                "🌙 被静音吞"
            } else if e.bypass_quiet_hours {
                "✅ 强制不静音"
            } else {
                "✅"
            };
            let _ = writeln!(
                out,
                "• {} {} · {} · {} {}",
                e.time_local, kind, e.content_hint, e.frequency, active
            );
        }
    }
    out.push('\n');
    write_immediate_section(&mut out, overview);
    out
}

fn write_header(out: &mut String, overview: &ScheduleOverview) {
    use std::fmt::Write;
    let _ = writeln!(out, "你的推送日程");
    let _ = writeln!(out, "时区：{}", overview.timezone);
    if let Some(qh) = &overview.quiet_hours {
        let exempt = if qh.exempt_kinds.is_empty() {
            String::new()
        } else {
            format!("（豁免: {}）", qh.exempt_kinds.join(", "))
        };
        let _ = writeln!(out, "勿扰时段：🌙 {} – {}{}", qh.from, qh.to, exempt);
    } else {
        let _ = writeln!(out, "勿扰时段：未启用");
    }
    out.push('\n');
}

fn write_immediate_section(out: &mut String, overview: &ScheduleOverview) {
    use std::fmt::Write;
    let _ = writeln!(out, "即时推：");
    let _ = writeln!(
        out,
        "• 总开关：{}",
        if overview.immediate.enabled {
            "✅ 启用"
        } else {
            "❌ 已 disable"
        }
    );
    let _ = writeln!(out, "• 最低严重度：{}", overview.immediate.min_severity);
    if overview.immediate.portfolio_only {
        let _ = writeln!(out, "• 只推命中持仓的事件");
    }
    if let Some(p) = overview.immediate.price_high_pct {
        let _ = writeln!(out, "• 价格异动阈值：{p}%");
    }
    if !overview.immediate.blocked_kinds.is_empty() {
        let _ = writeln!(
            out,
            "• 屏蔽 kind：{}",
            overview.immediate.blocked_kinds.join(", ")
        );
    }
    if let Some(allow) = overview.immediate.allow_kinds.as_ref() {
        if !allow.is_empty() {
            let _ = writeln!(out, "• 仅允许 kind：{}", allow.join(", "));
        }
    }
    if !overview.immediate.exempt_in_quiet.is_empty() {
        let _ = writeln!(
            out,
            "• 静音期间豁免：{}",
            overview.immediate.exempt_in_quiet.join(", ")
        );
    }
}

fn source_label(s: ScheduleSource) -> &'static str {
    match s {
        ScheduleSource::PortfolioDigest => "持仓 digest",
        ScheduleSource::GlobalDigest => "全球要闻",
        ScheduleSource::CronJob => "自定义",
    }
}

/// 简化的 display width:ASCII = 1,其它(CJK / emoji) = 2。
/// 不引入 unicode-width crate,对中文场景已经够用。
fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

fn pad_to(s: &str, width: usize) -> String {
    let cur = display_width(s);
    if cur >= width {
        s.to_string()
    } else {
        let pad = " ".repeat(width - cur);
        format!("{s}{pad}")
    }
}

fn severity_str(s: &Severity) -> String {
    match s {
        Severity::Low => "low".into(),
        Severity::Medium => "medium".into(),
        Severity::High => "high".into(),
    }
}

fn describe_cron_frequency(job: &CronJob) -> String {
    let repeat = job.schedule.repeat.as_str();
    match repeat {
        "daily" => "每日".to_string(),
        "workday" => "工作日".to_string(),
        "trading_day" => "交易日".to_string(),
        "holiday" => "节假日".to_string(),
        "once" => "一次性".to_string(),
        "heartbeat" => "心跳（每 30 分钟检查）".to_string(),
        "weekly" => match job.schedule.weekday {
            Some(0) => "每周一".into(),
            Some(1) => "每周二".into(),
            Some(2) => "每周三".into(),
            Some(3) => "每周四".into(),
            Some(4) => "每周五".into(),
            Some(5) => "每周六".into(),
            Some(6) => "每周日".into(),
            _ => "每周".into(),
        },
        other => other.to_string(),
    }
}

/// 判断给定本地 HH:MM 是否落在 quiet_hours 区间内。语义跟
/// `hone_core::quiet::quiet_window_active` 对齐，但只看本地时刻不需要 now。
fn time_in_quiet(local_hhmm: &str, qh: Option<&QuietHours>) -> bool {
    let Some(qh) = qh else {
        return false;
    };
    let Ok(t) = NaiveTime::parse_from_str(local_hhmm, "%H:%M") else {
        return false;
    };
    let Ok(from_t) = NaiveTime::parse_from_str(&qh.from, "%H:%M") else {
        return false;
    };
    let Ok(to_t) = NaiveTime::parse_from_str(&qh.to, "%H:%M") else {
        return false;
    };
    let now_min = t.hour() as i32 * 60 + t.minute() as i32;
    let from_min = from_t.hour() as i32 * 60 + from_t.minute() as i32;
    let to_min = to_t.hour() as i32 * 60 + to_t.minute() as i32;
    if from_min == to_min {
        return false;
    }
    if from_min < to_min {
        now_min >= from_min && now_min < to_min
    } else {
        now_min >= from_min || now_min < to_min
    }
}

/// 把全局 digest 的 HH:MM 在 admin 时区下解释，再转成 actor 时区下的 HH:MM。
/// 用于让用户在自己时区里看到全球 digest 的命中时刻。
fn convert_to_actor_tz(
    src_hhmm: &str,
    src_tz: &str,
    dst_tz: &str,
    now: DateTime<Utc>,
) -> Option<String> {
    let t = NaiveTime::parse_from_str(src_hhmm, "%H:%M").ok()?;
    let src = src_tz.parse::<chrono_tz::Tz>().ok();
    let dst = dst_tz.parse::<chrono_tz::Tz>().ok();
    let utc = match src {
        Some(tz) => {
            let local = tz.from_utc_datetime(&now.naive_utc());
            let date = local.date_naive();
            let naive = date.and_time(t);
            tz.from_local_datetime(&naive).single()?.with_timezone(&Utc)
        }
        None => {
            // src tz 不可解析,假定 UTC+8 (与 EventEngine 默认一致)
            let off = FixedOffset::east_opt(8 * 3600).unwrap();
            let date = off.from_utc_datetime(&now.naive_utc()).date_naive();
            let naive = date.and_time(t);
            off.from_local_datetime(&naive).single()?.with_timezone(&Utc)
        }
    };
    let local = match dst {
        Some(tz) => tz.from_utc_datetime(&utc.naive_utc()),
        None => return Some(src_hhmm.to_string()),
    };
    Some(format!("{:02}:{:02}", local.hour(), local.minute()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_event_engine::prefs::{NotificationPrefs, QuietHours as QH};
    use tempfile::tempdir;

    fn actor() -> ActorIdentity {
        ActorIdentity::new("imessage", "u1", None::<String>).unwrap()
    }

    fn defaults() -> (GlobalDigestSlice, PortfolioDigestDefaults) {
        (
            GlobalDigestSlice {
                enabled: true,
                timezone: "Asia/Shanghai".into(),
                schedules: vec!["07:30".into(), "21:00".into()],
            },
            PortfolioDigestDefaults {
                pre_market: "08:30".into(),
                post_market: "09:00".into(),
            },
        )
    }

    #[test]
    fn build_overview_with_no_cron_or_prefs_returns_global_defaults() {
        let dir = tempdir().unwrap();
        let prefs_dir = dir.path().join("prefs");
        let cron_dir = dir.path().join("cron");
        std::fs::create_dir_all(&prefs_dir).unwrap();
        std::fs::create_dir_all(&cron_dir).unwrap();
        let (gd, pd) = defaults();
        let ov = build_overview(&prefs_dir, &cron_dir, &actor(), &gd, &pd, Utc::now()).unwrap();
        // 无 prefs → 默认 4 条:08:30/09:00 持仓 + 07:30/21:00 全球
        assert_eq!(ov.schedule.len(), 4);
        assert!(ov.quiet_hours.is_none());
        assert!(ov.immediate.enabled); // 默认 true
        assert!(
            ov.schedule
                .iter()
                .any(|e| e.source == ScheduleSource::PortfolioDigest && e.time_local == "08:30")
        );
        assert!(
            ov.schedule
                .iter()
                .any(|e| e.source == ScheduleSource::GlobalDigest && e.time_local == "07:30")
        );
    }

    #[test]
    fn build_overview_marks_cron_skipped_by_quiet() {
        let dir = tempdir().unwrap();
        let prefs_dir = dir.path().join("prefs");
        let cron_dir = dir.path().join("cron");
        std::fs::create_dir_all(&prefs_dir).unwrap();
        std::fs::create_dir_all(&cron_dir).unwrap();

        let prefs_storage = FilePrefsStorage::new(&prefs_dir).unwrap();
        let prefs = NotificationPrefs {
            quiet_hours: Some(QH {
                from: "23:00".into(),
                to: "07:00".into(),
                exempt_kinds: vec![],
            }),
            ..Default::default()
        };
        prefs_storage.save(&actor(), &prefs).unwrap();

        let cron_storage = CronJobStorage::new(&cron_dir);
        // 02:00 触发 → 在 quiet 内
        let r = cron_storage.add_job(
            &actor(),
            "夜半监控",
            Some(2),
            Some(0),
            "daily",
            "do something",
            "u1",
            None,
            None,
            true,
            None,
            true,
        );
        assert_eq!(r["success"], serde_json::json!(true), "add_job failed: {r}");
        // 09:00 触发 → 不在 quiet 内
        let r2 = cron_storage.add_job(
            &actor(),
            "盘后总结",
            Some(9),
            Some(0),
            "daily",
            "do something else",
            "u1",
            None,
            None,
            true,
            None,
            true,
        );
        assert_eq!(r2["success"], serde_json::json!(true), "add_job 2 failed: {r2}");

        let (gd, pd) = defaults();
        let ov = build_overview(&prefs_dir, &cron_dir, &actor(), &gd, &pd, Utc::now()).unwrap();

        let nighty = ov
            .schedule
            .iter()
            .find(|e| e.content_hint == "夜半监控")
            .expect("found cron 02:00");
        assert!(
            nighty.will_be_held_by_quiet,
            "02:00 cron 应被 quiet 吞掉"
        );
        let post = ov
            .schedule
            .iter()
            .find(|e| e.content_hint == "盘后总结")
            .expect("found cron 09:00");
        assert!(!post.will_be_held_by_quiet);
    }

    #[test]
    fn channel_render_format_maps_known_channels() {
        assert_eq!(
            channel_render_format("discord"),
            RenderFormat::DiscordMarkdown
        );
        assert_eq!(
            channel_render_format("Telegram"),
            RenderFormat::TelegramHtml
        );
        assert_eq!(channel_render_format("feishu"), RenderFormat::FeishuPost);
        assert_eq!(channel_render_format("imessage"), RenderFormat::Plain);
        assert_eq!(channel_render_format("anything-else"), RenderFormat::Plain);
    }

    fn make_overview() -> ScheduleOverview {
        let dir = tempdir().unwrap();
        let prefs_dir = dir.path().join("prefs");
        let cron_dir = dir.path().join("cron");
        std::fs::create_dir_all(&prefs_dir).unwrap();
        std::fs::create_dir_all(&cron_dir).unwrap();
        let (gd, pd) = defaults();
        build_overview(&prefs_dir, &cron_dir, &actor(), &gd, &pd, Utc::now()).unwrap()
    }

    #[test]
    fn render_overview_discord_uses_codeblock_table() {
        let ov = make_overview();
        let s = render_overview(&ov, RenderFormat::DiscordMarkdown);
        assert!(s.contains("你的推送日程"));
        assert!(s.contains("Asia/Shanghai"));
        // ``` 包住表
        assert!(s.contains("```\n"), "should open code block: {s}");
        assert!(s.contains("\n```\n"), "should close code block: {s}");
        // 表头列名
        assert!(s.contains("时刻"));
        assert!(s.contains("类型"));
        // 不应再出现 markdown table 字符
        assert!(!s.contains("| --- |"));
        assert!(!s.contains("## "));
    }

    #[test]
    fn render_overview_telegram_uses_pre_block() {
        let ov = make_overview();
        let s = render_overview(&ov, RenderFormat::TelegramHtml);
        assert!(s.contains("<pre>\n"));
        assert!(s.contains("\n</pre>"));
        assert!(s.contains("时刻"));
    }

    #[test]
    fn render_overview_feishu_and_imessage_use_bullet_list() {
        let ov = make_overview();
        for fmt in [RenderFormat::FeishuPost, RenderFormat::Plain] {
            let s = render_overview(&ov, fmt);
            assert!(s.contains("你的推送日程"));
            assert!(s.contains("定时推送："));
            // 不应出现代码块或 HTML 标签
            assert!(!s.contains("```"));
            assert!(!s.contains("<pre>"));
            // 每条 schedule 单行带 •
            assert!(s.contains("• 07:30") || s.contains("• 08:30"));
        }
    }

    #[test]
    #[ignore]
    fn dump_all_renders_for_visual_inspection() {
        let ov = make_overview();
        for (label, fmt) in [
            ("Discord", RenderFormat::DiscordMarkdown),
            ("Telegram", RenderFormat::TelegramHtml),
            ("Feishu", RenderFormat::FeishuPost),
            ("iMessage (Plain)", RenderFormat::Plain),
        ] {
            println!("\n========== {label} ==========");
            println!("{}", render_overview(&ov, fmt));
        }
    }

    #[test]
    fn pad_to_handles_cjk_width() {
        // "时刻" display width = 4 (2 CJK chars * 2)
        assert_eq!(pad_to("时刻", 8), "时刻    "); // 4 个空格补到 8
        assert_eq!(pad_to("ascii", 8), "ascii   "); // 5 + 3
        // 已经够宽不补
        assert_eq!(pad_to("时刻", 2), "时刻");
    }
}
