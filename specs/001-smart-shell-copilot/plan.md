# Implementation Plan: Smart Shell Copilot

**Branch**: `001-smart-shell-copilot` | **Date**: 2026-08-05 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-smart-shell-copilot/spec.md`

## Summary

Build a local-first terminal completion copilot using a lightweight shell hook and
a resident Rust daemon. The shell hook captures command-line state without
blocking native input, sends JSON-RPC requests over a Unix domain socket, and
renders at most one gray same-line Ghost Text suffix. The daemon handles request
cancellation, SQLite FTS5 command-memory retrieval, constrained local model
inference through a safe Rust llama.cpp binding, and silent degradation whenever
the background path is unavailable or too slow. Distribution uses Homebrew with
launchd/systemd service registration and reversible shell hook loading.

## Technical Context

**Language/Version**: Rust stable 1.80+ for the daemon and shared crates; POSIX
shell/Zsh script for shell hooks; Ruby Homebrew Formula DSL for packaging;
Bash-compatible validation harnesses.

**Primary Dependencies**: `tokio` for async runtime; Unix domain socket support
from Tokio/std; `serde`/`serde_json` for JSON-RPC packets; `rusqlite` with
bundled SQLite/FTS5 support; `llama-cpp-rs` as the preferred safe Rust wrapper
over llama.cpp; `tracing` for file/syslog diagnostics that never print into the
terminal; Homebrew Services plus launchd/systemd for service management.

**Storage**: Local SQLite database with `command_history` table and FTS5 virtual
table for command text. Database is per-user and local-only; no sync or cloud
storage.

**Testing**: `cargo test` for daemon units/integration; coverage via cargo-based
coverage tooling; Criterion or custom release benchmark harness for latency,
retrieval, TTFT, RSS, and idle CPU; shell compatibility harness for Zsh/Bash;
Homebrew install/uninstall validation on macOS and Linux.

**Target Platform**: macOS Intel, macOS Apple Silicon, Ubuntu Linux, and Arch
Linux; supported shells are Zsh and Bash. Hardware acceleration targets are
Metal on macOS and AVX2-capable CPU execution on Linux where available.

**Project Type**: Local developer tool with a Rust daemon, shell frontend hook,
local database, model runtime adapter, packaging formula, and validation suites.

**Performance Goals**: End-to-end keystroke-to-Ghost-Text latency <=30ms;
SQLite FTS5 hybrid retrieval <=3ms; warmed prefill plus incremental decode TTFT
<=15ms; daemon RSS <=600MB including model memory; idle CPU approximately 0%; no
skipped typed characters under request load.

**Constraints**: No uncontrolled `unsafe`; any native inference boundary must be
behind safe Rust wrapper APIs with entry validation. Shell Hook, daemon,
retrieval, and inference layers communicate only through JSON-RPC packets or
explicit Rust traits. Non-fatal failures silently degrade to native shell
behavior. UI remains single-line gray Ghost Text only. Shell main thread never
blocks on IPC, retrieval, inference, filesystem, or service state. Install must
be zero-touch and uninstall zero-residue.

**Scale/Scope**: Single-user local command memory with multiple concurrent
terminal sessions. Initial release covers local shell completion only; no cloud
inference, telemetry upload, remote sync, multi-user daemon, GUI, chat UI, or
multi-line command generation.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Memory and concurrency safety**: PASS. Daemon is Rust-only; `llama-cpp-rs` is
  the planned FFI boundary to avoid handwritten C++ FFI. The model adapter trait
  will validate empty prompts, oversized prompts, null/invalid model handles as
  exposed by the wrapper, cancellation, and timeout paths at its public entry.
- **Layer boundaries**: PASS. Shell Hook communicates with daemon only through
  JSON-RPC over a Unix domain socket. Daemon communicates with memory and model
  layers through `MemoryStore` and `CompletionModel` traits. No direct
  cross-layer imports from shell scripts into storage/model code are allowed.
- **Silent degradation**: PASS. Socket errors, SQLite locks, unavailable daemon,
  invalid suggestions, inference timeout, and service restart paths return no
  suggestion to the hook and preserve native shell behavior with no terminal
  error output.
- **Testing gates**: PASS. Planning artifacts require unit/integration tests for
  scheduling, debounce/cancellation, ranking, IPC, failure handling,
  compatibility, install/uninstall behavior, and >=85% core daemon coverage.
- **Latency/resource budgets**: PASS WITH BENCHMARK RISK. Architecture is chosen
  to meet <=30ms end-to-end, <=3ms retrieval, <=15ms warmed TTFT, <=600MB RSS,
  and idle CPU approximately 0%; model choice remains benchmark-gated in
  research and tasks.
- **Ghost Text UX**: PASS. Contracts constrain rendering to a same-line suffix;
  `Tab` and `Right Arrow` accept only when a suggestion is active; unrelated
  shell shortcuts fall through.
- **Zero-touch install**: PASS WITH PLATFORM RISK. Homebrew Services plus
  launchd/systemd service files are planned. Shell hook loading must use a
  reversible package-managed load point and may not require manual `.zshrc`
  edits for the default path.

## Project Structure

### Documentation (this feature)

```text
specs/001-smart-shell-copilot/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── json-rpc.md
│   ├── shell-hook.md
│   ├── memory-store.md
│   ├── model-adapter.md
│   └── packaging.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/
├── daemon/
│   └── src/
│       ├── main.rs
│       ├── ipc/
│       ├── scheduler/
│       ├── memory/
│       ├── model/
│       ├── config/
│       └── diagnostics/
├── protocol/
│   └── src/lib.rs
└── bench-harness/
    └── src/

