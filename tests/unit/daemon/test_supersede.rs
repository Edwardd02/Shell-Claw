#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct SupersedeTracker {
        sessions: Mutex<HashMap<String, String>>,
    }

    impl SupersedeTracker {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
            }
        }

        async fn register(&self, session_id: &str, request_id: &str) -> Option<String> {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session_id.to_string(), request_id.to_string())
        }

        async fn is_latest(&self, session_id: &str, request_id: &str) -> bool {
            let sessions = self.sessions.lock().await;
            sessions
                .get(session_id)
                .map(|current| current == request_id)
                .unwrap_or(false)
        }
    }

    #[tokio::test]
    async fn test_newer_request_supersedes_prior() {
        let tracker = SupersedeTracker::new();
        tracker.register("s1", "r1").await;
        tracker.register("s1", "r2").await;
        assert!(!tracker.is_latest("s1", "r1").await);
        assert!(tracker.is_latest("s1", "r2").await);
    }

    #[tokio::test]
    async fn test_different_sessions_independent() {
        let tracker = SupersedeTracker::new();
        tracker.register("s1", "r1").await;
        tracker.register("s2", "r2").await;
        assert!(tracker.is_latest("s1", "r1").await);
        assert!(tracker.is_latest("s2", "r2").await);
    }

    #[tokio::test]
    async fn test_multiple_supersedes_in_sequence() {
        let tracker = SupersedeTracker::new();
        tracker.register("s1", "r1").await;
        tracker.register("s1", "r2").await;
        tracker.register("s1", "r3").await;
        assert!(!tracker.is_latest("s1", "r1").await);
        assert!(!tracker.is_latest("s1", "r2").await);
        assert!(tracker.is_latest("s1", "r3").await);
    }

    #[tokio::test]
    async fn test_supersede_cleans_up_in_flight() {
        let in_flight: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let tracker = Arc::new(SupersedeTracker::new());
        tracker.register("s1", "r1").await;

        {
            let mut ifc = in_flight.lock().await;
            ifc.push("r1".to_string());
        }

        tracker.register("s1", "r2").await;

        {
            let ifc = in_flight.lock().await;
            assert!(!tracker.is_latest("s1", &ifc[0]).await);
        }
    }
}
