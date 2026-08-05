//! # Protocol crate
//!
//! 在 **Shell 钩子**（Zsh/Bash）和 **Rust 守护进程** 之间共享的 JSON-RPC 2.0
//! 传输类型。
//!
//! 本 crate 刻意保持极少的依赖：它只通过 `serde` 定义可序列化的数据结构，
//! **不含**任何关于守护进程如何存储记忆或运行模型的具体逻辑。把线缆协议
//! 收敛到单一位置，可以让两个进程各自独立演进内部实现，而不会破坏彼此的
//! 消息格式。
//!
//! ## 传输方式
//! 消息通过本机 **Unix domain socket** 传输，采用换行分隔的 JSON 文本。
//! 每个请求体对应 [`JsonRpcMessage`] 枚举中的一个变体；守护进程以对应的
//! 响应结构体回复。

use serde::{Deserialize, Serialize};

/// 每个 JSON-RPC 2.0 信封中固定的版本标记。
pub const JSONRPC_VERSION: &str = "2.0";

// ===========================================================================
// completion.request —— 向守护进程请求一个补全后缀
// ===========================================================================

/// Shell 钩子发出的 *completion.request* 信封。
///
/// 它包装了每个请求的参数以及传输层字段（`jsonrpc`、`id`、`method`）。
/// 其中 `id` 是请求标识，用于把响应与请求对应起来，从而忽略过期的回复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// 恒为 `"2.0"`。
    pub jsonrpc: String,
    /// 请求标识，例如 `"session-1:42"`。在同一会话内唯一，用于过期响应检测。
    pub id: String,
    /// RPC 方法名；对于该信封固定为 `"completion.request"`。
    pub method: String,
    /// 描述请求时 shell 状态的数据体。
    pub params: CompletionParams,
}

impl CompletionRequest {
    /// 便捷构造函数：自动填充协议样板并固定方法名。
    pub fn new(id: String, params: CompletionParams) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: "completion.request".to_string(),
            params,
        }
    }
}

/// 补全请求的主体：交互式 shell 状态的快照，守护进程据此生成建议。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    /// 所属 shell 会话的不透明稳定标识。用于使同一终端中更早的进行中请求
    /// 失效，并用于请求计数的作用域。
    pub session_id: String,
    /// shell 类型，目前为 `"zsh"` 或 `"bash"`。
    pub shell_kind: String,
    /// 用户到目前为止输入完整的命令行文本。
    pub line: String,
    /// 光标在 `line` 内的字节/字符位置。
    pub cursor: usize,
    /// 当前工作目录的绝对路径。守护进程用它做路径相关性排序。
    pub cwd: String,
    /// 从钩子视角看，允许的最大响应预算（毫秒）。超时后守护进程降级为
    /// “none”响应。
    pub deadline_ms: u64,
    /// 钩子的单调时钟时间戳（毫秒）——仅用于延迟测量。
    pub client_sent_at_ms: u64,
}

/// 守护进程返回的 *completion* 回复。
///
/// `id` 回显匹配请求的 id，以便钩子丢弃过期的回复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// 恒为 `"2.0"`。
    pub jsonrpc: String,
    /// 回显请求的 `id`。
    pub id: String,
    /// 要么是具体的建议，要么是“none”。
    pub result: CompletionResult,
}

/// 补全请求的判别结果。
///
/// 通过 `kind` 标签序列化，让 shell 钩子可以低成本地分支处理：
///     - `{"kind":"suggestion", ...}` —— 可渲染的后缀
///     - `{"kind":"none"}`           —— 无内容可展示（回退到原生行为）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CompletionResult {
    /// 一个可渲染为 Ghost Text 的具体单行后缀。
    #[serde(rename = "suggestion")]
    Suggestion(SuggestionData),
    /// 无建议。既用于空结果，也用于所有静默降级路径（socket 错误、SQLite
    /// 锁、推理超时等等）。
    #[serde(rename = "none")]
    None,
}

impl CompletionResult {
    /// 构造“无建议”变体。守护进程在出现任何可恢复的失败或无匹配时返回它。
    pub fn no_suggestion() -> Self {
        Self::None
    }

    /// 构造建议变体。各字段含义见 [`SuggestionData`]。
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

/// 包含在 [`CompletionResult::Suggestion`] 内的具体建议数据体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionData {
    /// 与当前命令同行的后缀，在光标后渲染（绝不包含换行、回车符或 NUL）。
    pub suffix: String,
    /// 后缀开始处的光标偏移；通常等于请求的光标。保留给钩子用于正确定位。
    pub replacement_start: usize,
    /// 请求行的指纹（哈希）。钩子在渲染前会将其与当前行比对，从而绝不会
    /// 显示过期建议。
    pub valid_for_line_hash: String,
    /// 建议的来源（模型还是记忆）——用于诊断和基准测试。
    pub source: SuggestionSource,
    /// 守护进程内的延迟（毫秒），从接收请求那一刻起测量。
    pub daemon_latency_ms: u64,
}

