use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const FAILURE_WINDOW: Duration = Duration::from_secs(10 * 60);
const FAILURE_LIMIT: usize = 8;
const BLOCK_DURATION: Duration = Duration::from_secs(15 * 60);
const ENTRY_RETENTION: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicAuthLimitStatus {
    Allowed,
    Blocked { retry_after_secs: u64 },
}

#[derive(Debug)]
struct FailureTracker {
    failures: Vec<Instant>,
    blocked_until: Option<Instant>,
    last_seen_at: Instant,
}

impl FailureTracker {
    fn new(now: Instant) -> Self {
        Self {
            failures: Vec::new(),
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

#[derive(Default)]
pub struct PublicAuthLimiter {
    state: Mutex<HashMap<String, FailureTracker>>,
}

impl PublicAuthLimiter {
    pub fn check(&self, key: &str) -> PublicAuthLimitStatus {
        let now = Instant::now();
        let mut state = self.state.lock().unwrap();
        prune_stale_entries(&mut state, now);
        let tracker = state
            .entry(key.to_string())
            .or_insert_with(|| FailureTracker::new(now));
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
        prune_stale_entries(&mut state, now);
        let tracker = state
            .entry(key.to_string())
            .or_insert_with(|| FailureTracker::new(now));
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
        state.remove(key);
    }
}

fn prune_stale_entries(state: &mut HashMap<String, FailureTracker>, now: Instant) {
    state.retain(|_, tracker| !tracker.is_stale(now));
}

#[cfg(test)]
mod tests {
    use super::{BLOCK_DURATION, PublicAuthLimitStatus, PublicAuthLimiter};

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
}
