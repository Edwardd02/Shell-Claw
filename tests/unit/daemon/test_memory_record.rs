#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cwd TEXT NOT NULL,
                command TEXT NOT NULL,
                last_used_at INTEGER NOT NULL,
                use_count INTEGER DEFAULT 1
            );
            CREATE VIRTUAL TABLE command_fts USING fts5(
                command,
                content='command_history',
                content_rowid='id'
            );",
        )
        .unwrap();
        conn
    }

    fn record_command(conn: &Connection, cwd: &str, command: &str, used_at: i64) {
        conn.execute(
            "INSERT INTO command_history (cwd, command, last_used_at, use_count)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(id) DO UPDATE SET use_count = use_count + 1, last_used_at = ?3",
            rusqlite::params![cwd, command, used_at],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO command_fts(rowid, command) VALUES (last_insert_rowid(), ?1)",
            rusqlite::params![command],
        )
        .unwrap();
    }

    #[test]
    fn test_record_command_inserts_entry() {
        let conn = setup_db();
        record_command(&conn, "/tmp", "ls -la", 1000);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM command_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_record_command_stores_cwd() {
        let conn = setup_db();
        record_command(&conn, "/home/user/project", "cargo build", 2000);
        let cwd: String = conn
            .query_row(
                "SELECT cwd FROM command_history WHERE command = 'cargo build'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cwd, "/home/user/project");
    }

    #[test]
    fn test_record_command_upserts_use_count() {
        let conn = setup_db();
        record_command(&conn, "/tmp", "git status", 1000);
        record_command(&conn, "/tmp", "git status", 2000);
        let count: i64 = conn
            .query_row(
                "SELECT use_count FROM command_history WHERE command = 'git status'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }
}
