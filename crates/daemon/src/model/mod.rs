pub mod adapter;
pub mod grammar;
pub mod validate;

use tokio_util::sync::CancellationToken;

pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct ModelError {
    pub message: String,
}

impl ModelError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ModelError {}

#[derive(Debug, Clone)]
pub struct ModelContext {
    pub line_prefix: String,
}

#[derive(Debug, Clone)]
pub struct ModelOutput {
    pub suffix: String,
    pub ttft_ms: u64,
    /// 建议真实来源:记忆快路径标 `Memory`;模型推理标 `Model`。
    pub source: protocol::SuggestionSource,
}

pub trait CompletionModel: Send + Sync {
    fn complete_suffix(
        &self,
        context: ModelContext,
        cancel: CancellationToken,
    ) -> ModelResult<ModelOutput>;

    /// Release heavyweight runtime state after an idle period. Implementations
    /// that do not retain native resources can keep the default no-op.
    fn unload_if_idle(&self, _max_idle: std::time::Duration) {}
}
