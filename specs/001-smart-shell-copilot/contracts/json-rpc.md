# Contract: Shell Hook <-> Daemon JSON-RPC

## Transport

- Local Unix domain socket.
- Default planned socket path: `/tmp/smart-shell-copilot.sock`.
- Payload encoding: UTF-8 JSON lines or length-delimited JSON. The exact frame
  choice must be consistent across hook and daemon and benchmarked.
- The shell hook must use short non-blocking deadlines and silently return to
  native shell behavior on connection, write, read, parse, or timeout failure.

## Method: `completion.request`

### Request

```json
{
  "jsonrpc": "2.0",
  "id": "session-1:42",
  "method": "completion.request",
  "params": {
    "session_id": "session-1",
    "shell_kind": "zsh",
    "line": "git che",
    "cursor": 7,
    "cwd": "/Users/example/project",
    "deadline_ms": 25,
    "client_sent_at_ms": 123456789
  }
}
```

### Successful suggestion response

```json
{
  "jsonrpc": "2.0",
  "id": "session-1:42",
  "result": {
    "kind": "suggestion",
    "suffix": "ckout main",
    "replacement_start": 7,
    "valid_for_line_hash": "opaque-client-or-daemon-fingerprint",
    "source": "model",
    "daemon_latency_ms": 14
  }
}
```

### No suggestion response

```json
{
  "jsonrpc": "2.0",
  "id": "session-1:42",
  "result": {
    "kind": "none"
  }
}
```

### Error handling rule

JSON-RPC error objects are allowed for daemon diagnostics and tests, but the
shell hook must treat all error objects as `kind: none` and must not print the
error into the terminal.

## Method: `session.cancel`

Optional explicit cancellation when the shell hook knows a request is obsolete.
The daemon must also supersede older requests automatically when a newer
`completion.request` arrives for the same `session_id`.

### Request

```json
{
  "jsonrpc": "2.0",
  "id": "session-1:43",
  "method": "session.cancel",
  "params": {
    "session_id": "session-1",
    "request_id": "session-1:42"
  }
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "id": "session-1:43",
  "result": {
    "cancelled": true
  }
}
```

## Validation Requirements

- Oversized, malformed, missing-field, invalid-cursor, or stale requests must not
  crash the daemon.
- Daemon must never return multiline suggestions.
- Hook must verify response `id` and current line/cursor state before rendering.
- Hook must ignore any response that arrives after the deadline or after a newer
  request has been sent.
- Non-fatal failures produce no terminal-visible output.
