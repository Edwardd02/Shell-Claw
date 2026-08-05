# Data Model: Smart Shell Copilot

## Entity: Shell Session

Represents one active interactive terminal session with completion integration.

**Fields**:

- `session_id`: Stable opaque identifier for the active shell session.
- `shell_kind`: `zsh` or `bash`.
- `pid`: Shell process identifier where available.
- `cwd`: Current working directory observed at request time.
- `line`: Current command line text.
- `cursor`: Cursor byte/character offset in `line`.
- `active_request_id`: Latest completion request identity for stale-response
  prevention.
- `visible_suggestion_id`: Current rendered suggestion identity, if any.

**Validation rules**:

- `session_id` must be present in every request.
- `cwd` must be absolute or the request is treated as no-suggestion.
- Responses must match `active_request_id` and current line/cursor state before
  rendering.

**State transitions**:

```text
No Hook -> Hook Loaded -> Service Available -> Completion Active
                              │
                              └── Service Unavailable -> Native Shell Fallback
```

## Entity: Completion Request

Transient request generated from shell state and sent to the daemon.

**Fields**:

- `jsonrpc`: Protocol marker, `2.0`.
- `id`: Monotonic or UUID request identifier scoped to a session.
- `method`: `completion.request`.
- `session_id`: Origin shell session.
- `line`: Full command line text at request time.
- `cursor`: Cursor offset at request time.
- `cwd`: Working directory at request time.
- `shell_kind`: `zsh` or `bash`.
- `deadline_ms`: Maximum response budget from hook perspective.
- `client_sent_at_ms`: Client monotonic timestamp for benchmarks.

**Validation rules**:

- `line` length is bounded by the daemon config; oversized requests return
  no-suggestion.
- `cursor` must be within `line` bounds.
- Empty or whitespace-only lines may return no-suggestion.
- A newer request for the same session supersedes older in-flight work.

**State transitions**:

```text
Created -> Sent -> Accepted by Daemon -> Running
                                     ├── Superseded
                                     ├── Timed Out
                                     ├── Failed Silently
                                     └── Completed
```

## Entity: Completion Suggestion

Same-line command suffix eligible for Ghost Text rendering.

**Fields**:

- `request_id`: Request this suggestion answers.
- `suffix`: Suggested command-line suffix only.
- `replacement_start`: Cursor offset where suffix applies; initially equal to
  request cursor.
- `valid_for_line_hash`: Hash/fingerprint of request line state.
- `source`: `model`, `memory`, or `none` for diagnostics/benchmarks.
- `latency_ms`: End-to-end or daemon-side latency measurement.

**Validation rules**:

- `suffix` must be non-empty to render.
- `suffix` must not contain `\r`, `\n`, NUL, Markdown fences, or explanatory
  prose markers.
- `suffix` must not duplicate the already typed suffix at the cursor.
- `request_id` and line fingerprint must match the current shell state before
  rendering.
- Invalid suggestions degrade to no-suggestion without terminal output.

**State transitions**:

```text
Candidate -> Validated -> Rendered -> Accepted
                         ├── Cleared by typing
                         ├── Superseded by newer suggestion
                         └── Rejected as stale/invalid
```

## Entity: Command Memory Entry

Historical command used by local retrieval.

**Fields**:

- `id`: Integer primary key.
- `cwd`: Execution path.
- `command`: Full command line.
- `last_used_at`: Unix timestamp.
- `use_count`: Positive integer frequency count.

**SQLite schema baseline**:

```sql
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
```

**Validation rules**:

- `cwd` and `command` must be non-empty.
- `use_count` must be >=1.
- Commands containing NUL bytes are rejected.
- Memory updates must be best-effort and must never block shell input.

## Entity: Retrieval Candidate

Ranked local command memory result.

**Fields**:

- `entry_id`: Command memory entry id.
- `command`: Full command text.
- `cwd`: Stored command path.
- `bm25_score`: FTS text score.
- `cwd_match`: Boolean or weighted path relevance score.
- `frequency_score`: Function of `use_count`.
- `recency_score`: Time-decay score.
- `final_score`: Weighted combined score.

**Ranking baseline**:

```text
Score = w1 * BM25(query)
      + w2 * cwd_match
      + w3 * log(1 + use_count) * exp(-lambda * delta_time)
```

**Validation rules**:

- Retrieval must complete within 3ms in release benchmark conditions.
- Retrieval failure returns an empty candidate list rather than an error to the
  shell.
- Ranking weights must be configurable or centrally defined for benchmark
  tuning.

## Entity: Model Context

Input package prepared for constrained model inference.

**Fields**:

- `system_prompt_id`: Identifier for pre-warmed prompt constraints.
- `line_prefix`: Command text before cursor.
- `cwd`: Current working directory.
- `retrieval_candidates`: Bounded list of relevant historical commands.
- `grammar_id`: `single_line` grammar identifier.
- `deadline_ms`: Inference deadline.

**Validation rules**:

- Context must be bounded by configured token/byte limits.
- Grammar must constrain output to a single line with no NUL.
- Missing retrieval candidates are valid.
- Expired/cancelled contexts must stop inference and return no-suggestion.

## Entity: Service Registration

Platform service state created by package install.

**Fields**:

- `platform`: `macos` or `linux`.
- `manager`: `launchd` or `systemd-user`.
- `service_name`: Stable service identifier.
- `daemon_path`: Installed daemon path.
- `socket_path`: Configured Unix socket path.
- `enabled_on_boot`: Boolean.
- `running`: Boolean observed during validation.

**Validation rules**:

- Install must register and start the service without manual user commands.
- Uninstall must stop and unregister the service.
- Service failures must not produce terminal hook errors.

## Entity: Shell Hook Registration

Package-managed shell integration loaded by new shell sessions.

**Fields**:

- `shell_kind`: `zsh` or `bash`.
- `load_point`: Package-managed global or Homebrew-managed load location.
- `hook_path`: Installed hook file.
- `probe_timeout_ms`: Socket probe timeout.
- `enabled`: Whether hook activation is expected.

**Validation rules**:

- Hook loading must not require manual `.zshrc` or shell profile edits.
- Hook must silently no-op if daemon/socket is unavailable.
- Uninstall must remove hook load point and hook file residue.
- Hook must preserve unrelated native shortcuts.