/// 建议的来源，主要用于诊断和基准测试。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionSource {
    /// 由本地模型推理（llama.cpp）产生。
    Model,
    /// 直接由本地命令记忆检索得到。
    Memory,
    /// 无来源（通常在具体建议中不应出现）。
    None,
}

// ===========================================================================
// session.cancel —— 可选地通知守护进程某个请求已经过期
// ===========================================================================

/// 外发的 *session.cancel* 信封。
///
/// 这只是一个提示：守护进程一旦收到同一 `session_id` 的更新的
/// `completion.request`，也会自动使旧的请求失效。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    /// 恒为 `"2.0"`。
    pub jsonrpc: String,
    /// 取消事务的标识。
    pub id: String,
    /// 恒为 `"session.cancel"`。
    pub method: String,
    /// 要取消哪个请求。
    pub params: CancelParams,
}

impl CancelRequest {
    /// 构造函数：固定 `jsonrpc` 和 `method`。
    pub fn new(id: String, params: CancelParams) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: "session.cancel".to_string(),
            params,
        }
    }
}

/// 在某个会话内要取消的请求的标识。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelParams {
    /// 拥有该请求的会话。
    pub session_id: String,
    /// 要取消的请求 id（例如 `"session-1:42"`）。
    pub request_id: String,
}

/// 对取消请求的入站回复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    /// 恒为 `"2.0"`。
    pub jsonrpc: String,
    /// 回显取消请求的 id。
    pub id: String,
    /// 取消的结果。
    pub result: CancelResult,
}

/// 守护进程是否接收了取消。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResult {
    /// 如果请求已被取消（或本来就已过期）则为 `true`。
    pub cancelled: bool,
}

// ===========================================================================
// error response —— 仅用于诊断，绝不会渲染到终端
// ===========================================================================

/// 通用的 JSON-RPC 错误响应。
///
/// **契约：** shell 钩子把任何错误对象都当作 `kind: none`。错误主体只供
/// 守护进程诊断和测试使用；它**绝不**被打印进用户的终端。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// 恒为 `"2.0"`。
    pub jsonrpc: String,
    /// 回显请求的 id；对于 id 未知的解析错误为 `None`。
    pub id: Option<String>,
    /// 结构化的错误信息。
    pub error: ErrorData,
}

/// 结构化的 JSON-RPC 错误主体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    /// 数值错误码（例如 `-32700` 解析错误、`-32600` 非法请求）。
    pub code: i32,
    /// 简短的人类可读错误消息。
    pub message: String,
}

// ===========================================================================
// memory.record —— 把一条已执行的命令持久化到本地记忆
// ===========================================================================

/// 外发的 *memory.record* 信封。
///
/// 由 shell 钩子在命令执行之后发送，以便守护进程把命令存入命令记忆，
/// 从而改进以后的补全。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecordRequest {
    /// 恒为 `"2.0"`。
    pub jsonrpc: String,
    /// 记录事务的标识。
    pub id: String,
    /// 恒为 `"memory.record"`。
    pub method: String,
    /// 要存储的命令与上下文。
    pub params: MemoryRecordParams,
}

impl MemoryRecordRequest {
    /// 构造函数：固定 `jsonrpc` 和 `method`。
    pub fn new(id: String, params: MemoryRecordParams) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: "memory.record".to_string(),
            params,
        }
    }
}

/// 要持久化的命令行执行记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecordParams {
    /// 执行该命令的会话（审计/诊断）。
    pub session_id: String,
    /// 命令运行的目录。
    pub cwd: String,
    /// 执行时完整的命令行。
    pub command: String,
}

// ===========================================================================
// JsonRpcMessage —— 非标签化（untagged）的分发联合
// ===========================================================================

/// 所有受支持的入站请求类型的非标签化联合。
///
/// `#[serde(untagged)]` 表示 serde 会按顺序尝试每个变体，选取第一个能成功
/// 反序列化的（依据有区分度的字段来识别）：
///   - `method == "completion.request"` **且** 参数包含 `cursor`/`cwd` → Request
///   - `method == "session.cancel"` **且** 参数包含 `request_id` → Cancel
///   - `method == "memory.record"` **且** 参数包含 `command`/`cwd` → Record
///
/// 守护进程的 IPC handler 基于该枚举进行匹配，从而把原始帧路由出去。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// 补全请求（核心交互路径）。
    Request(CompletionRequest),
    /// 取消请求（可选、尽力而为）。
    Cancel(CancelRequest),
    /// 命令记录请求（记忆持久化）。
    Record(MemoryRecordRequest),
}

impl JsonRpcMessage {
    /// 返回被封装的请求的 RPC 方法名，而不重新序列化。用于日志和路由。
    pub fn method(&self) -> &str {
        match self {
            Self::Request(_) => "completion.request",
            Self::Cancel(_) => "session.cancel",
            Self::Record(_) => "memory.record",
        }
    }

    /// 返回被封装的请求的 id，用于响应关联。
    pub fn id(&self) -> &str {
        match self {
            Self::Request(r) => &r.id,
            Self::Cancel(r) => &r.id,
            Self::Record(r) => &r.id,
        }
    }
}
