use std::{collections::HashMap, sync::Mutex, time::Duration};

const BASE_DELAY: Duration = Duration::from_secs(5);
const MAX_DELAY: Duration = Duration::from_mins(5);

pub struct ErrorBackoff {
    state: Mutex<HashMap<String, u32>>,
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
            let count = state.entry(uid.into()).or_insert(0);
            *count = count.saturating_add(1);
            let shift = count.saturating_sub(1).min(6);
            BASE_DELAY.saturating_mul(1u32 << shift).min(MAX_DELAY)
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

    fn resource(uid: String) -> ConfigMap {
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
        let r = resource("uid-1".to_string());
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
        let r1 = resource("uid-1".to_string());
        let r2 = resource("uid-2".to_string());
        backoff.next_delay(&r1);
        backoff.next_delay(&r1);

        assert_eq!(backoff.next_delay(&r2), Duration::from_secs(5));
        assert_eq!(backoff.next_delay(&r1), Duration::from_secs(20));
    }

    #[test]
    fn recovery_resets_delay_to_base() {
        let backoff = ErrorBackoff::default();
        let r = resource("uid-1".to_string());
        for _ in 0..8 {
            backoff.next_delay(&r);
        }
        backoff.reset(&r);
        assert_eq!(backoff.next_delay(&r), Duration::from_secs(5));
    }
}
