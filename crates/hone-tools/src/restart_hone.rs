//! RestartHoneTool — 管理员专属重启工具
//!
//! 仅对管理员用户开放。读取 data/runtime/current.pid，
//! 调用 scripts/restart_hone.sh 在后台安全地 kill 旧进程并重启 launch.sh。
//!
//! 状态机：
//!   launch.sh 启动 → 进程就绪 → 写 current.pid
//!   restart 触发   → 读 current.pid → 后台脚本 kill 旧 PID → 启动新 launch.sh
//!   新 launch.sh   → 进程就绪 → 写新 current.pid

use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;

use crate::base::{Tool, ToolParameter};

/// 管理员专属重启工具
pub struct RestartHoneTool {
    /// Hone 项目根目录（launch.sh 所在目录）
    project_root: PathBuf,
}

impl RestartHoneTool {
    /// 创建实例，传入项目根目录路径
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }
}

#[async_trait]
impl Tool for RestartHoneTool {
    fn name(&self) -> &str {
        "restart_hone"
    }

    fn description(&self) -> &str {
        "【仅管理员可用】重启 Hone 服务。\
        将在约 3 秒后优雅终止当前 Hone 进程并重新执行 launch.sh（包含重新构建）。\
        重启前请确保源码修改已完成，重启过程约需 1-3 分钟（含编译）。\
        重启日志写入 data/logs/restart.log。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "confirm".to_string(),
            param_type: "string".to_string(),
            description: "确认执行重启，必须填写 \"yes\"".to_string(),
            required: true,
            r#enum: Some(vec!["yes".to_string()]),
            items: None,
        }]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let confirm = args.get("confirm").and_then(|v| v.as_str()).unwrap_or("");

        if confirm != "yes" {
            return Ok(serde_json::json!({
                "success": false,
                "error": "请传入 confirm=\"yes\" 以确认重启操作"
            }));
        }

        let project_root = self.project_root.clone();
        let pid_file = project_root.join("data/runtime/current.pid");
        let restart_lock = project_root.join("data/runtime/restart.lock");
        let log_dir = project_root.join("data/logs");
        let restart_script = project_root.join("scripts/restart_hone.sh");

        // 防止并发重启：检查 lock 文件（超过 60s 的旧锁视为无效）
        if restart_lock.exists() {
            if let Ok(meta) = std::fs::metadata(&restart_lock) {
                if let Ok(modified) = meta.modified() {
                    let age = std::time::SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default();
                    if age.as_secs() < 60 {
                        return Ok(serde_json::json!({
                            "success": false,
                            "error": "重启已在进行中，请等待约 1-3 分钟后重试"
                        }));
                    }
                }
            }
        }

        // 检查 restart_hone.sh 是否存在
        if !restart_script.exists() {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!(
                    "重启脚本不存在：{}，请确认项目完整性",
                    restart_script.display()
                )
            }));
        }

        // 读取当前 launch.sh PID（可能为空——launch.sh 尚未进入就绪状态）
        let current_pid = std::fs::read_to_string(&pid_file)
            .ok()
            .unwrap_or_default()
            .trim()
            .to_string();

        // 验证 PID 格式（空字符串也允许，脚本会直接跳过 kill）
        if !current_pid.is_empty() && current_pid.parse::<u32>().is_err() {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("current.pid 内容异常（{}），中止重启", current_pid)
            }));
        }

        // 创建日志目录
        let _ = std::fs::create_dir_all(&log_dir);

        // 写入重启锁文件
        let _ = std::fs::write(&restart_lock, format!("{}", std::process::id()));

        tracing::info!(
            "[RestartHoneTool] 准备重启 Hone，当前 PID={}, 项目根={}",
            if current_pid.is_empty() {
                "未知（进程可能尚未就绪）"
            } else {
                &current_pid
            },
            project_root.display()
        );

        // 启动 scripts/restart_hone.sh，通过 nohup 在后台独立运行
        // 脚本接受两个参数：<project_root> <old_pid>
        // 脚本内部会将日志追加到 data/logs/restart.log，Rust 侧直接丢弃 stdout/stderr
        let spawn_result = std::process::Command::new("nohup")
            .arg("bash")
            .arg(&restart_script)
            .arg(project_root.as_os_str())
            .arg(&current_pid)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        // 无论是否成功，都清除锁文件（脚本成功后会自动退出，不需要保留锁）
        // 注意：脚本本身是 nohup 后台进程，spawn() 成功即可 detach
        match spawn_result {
            Ok(child) => {
                // 不 wait，直接 drop（Rust drop 不会 kill 子进程）
                std::mem::drop(child);

                // 延迟清除锁文件（由脚本实际完成后再清，这里 spawn 成功即 detach）
                // 锁文件由 60s 超时自动失效，无需显式删除
                tracing::info!("[RestartHoneTool] 重启脚本已后台启动，约 3 秒后生效");

                Ok(serde_json::json!({
                    "success": true,
                    "message": format!(
                        "Hone 重启指令已发出（当前 PID={}）。\
                        将在约 3 秒后优雅停止当前服务，重新编译并启动（约需 1-3 分钟）。\
                        消息通道将短暂中断，请耐心等待。重启日志：data/logs/restart.log",
                        if current_pid.is_empty() { "未知" } else { &current_pid }
                    ),
                    "current_pid": current_pid,
                    "restart_script": restart_script.display().to_string()
                }))
            }
            Err(e) => {
                // 清除锁文件，允许下次重试
                let _ = std::fs::remove_file(&restart_lock);
                tracing::error!("[RestartHoneTool] 启动重启脚本失败: {}", e);
                Ok(serde_json::json!({
                    "success": false,
                    "error": format!("重启失败，无法启动后台脚本: {}", e)
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name_and_description() {
        let tool = RestartHoneTool::new(PathBuf::from("/tmp/test"));
        assert_eq!(tool.name(), "restart_hone");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_parameters() {
        let tool = RestartHoneTool::new(PathBuf::from("/tmp/test"));
        let params = tool.parameters();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "confirm");
        assert!(params[0].required);
    }

    #[tokio::test]
    async fn test_execute_without_confirm() {
        let tool = RestartHoneTool::new(PathBuf::from("/tmp/test"));
        let result = tool
            .execute(serde_json::json!({ "confirm": "no" }))
            .await
            .unwrap();
        assert_eq!(result["success"], false);
    }
}
