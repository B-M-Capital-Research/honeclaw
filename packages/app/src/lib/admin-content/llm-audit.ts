import { makeContentProxy } from "../i18n"

const ZH = {
  capability: {
    unavailable: "当前 backend 未开放 LLM 审计能力。",
  },
  toolbar: {
    title: "LLM 审计",
    filter_user_placeholder: "过滤用户 ID",
    filter_session_placeholder: "过滤 Session",
    status_all: "全部状态",
    status_success: "成功",
    status_failed: "失败",
    refresh_button: "刷新",
    total_count: "共 {count} 条",
  },
  table: {
    col_time: "时间",
    col_actor_session: "Actor / Session",
    col_provider_model: "服务 / 模型",
    col_operation: "操作",
    col_status: "状态",
    col_tokens: "Tokens",
    col_latency: "耗时",
    empty: "暂无审计记录",
    status_success: "成功",
    status_failed: "失败",
    actor_user_none: "无",
  },
  pagination: {
    prev: "上一页",
    next: "下一页",
    page_label: "第 {page} 页",
  },
  detail: {
    title: "记录详情",
    loading: "加载中...",
    load_failed_title: "加载失败",
    related_entities: "关联实体",
    error_section: "错误信息",
    tokens_section: "Token 使用",
    tokens_prompt: "提示 (Prompt)",
    tokens_completion: "补全 (Completion)",
    tokens_total: "总计 (Total)",
    request_json: "请求 JSON",
    response_json: "响应 JSON",
    metadata: "元数据",
  },
}

const EN: typeof ZH = {
  capability: {
    unavailable: "This backend does not expose the LLM audit capability.",
  },
  toolbar: {
    title: "LLM audit",
    filter_user_placeholder: "Filter user ID",
    filter_session_placeholder: "Filter session",
    status_all: "All statuses",
    status_success: "Success",
    status_failed: "Failed",
    refresh_button: "Refresh",
    total_count: "{count} records",
  },
  table: {
    col_time: "Time",
    col_actor_session: "Actor / Session",
    col_provider_model: "Provider / Model",
    col_operation: "Operation",
    col_status: "Status",
    col_tokens: "Tokens",
    col_latency: "Latency",
    empty: "No audit records",
    status_success: "Success",
    status_failed: "Failed",
    actor_user_none: "none",
  },
  pagination: {
    prev: "Prev",
    next: "Next",
    page_label: "Page {page}",
  },
  detail: {
    title: "Record detail",
    loading: "Loading…",
    load_failed_title: "Load failed",
    related_entities: "Related entities",
    error_section: "Error",
    tokens_section: "Token usage",
    tokens_prompt: "Prompt",
    tokens_completion: "Completion",
    tokens_total: "Total",
    request_json: "Request JSON",
    response_json: "Response JSON",
    metadata: "Metadata",
  },
}

export const LLM_AUDIT = makeContentProxy(ZH, EN)
export const __LLM_AUDIT_TREES__ = { zh: ZH, en: EN } as const
