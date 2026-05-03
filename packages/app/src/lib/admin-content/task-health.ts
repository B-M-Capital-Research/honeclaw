import { makeContentProxy } from "../i18n"

const ZH = {
  page: {
    title: "任务健康",
    window_label: "窗口",
    filter_task_label: "过滤 task",
    filter_all: "全部",
    refresh_button: "刷新",
    refreshing_button: "刷新中…",
  },
  summary: {
    eyebrow: "24h 汇总(每个 task 一行)",
    col_task: "Task",
    col_last_seen: "最近一次",
    col_runs_24h: "24h 总",
    col_ok: "ok",
    col_skipped: "skipped",
    col_failed: "failed",
    col_success_rate: "成功率",
    col_last_error: "最近错误",
    empty_no_records: "过去 24h 没有 task_runs.jsonl 记录。检查 web-api 是否已经启动且 with_task_runs_dir 已配置。",
    badge_latest_failure: "最新失败",
    badge_recovered: "已恢复 +{count}",
  },
  runs: {
    eyebrow: "最近运行(最多 500 条,倒序)",
    col_started: "起始",
    col_task: "Task",
    col_outcome: "Outcome",
    col_items: "Items",
    col_duration: "耗时",
    col_error: "错误",
    empty_no_match: "没有匹配的记录。",
  },
  relative: {
    seconds_ago: "{count}s ago",
    minutes_ago: "{count}m ago",
    hours_ago: "{count}h ago",
    days_ago: "{count}d ago",
  },
}

const EN: typeof ZH = {
  page: {
    title: "Task health",
    window_label: "Window",
    filter_task_label: "Filter task",
    filter_all: "All",
    refresh_button: "Refresh",
    refreshing_button: "Refreshing…",
  },
  summary: {
    eyebrow: "24h summary (one row per task)",
    col_task: "Task",
    col_last_seen: "Last seen",
    col_runs_24h: "24h total",
    col_ok: "ok",
    col_skipped: "skipped",
    col_failed: "failed",
    col_success_rate: "Success rate",
    col_last_error: "Last error",
    empty_no_records: "No task_runs.jsonl records in the past 24h. Check that web-api is running and with_task_runs_dir is configured.",
    badge_latest_failure: "Latest failure",
    badge_recovered: "Recovered +{count}",
  },
  runs: {
    eyebrow: "Recent runs (up to 500, newest first)",
    col_started: "Started",
    col_task: "Task",
    col_outcome: "Outcome",
    col_items: "Items",
    col_duration: "Duration",
    col_error: "Error",
    empty_no_match: "No matching records.",
  },
  relative: {
    seconds_ago: "{count}s ago",
    minutes_ago: "{count}m ago",
    hours_ago: "{count}h ago",
    days_ago: "{count}d ago",
  },
}

export const TASK_HEALTH = makeContentProxy(ZH, EN)
export const __TASK_HEALTH_TREES__ = { zh: ZH, en: EN } as const
