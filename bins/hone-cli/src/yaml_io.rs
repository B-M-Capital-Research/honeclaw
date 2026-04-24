//! CLI 级别的 YAML / JSON 输出辅助 + 应用 mutations 到 canonical config 的胶水。
//!
//! 这里放的是「所有 hone-cli 子命令都可能用到的 IO 片段」,不含业务 logic:
//! - [`apply_message`] —— 把 `ConfigApplyPlan` 翻译成人类可读的反馈文案
//! - [`apply_mutations_and_generate`] —— apply_config_mutations + 立即重写
//!   effective-config snapshot,返回带新 revision 的 result
//! - [`yaml_value_from_cli`] —— 把命令行原始字符串解析成 `serde_yaml::Value`
//!   (既支持 `foo`、`"foo"`,也支持 `{a: 1}` 这种内联 YAML)
//! - [`value_to_pretty_text`] —— 标量直接显示,非标量走 yaml 多行格式
//! - [`print_json`] —— 给 `--json` 输出路径用的 pretty JSON printer

use serde::Serialize;
use serde_yaml::Value;

use hone_core::config::{
    ConfigApplyPlan, ConfigMutation, ConfigMutationResult, apply_config_mutations,
    generate_effective_config,
};

use crate::common;

pub(crate) fn apply_message(plan: &ConfigApplyPlan) -> String {
    if plan.restart_required {
        return "配置已保存，需重启运行时".to_string();
    }
    if !plan.restarted_components.is_empty() {
        return format!(
            "配置已保存，并需重启组件：{}",
            plan.restarted_components.join(", ")
        );
    }
    "配置已保存，已立即生效".to_string()
}

/// 把 mutations 写入 canonical config,然后立刻重生成 effective-config 快照,
/// 把 result 的 revision 替换成新快照的 revision（避免两边漂移）。
pub(crate) fn apply_mutations_and_generate(
    paths: &common::ResolvedRuntimePaths,
    mutations: &[ConfigMutation],
) -> Result<ConfigMutationResult, String> {
    let mut result = apply_config_mutations(&paths.canonical_config_path, mutations)
        .map_err(|e| e.to_string())?;
    result.config_revision =
        generate_effective_config(&paths.canonical_config_path, &paths.effective_config_path)
            .map_err(|e| e.to_string())?;
    Ok(result)
}

pub(crate) fn yaml_value_from_cli(raw: &str) -> Result<Value, String> {
    serde_yaml::from_str(raw).map_err(|e| format!("无法解析配置值: {e}"))
}

pub(crate) fn value_to_pretty_text(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        _ => serde_yaml::to_string(value)
            .unwrap_or_else(|_| "<unable to render>".to_string())
            .trim()
            .to_string(),
    }
}

pub(crate) fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let rendered = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    println!("{rendered}");
    Ok(())
}