shell/
├── zsh/smart-shell-copilot.zsh
└── bash/smart-shell-copilot.bash

packaging/
├── homebrew/smart-shell-copilot.rb
├── launchd/com.smart-shell-copilot.daemon.plist
└── systemd/smart-shell-copilot.service

models/
└── README.md

reference/
└── README.md

tests/
├── unit/
├── integration/
├── compat/
├── benchmarks/
├── packaging/
└── fixtures/
```

**Structure Decision**: Use a Rust workspace for daemon/protocol/benchmark code,
plain shell directories for frontend hooks, packaging directories for Homebrew
and service manager definitions, and separate test folders for unit,
integration, compatibility, benchmark, and packaging validation. Keep third-party
reference checkouts under `reference/` and model acquisition notes under
`models/` so source crates do not depend on vendored experiments.

## Phase 0: Research Summary

See [research.md](./research.md) for decisions and alternatives on runtime,
model, local memory, IPC, shell hooks, packaging, benchmarking, and failure
handling.

## Phase 1: Design Summary

See [data-model.md](./data-model.md) for entities, state transitions, and
validation rules. See [contracts/](./contracts/) for JSON-RPC, shell hook,
memory-store, model-adapter, and packaging contracts. See
[quickstart.md](./quickstart.md) for validation scenarios.

## Post-Design Constitution Re-check

- **Memory and concurrency safety**: PASS. The design uses `CompletionModel` as
  the only model boundary and requires failure/cancellation validation.
- **Layer boundaries**: PASS. Contracts define JSON-RPC externally and traits
  internally.
- **Silent degradation**: PASS. Contracts explicitly encode no-suggestion fallback
  rather than terminal-visible errors.
- **Testing gates**: PASS. Quickstart and contracts identify required benchmark,
  compatibility, and packaging validation paths for `/speckit-tasks`.
- **Latency/resource budgets**: PASS WITH REQUIRED BENCHMARKS. Targets are
  documented and must become blocking tasks.
- **Ghost Text UX**: PASS. Shell contract prohibits multiline/popup/chat output.
- **Zero-touch install**: PASS WITH REQUIRED PLATFORM VALIDATION. Packaging
  contract requires install/start and uninstall cleanup checks.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
