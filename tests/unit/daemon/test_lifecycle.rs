#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use std::time::Duration;
    use tokio::sync::Mutex;

    struct MockDaemon {
        running: Mutex<bool>,
    }

    impl MockDaemon {
        fn new() -> Self {
            Self {
                running: Mutex::new(false),
            }
        }

        async fn start(&self) {
            let mut r = self.running.lock().await;
            *r = true;
        }

        async fn stop(&self) {
            let mut r = self.running.lock().await;
            *r = false;
        }

        async fn is_running(&self) -> bool {
            *self.running.lock().await
        }
    }

    #[tokio::test]
    async fn test_daemon_startup_sets_running() {
        let daemon = MockDaemon::new();
        assert!(!daemon.is_running().await);
        daemon.start().await;
        assert!(daemon.is_running().await);
    }

    #[tokio::test]
    async fn test_daemon_shutdown_clears_running() {
        let daemon = MockDaemon::new();
        daemon.start().await;
        assert!(daemon.is_running().await);
        daemon.stop().await;
        assert!(!daemon.is_running().await);
    }

    #[tokio::test]
    async fn test_daemon_restart_cycle() {
        let daemon = MockDaemon::new();
        daemon.start().await;
        daemon.stop().await;
        daemon.start().await;
        assert!(daemon.is_running().await);
    }

    #[tokio::test]
    async fn test_graceful_shutdown_preserves_state() {
        let daemon = MockDaemon::new();
        daemon.start().await;
        let was_running = daemon.is_running().await;
        daemon.stop().await;
        let is_running = daemon.is_running().await;
        assert!(was_running);
        assert!(!is_running);
    }
}
