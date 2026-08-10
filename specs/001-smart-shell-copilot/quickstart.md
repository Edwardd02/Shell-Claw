# Quickstart Validation Guide: Smart Shell Copilot

This guide defines runnable validation scenarios for the feature after tasks are
generated and implemented. Commands are intentionally placeholders until
`/speckit-tasks` maps them to exact scripts and CI jobs.

## Prerequisites

- Rust stable toolchain installed.
- Supported shell available: Zsh and/or Bash.
- SQLite with FTS5 available through the project build.
- Local edge model file acquired according to `models/README.md`.
- Platform service manager available:
  - macOS: launchd/Homebrew Services.
  - Linux: user-level systemd/Homebrew Services.

## 1. Daemon Unit and Integration Validation

```bash
cargo test --workspace
```

Expected outcomes:

- Scheduler cancellation tests pass.
- Debounce/supersede behavior tests pass.
- Memory ranking tests pass for BM25, cwd relevance, frequency, and recency.
- Failure handling tests prove socket, SQLite, and inference failures produce
  no-suggestion outcomes rather than terminal-visible errors.

## 2. Coverage Gate

```bash
./tests/benchmarks/run-coverage-gate.sh
```

Expected outcome: core daemon coverage is >=85% for scheduling,
debounce/cancellation, ranking, IPC, and failure handling modules.

## 3. End-to-End Latency Benchmark

```bash
./tests/benchmarks/e2e-latency.sh --shell zsh --samples 1000
./tests/benchmarks/e2e-latency.sh --shell bash --samples 1000
```

Expected outcomes:

- 100% of simulated typed characters appear in order with zero skipped
  characters.
- End-to-end keystroke-to-Ghost-Text latency is <=30ms.
- Completion updates show no visible stutter in at least 99% of samples.

## 4. Retrieval Benchmark

```bash
./tests/benchmarks/retrieval.sh --fixture tests/fixtures/command-history-large.sqlite
```

Expected outcome: hybrid SQLite FTS5 retrieval completes in <=3ms while ranking
current-directory, frequent, and recent commands above unrelated matches.

## 5. Model Warmed TTFT and Resource Benchmark

```bash
./tests/benchmarks/model-ttft.sh --model models/qwen2.5-coder-0.5b-instruct-finetuned.gguf
./tests/benchmarks/daemon-resources.sh
```

Expected outcomes:

- Warmed prefill plus incremental decode TTFT is <=15ms.
- Resident daemon RSS is <=600MB including model memory.
- Idle CPU is approximately 0% when no keystroke requests are active.
- Outputs are rejected if multiline, empty, explanatory, Markdown, or stale.

## 6. Ghost Text UX Validation

```bash
./tests/compat/ghost-text.sh --shell zsh
./tests/compat/ghost-text.sh --shell bash
```

Expected outcomes:

- Suggestions render only as gray same-line Ghost Text.
- `Tab` accepts visible suggestion.
- `Right Arrow` accepts visible suggestion.
- Continuing to type clears or replaces suggestion without corrupting typed text.
- No popup, chat UI, Markdown block, or multiline text appears.
- Native shortcuts fall through when no suggestion is active.

## 7. Silent Degradation Validation

```bash
./tests/integration/failure-injection.sh
```

Inject failures:

- Daemon unavailable.
- Stale or permission-denied socket.
- SQLite lock.
- Empty memory store.
- Inference timeout.
- Invalid multiline model output.
- Daemon restart during active typing.

Expected outcome: every case falls back to native shell behavior with no terminal
error output, no panic stack, and no skipped characters.

## 8. Package Install/Uninstall Validation

```bash
./tests/packaging/homebrew-install.sh --platform current --shell zsh
./tests/packaging/homebrew-install.sh --platform current --shell bash
./tests/packaging/homebrew-uninstall.sh --platform current --shell zsh
./tests/packaging/homebrew-uninstall.sh --platform current --shell bash
```

Expected outcomes:

- Install registers and starts the daemon service automatically.
- Opening a new shell loads the hook without manual shell-file edits.
- Hook silently no-ops if daemon is unavailable.
- Uninstall stops daemon, unregisters service, removes hook integration, and
  leaves no running daemon process or active service socket.

## 9. Release Matrix Validation

Run the benchmark, compatibility, and packaging suites across:

- macOS Intel + Zsh
- macOS Intel + Bash
- macOS Apple Silicon + Zsh
- macOS Apple Silicon + Bash
- Ubuntu Linux + Zsh
- Ubuntu Linux + Bash
- Arch Linux + Zsh
- Arch Linux + Bash

Expected outcome: all constitution gates pass before release.
