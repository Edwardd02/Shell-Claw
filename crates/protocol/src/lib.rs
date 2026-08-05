use serde::{Deserialize, Serialize};

pub const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: CompletionParams,
}

impl CompletionRequest {
    pub fn new(id: String, params: CompletionParams) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: "completion.request".to_string(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    pub session_id: String,
    pub shell_kind: String,
    pub line: String,
    pub cursor: usize,
    pub cwd: String,
    pub deadline_ms: u64,
    pub client_sent_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub jsonrpc: String,
    pub id: String,
    pub result: CompletionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CompletionResult {
    #[serde(rename = "suggestion")]
    Suggestion(SuggestionData),
    #[serde(rename = "none")]
    None,
}

impl CompletionResult {
    pub fn no_suggestion() -> Self {
        Self::None
    }

    pub fn suggestion(
        suffix: String,
        replacement_start: usize,
        valid_for_line_hash: String,
        source: SuggestionSource,
        daemon_latency_ms: u64,
    ) -> Self {
        Self::Suggestion(SuggestionData {
            suffix,
            replacement_start,
            valid_for_line_hash,
            source,
            daemon_latency_ms,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionData {
    pub suffix: String,
    pub replacement_start: usize,
    pub valid_for_line_hash: String,
    pub source: SuggestionSource,
    pub daemon_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionSource {
    Model,
    Memory,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: CancelParams,
}

impl CancelRequest {
    pub fn new(id: String, params: CancelParams) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: "session.cancel".to_string(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelParams {
    pub session_id: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    pub jsonrpc: String,
    pub id: String,
    pub result: CancelResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResult {
    pub cancelled: bool,
}

/// Generic JSON-RPC error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    pub id: Option<String>,
    pub error: ErrorData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecordRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: MemoryRecordParams,
}

impl MemoryRecordRequest {
    pub fn new(id: String, params: MemoryRecordParams) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: "memory.record".to_string(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecordParams {
    pub session_id: String,
    pub cwd: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(CompletionRequest),
    Cancel(CancelRequest),
    Record(MemoryRecordRequest),
}

impl JsonRpcMessage {
    pub fn method(&self) -> &str {
        match self {
            Self::Request(_) => "completion.request",
            Self::Cancel(_) => "session.cancel",
            Self::Record(_) => "memory.record",
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Request(r) => &r.id,
            Self::Cancel(r) => &r.id,
            Self::Record(r) => &r.id,
        }
    }
}
