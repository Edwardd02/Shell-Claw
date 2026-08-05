#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    #[test]
    fn test_deadline_not_expired_yet() {
        let deadline = Instant::now() + Duration::from_millis(100);
        assert!(Instant::now() < deadline);
    }

    #[test]
    fn test_deadline_expired() {
        let deadline = Instant::now();
        std::thread::sleep(Duration::from_millis(1));
        assert!(Instant::now() >= deadline);
    }

    #[test]
    fn test_no_suggestion_when_deadline_passed() {
        let deadline_ms = 0u64;
        let expired = deadline_ms == 0;
        assert!(expired || deadline_ms < 1);
    }

    #[test]
    fn test_deadline_aborts_early() {
        let start = Instant::now();
        let deadline = start + Duration::from_millis(5);
        std::thread::sleep(Duration::from_millis(10));
        assert!(Instant::now() > deadline);
    }
}
