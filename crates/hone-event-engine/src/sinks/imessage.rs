//! iMessage OutboundSink —— 调 `osascript` 驱动 macOS Messages.app 发消息。
//!
//! `actor.user_id` 必须是 Messages.app 能识别的 participant(`+1xxxxxxxxxx` /
//! `foo@bar.com` / iCloud Apple ID)。Engine 不区分 direct/group —— iMessage 的
//! 群聊参与人同样以 participant 形式寻址,`channel_scope` 目前只作为元数据,不
//! 影响目的地。
//!
//! 限制:
//! - 只能在 macOS + Messages.app 已登录的机器上跑。其它 OS 会直接 bail。
//! - osascript 是阻塞调用,用 `tokio::task::spawn_blocking` 隔离。

use async_trait::async_trait;
use hone_core::ActorIdentity;

use crate::renderer::RenderFormat;
use crate::router::OutboundSink;

pub struct IMessageSink;

impl IMessageSink {
    pub fn new() -> Self {
        Self
    }

    fn build_script(handle: &str, text: &str) -> String {
        let escaped_handle = handle.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            r#"tell application "Messages"
    set targetService to 1st account whose service type = iMessage
    set targetBuddy to participant "{escaped_handle}" of targetService
    send "{escaped_text}" to targetBuddy
end tell"#
        )
    }
}

impl Default for IMessageSink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OutboundSink for IMessageSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        if !cfg!(target_os = "macos") {
            anyhow::bail!("iMessage sink only available on macOS");
        }
        let script = Self::build_script(&actor.user_id, body);
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
        })
        .await??;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            anyhow::bail!("osascript failed: {stderr}");
        }
        Ok(())
    }

    fn format(&self) -> RenderFormat {
        RenderFormat::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_escapes_quotes_and_backslashes_in_handle() {
        let script = IMessageSink::build_script(r#"a\"b"#, "hello");
        assert!(script.contains(r#"participant "a\\\"b""#));
    }

    #[test]
    fn script_escapes_quotes_in_body() {
        let script = IMessageSink::build_script("+12025550100", r#"hi "world""#);
        assert!(script.contains(r#"send "hi \"world\"""#));
    }
}
