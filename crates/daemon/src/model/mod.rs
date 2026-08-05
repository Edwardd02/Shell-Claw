pub mod adapter;
pub mod context;
pub mod grammar;
pub mod safe_wrapper;
pub mod validate;
pub mod warmup;

use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::memory::RetrievalCandidate;

pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct ModelError {
    pub message: String,
}

impl ModelError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GrammarId {
    SingleLine,
}

#[derive(Debug, Clone)]
pub struct ModelContext {
    pub line_prefix: String,
    pub cwd: String,
    pub retrieval_candidates: Vec<RetrievalCandidate>,
    pub grammar_id: GrammarId,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ModelOutput {
    pub suffix: String,
    pub ttft_ms: u64,
    pub model_id: String,
}

pub trait CompletionModel: Send + Sync {
    fn complete_suffix(
        &self,
        context: ModelContext,
        cancel: CancellationToken,
    ) -> ModelResult<ModelOutput>;
}

pub type SharedCompletionModel = Arc<dyn CompletionModel>;
