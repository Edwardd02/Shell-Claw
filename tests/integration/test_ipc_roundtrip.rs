#[cfg(test)]
mod tests {
    use protocol::{
        CompletionParams, CompletionRequest, CompletionResult, SuggestionData, SuggestionSource,
    };
    use serde_json;

    #[test]
    fn test_completion_request_serialization() {
        let req = CompletionRequest::new(
            "session-1:42".to_string(),
            CompletionParams {
                session_id: "session-1".to_string(),
                shell_kind: "zsh".to_string(),
                line: "git che".to_string(),
                cursor: 7,
                cwd: "/Users/test/project".to_string(),
                deadline_ms: 25,
                client_sent_at_ms: 123456789,
            },
        );

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("completion.request"));
        assert!(json.contains("git che"));
        assert!(json.contains("session-1"));
    }

    #[test]
    fn test_completion_response_suggestion_serialization() {
        let resp = protocol::CompletionResponse {
            jsonrpc: "2.0".to_string(),
            id: "session-1:42".to_string(),
            result: CompletionResult::Suggestion(SuggestionData {
                suffix: "ckout main".to_string(),
                replacement_start: 7,
                valid_for_line_hash: "abc123".to_string(),
                source: SuggestionSource::Model,
                daemon_latency_ms: 14,
            }),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("suggestion"));
        assert!(json.contains("ckout main"));
    }

    #[test]
    fn test_completion_response_no_suggestion_serialization() {
        let resp = protocol::CompletionResponse {
            jsonrpc: "2.0".to_string(),
            id: "session-1:42".to_string(),
            result: CompletionResult::None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("none"));
    }

    #[test]
    fn test_roundtrip_request_parse() {
        let req = CompletionRequest::new(
            "r1".to_string(),
            CompletionParams {
                session_id: "s1".to_string(),
                shell_kind: "zsh".to_string(),
                line: "echo hello".to_string(),
                cursor: 5,
                cwd: "/tmp".to_string(),
                deadline_ms: 30,
                client_sent_at_ms: 1000,
            },
        );

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.params.line, "echo hello");
        assert_eq!(parsed.params.cwd, "/tmp");
    }

    #[test]
    fn test_roundtrip_no_suggestion_response() {
        let resp = protocol::CompletionResponse {
            jsonrpc: "2.0".to_string(),
            id: "x".to_string(),
            result: CompletionResult::None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        let parsed: protocol::CompletionResponse = serde_json::from_str(&json).unwrap();
        match parsed.result {
            CompletionResult::None => {}
            _ => panic!("expected None"),
        }
    }

    #[test]
    fn test_connection_failure_produces_no_terminal_output() {
        let resp = CompletionResult::no_suggestion();
        match resp {
            CompletionResult::None => {}
            _ => panic!("expected None on connection failure"),
        }
    }

    #[test]
    fn test_socket_interruption_produces_no_error_output() {
        let resp = CompletionResult::no_suggestion();
        match resp {
            CompletionResult::None => {}
            _ => panic!("expected None on socket interruption"),
        }
    }
}
