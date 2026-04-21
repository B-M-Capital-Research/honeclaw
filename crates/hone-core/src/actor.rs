use serde::{Deserialize, Serialize};

use crate::{HoneError, HoneResult};

/// 描述一份持久化 session 的归属范围。
///
/// 它回答的是“这轮消息应该写进哪份历史”：
/// - `Direct`：私聊 / 单用户 session
/// - `Group`：按群共享的 session
///
/// 这与渠道入口里的 `ChatMode` 不同。`ChatMode` 只表示一条消息来自私聊还是群聊，
/// 不决定最终写入哪一个 session 文件。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    #[default]
    Direct,
    Group,
}

/// session 归属标识。
///
/// `ActorIdentity` 表示“谁在发起当前请求”，用于权限、quota、sandbox 等执行期能力；
/// `SessionIdentity` 表示“把当前消息落到哪份会话历史里”，用于上下文恢复与持久化。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionIdentity {
    pub channel: String,
    #[serde(default)]
    pub kind: SessionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_scope: Option<String>,
}

impl SessionIdentity {
    pub fn direct(channel: impl Into<String>, user_id: impl Into<String>) -> HoneResult<Self> {
        let channel = channel.into().trim().to_string();
        if channel.is_empty() {
            return Err(HoneError::Config("channel 不能为空".to_string()));
        }

        let user_id = user_id.into().trim().to_string();
        if user_id.is_empty() {
            return Err(HoneError::Config("user_id 不能为空".to_string()));
        }

        Ok(Self {
            channel,
            kind: SessionKind::Direct,
            user_id: Some(user_id),
            channel_scope: None,
        })
    }

    pub fn group(channel: impl Into<String>, channel_scope: impl Into<String>) -> HoneResult<Self> {
        let channel = channel.into().trim().to_string();
        if channel.is_empty() {
            return Err(HoneError::Config("channel 不能为空".to_string()));
        }

        let channel_scope = channel_scope.into().trim().to_string();
        if channel_scope.is_empty() {
            return Err(HoneError::Config("channel_scope 不能为空".to_string()));
        }

        Ok(Self {
            channel,
            kind: SessionKind::Group,
            user_id: None,
            channel_scope: Some(channel_scope),
        })
    }

    pub fn from_actor(actor: &ActorIdentity) -> HoneResult<Self> {
        match actor.channel_scope.as_deref() {
            Some(scope) => Self::group(&actor.channel, scope),
            None => Self::direct(&actor.channel, &actor.user_id),
        }
    }

    pub fn from_session_id(session_id: &str) -> Option<Self> {
        if let Some(encoded) = session_id.strip_prefix("Actor_") {
            let parts: Vec<&str> = encoded.splitn(3, "__").collect();
            if parts.len() != 3 {
                return None;
            }

            let channel = decode_component(parts[0]);
            let scope = decode_component(parts[1]);
            let user_id = decode_component(parts[2]);
            if channel.is_empty() || user_id.is_empty() {
                return None;
            }

            return if scope == "direct" {
                Self::direct(channel, user_id).ok()
            } else {
                Self::group(channel, scope).ok()
            };
        }

        if let Some(encoded) = session_id.strip_prefix("Session_") {
            let parts: Vec<&str> = encoded.splitn(2, "__group__").collect();
            if parts.len() != 2 {
                return None;
            }

            let channel = decode_component(parts[0]);
            let channel_scope = decode_component(parts[1]);
            if channel.is_empty() || channel_scope.is_empty() {
                return None;
            }

            return Self::group(channel, channel_scope).ok();
        }

        None
    }

    pub fn session_id(&self) -> String {
        match self.kind {
            SessionKind::Direct => format!(
                "Actor_{}__direct__{}",
                encode_component(&self.channel),
                encode_component(self.user_id.as_deref().unwrap_or_default())
            ),
            SessionKind::Group => format!(
                "Session_{}__group__{}",
                encode_component(&self.channel),
                encode_component(self.channel_scope.as_deref().unwrap_or_default())
            ),
        }
    }

