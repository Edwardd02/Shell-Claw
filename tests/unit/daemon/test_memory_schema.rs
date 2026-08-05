#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn create_schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cwd TEXT NOT NULL,
                command TEXT NOT NULL,
                last_used_at INTEGER NOT NULL,
                use_count INTEGER DEFAULT 1
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS command_fts USING fts5(
                command,
                content='command_history',
                content_rowid='id'
            );",
        )
        .unwrap();
    }

    #[test]
    fn test_schema_creation_succeeds() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(create_schema(&conn).is_ok(()));
    }

    #[test]
    fn test_command_history_table_exists() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn);
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='command_history'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fts_table_exists() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn);
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='command_fts'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn);
        create_schema(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM command_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
