import { makeContentProxy } from "../i18n"

const ZH = {
  capability: {
    unavailable: "当前后端未开放日志能力。",
  },
  toolbar: {
    title: "日志",
    search_placeholder: "搜索日志…",
    user_filter_placeholder: "按用户 ID 筛选…",
    user_filter_title: "只显示与该用户相关的日志（匹配结构化用户主体或消息文本）",
    pause_button: "暂停",
    resume_button: "继续",
    clear_button: "清空",
    count_label: "{count} 条",
    status_live: "实时",
    status_disconnected: "断开",
  },
  list: {
    empty: "暂无匹配日志",
    msg_id_prefix: "MSG_ID: {id}",
  },
}

const EN: typeof ZH = {
  capability: {
    unavailable: "The current backend does not expose the logs capability.",
  },
  toolbar: {
    title: "Logs",
    search_placeholder: "Search logs…",
    user_filter_placeholder: "Filter by user ID…",
    user_filter_title: "Only show logs related to this user (matches structured actor or message text).",
    pause_button: "Pause",
    resume_button: "Resume",
    clear_button: "Clear",
    count_label: "{count} entries",
    status_live: "Live",
    status_disconnected: "Disconnected",
  },
  list: {
    empty: "No matching logs",
    msg_id_prefix: "MSG_ID: {id}",
  },
}

export const LOGS = makeContentProxy(ZH, EN)
export const __LOGS_TREES__ = { zh: ZH, en: EN } as const
