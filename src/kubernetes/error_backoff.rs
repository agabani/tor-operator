use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

const BASE_DELAY: Duration = Duration::from_secs(5);
const MAX_DELAY: Duration = Duration::from_mins(5);
const STALE_THRESHOLD: Duration = Duration::from_mins(10);

pub struct ErrorBackoff {
    state: Mutex<HashMap<String, (u32, Instant)>>,
}

impl Default for ErrorBackoff {
    fn default() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl ErrorBackoff {
    pub fn next_delay<R>(&self, resource: &R) -> Duration
    where
        R: super::Resource,
    {
        if let Ok(uid) = resource.try_uid() {
            let mut state = self.state.lock().expect("error backoff mutex poisoned");

            let now = Instant::now();

            state.retain(|_, entry| now.duration_since(entry.1) < STALE_THRESHOLD);

            let entry = state.entry(uid.into()).or_insert((0, now));

            entry.0 = entry.0.saturating_add(1);
            entry.1 = now;

            BASE_DELAY
                .saturating_mul(1u32 << entry.0.saturating_sub(1).min(6))
                .min(MAX_DELAY)
        } else {
            BASE_DELAY
        }
    }

    pub fn reset<R>(&self, resource: &R)
    where
        R: super::Resource,
    {
        if let Ok(uid) = resource.try_uid() {
            self.state
                .lock()
                .expect("error backoff mutex poisoned")
                .remove(&*uid);
        }
    }
}

#[cfg(test)]
mod tests {
    use k8s_openapi::{api::core::v1::ConfigMap, apimachinery::pkg::apis::meta::v1::ObjectMeta};

    use super::*;

    fn test_resource(uid: String) -> ConfigMap {
        ConfigMap {
            metadata: ObjectMeta {
                uid: Some(uid),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn delay_doubles_per_failure_then_caps_at_max() {
        let backoff = ErrorBackoff::default();
        let r = test_resource("uid-1".to_string());
        for expected in [5, 10, 20, 40, 80, 160] {
            assert_eq!(backoff.next_delay(&r), Duration::from_secs(expected));
        }
        for _ in 0..10 {
            assert_eq!(backoff.next_delay(&r), MAX_DELAY);
        }
    }

    #[test]
    fn different_keys_are_tracked_independently() {
        let backoff = ErrorBackoff::default();
        let r1 = test_resource("uid-1".to_string());
        let r2 = test_resource("uid-2".to_string());
        backoff.next_delay(&r1);
        backoff.next_delay(&r1);

        assert_eq!(backoff.next_delay(&r2), Duration::from_secs(5));
        assert_eq!(backoff.next_delay(&r1), Duration::from_secs(20));
    }

    #[test]
    fn recovery_resets_delay_to_base() {
        let backoff = ErrorBackoff::default();
        let r = test_resource("uid-1".to_string());
        for _ in 0..8 {
            backoff.next_delay(&r);
        }
        backoff.reset(&r);
        assert_eq!(backoff.next_delay(&r), Duration::from_secs(5));
    }

    #[test]
    fn missing_uid_returns_base_delay() {
        let backoff = ErrorBackoff::default();
        let r = ConfigMap::default();
        assert_eq!(backoff.next_delay(&r), BASE_DELAY);
    }

    #[test]
    fn reset_missing_uid_does_not_panic() {
        let backoff = ErrorBackoff::default();
        let r = ConfigMap::default();
        backoff.reset(&r);
    }

    #[test]
    fn reset_unknown_uid_does_not_panic() {
        let backoff = ErrorBackoff::default();
        let r = test_resource("never-seen".to_string());
        backoff.reset(&r);
    }
}
