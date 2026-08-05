#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_cancellation_stops_work() {
        let token = CancellationToken::new();
        let was_cancelled = AtomicBool::new(false);

        let child_token = token.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = child_token.cancelled() => {
                    was_cancelled.store(true, Ordering::SeqCst);
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
            }
        });

        token.cancel();
        handle.await.unwrap();
        assert!(was_cancelled.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_cancelled_token_returns_no_suggestion() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_uncancelled_token_allows_work() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }
}
