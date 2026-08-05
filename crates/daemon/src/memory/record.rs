use super::db::Database;
use super::{CommandMemoryInput, MemoryError, MemoryResult};

pub fn record_command(db: &Database, entry: CommandMemoryInput) -> MemoryResult<()> {
    if entry.cwd.is_empty() || entry.command.is_empty() {
        return Err(MemoryError::new("cwd and command must not be empty"));
    }
    if entry.command.contains('\0') {
        return Err(MemoryError::new("command must not contain NUL bytes"));
    }

    db.execute_insert(&entry.cwd, &entry.command, entry.used_at_unix)
}
