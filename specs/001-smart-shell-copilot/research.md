# Research: Smart Shell Copilot

## Decision: Rust daemon with Tokio async runtime

**Rationale**: Rust provides memory safety without garbage collection pauses,
strong concurrency primitives, and predictable resident resource use. Tokio fits
the daemon's socket server, cancellation, timeout, and background scheduling
requirements without blocking the shell-facing path.

**Alternatives considered**:

- Go daemon: simpler service development but introduces GC behavior and less
  direct alignment with the constitution's Rust daemon requirement.
- Python/Node daemon: faster prototyping but unacceptable latency and resident
  runtime overhead risk for keystroke-triggered completion.
- Shell-only implementation: avoids daemon packaging but cannot safely host local
  model inference, memory retrieval, cancellation, and benchmarking within the
  latency budget.

## Decision: Unix domain socket with JSON-RPC packets

**Rationale**: Unix sockets provide low-latency local IPC with filesystem
permissions and no network exposure. JSON-RPC keeps the shell hook/daemon
boundary explicit and debuggable while remaining easy to construct from shell
scripts.

**Alternatives considered**:

- stdin/stdout daemon protocol: fragile when launched under service managers and
  harder to multiplex across terminal sessions.
- TCP localhost: broader attack surface and unnecessary network stack overhead.
- Binary protocol: lower overhead but premature complexity; JSON payloads are
  small and bounded for command-line completion.

## Decision: Request generation with debounce plus daemon-side cancellation

**Rationale**: The shell hook must avoid flooding the daemon during rapid typing,
while the daemon must cancel or supersede stale work after request arrival. This
two-level strategy protects latency and prevents obsolete Ghost Text rendering.

**Alternatives considered**:

- Debounce only in shell hook: insufficient when slow inference is already in
  progress.
- Cancellation only in daemon: still wastes socket traffic and shell work during
  fast typing.
- Fixed-rate polling: violates idle CPU and main-thread non-blocking goals.

## Decision: SQLite with FTS5 for local command memory

**Rationale**: SQLite is single-file, local, zero-configuration, and FTS5 supports
fast full-text retrieval for command history. `rusqlite` can use bundled SQLite
with FTS5 enabled, keeping install behavior predictable.

**Alternatives considered**:

- Plain shell history grep: simpler but weak ranking, difficult path/frequency
  modeling, and inconsistent shell formats.
- Tantivy/Lucene-style embedded index: powerful but heavier operational and
  storage complexity than the product needs.
- In-memory-only index: fastest but loses useful history and complicates startup
  warmup.

## Decision: Hybrid ranking with BM25, path relevance, frequency, and recency

**Rationale**: Shell completions are highly project-local. Combining textual
match, current working directory relevance, frequency, and recency gives useful
suggestions while remaining explainable and benchmarkable.

**Alternatives considered**:

- BM25 only: misses project-local command habits.
- Recency only: tends to overfit the last command and ignore typed intent.
- Model-only retrieval: too slow and less deterministic for the 3ms retrieval
  budget.

## Decision: `llama-cpp-rs` as preferred inference wrapper

**Rationale**: The project constitution allows native FFI only behind safe
abstractions. A mature Rust binding avoids handwritten C++ FFI in the project,
while preserving llama.cpp performance, Metal support on macOS, AVX2 CPU support
on Linux, grammar sampling, and memory-locking capabilities where exposed.

**Alternatives considered**:

- Handwritten llama.cpp FFI: rejected because it increases unsafe surface and
  maintenance risk.
- External model server: easier isolation but adds IPC/process latency and more
  service lifecycle complexity.
- Pure Rust inference engines: attractive safety story but higher risk of missing
  Metal/AVX2 performance and grammar sampling maturity.

## Decision: Qwen/Qwen3-0.6B-Base as baseline model, benchmark-gated fallback

**Rationale**: The stack proposal names Qwen/Qwen3-0.6B-Base as the baseline
edge model with roughly 500MB memory use and code/shell pretraining. Planning
will benchmark it against the 600MB RSS and 15ms warmed TTFT gates. If it misses
either gate or produces weak shell suffixes, the fallback evaluation candidate is
Qwen2.5-Coder-0.5B-Instruct because the roadmap explicitly names it for pipeline
setup.

**Alternatives considered**:

- Commit directly to Qwen2.5-Coder-0.5B-Instruct: stronger coding prior but may
  conflict with the requested baseline and memory target.
- Larger local coder models: likely better quality but exceed memory/TTFT budget.
- Rule-based completions only: extremely fast but does not satisfy smart local
  copilot behavior.

## Decision: GBNF single-line suffix grammar plus post-validation

**Rationale**: Grammar sampling constrains generation before it reaches the shell
UI, and post-validation catches empty, multiline, duplicate, stale, or unsafe
outputs. Both are needed to satisfy Ghost Text-only UX and silent degradation.

**Alternatives considered**:

- Prompt-only instruction: too weak; model can still emit Markdown or
  explanations.
- Post-validation only: protects UI but wastes inference time and may increase
  no-suggestion rate.
- Multi-line grammar: explicitly violates constitution and spec.

## Decision: Homebrew Services with launchd/systemd service definitions

**Rationale**: The product's default distribution path is Homebrew. Homebrew
Services maps naturally to launchd on macOS and systemd user services on Linux,
allowing automatic startup and boot persistence with package-manager lifecycle
integration.

**Alternatives considered**:

- Manual install script: violates zero-touch package-manager requirement.
- User manually edits `.zshrc`: explicitly forbidden by constitution.
- Always-on shell-spawned daemon: easier but less reliable across shells and
  terminal windows, and harder to manage on boot.

## Decision: Reversible package-managed shell hook loading

**Rationale**: The shell hook must load automatically without user edits and must
be removed on uninstall. The packaging design will use a package-managed global
load point when permitted by the platform and a reversible Homebrew-managed
shim/probe file. The hook must probe the socket and silently no-op if unavailable.

**Alternatives considered**:

- Editing user dotfiles: rejected because it violates zero-touch and
  zero-residue cleanup.
- Shell plugin manager integration only: not universal and requires user action.
- Per-terminal manual source command: violates out-of-the-box behavior.

## Decision: Diagnostics outside terminal output

**Rationale**: Failures must not disturb shell input/output. Diagnostics will go
to rotating files or platform logging controlled by the daemon/service, while the
shell hook receives only suggestion/no-suggestion outcomes.

**Alternatives considered**:

- Print warnings in shell: violates silent degradation.
- Suppress all diagnostics: makes debugging and release validation too hard.
- Popup notifications: violates Ghost Text-only UX.

## Decision: Release benchmarks as blocking gates

**Rationale**: The constitution makes latency, retrieval, TTFT, RSS, idle CPU,
coverage, shell compatibility, and install/uninstall behavior non-negotiable.
Benchmarks and compatibility tests must run in CI/release validation before
merge/release.

**Alternatives considered**:

- Manual measurement only: too easy to regress.
- Benchmark after implementation only: too late; tasks must include harness work.
- Quality-only model evaluation: ignores terminal responsiveness, which is the
  primary user trust metric.
