use std::sync::Arc;

use super::db::Database;
use super::{CommandMemoryInput, MemoryResult, MemoryStore, RetrievalCandidate, RetrievalQuery};

pub struct SqliteMemoryStore {
    db: Arc<Database>,
}

impl SqliteMemoryStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl MemoryStore for SqliteMemoryStore {
    fn record_command(&self, entry: CommandMemoryInput) -> MemoryResult<()> {
        super::record::record_command(&self.db, entry)
    }

    fn retrieve(&self, query: RetrievalQuery) -> MemoryResult<Vec<RetrievalCandidate>> {
        super::retrieve::retrieve(&self.db, query)
    }
}
