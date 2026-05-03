import { makeContentProxy } from "../i18n"

const ZH = {
  page: {
    title: "推送日程",
    user_label: "用户",
    query_button: "查询",
    loading_button: "加载中…",
    err_pick_user: "请选择用户",
  },
  source: {
    digest: "Digest",
    cron_job: "自定义",
  },
  card: {
    actor: "actor",
    timezone: "时区",
    quiet_hours: "勿扰时段",
    quiet_disabled: "未启用",
    quiet_exempt_prefix: "豁免: {kinds}",
    immediate: "即时推",
    immediate_enabled: "✅ 启用",
    immediate_disabled: "❌ 已 disable",
    immediate_min_prefix: " · 最低: ",
    immediate_only_portfolio: " · 仅持仓",
    immediate_price_threshold: " · 价格阈值 {pct}%",
  },
  table: {
    col_time: "时刻",
    col_type: "类型",
    col_content: "内容",
    col_freq: "频率",
    col_active: "当日生效",
    col_hint: "操作提示",
    empty: "无定时推送（所有事件走即时推）",
    cell_quiet_held: "🌙 被静音吞",
    cell_bypass_quiet: "✅ 强制不静音",
    cell_active: "✅",
  },
  filters: {
    blocked_kinds: "屏蔽 kind: ",
    allow_kinds: "仅允许 kind: ",
    exempt_in_quiet: "静音期间豁免: ",
  },
}

const EN: typeof ZH = {
  page: {
    title: "Push schedule",
    user_label: "User",
    query_button: "Query",
    loading_button: "Loading…",
    err_pick_user: "Pick a user first.",
  },
  source: {
    digest: "Digest",
    cron_job: "Custom",
  },
  card: {
    actor: "actor",
    timezone: "Timezone",
    quiet_hours: "Quiet hours",
    quiet_disabled: "Disabled",
    quiet_exempt_prefix: "exempt: {kinds}",
    immediate: "Immediate",
    immediate_enabled: "✅ Enabled",
    immediate_disabled: "❌ Disabled",
    immediate_min_prefix: " · min: ",
    immediate_only_portfolio: " · portfolio only",
    immediate_price_threshold: " · price ≥ {pct}%",
  },
  table: {
    col_time: "Time",
    col_type: "Type",
    col_content: "Content",
    col_freq: "Frequency",
    col_active: "Today",
    col_hint: "Edit hint",
    empty: "No scheduled pushes (all events go immediate).",
    cell_quiet_held: "🌙 Held by quiet",
    cell_bypass_quiet: "✅ Bypasses quiet",
    cell_active: "✅",
  },
  filters: {
    blocked_kinds: "Blocked kinds: ",
    allow_kinds: "Allow kinds: ",
    exempt_in_quiet: "Quiet exempt: ",
  },
}

export const SCHEDULE = makeContentProxy(ZH, EN)
export const __SCHEDULE_TREES__ = { zh: ZH, en: EN } as const
