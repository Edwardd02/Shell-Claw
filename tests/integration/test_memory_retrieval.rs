#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn setup_and_seed() -> Connection {
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

        let entries = vec![
            ("/home/user/project", "git status", 5000, 20),
            ("/home/user/project", "git checkout main", 4500, 15),
            ("/home/user/project", "cargo build --release", 4000, 30),
            ("/tmp", "ls -la", 3000, 50),
            ("/tmp", "rm -rf test", 2000, 3),
            ("/home/user/other", "npm install", 1000, 10),
        ];

        for (cwd, cmd, ts, count) in entries {
            conn.execute(
                "INSERT INTO command_history (cwd, command, last_used_at, use_count) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![cwd, cmd, ts, count],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO command_fts(rowid, command) VALUES (last_insert_rowid(), ?1)",
                rusqlite::params![cmd],
            )
            .unwrap();
        }

        conn
    }

    fn retrieval_write_ahead_log(conn: &Connection, prefix: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(
                "SELECT ch.command FROM command_history ch
                 JOIN command_fts fts ON ch.id = fts.rowid
                 WHERE command_fts MATCH ?1
                 ORDER BY ch.last_used_at DESC
                 LIMIT 5",
            )
            .unwrap();
        let results: Vec<String> = stmt
            .query_map(rusqlite::params![prefix], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        results
    }

    #[test]
    fn test_retrieval_returns_matching_commands() {
        let conn = setup_and_seed();
        let results = retrieval_write_ahead_log(&conn, "git");
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.contains("git")));
    }

    #[test]
    fn test_retrieval_orders_by_recent_first() {
        let conn = setup_and_seed();
        let results = retrieval_write_ahead_log(&conn, "git");
        assert_eq!(results[0], "git status");
    }

    #[test]
    fn test_retrieval_no_match_returns_empty() {
        let conn = setup_and_seed();
        let results = retrieval_write_ahead_log(&conn, "docker");
        assert!(results.is_empty());
    }

    #[test]
    fn test_retrieval_respects_limit() {
        let conn = setup_and_seed();
        let results = retrieval_write_ahead_log(&conn, "git");
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_seeded_data_present() {
        let conn = setup_and_seed();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM command_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 6);
    }
}
