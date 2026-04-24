//! `AgentSession::run` 用的 daily-conversation 配额 RAII guard。
//!
//! 主流程里任何一个失败分支(ensure_session 失败、slash skill 展开失败、
//! prepare_execution 失败、sandbox guard 不通过等)都必须把已经预留的
//! 配额「释放」掉,否则同一个用户当天能用的对话次数会被错误消耗。
//!
//! 引入这个 guard 之前,`run()` 里重复了 5 处
//! `if let Some(reservation) = quota_reservation.as_ref() { release... }`
//! 的样板,非常容易漏改；现在统一成「默认 drop = release,成功路径调用
//! `commit()` 消耗自身阻止 drop 自动释放」。

use hone_memory::ConversationQuotaReservation;
use std::sync::Arc;

use crate::HoneBotCore;

pub(super) struct QuotaReservationGuard {
    core: Arc<HoneBotCore>,
    reservation: Option<ConversationQuotaReservation>,
}

impl QuotaReservationGuard {
    pub(super) fn new(
        core: Arc<HoneBotCore>,
        reservation: Option<ConversationQuotaReservation>,
    ) -> Self {
        Self { core, reservation }
    }

    /// 成功路径：把预留的额度正式 commit 到当日计数。
    /// 消耗 self 防止随后的 Drop 再次执行 release。
    pub(super) fn commit(mut self) {
        if let Some(reservation) = self.reservation.take() {
            let _ = self
                .core
                .conversation_quota_storage
                .commit_daily_conversation(&reservation);
        }
    }
}

impl Drop for QuotaReservationGuard {
    fn drop(&mut self) {
        // 默认退出路径 = 失败 / panic / 提前 return：把 reservation 原样还回去,
        // 避免用户当日配额被白白消耗。`commit` 已经 take 过时这里是 no-op。
        if let Some(reservation) = self.reservation.take() {
            let _ = self
                .core
                .conversation_quota_storage
                .release_daily_conversation(&reservation);
        }
    }
}
