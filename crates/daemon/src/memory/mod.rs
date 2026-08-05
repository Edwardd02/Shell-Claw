pub mod db;
pub mod record;
pub mod retrieve;
pub mod schema;
pub mod store;

use std::sync::Arc;

pub type MemoryResult<T> = Result<T, MemoryError>;

#[derive(Debug, Clone)]
pub struct MemoryError {
    pub message: String,
}

impl MemoryError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandMemoryInput {
    pub cwd: String,
    pub command: String,
    pub used_at_unix: i64,
}

#[derive(Debug, Clone)]
pub struct RetrievalQuery {
    pub cwd: String,
    pub line_prefix: String,
    pub limit: usize,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RetrievalCandidate {
    pub entry_id: i64,
    pub command: String,
    pub cwd: String,
    pub bm25_score: f64,
    pub cwd_score: f64,
    pub frequency_score: f64,
    pub recency_score: f64,
    pub final_score: f64,
}

pub trait MemoryStore: Send + Sync {
    fn record_command(&self, entry: CommandMemoryInput) -> MemoryResult<()>;
    fn retrieve(&self, query: RetrievalQuery) -> MemoryResult<Vec<RetrievalCandidate>>;
}

pub type SharedMemoryStore = Arc<dyn MemoryStore>;
