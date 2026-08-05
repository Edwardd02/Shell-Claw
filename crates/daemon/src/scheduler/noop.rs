use protocol::CompletionResult;

pub struct NoopResponse;

impl NoopResponse {
    pub fn suggestion() -> CompletionResult {
        CompletionResult::no_suggestion()
    }

    pub fn response_json(request_id: &str) -> String {
        let resp = protocol::CompletionResponse {
            jsonrpc: "2.0".to_string(),
            id: request_id.to_string(),
            result: CompletionResult::no_suggestion(),
        };
        serde_json::to_string(&resp).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":"","result":{"kind":"none"}}"#.to_string()
        })
    }
}
