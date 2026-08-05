# Contract: Memory Store Trait

The daemon must access command memory through an explicit trait boundary.

## Trait Shape

```rust
pub trait MemoryStore: Send + Sync {
    fn record_command(&self, entry: CommandMemoryInput) -> MemoryResult<()>;
    fn retrieve(&self, query: RetrievalQuery) -> MemoryResult<Vec<RetrievalCandidate>>;
}
```

Exact async/sync shape may be adjusted during implementation, but the daemon
scheduler must not depend directly on SQLite internals.

## `CommandMemoryInput`

- `cwd: String`
- `command: String`
- `used_at_unix: i64`

## `RetrievalQuery`

- `cwd: String`
- `line_prefix: String`
- `limit: usize`
- `deadline_ms: u64`

## `RetrievalCandidate`

- `entry_id: i64`
- `command: String`
- `cwd: String`
- `bm25_score: f64`
- `cwd_score: f64`
- `frequency_score: f64`
- `recency_score: f64`
- `final_score: f64`

## Failure Rules

- SQLite lock, corruption, missing table, or timeout maps to an empty candidate
  list for completion purposes.
- Memory failures are diagnosable through daemon diagnostics, not terminal output.
- Retrieval must be deadline-aware and benchmarked against <=3ms.
