pub const CREATE_COMMAND_HISTORY: &str = "
CREATE TABLE IF NOT EXISTS command_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cwd TEXT NOT NULL,
    command TEXT NOT NULL,
    last_used_at INTEGER NOT NULL,
    use_count INTEGER DEFAULT 1
);
";

pub const CREATE_COMMAND_FTS: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS command_fts USING fts5(
    command,
    content='command_history',
    content_rowid='id'
);
";

pub const INSERT_COMMAND: &str = "
INSERT INTO command_history (cwd, command, last_used_at, use_count)
VALUES (?1, ?2, ?3, 1)
ON CONFLICT(id) DO UPDATE SET use_count = use_count + 1, last_used_at = ?3;
";

pub const INSERT_FTS: &str = "
INSERT INTO command_fts(rowid, command) VALUES (last_insert_rowid(), ?1);
";

pub const RETRIEVE_BY_QUERY: &str = "
SELECT
    ch.id,
    ch.command,
    ch.cwd,
    ch.last_used_at,
    ch.use_count,
    fts.rank AS bm25_score
FROM command_history ch
JOIN command_fts fts ON ch.id = fts.rowid
WHERE command_fts MATCH ?1
ORDER BY fts.rank
LIMIT ?2
";
