use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const FAILURE_WINDOW: Duration = Duration::from_secs(10 * 60);
const FAILURE_LIMIT: usize = 8;
const BLOCK_DURATION: Duration = Duration::from_secs(15 * 60);
const ENTRY_RETENTION: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_TRACKERS: usize = 16_384;
const FULL_PRUNE_INTERVAL: Duration = Duration::from_secs(60);
const SMS_SEND_COOLDOWN: Duration = Duration::from_secs(60);
const SMS_SEND_PHONE_WINDOW: Duration = Duration::from_secs(24 * 60 * 60);
const SMS_SEND_PHONE_LIMIT: usize = 10;
const SMS_SEND_IP_WINDOW: Duration = Duration::from_secs(60 * 60);
const SMS_SEND_IP_LIMIT: usize = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicAuthLimitStatus {
    Allowed,
    Blocked { retry_after_secs: u64 },
}

#[derive(Debug)]
struct FailureTracker {
    failures: Vec<Instant>,
    attempts: Vec<Instant>,
    blocked_until: Option<Instant>,
    last_seen_at: Instant,
}

impl FailureTracker {
    fn new(now: Instant) -> Self {
        Self {
            failures: Vec::new(),
            attempts: Vec::new(),
            blocked_until: None,
            last_seen_at: now,
        }
    }

    fn compact(&mut self, now: Instant) {
        self.failures
            .retain(|attempt| now.saturating_duration_since(*attempt) <= FAILURE_WINDOW);
        if self.blocked_until.is_some_and(|until| until <= now) {
            self.blocked_until = None;
        }
        self.last_seen_at = now;
    }

    fn is_stale(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.last_seen_at) > ENTRY_RETENTION
    }
}

#[derive(Debug)]
struct LimiterState {
    trackers: HashMap<String, FailureTracker>,
    last_full_prune_at: Instant,
}

impl Default for LimiterState {
    fn default() -> Self {
        Self {
            trackers: HashMap::new(),
            last_full_prune_at: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct PublicAuthLimiter {
    state: Mutex<LimiterState>,
}

impl PublicAuthLimiter {
    pub fn check(&self, key: &str) -> PublicAuthLimitStatus {
        let now = Instant::now();
        let mut state = self.state.lock().unwrap();
        if state
            .trackers
            .get(key)
            .is_some_and(|tracker| tracker.is_stale(now))
        {
            state.trackers.remove(key);
            return PublicAuthLimitStatus::Allowed;
        }
        let Some(tracker) = state.trackers.get_mut(key) else {
            return PublicAuthLimitStatus::Allowed;
        };
        tracker.compact(now);
        if let Some(until) = tracker.blocked_until {
            return PublicAuthLimitStatus::Blocked {
                retry_after_secs: until.saturating_duration_since(now).as_secs().max(1),
            };
        }
        PublicAuthLimitStatus::Allowed
    }

    pub fn record_failure(&self, key: &str) -> Option<u64> {
        let now = Instant::now();
        let mut state = self.state.lock().unwrap();
        let Some(tracker) = tracker_for_key(&mut state, key, now) else {
            return Some(FULL_PRUNE_INTERVAL.as_secs());
        };
        tracker.compact(now);
        tracker.failures.push(now);
        if tracker.failures.len() >= FAILURE_LIMIT {
            tracker.failures.clear();
            tracker.blocked_until = Some(now + BLOCK_DURATION);
            return Some(BLOCK_DURATION.as_secs());
        }
        None
    }

    pub fn record_success(&self, key: &str) {
        let mut state = self.state.lock().unwrap();
        state.trackers.remove(key);
    }

    pub fn consume_sms_send(&self, ip_key: &str, phone_key: &str) -> PublicAuthLimitStatus {
        let now = Instant::now();
        let mut state = self.state.lock().unwrap();
        let ip_key = format!("sms-send:{ip_key}");

        if tracker_for_key(&mut state, &ip_key, now).is_none()
            || tracker_for_key(&mut state, phone_key, now).is_none()
        {
            return PublicAuthLimitStatus::Blocked {
                retry_after_secs: FULL_PRUNE_INTERVAL.as_secs(),
            };
        }

        let ip_status = {
            let tracker = state.trackers.get_mut(&ip_key).expect("tracker inserted");
            check_attempt_window(tracker, now, SMS_SEND_IP_WINDOW, SMS_SEND_IP_LIMIT, None)
        };
        if let PublicAuthLimitStatus::Blocked { .. } = ip_status {
            return ip_status;
        }

        let phone_status = {
            let tracker = state.trackers.get_mut(phone_key).expect("tracker inserted");
            check_attempt_window(
                tracker,
                now,
                SMS_SEND_PHONE_WINDOW,
                SMS_SEND_PHONE_LIMIT,
                Some(SMS_SEND_COOLDOWN),
            )
        };
        if let PublicAuthLimitStatus::Blocked { .. } = phone_status {
            return phone_status;
        }

        state
            .trackers
            .get_mut(&ip_key)
            .expect("tracker inserted")
            .attempts
            .push(now);
        state
            .trackers
            .get_mut(phone_key)
            .expect("tracker inserted")
            .attempts
            .push(now);
        PublicAuthLimitStatus::Allowed
    }
}

fn tracker_for_key<'a>(
    state: &'a mut LimiterState,
    key: &str,
    now: Instant,
) -> Option<&'a mut FailureTracker> {
    if !state.trackers.contains_key(key) {
        if state.trackers.len() >= MAX_TRACKERS {
            if now.saturating_duration_since(state.last_full_prune_at) >= FULL_PRUNE_INTERVAL {
                state.trackers.retain(|_, tracker| !tracker.is_stale(now));
                state.last_full_prune_at = now;
            }
            if state.trackers.len() >= MAX_TRACKERS {
                return None;
            }
        }
        state
            .trackers
            .insert(key.to_string(), FailureTracker::new(now));
    }
    state.trackers.get_mut(key)
}

fn check_attempt_window(
    tracker: &mut FailureTracker,
    now: Instant,
    window: Duration,
    limit: usize,
    cooldown: Option<Duration>,
) -> PublicAuthLimitStatus {
    tracker
        .attempts
        .retain(|attempt| now.saturating_duration_since(*attempt) <= window);
    tracker.last_seen_at = now;

    if let (Some(cooldown), Some(last_attempt)) = (cooldown, tracker.attempts.last()) {
        let elapsed = now.saturating_duration_since(*last_attempt);
        if elapsed < cooldown {
            return PublicAuthLimitStatus::Blocked {
                retry_after_secs: cooldown.saturating_sub(elapsed).as_secs().max(1),
            };
        }
    }

    if tracker.attempts.len() >= limit {
        let retry_after_secs = tracker
            .attempts
            .first()
            .map(|first| window.saturating_sub(now.saturating_duration_since(*first)))
            .unwrap_or(window)
            .as_secs()
            .max(1);
        return PublicAuthLimitStatus::Blocked { retry_after_secs };
    }

    PublicAuthLimitStatus::Allowed
}

#[cfg(test)]
mod tests {
    use super::{BLOCK_DURATION, MAX_TRACKERS, PublicAuthLimitStatus, PublicAuthLimiter};