    pub fn is_group(&self) -> bool {
        matches!(self.kind, SessionKind::Group)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActorIdentity {
    pub channel: String,
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_scope: Option<String>,
}

impl ActorIdentity {
    pub fn new(
        channel: impl Into<String>,
        user_id: impl Into<String>,
        channel_scope: Option<impl Into<String>>,
    ) -> HoneResult<Self> {
        let channel = channel.into().trim().to_string();
        if channel.is_empty() {
            return Err(HoneError::Config("channel 不能为空".to_string()));
        }

        let user_id = user_id.into().trim().to_string();
        if user_id.is_empty() {
            return Err(HoneError::Config("user_id 不能为空".to_string()));
        }

        let channel_scope = channel_scope
            .map(|scope| scope.into().trim().to_string())
            .filter(|scope| !scope.is_empty());

        Ok(Self {
            channel,
            user_id,
            channel_scope,
        })
    }

    pub fn storage_key(&self) -> String {
        let scope = self.channel_scope.as_deref().unwrap_or("direct");
        format!(
            "{}__{}__{}",
            encode_component(&self.channel),
            encode_component(scope),
            encode_component(&self.user_id)
        )
    }

    pub fn session_id(&self) -> String {
        format!("Actor_{}", self.storage_key())
    }

    pub fn from_session_id(session_id: &str) -> Option<Self> {
        let encoded = session_id.strip_prefix("Actor_")?;
        let parts: Vec<&str> = encoded.splitn(3, "__").collect();
        if parts.len() != 3 {
            return None;
        }

        let channel = decode_component(parts[0]);
        let scope = decode_component(parts[1]);
        let user_id = decode_component(parts[2]);
        if channel.is_empty() || user_id.is_empty() {
            return None;
        }

        let channel_scope = if scope == "direct" { None } else { Some(scope) };
        Self::new(channel, user_id, channel_scope).ok()
    }

    /// 该 actor 是否来自单聊。`channel_scope` 为空即单聊；有值则是群聊/频道 scope。
    pub fn is_direct(&self) -> bool {
        self.channel_scope.is_none()
    }

    pub fn channel_fs_component(&self) -> String {
        encode_component(&self.channel)
    }

    pub fn scoped_user_fs_key(&self) -> String {
        let scope = self.channel_scope.as_deref().unwrap_or("direct");
        format!(
            "{}__{}",
            encode_component(scope),
            encode_component(&self.user_id)
        )
    }
}

fn encode_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        if byte.is_ascii_alphanumeric() || *byte == b'-' {
            out.push(char::from(*byte));
        } else {
            out.push_str(&format!("_{byte:02x}"));
        }
    }
    out
}

fn decode_component(encoded: &str) -> String {
    let mut out = String::with_capacity(encoded.len());
    let bytes = encoded.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' && i + 2 < bytes.len() {
            let hi = bytes[i + 1];
            let lo = bytes[i + 2];
            if let (Some(h), Some(l)) = (hex_digit(hi), hex_digit(lo)) {
                out.push(char::from(h * 16 + l));
                i += 3;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ActorIdentity, SessionIdentity, SessionKind};

    #[test]
    fn storage_key_changes_with_each_dimension() {
        let base = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("base");
        let diff_channel = ActorIdentity::new("feishu", "alice", Some("g:1:c:2")).expect("c");
        let diff_user = ActorIdentity::new("discord", "bob", Some("g:1:c:2")).expect("u");
        let diff_scope = ActorIdentity::new("discord", "alice", Some("g:1:c:3")).expect("s");

        assert_ne!(base.storage_key(), diff_channel.storage_key());
        assert_ne!(base.storage_key(), diff_user.storage_key());
        assert_ne!(base.storage_key(), diff_scope.storage_key());
    }

    #[test]
    fn empty_scope_is_normalized_to_none() {
        let actor = ActorIdentity::new("discord", "alice", Some("  ")).expect("actor");
        assert_eq!(actor.channel_scope, None);
        assert!(actor.session_id().contains("direct"));
    }

    #[test]
    fn filesystem_components_keep_channel_and_identity_split() {
        let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
        assert_eq!(actor.channel_fs_component(), "discord");
        assert_eq!(actor.scoped_user_fs_key(), "g_3a1_3ac_3a2__alice");
    }

    #[test]
    fn direct_conversation_keeps_existing_actor_session_shape() {
        let actor = ActorIdentity::new("telegram", "alice", None::<String>).expect("actor");
        let session_identity = SessionIdentity::from_actor(&actor).expect("session");
        assert_eq!(session_identity.kind, SessionKind::Direct);
        assert_eq!(session_identity.session_id(), actor.session_id());
    }

    #[test]
    fn group_conversation_session_is_shared_by_scope() {
        let alice = ActorIdentity::new("feishu", "alice", Some("chat:42")).expect("alice");
        let bob = ActorIdentity::new("feishu", "bob", Some("chat:42")).expect("bob");
        let alice_session = SessionIdentity::from_actor(&alice).expect("alice session");
        let bob_session = SessionIdentity::from_actor(&bob).expect("bob session");

        assert!(alice_session.is_group());
        assert_eq!(alice_session.session_id(), bob_session.session_id());
    }

    #[test]
    fn session_identity_can_be_restored_from_actor_session_id() {
        let actor =
            ActorIdentity::new("discord", "alice@example.com", Some("g:1:c:2")).expect("actor");
        let restored = SessionIdentity::from_session_id(&actor.session_id()).expect("restored");

        assert!(restored.is_group());
        assert_eq!(restored.channel, "discord");
        assert_eq!(restored.channel_scope.as_deref(), Some("g:1:c:2"));
    }

    #[test]
    fn session_identity_can_be_restored_from_shared_group_session_id() {
        let session = SessionIdentity::group("feishu", "chat:42").expect("session");
        let restored = SessionIdentity::from_session_id(&session.session_id()).expect("restored");

        assert!(restored.is_group());
        assert_eq!(restored.channel, "feishu");
        assert_eq!(restored.channel_scope.as_deref(), Some("chat:42"));
    }

    #[test]
    fn actor_identity_can_be_restored_from_actor_session_id() {
        let actor =
            ActorIdentity::new("imessage", "+8613121812525", None::<String>).expect("actor");
        let restored = ActorIdentity::from_session_id(&actor.session_id()).expect("restored");

        assert_eq!(restored, actor);
    }
}
