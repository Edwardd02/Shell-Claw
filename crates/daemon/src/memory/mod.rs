pub mod db;
pub mod record;
pub mod retrieve;
pub mod schema;
pub mod store;

pub type MemoryResult<T> = Result<T, MemoryError>;

#[derive(Debug, Clone)]
pub struct MemoryError {
    pub message: String,
}

impl MemoryError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MemoryError {}

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
    pub command: String,
    pub final_score: f64,
}

pub trait MemoryStore: Send + Sync {
    fn record_command(&self, entry: CommandMemoryInput) -> MemoryResult<()>;
    fn retrieve(&self, query: RetrievalQuery) -> MemoryResult<Vec<RetrievalCandidate>>;
}
