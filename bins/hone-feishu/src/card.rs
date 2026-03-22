use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant as StdInstant};

use super::client::FeishuApiClient;
use tracing::warn;

const CARDKIT_THROTTLE_MS: u64 = 200;

pub(crate) struct CardKitSession {
    facade: FeishuApiClient,
    card_id: String,
    sequence: AtomicU64,
    last_sent: Mutex<String>,
    last_update_at: Mutex<StdInstant>,
}

fn truncate_summary(text: &str, max: usize) -> String {
    let clean: String = text.chars().take(max + 3).collect::<String>();
    let clean = clean.replace('\n', " ");
    if clean.chars().count() <= max {
        clean
    } else {
        let s: String = clean.chars().take(max - 3).collect();
        format!("{s}...")
    }
}

impl CardKitSession {
    pub(crate) fn new(facade: FeishuApiClient, card_id: String) -> Self {
        Self {
            facade,
            card_id,
            sequence: AtomicU64::new(1),
            last_sent: Mutex::new(String::new()),
            last_update_at: Mutex::new(
                StdInstant::now() - Duration::from_millis(CARDKIT_THROTTLE_MS + 1),
            ),
        }
    }

    fn next_seq(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::SeqCst)
    }

    pub(crate) async fn update(&self, text: &str) {
        let now = StdInstant::now();
        {
            let mut last_at = self.last_update_at.lock().unwrap();
            if now.duration_since(*last_at) < Duration::from_millis(CARDKIT_THROTTLE_MS) {
                return;
            }
            *last_at = now;
        }

        {
            let mut last = self.last_sent.lock().unwrap();
            if *last == text {
                return;
            }
            *last = text.to_string();
        }

        let seq = self.next_seq();
        let uuid = format!("u_{}_{}", self.card_id, seq);
        if let Err(e) = self
            .facade
            .update_card_element(&self.card_id, "content", text, seq, &uuid)
            .await
        {
            warn!("[Feishu/CardKit] update element 失败: {e}");
        }
    }

    pub(crate) async fn force_update(&self, text: &str) {
        *self.last_update_at.lock().unwrap() = StdInstant::now();
        {
            let mut last = self.last_sent.lock().unwrap();
            if *last == text {
                return;
            }
            *last = text.to_string();
        }
        let seq = self.next_seq();
        let uuid = format!("fu_{}_{}", self.card_id, seq);
        if let Err(e) = self
            .facade
            .update_card_element(&self.card_id, "content", text, seq, &uuid)
            .await
        {
            warn!("[Feishu/CardKit] force_update element 失败: {e}");
        }
    }

    pub(crate) async fn close(&self, final_text: &str) {
        let last = self.last_sent.lock().unwrap().clone();
        if final_text != last {
            let seq = self.next_seq();
            let uuid = format!("f_{}_{}", self.card_id, seq);
            if let Err(e) = self
                .facade
                .update_card_element(&self.card_id, "content", final_text, seq, &uuid)
                .await
            {
                warn!("[Feishu/CardKit] close: final update element 失败: {e}");
            }
        }

        let seq = self.next_seq();
        let uuid = format!("c_{}_{}", self.card_id, seq);
        let summary = truncate_summary(final_text, 50);
        if let Err(e) = self
            .facade
            .close_card_streaming(&self.card_id, &summary, seq, &uuid)
            .await
        {
            warn!("[Feishu/CardKit] close streaming 失败: {e}");
        }
    }
}
