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

pub const CREATE_COMMAND_LOOKUP_INDEX: &str = "
CREATE INDEX IF NOT EXISTS command_history_cwd_command_idx
ON command_history(cwd, command);
";

pub const INSERT_COMMAND: &str = "
INSERT INTO command_history (cwd, command, last_used_at, use_count)
VALUES (?1, ?2, ?3, 1);
";

pub const FIND_COMMAND: &str = "
SELECT id FROM command_history WHERE cwd = ?1 AND command = ?2 LIMIT 1;
";

pub const UPDATE_COMMAND: &str = "
UPDATE command_history
SET use_count = use_count + 1, last_used_at = ?2
WHERE id = ?1;
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
