import { makeContentProxy } from "../i18n"

const ZH = {
  channels: {
    multi_agent_desc: "MiniMax 搜索 + Gemini 回答",
    codex_acp_desc: "通过 codex-acp 驱动当前会话",
    opencode_acp_name: "自定义 OpenAI 协议",
    opencode_acp_desc: "OpenAI compatible / 推荐 OpenRouter",
    gemini_cli_desc: "复用本机 Gemini 命令行",
    codex_cli_desc: "复用本机 Codex 命令行",
  },

  status_panel: {
    backend_label: "后端连接",
    backend_connected: "已连接",
    backend_disconnected: "未连接",
    channels_label: "渠道",
    channels_summary: "{live} / {total} 在线",
    active_research_label: "进行中研究",
    active_research_summary: "{count} 个任务",
    research_open_link: "打开研究模块 →",
  },

  quick_chat: {
    title: "快速发起对话",
    subtitle: "输入问题直接发给 ME 渠道，Enter 发送",
    runner_chip_title: "{desc} · 点击前往配置",
    placeholder: "输入你想探索的投研问题…",
    shift_enter_hint: "Shift + Enter 换行",
    loading_settings: "加载配置中…",
    send_aria: "发送",
  },

  recent: {
    sessions_title: "最近会话",
    sessions_empty: "暂无最近会话",
    sessions_view_all: "全部 →",
    research_title: "进行中研究",
    research_empty: "暂无研究任务",
    research_view_all: "全部 →",
    last_message_empty: "(空)",
    created_prefix: "创建于 {time}",
  },
}

const EN: typeof ZH = {
  channels: {
    multi_agent_desc: "MiniMax search + Gemini answer",
    codex_acp_desc: "Drive sessions via codex-acp",
    opencode_acp_name: "Custom OpenAI protocol",
    opencode_acp_desc: "OpenAI compatible / OpenRouter recommended",
    gemini_cli_desc: "Reuse local Gemini CLI",
    codex_cli_desc: "Reuse local Codex CLI",
  },

  status_panel: {
    backend_label: "Backend",
    backend_connected: "Connected",
    backend_disconnected: "Disconnected",
    channels_label: "Channels",
    channels_summary: "{live} / {total} online",
    active_research_label: "Active research",
    active_research_summary: "{count} tasks",
    research_open_link: "Open research →",
  },

  quick_chat: {
    title: "Quick chat",
    subtitle: "Send a message to your ME session — press Enter to submit",
    runner_chip_title: "{desc} · click to configure",
    placeholder: "What research question do you want to explore?",
    shift_enter_hint: "Shift + Enter for newline",
    loading_settings: "Loading settings…",
    send_aria: "Send",
  },

  recent: {
    sessions_title: "Recent sessions",
    sessions_empty: "No recent sessions",
    sessions_view_all: "View all →",
    research_title: "Active research",
    research_empty: "No research tasks",
    research_view_all: "View all →",
    last_message_empty: "(empty)",
    created_prefix: "Created {time}",
  },
}

export const DASH = makeContentProxy(ZH, EN)
export const __DASH_TREES__ = { zh: ZH, en: EN } as const
