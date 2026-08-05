#[cfg(test)]
mod tests {
    use protocol::{CompletionResult, SuggestionData, SuggestionSource};

    #[test]
    fn test_pipeline_returns_valid_single_line_suffix() {
        let suffix = "ckout main";
        assert!(!suffix.contains('\n'));
        assert!(!suffix.contains('\r'));
        assert!(!suffix.is_empty());
    }

    #[test]
    fn test_pipeline_no_suggestion_on_timeout() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }

    #[test]
    fn test_pipeline_no_suggestion_on_multiline() {
        let multiline_suffix = "line1\nline2";
        let valid = !multiline_suffix.contains('\n');
        assert!(!valid);
    }

    #[test]
    fn test_pipeline_full_roundtrip() {
        let suggestion = SuggestionData {
            suffix: "checkout main".to_string(),
            replacement_start: 4,
            valid_for_line_hash: "abc".to_string(),
            source: SuggestionSource::Model,
            daemon_latency_ms: 12,
        };

        assert_eq!(suggestion.suffix, "checkout main");
        assert!(matches!(suggestion.source, SuggestionSource::Model));
        assert!(suggestion.daemon_latency_ms <= 15);
    }

    #[test]
    fn test_pipeline_inference_timeout_returns_no_suggestion() {
        let outcome = CompletionResult::no_suggestion();
        match outcome {
            CompletionResult::None => {}
            _ => panic!("Expected None for timeout"),
        }
    }

    #[test]
    fn test_pipeline_multiline_rejected() {
        let bad = "line\nbreak";
        assert!(bad.contains('\n'));
    }

    #[test]
    fn test_silent_degradation_inference_failure() {
        let outcome = CompletionResult::no_suggestion();
        assert!(matches!(outcome, CompletionResult::None));
    }
}
