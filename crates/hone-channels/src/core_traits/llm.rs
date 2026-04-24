//! LLM provider / audit sink 访问器 trait。
//!
//! 当前 `HoneBotCore` 把主 LLM / auxiliary LLM / audit sink 暴露成 `pub`
//! 字段,直接 `core.llm.clone()` 就能拿到 `Arc<dyn LlmProvider>`。
//! 这样很顺手但也让「依赖了一个 LLM 路由」这件事隐形：审视一个模块是否
//! 需要 LLM 得翻代码。
//!
//! 抽 trait 之后把这种依赖显式化成 `&dyn LlmProviderBundle`,
//! 同时为测试 mock 打开通道。字段仍然是 `pub`,调用方的现有读法不变。

use std::sync::Arc;

use hone_core::LlmAuditSink;
use hone_llm::LlmProvider;

use crate::core::HoneBotCore;

/// 主 / 辅助 LLM provider + audit sink 的访问入口。
pub trait LlmProviderBundle: Send + Sync {
    /// 主对话 LLM（走 `agent.runner` 选定的路径;未配置则返回 `None`）。
    fn primary_llm(&self) -> Option<Arc<dyn LlmProvider>>;

    /// 辅助 LLM（heartbeat / session compaction 等后台任务使用)。
    fn auxiliary_llm(&self) -> Option<Arc<dyn LlmProvider>>;

    /// 审计落盘 sink（启用时把 LLM 请求 / 响应保存到 SQLite）。
    fn llm_audit_sink(&self) -> Option<Arc<dyn LlmAuditSink>>;

    /// 辅助 LLM 的显示用模型名称（降级时会回落到 `openrouter.sub_model`）。
    fn auxiliary_model_name(&self) -> String;

    /// 辅助 LLM 的 `(provider_display, model_display)`,供前端展示。
    fn auxiliary_provider_hint(&self) -> (String, String);
}

impl LlmProviderBundle for HoneBotCore {
    fn primary_llm(&self) -> Option<Arc<dyn LlmProvider>> {
        self.llm.clone()
    }

    fn auxiliary_llm(&self) -> Option<Arc<dyn LlmProvider>> {
        self.auxiliary_llm.clone()
    }

    fn llm_audit_sink(&self) -> Option<Arc<dyn LlmAuditSink>> {
        self.llm_audit.clone()
    }

    fn auxiliary_model_name(&self) -> String {
        HoneBotCore::auxiliary_model_name(self)
    }

    fn auxiliary_provider_hint(&self) -> (String, String) {
        HoneBotCore::auxiliary_provider_hint(self)
    }
}
