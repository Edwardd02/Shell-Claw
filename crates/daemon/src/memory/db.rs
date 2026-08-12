use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use super::schema;
use super::{MemoryError, MemoryResult};

pub struct Database {
    conn: Mutex<Connection>,
}

pub type CommandRow = (i64, String, String, i64, i32, f64);

impl Database {
    pub fn open(path: &PathBuf) -> MemoryResult<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(path).map_err(|e| MemoryError::new(e.to_string()))?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| MemoryError::new(e.to_string()))?;

        conn.execute_batch("PRAGMA busy_timeout=2;")
            .map_err(|e| MemoryError::new(e.to_string()))?;

        conn.execute_batch(schema::CREATE_COMMAND_HISTORY)
            .and_then(|_| conn.execute_batch(schema::CREATE_COMMAND_LOOKUP_INDEX))
            .and_then(|_| conn.execute_batch(schema::CREATE_COMMAND_FTS))
            .map_err(|e| MemoryError::new(e.to_string()))?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn execute_insert(&self, cwd: &str, command: &str, used_at: i64) -> MemoryResult<()> {
        let conn = self.conn.lock().map_err(|e| MemoryError::new(e.to_string()))?;

        let tx = conn.unchecked_transaction().map_err(|e| MemoryError::new(e.to_string()))?;

        let existing_id = tx
            .query_row(schema::FIND_COMMAND, rusqlite::params![cwd, command], |row| {
                row.get::<_, i64>(0)
            })
            .optional()
            .map_err(|e| MemoryError::new(e.to_string()))?;

        if let Some(id) = existing_id {
            tx.execute(schema::UPDATE_COMMAND, rusqlite::params![id, used_at])
                .map_err(|e| MemoryError::new(e.to_string()))?;
        } else {
            tx.execute(schema::INSERT_COMMAND, rusqlite::params![cwd, command, used_at])
                .map_err(|e| MemoryError::new(e.to_string()))?;
            tx.execute(schema::INSERT_FTS, rusqlite::params![command])
                .map_err(|e| MemoryError::new(e.to_string()))?;
        }

        tx.commit().map_err(|e| MemoryError::new(e.to_string()))?;

        Ok(())
    }

    pub fn retrieve(&self, query: &str, limit: usize) -> MemoryResult<Vec<CommandRow>> {
        let conn = self.conn.lock().map_err(|e| MemoryError::new(e.to_string()))?;

        let mut stmt =
            conn.prepare(schema::RETRIEVE_BY_QUERY).map_err(|e| MemoryError::new(e.to_string()))?;

        let results = stmt
            .query_map(rusqlite::params![query, limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, f64>(5)?,
                ))
            })
            .map_err(|e| MemoryError::new(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }
}

use rusqlite::OptionalExtension;