    #[test]
    fn limiter_blocks_after_too_many_failures() {
        let limiter = PublicAuthLimiter::default();
        for _ in 0..7 {
            assert_eq!(limiter.check("ip:1"), PublicAuthLimitStatus::Allowed);
            assert_eq!(limiter.record_failure("ip:1"), None);
        }

        assert_eq!(
            limiter.record_failure("ip:1"),
            Some(BLOCK_DURATION.as_secs())
        );
        match limiter.check("ip:1") {
            PublicAuthLimitStatus::Blocked { retry_after_secs } => {
                assert!(retry_after_secs > 0);
            }
            PublicAuthLimitStatus::Allowed => panic!("expected limiter to block"),
        }
    }

    #[test]
    fn successful_login_clears_previous_failures() {
        let limiter = PublicAuthLimiter::default();
        for _ in 0..7 {
            assert_eq!(limiter.record_failure("ip:2"), None);
        }

        limiter.record_success("ip:2");

        assert_eq!(limiter.check("ip:2"), PublicAuthLimitStatus::Allowed);
        assert_eq!(limiter.record_failure("ip:2"), None);
        assert_eq!(limiter.check("ip:2"), PublicAuthLimitStatus::Allowed);
    }

    #[test]
    fn checking_unseen_keys_does_not_grow_limiter_state() {
        let limiter = PublicAuthLimiter::default();

        for index in 0..(MAX_TRACKERS * 2) {
            assert_eq!(
                limiter.check(&format!("sms-login:{index}")),
                PublicAuthLimitStatus::Allowed
            );
        }

        assert!(limiter.state.lock().unwrap().trackers.is_empty());
    }

    #[test]
    fn failure_tracker_cardinality_is_hard_bounded() {
        let limiter = PublicAuthLimiter::default();

        for index in 0..(MAX_TRACKERS * 2) {
            let _ = limiter.record_failure(&format!("sms-login:{index}"));
        }

        assert_eq!(limiter.state.lock().unwrap().trackers.len(), MAX_TRACKERS);
    }

    #[test]
    fn sms_send_success_budget_enforces_phone_cooldown() {
        let limiter = PublicAuthLimiter::default();

        assert_eq!(
            limiter.consume_sms_send("ip:203.0.113.1", "sms-send:13800138000"),
            PublicAuthLimitStatus::Allowed
        );
        match limiter.consume_sms_send("ip:203.0.113.1", "sms-send:13800138000") {
            PublicAuthLimitStatus::Blocked { retry_after_secs } => {
                assert!(retry_after_secs > 0);
            }
            PublicAuthLimitStatus::Allowed => panic!("expected SMS resend cooldown"),
        }
    }
}
