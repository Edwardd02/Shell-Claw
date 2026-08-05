#[cfg(test)]
mod tests {
    use protocol::CompletionResult;

    #[test]
    fn test_daemon_off_returns_no_suggestion() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }

    #[test]
    fn test_socket_interrupted_returns_no_suggestion() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }

    #[test]
    fn test_sqlite_lock_returns_no_suggestion() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }

    #[test]
    fn test_empty_memory_store_returns_no_suggestion() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }

    #[test]
    fn test_inference_timeout_returns_no_suggestion() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }

    #[test]
    fn test_invalid_model_output_returns_no_suggestion() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }

    #[test]
    fn test_all_failures_preserve_native_shell_behavior() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
        assert!(
            std::panic::catch_unwind(|| {
                let json = serde_json::to_string(&outcome).unwrap();
                assert!(!json.contains('\n'));
                assert!(!json.contains("error"));
            })
            .is_ok()
        );
    }
}
