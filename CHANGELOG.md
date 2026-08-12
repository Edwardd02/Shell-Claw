# Changelog

## 0.0.2 - 2026-08-12

- Make completion, cancellation, SQLite retrieval, and llama.cpp inference run
  behind explicit protocol/trait boundaries without blocking Tokio I/O tasks.
- Add real per-session cancellation and concurrent responses over persistent
  Unix sockets; limit frames and set socket permissions to `0600`.
- Render Zsh ghost text through `POSTDISPLAY`, preserve existing widgets, use
  shell-safe JSON/UTF-8 transport, and disable interaction logging by default.
- Add idempotent Homebrew Zsh setup, daemon startup, a service definition, and
  resumable dual-source model downloads without a `bc` dependency.
- Fix SQLite command deduplication and FTS5 ranking. The measured 2,000-entry
  memory fast path is below 1 ms on the release test machine.
- Keep the Metal model warm for active sessions, then unload it after 30 idle
  seconds. The measured idle daemon is about 109 MB RSS and approximately 0%
  CPU; model inference temporarily uses more memory.
- Report end-to-end daemon latency separately from model time-to-first-token.
- Add repeatable release packaging and archive verification scripts.

## 0.0.1 - 2026-08-12

- Initial Apple Silicon alpha release.
