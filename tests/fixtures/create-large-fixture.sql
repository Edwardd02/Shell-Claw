#!/usr/bin/env bash
# Large command-history fixture for retrieval benchmarking.
# Run: sqlite3 tests/fixtures/command-history-large.sqlite < tests/fixtures/create-large-fixture.sql
set -euo pipefail

echo "=== Creating large command-history fixture ==="

FIXTURE="tests/fixtures/command-history-large.sqlite"

sqlite3 "$FIXTURE" <<'SQL'
CREATE TABLE IF NOT EXISTS command_history (
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
);
SQL

DIRS=("/home/user/project" "/home/user/other" "/tmp" "/var/log" "/etc")
COMMANDS=("git status" "git diff" "git add ." "git commit -m" "git push" "cargo build" "cargo test" "cargo run" "npm install" "npm run dev" "npm test" "ls -la" "cd .." "find . -name" "grep -r" "docker compose up" "docker ps" "ssh" "curl" "wget" "make" "pip install" "python -m" "echo" "cat" "vim" "mkdir" "rm -rf" "chmod +x" "tar xzf")

COUNT=0
while [ $COUNT -lt 1000 ]; do
    DIR="${DIRS[$((RANDOM % 5))]}"
    CMD="${COMMANDS[$((RANDOM % 30))]}"
    TS=$(( (RANDOM % 86400) + 1710000000 ))
    USE_COUNT=$(( (RANDOM % 50) + 1 ))

    sqlite3 "$FIXTURE" "INSERT INTO command_history (cwd, command, last_used_at, use_count) VALUES ('$DIR', '$CMD', $TS, $USE_COUNT);"
    ROWID=$(sqlite3 "$FIXTURE" "SELECT last_insert_rowid();")
    sqlite3 "$FIXTURE" "INSERT INTO command_fts(rowid, command) VALUES ($ROWID, '$CMD');"

    COUNT=$((COUNT + 1))
done

echo "Fixture created: $FIXTURE with 1000 entries"
