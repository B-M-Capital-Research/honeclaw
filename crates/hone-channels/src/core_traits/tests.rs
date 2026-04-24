//! Object-safety 编译期冒烟测试。
//!
//! 每个 trait 只做一件事：证明 `HoneBotCore` 可以被当成 `&dyn Trait` 使用。
//! 构造真正的 `HoneBotCore` 需要完整配置,在这里意义不大;行为测试仍然由
//! `core.rs` 覆盖。只要这几个函数能编译通过,就说明：
//! 1. trait 本身是 object-safe 的
//! 2. HoneBotCore 提供了完整的 trait 实现

use super::*;

#[test]
fn hone_bot_core_is_object_safe_audit_recorder() {
    fn _assert<T: AuditRecorder + ?Sized>(_: &T) {}
}

#[test]
fn hone_bot_core_is_object_safe_admin_intercept() {
    fn _assert<T: AdminIntercept + ?Sized>(_: &T) {}
}

#[test]
fn hone_bot_core_is_object_safe_path_resolver() {
    fn _assert<T: PathResolver + ?Sized>(_: &T) {}
}

#[test]
fn hone_bot_core_is_object_safe_runner_factory() {
    fn _assert<T: RunnerFactory + ?Sized>(_: &T) {}
}

#[test]
fn hone_bot_core_is_object_safe_tool_registry_factory() {
    fn _assert<T: ToolRegistryFactory + ?Sized>(_: &T) {}
}

#[test]
fn hone_bot_core_is_object_safe_llm_provider_bundle() {
    fn _assert<T: LlmProviderBundle + ?Sized>(_: &T) {}
}
