#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Notify;
    use tokio::time::timeout;

    struct Debouncer {
        interval: Duration,
        last_trigger: tokio::sync::Mutex<tokio::time::Instant>,
    }

    impl Debouncer {
        fn new(interval_ms: u64) -> Self {
            Self {
                interval: Duration::from_millis(interval_ms),
                last_trigger: tokio::sync::Mutex::new(tokio::time::Instant::now()),
            }
        }

        async fn should_fire(&self) -> bool {
            let mut last = self.last_trigger.lock().await;
            let now = tokio::time::Instant::now();
            if now - *last >= self.interval {
                *last = now;
                true
            } else {
                false
            }
        }

        async fn reset(&self) {
            *self.last_trigger.lock().await = tokio::time::Instant::now();
        }
    }

    #[tokio::test]
    async fn test_debounce_initial_fires_after_interval() {
        let debouncer = Debouncer::new(30);
        assert!(debouncer.should_fire().await);
    }

    #[tokio::test]
    async fn test_debounce_rapid_typing_resets_timer() {
        let debouncer = Debouncer::new(30);
        assert!(debouncer.should_fire().await);
        assert!(!debouncer.should_fire().await);
        debouncer.reset();
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(debouncer.should_fire().await);
    }

    #[tokio::test]
    async fn test_debounce_multiple_resets_within_window() {
        let debouncer = Debouncer::new(30);
        assert!(debouncer.should_fire().await);
        for _ in 0..5 {
            assert!(!debouncer.should_fire().await);
        }
        tokio::time::sleep(Duration::from_millis(31)).await;
        assert!(debouncer.should_fire().await);
    }
}
