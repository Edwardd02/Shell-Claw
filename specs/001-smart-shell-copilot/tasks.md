# Tasks: Smart Shell Copilot

**Input**: Design documents from `/specs/001-smart-shell-copilot/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md, .specify/memory/constitution.md

**Tests**: Constitution-governed tests are mandatory for daemon safety, scheduling, ranking, IPC, shell UX, latency, resource usage, compatibility, and install/uninstall behavior.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- Rust workspace: `crates/daemon/`, `crates/protocol/`, `crates/bench-harness/`
- Shell hooks: `shell/zsh/`, `shell/bash/`
- Packaging: `packaging/homebrew/`, `packaging/launchd/`, `packaging/systemd/`
- Model notes: `models/`
- Tests: `tests/unit/`, `tests/integration/`, `tests/compat/`, `tests/benchmarks/`, `tests/packaging/`, `tests/fixtures/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create Rust workspace root with `Cargo.toml` declaring members: `crates/daemon`, `crates/protocol`, `crates/bench-harness`
- [X] T002 [P] Initialize `crates/protocol` crate with `serde`, `serde_json` dependencies in `crates/protocol/Cargo.toml`
- [X] T003 [P] Initialize `crates/daemon` crate with `tokio` (full features), `serde`, `serde_json`, `tracing`, `tracing-subscriber` in `crates/daemon/Cargo.toml`
- [X] T004 [P] Initialize `crates/bench-harness` crate with criterion or manual benchmark deps in `crates/bench-harness/Cargo.toml`
- [X] T005 [P] Create directory structure: `crates/daemon/src/{ipc,scheduler,memory,model,config,diagnostics}/`, `shell/zsh/`, `shell/bash/`, `packaging/homebrew/`, `packaging/launchd/`, `packaging/systemd/`, `models/`, `tests/{unit,integration,compat,benchmarks,packaging,fixtures}/`
- [X] T006 [P] Create `models/README.md` with model acquisition instructions for Qwen/Qwen3-0.6B-Base and Qwen2.5-Coder-0.5B-Instruct fallback
- [X] T007 [P] Configure `rustfmt.toml` and `.cargo/config.toml` for workspace-wide formatting and build settings
- [X] T008 [P] Add `.gitignore` entries for `target/`, `.gguf`, `*.sock`, SQLite files, and diagnostic logs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [X] T009 Define JSON-RPC 2.0 protocol types (`CompletionRequest`, `CompletionResponse`, `CancelRequest`, `CancelResponse`) in `crates/protocol/src/lib.rs` with serde derives
- [X] T010 [P] Define shared error types and `kind: none` no-suggestion response variant in `crates/protocol/src/lib.rs`
- [X] T011 Implement Unix domain socket listener with Tokio in `crates/daemon/src/ipc/server.rs` accepting connections and spawning per-session handlers
- [X] T012 Implement JSON-RPC request parsing, method dispatch, and response serialization in `crates/daemon/src/ipc/handler.rs`
- [X] T013 Implement daemon configuration loading (socket path, model path, DB path, log path, deadline defaults) in `crates/daemon/src/config/mod.rs`
- [X] T014 [P] Implement daemon diagnostics/logging setup using `tracing` to file/syslog (never stdout/stderr) in `crates/daemon/src/diagnostics/mod.rs`
- [X] T015 Implement `MemoryStore` trait definition matching contract in `crates/daemon/src/memory/mod.rs` with `record_command` and `retrieve` methods
- [X] T016 [P] Implement `CompletionModel` trait definition matching contract in `crates/daemon/src/model/mod.rs` with `complete_suffix` method and `CancellationToken`
- [X] T017 Implement daemon `main.rs` entry point: config load, socket bind, tracing init, graceful shutdown in `crates/daemon/src/main.rs`
- [X] T018 [P] Add end-to-end latency benchmark harness scaffold in `tests/benchmarks/e2e-latency.sh` with `--shell` and `--samples` flags
- [X] T019 [P] Add retrieval benchmark harness scaffold in `tests/benchmarks/retrieval.sh` with `--fixture` flag
- [X] T020 [P] Add model TTFT and resources benchmark harness scaffold in `tests/benchmarks/model-ttft.sh` and `tests/benchmarks/daemon-resources.sh`
- [X] T021 [P] Add coverage gate harness scaffold in `tests/benchmarks/run-coverage-gate.sh`
- [X] T022 [P] Add Ghost Text UX validation harness scaffold in `tests/compat/ghost-text.sh` with `--shell` flag
- [X] T023 [P] Add silent degradation validation harness scaffold in `tests/integration/failure-injection.sh`
- [X] T024 [P] Add packaging install/uninstall validation harness scaffolds in `tests/packaging/homebrew-install.sh` and `tests/packaging/homebrew-uninstall.sh` with `--platform` and `--shell` flags
- [X] T025 Configure CI pipeline to run `cargo test --workspace`, benchmark gates, compatibility tests, and packaging validation, blocking merge on any failure

**Checkpoint**: Foundation ready — user story implementation can now begin in parallel

---

## Phase 3: User Story 1 — Receive Non-Blocking Shell Completion (Priority: P1)

**Goal**: A developer types a command in an interactive shell and receives a gray single-line completion suggestion to the right of the cursor without any skipped characters, visible pause, popup, or chat interface.

**Independent Test**: Start the local background service, open a supported shell, type a partial command, and verify the shell remains responsive while a single-line ghost suggestion appears and can be ignored by continuing to type.

### Tests for User Story 1 (MANDATORY — constitution-governed behavior)

> **Write these tests FIRST, ensure they FAIL before implementation.**

- [X] T026 [P] [US1] Unit test for debounce timer (30ms window, rapid typing resets timer) in `tests/unit/daemon/test_debounce.rs`
- [X] T027 [P] [US1] Unit test for request superseding (newer request cancels prior in-flight work for same session) in `tests/unit/daemon/test_supersede.rs`
- [X] T028 [P] [US1] Unit test for stale response rejection (response id mismatch, line/cursor mismatch) in `tests/unit/daemon/test_stale_response.rs`
- [X] T029 [P] [US1] Unit test for daemon startup, socket bind, graceful shutdown lifecycle in `tests/unit/daemon/test_lifecycle.rs`
- [X] T030 [P] [US1] Integration test for end-to-end JSON-RPC request/response round-trip through socket in `tests/integration/test_ipc_roundtrip.rs`
- [X] T031 [P] [US1] Integration test for connection failure (daemon off) producing no terminal output in `tests/integration/test_silent_degradation.rs`
- [X] T032 [P] [US1] Integration test for socket interruption mid-request producing no terminal output in `tests/integration/test_silent_degradation.rs`
- [X] T033 [P] [US1] Ghost Text rendering test: suggestion appears as same-line gray text in `tests/compat/ghost-text.sh`
- [X] T034 [P] [US1] Ghost Text rendering test: continuing to type clears or replaces suggestion in `tests/compat/ghost-text.sh`
- [X] T035 [P] [US1] Latency benchmark: zero skipped characters under 1000-sample typing simulation in `tests/benchmarks/e2e-latency.sh`

### Implementation for User Story 1

- [X] T036 [P] [US1] Implement Zsh shell hook: probe daemon socket, set up preexec/precmd non-blocking request sourcing in `shell/zsh/smart-shell-copilot.zsh`
- [X] T037 [P] [US1] Implement Bash shell hook with equivalent non-blocking behavior in `shell/bash/smart-shell-copilot.bash`
- [X] T038 [US1] Implement daemon request scheduler: accept `completion.request`, assign monotonic request id, queue work, handle cancel/supersede in `crates/daemon/src/scheduler/mod.rs`
- [X] T039 [US1] Implement request validation: reject oversized lines, invalid cursors, empty/whitespace-only lines, missing session_id in `crates/daemon/src/scheduler/validate.rs`
- [X] T040 [US1] Implement daemon-side deadline enforcement: check `deadline_ms` before starting retrieval/inference, expire in-flight work in `crates/daemon/src/scheduler/deadline.rs`
- [X] T041 [US1] Implement `session.cancel` JSON-RPC method handler in `crates/daemon/src/ipc/handler.rs`
- [X] T042 [US1] Implement no-suggestion response path (kind: none) for all failure/degradation cases in `crates/daemon/src/scheduler/noop.rs`
- [X] T043 [US1] Wire scheduler into IPC handler: dispatch `completion.request` and `session.cancel` through scheduler, return JSON-RPC response in `crates/daemon/src/ipc/handler.rs`
- [X] T044 [US1] Implement Ghost Text rendering in Zsh: gray suffix display using zle region_highlight, clear on new input in `shell/zsh/smart-shell-copilot.zsh`
- [X] T045 [US1] Implement Ghost Text rendering in Bash: gray suffix display using READLINE_LINE manipulation, clear on new input in `shell/bash/smart-shell-copilot.bash`
- [X] T046 [US1] Add diagnostics for IPC connection lifecycle, request counts, deadlines missed to `crates/daemon/src/diagnostics/mod.rs`

**Checkpoint**: User Story 1 — non-blocking completion with Ghost Text rendering — independently testable. Shell hook sends requests, daemon responds with `kind: none` (no real retrieval/inference yet), Ghost Text appears and clears correctly.

---

## Phase 4: User Story 2 — Accept Completion With Native Keystrokes (Priority: P1)

**Goal**: A developer can accept the visible suggestion using `Tab` or `Right Arrow` while all unrelated shell shortcuts continue to behave normally.

**Independent Test**: Produce a visible suggestion, press `Tab` and `Right Arrow` in separate runs, and verify each accepts the suggestion instantly while unrelated native shortcuts still work.

### Tests for User Story 2 (MANDATORY — constitution-governed behavior)

> **Write these tests FIRST, ensure they FAIL before implementation.**

- [X] T047 [P] [US2] Unit test for Tab key acceptance when suggestion is active in `tests/unit/zsh/test_acceptance.zsh`
- [X] T048 [P] [US2] Unit test for Right Arrow key acceptance when suggestion is active in `tests/unit/zsh/test_acceptance.zsh`
- [X] T049 [P] [US2] Unit test for Tab fallthrough to native behavior when no suggestion is active in `tests/unit/zsh/test_acceptance.zsh`
- [X] T050 [P] [US2] Unit test for unrelated shortcuts (Ctrl+C, Ctrl+D, Up Arrow, etc.) passing through when suggestion is active in `tests/unit/zsh/test_fallback.zsh`
- [X] T051 [P] [US2] Ghost Text acceptance test: Tab inserts suggestion in `tests/compat/ghost-text.sh`
- [X] T052 [P] [US2] Ghost Text acceptance test: Right Arrow inserts suggestion in `tests/compat/ghost-text.sh`
- [X] T053 [P] [US2] Ghost Text acceptance test: native shortcuts fall through when no suggestion in `tests/compat/ghost-text.sh`

### Implementation for User Story 2

- [X] T054 [P] [US2] Implement Zsh Tab key binding: accept visible suggestion, else fall through to native Tab in `shell/zsh/smart-shell-copilot.zsh`
- [X] T055 [P] [US2] Implement Zsh Right Arrow key binding: accept visible suggestion, else fall through to native Right Arrow in `shell/zsh/smart-shell-copilot.zsh`
- [X] T056 [P] [US2] Implement Bash Tab key binding with equivalent acceptance/fallthrough in `shell/bash/smart-shell-copilot.bash`
- [X] T057 [P] [US2] Implement Bash Right Arrow key binding with equivalent acceptance/fallthrough in `shell/bash/smart-shell-copilot.bash`
- [X] T058 [US2] Implement suggestion acceptance state machine: transition from Rendered to Accepted, clear Ghost Text, update command line in Zsh hook in `shell/zsh/smart-shell-copilot.zsh`
- [X] T059 [US2] Implement suggestion acceptance state machine in Bash hook in `shell/bash/smart-shell-copilot.bash`
- [X] T060 [US2] Verify all native Zsh shortcuts (expand-or-complete, backward-delete-char, up-line-or-history, etc.) are not shadowed when no suggestion in `shell/zsh/smart-shell-copilot.zsh`
- [X] T061 [US2] Verify all native Bash shortcuts are not shadowed when no suggestion in `shell/bash/smart-shell-copilot.bash`

**Checkpoint**: User Story 2 — keystroke acceptance — independently testable. Tab/Right Arrow accept suggestion, all other shortcuts fall through, no keybinding conflicts.

---

## Phase 5: User Story 3 — Improve Suggestions From Local Command Memory (Priority: P2)

**Goal**: A developer receives suggestions that account for the current working directory and frequently or recently used commands while preserving strict latency and local-only behavior.

**Independent Test**: Seed command history for multiple directories, type matching prefixes in each directory, and verify suggestions favor relevant local commands while meeting retrieval and full-path latency targets.

### Tests for User Story 3 (MANDATORY — constitution-governed behavior)

> **Write these tests FIRST, ensure they FAIL before implementation.**

- [X] T062 [P] [US3] Unit test for SQLite FTS5 table creation and migration in `tests/unit/daemon/test_memory_schema.rs`
- [X] T063 [P] [US3] Unit test for `record_command` inserting/upserting command memory entries in `tests/unit/daemon/test_memory_record.rs`
- [X] T064 [P] [US3] Unit test for BM25 text scoring correctness in `tests/unit/daemon/test_memory_ranking.rs`
- [X] T065 [P] [US3] Unit test for cwd relevance scoring (same directory > parent > unrelated) in `tests/unit/daemon/test_memory_ranking.rs`
- [X] T066 [P] [US3] Unit test for frequency and recency time-decay scoring in `tests/unit/daemon/test_memory_ranking.rs`
- [X] T067 [P] [US3] Unit test for hybrid ranking: combined score ordering matches expected relevance in `tests/unit/daemon/test_memory_ranking.rs`
- [X] T068 [P] [US3] Unit test for SQLite lock/corruption/missing-table producing empty candidate list (no crash, no error to terminal) in `tests/unit/daemon/test_memory_failure.rs`
- [X] T069 [P] [US3] Integration test for memory retrieval returning candidates ordered by relevance with seeded data in `tests/integration/test_memory_retrieval.rs`
- [X] T070 [P] [US3] Integration test for retrieval latency meeting <=3ms budget with large fixture in `tests/integration/test_memory_latency.rs`
- [X] T071 [P] [US3] Retrieval benchmark: <=3ms hybrid retrieval with large command-history fixture in `tests/benchmarks/retrieval.sh`

### Implementation for User Story 3

- [X] T072 [P] [US3] Implement SQLite FTS5 schema: `command_history` table and `command_fts` virtual table in `crates/daemon/src/memory/schema.rs`
- [X] T073 [P] [US3] Implement database initialization, connection pool, and migration logic (if applicable) in `crates/daemon/src/memory/db.rs`
- [X] T074 [US3] Implement `MemoryStore::record_command` — upsert command_history, maintain use_count and last_used_at, update FTS index in `crates/daemon/src/memory/record.rs`
- [X] T075 [US3] Implement `MemoryStore::retrieve` — FTS5 BM25 query + cwd match + frequency + recency hybrid scoring in `crates/daemon/src/memory/retrieve.rs`
- [X] T076 [US3] Implement retrieval deadline guard: abort and return empty if elapsed >= deadline_ms in `crates/daemon/src/memory/retrieve.rs`
- [X] T077 [US3] Implement ranking weight configuration (w1, w2, w3, lambda for time decay) through daemon config in `crates/daemon/src/config/mod.rs`
- [X] T078 [US3] Wire memory retrieval into scheduler: run retrieve before inference, pass candidates as model context in `crates/daemon/src/scheduler/mod.rs`
- [X] T079 [US3] Wire command recording into scheduler: record executed commands after user accepts suggestion or presses Enter in `crates/daemon/src/scheduler/mod.rs`
- [X] T080 [US3] Implement `kind: none` fallback for locked/corrupt/empty memory store in scheduler's no-suggestion path
- [X] T081 [US3] Add diagnostics for memory hit rate, retrieval latency distribution, ranking decision logs in `crates/daemon/src/diagnostics/mod.rs`
- [X] T082 [US3] Create large command-history fixture for benchmarking in `tests/fixtures/command-history-large.sqlite`

**Checkpoint**: User Story 3 — local command memory with hybrid ranking — independently testable. Retrieval returns cwd/frequency/recency-ranked candidates within 3ms, degrades silently on failure.

---

## Phase 6: Model Inference Integration (Cross-cutting US1+US3 — Priority: P2)

**Purpose**: This phase connects the local model to the scheduler, enabling smart suffix generation. It is a cross-cutting concern between US1 (rendering) and US3 (retrieval provides candidates). Listed as its own phase because it introduces the FFI boundary and GBNF grammar.

**Independent Test**: With model loaded, type a partial command; verify the daemon returns a valid single-line suffix from inference combined with retrieval candidates, meeting TTFT and output validation constraints.

### Tests for Model Integration (MANDATORY — constitution-governed behavior)

> **Write these tests FIRST, ensure they FAIL before implementation.**

- [X] T083 [P] [US1] Unit test for GBNF grammar constraint: outputs with `\n`, `\r`, `\0`, Markdown fences rejected in `tests/unit/daemon/test_grammar.rs`
- [X] T084 [P] [US1] Unit test for post-validation: empty suffix rejected, duplicate-of-input rejected, explanatory prose rejected in `tests/unit/daemon/test_suffix_validation.rs`
- [X] T085 [P] [US1] Unit test for model cancellation: cancelled token stops inference, returns no-suggestion in `tests/unit/daemon/test_model_cancel.rs`
- [X] T086 [P] [US1] Unit test for model deadline: expired deadline returns no-suggestion before starting or during inference in `tests/unit/daemon/test_model_deadline.rs`
- [X] T087 [P] [US1] Unit test for model error paths: missing model file, invalid GGUF, OOM produce no-suggestion not crash in `tests/unit/daemon/test_model_error.rs`
- [X] T088 [P] [US1] Unit test for bounded context: oversized prompt or candidate list rejected before inference in `tests/unit/daemon/test_model_context.rs`
- [X] T089 [P] [US1] Integration test for full pipeline: retrieval + inference produce valid single-line suffix in `tests/integration/test_pipeline_e2e.rs`
- [X] T090 [P] [US1] Integration test for inference timeout producing no-suggestion in `tests/integration/test_silent_degradation.rs`
- [X] T091 [P] [US1] Integration test for multiline model output rejected, no suggestion rendered in `tests/integration/test_silent_degradation.rs`
- [X] T092 [P] [US1] Model TTFT benchmark: warmed prefill + decode <=15ms in `tests/benchmarks/model-ttft.sh`
- [X] T093 [P] [US1] Daemon resource benchmark: RSS <=600MB, idle CPU ~0% in `tests/benchmarks/daemon-resources.sh`

### Implementation for Model Integration

- [X] T094 Implement `llama-cpp-rs` dependency addition to `crates/daemon/Cargo.toml` with Metal feature flag for macOS
- [X] T095 [P] Implement GBNF single-line grammar string and grammar loading wrapper in `crates/daemon/src/model/grammar.rs`
- [X] T096 [P] Implement model suffix post-validator: reject empty, multiline, Markdown, explanatory text, duplicate in `crates/daemon/src/model/validate.rs`
- [X] T097 Implement `CompletionModel::complete_suffix` using `llama-cpp-rs`: load model, apply GBNF grammar, run inference with cancellation support in `crates/daemon/src/model/adapter.rs`
- [X] T098 Implement model context builder: assemble line_prefix, cwd, retrieval candidates, grammar_id, deadline into bounded format in `crates/daemon/src/model/context.rs`
- [X] T099 Implement model warmup on daemon startup: pre-load model weights, warm KV cache, prime system prompt in `crates/daemon/src/model/warmup.rs`
- [X] T100 Implement safe FFI boundary wrapper: validate model handles, null pointers, buffer sizes at entry; no unsafe leakage into scheduler in `crates/daemon/src/model/safe_wrapper.rs`
- [X] T101 Implement inference deadline enforcement: check remaining budget before decode steps, abort on expiry in `crates/daemon/src/model/adapter.rs`
- [X] T102 Wire model inference into scheduler: call after retrieval, pass candidates as context, handle cancellation/timeout/no-suggestion in `crates/daemon/src/scheduler/mod.rs`
- [X] T103 Implement model selection: Qwen/Qwen3-0.6B-Base as primary, Qwen2.5-Coder-0.5B-Instruct as configurable fallback in `crates/daemon/src/config/mod.rs`
- [X] T104 Add diagnostics for inference TTFT, model load time, cancellation rate, rejection rate in `crates/daemon/src/diagnostics/mod.rs`

**Checkpoint**: Full pipeline works end-to-end: shell hook sends request → daemon retrieves memory → model generates suffix → Ghost Text rendered. All TTFT, RSS, latency, grammar, and validation gates passable.

---

## Phase 7: User Story 4 — Install, Start, and Uninstall Without Manual Configuration (Priority: P2)

**Goal**: A developer installs the product through the package manager, opens a new terminal, and the service plus shell hook are active automatically. When uninstalling, service and shell integration are removed cleanly.

**Independent Test**: Install on each supported platform/shell combination, open a new terminal without editing shell files manually, verify completion activation or silent native fallback, then uninstall and verify no service or hook residue remains.

### Tests for User Story 4 (MANDATORY — constitution-governed behavior)

> **Write these tests FIRST, ensure they FAIL before implementation.**

- [X] T105 [P] [US4] Packaging test: install registers and starts daemon service automatically in `tests/packaging/homebrew-install.sh`
- [X] T106 [P] [US4] Packaging test: new shell session loads hook without manual dotfile edits in `tests/packaging/homebrew-install.sh`
- [X] T107 [P] [US4] Packaging test: hook silently no-ops when daemon is unavailable in `tests/packaging/homebrew-install.sh`
- [X] T108 [P] [US4] Packaging test: uninstall stops daemon, unregisters service, removes hook, no residue in `tests/packaging/homebrew-uninstall.sh`
- [X] T109 [P] [US4] Packaging test: uninstall leaves no running daemon process or active socket in `tests/packaging/homebrew-uninstall.sh`
- [X] T110 [P] [US4] Packaging test: uninstall does not delete unrelated user shell configuration in `tests/packaging/homebrew-uninstall.sh`

### Implementation for User Story 4

- [X] T111 [P] [US4] Write Homebrew Formula: install daemon binary, model download instructions, shell hook files, service registration in `packaging/homebrew/smart-shell-copilot.rb`
- [X] T112 [P] [US4] Write macOS launchd plist: socket path, model path, config path, log path, restart policy in `packaging/launchd/com.smart-shell-copilot.daemon.plist`
- [X] T113 [P] [US4] Write Linux systemd user service: socket path, model path, config path, log path, restart policy in `packaging/systemd/smart-shell-copilot.service`
- [X] T114 [US4] Implement Homebrew install post-install: register service, start daemon, install shell hook load point in `packaging/homebrew/smart-shell-copilot.rb`
- [X] T115 [US4] Implement Homebrew uninstall post-uninstall: stop daemon, unregister service, remove hook load point and hook files in `packaging/homebrew/smart-shell-copilot.rb`
- [X] T116 [US4] Implement reversible shell hook loading: package-managed global load point (Homebrew site-functions / etc), daemon probe with short timeout in `packaging/homebrew/smart-shell-copilot.rb`
- [X] T117 [US4] Implement shell hook silent no-op: if daemon socket unreachable, hook loads but does nothing (no error, no terminal output) in `shell/zsh/smart-shell-copilot.zsh` and `shell/bash/smart-shell-copilot.bash`
- [X] T118 [US4] Add platform detection and conditional service manager logic (launchd vs systemd) in `packaging/homebrew/smart-shell-copilot.rb`

**Checkpoint**: User Story 4 — zero-touch install/uninstall — independently testable. Brew install activates everything, brew uninstall removes everything, hooks auto-load and silently degrade.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Release-quality hardening, complete validation matrix, and final gate checks

- [X] T119 [P] Verify end-to-end latency <=30ms under full pipeline load with `tests/benchmarks/e2e-latency.sh --shell zsh --samples 1000` and Bash variant
- [X] T120 [P] Verify retrieval <=3ms with large fixture `tests/benchmarks/retrieval.sh`
- [X] T121 [P] Verify warm TTFT <=15ms with `tests/benchmarks/model-ttft.sh`
- [X] T122 [P] Verify daemon RSS <=600MB and idle CPU ~0% with `tests/benchmarks/daemon-resources.sh`
- [X] T123 Verify core daemon coverage >=85% for scheduler, memory, model, IPC modules using `tests/benchmarks/run-coverage-gate.sh`
- [X] T124 [P] Run full release matrix validation: macOS Intel/Bash+Zsh, macOS Apple Silicon/Bash+Zsh, Ubuntu/Bash+Zsh, Arch/Bash+Zsh using `tests/compat/ghost-text.sh` and packaging scripts
- [X] T125 [P] Run failure injection validation: daemon off, socket stale, SQLite lock, empty memory, inference timeout, multiline output, daemon restart in `tests/integration/failure-injection.sh`
- [X] T126 [P] Verify 100% of failure-injection cases fall back to native shell with no terminal error in `tests/integration/failure-injection.sh`
- [X] T127 [P] Verify 100% of rendered suggestions are single-line gray Ghost Text, 0 Markdown/popups/chat/multi-line in `tests/compat/ghost-text.sh`
- [X] T128 [P] Verify zero skipped characters in 1000-sample automated typing simulation in `tests/benchmarks/e2e-latency.sh`
- [X] T129 [P] Verify completion updates show no visible stutter in >=99% of benchmarked samples in `tests/benchmarks/e2e-latency.sh`
- [X] T130 Code review: audit for uncontrolled `unsafe`, cross-layer imports, terminal-visible error output, hardcoded paths, missing deadline checks
- [X] T131 Run full quickstart.md validation: all 9 checklists pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational — non-blocking completion + Ghost Text rendering
- **User Story 2 (Phase 4)**: Depends on User Story 1 (needs Ghost Text to accept) — keystroke acceptance
- **User Story 3 (Phase 5)**: Depends on Foundational, can run parallel with US1/US2 — local memory ranking
- **Model Integration (Phase 6)**: Depends on US1 (scheduler needs requests) AND US3 (retrieval provides candidates) — model inference
- **User Story 4 (Phase 7)**: Depends on Foundational, can run parallel with US1/US2/US3 — packaging
- **Polish (Phase 8)**: Depends on all phases complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependencies on other stories
- **User Story 2 (P1)**: Depends on US1 (Ghost Text must exist before acceptance can be tested)
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) — independent from US1/US2 in implementation, but integration tested together
- **Model Integration**: Cross-cuts US1+US3 — US1 provides rendering path, US3 provides memory candidates
- **User Story 4 (P2)**: Can start after Foundational (Phase 2) — independent from other stories

### Within Each Phase

- Tests MUST be written and FAIL before implementation for constitution-governed behavior
- Shared infrastructure (traits, config, protocol) before feature code
- Core implementation before integration
- Integration before benchmarks
- Phase complete before moving to dependent phase

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- After Foundational: US1, US3, and US4 can start in parallel
- All tests within a phase marked [P] can run in parallel
- US1 models/services marked [P] can run in parallel
- US2 zsh/bash hook implementations marked [P] can run in parallel
- US3 schema/record/retrieval tasks marked [P] can run in parallel
- US4 packaging files marked [P] can run in parallel
- Phase 8 polish tasks marked [P] can run in parallel

---

## Parallel Example: Phase 3 (User Story 1)

```bash
# Launch all US1 tests together:
Task: "Unit test for debounce timer in tests/unit/daemon/test_debounce.rs"
Task: "Unit test for request superseding in tests/unit/daemon/test_supersede.rs"
Task: "Unit test for stale response rejection in tests/unit/daemon/test_stale_response.rs"
Task: "Unit test for daemon lifecycle in tests/unit/daemon/test_lifecycle.rs"

# Launch all US1 hook implementations together:
Task: "Implement Zsh shell hook in shell/zsh/smart-shell-copilot.zsh"
Task: "Implement Bash shell hook in shell/bash/smart-shell-copilot.bash"
```

## Parallel Example: Phase 5 (User Story 3)

```bash
# Launch all US3 tests together:
Task: "Unit test for SQLite FTS5 schema in tests/unit/daemon/test_memory_schema.rs"
Task: "Unit test for record_command in tests/unit/daemon/test_memory_record.rs"
Task: "Unit test for ranking in tests/unit/daemon/test_memory_ranking.rs"
Task: "Unit test for memory failure in tests/unit/daemon/test_memory_failure.rs"

# Launch all US3 memory implementations together:
Task: "Implement SQLite FTS5 schema in crates/daemon/src/memory/schema.rs"
Task: "Implement database init in crates/daemon/src/memory/db.rs"
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (non-blocking completion with Ghost Text)
4. Complete Phase 4: User Story 2 (keystroke acceptance)
5. **STOP and VALIDATE**: Basic Ghost Text + Tab/Right Arrow acceptance works
6. Deploy/demo if ready (no retrieval or inference yet — just `kind: none` responses)

### MVP + Memory (US1 + US2 + US3)

1. After MVP, add Phase 5: User Story 3 (local command memory ranking)
2. **VALIDATE**: Suggestions now come from local command history with path-aware ranking
3. Retrieval within 3ms, degrades silently

### Full Pipeline (US1–US4 + Model)

1. Add Phase 6: Model inference integration
2. Add Phase 7: User Story 4 (packaging)
3. **VALIDATE**: Full pipeline with model-generated completions, zero-touch install/uninstall
4. Complete Phase 8: Polish and release gates

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 + 2 (shell hook + rendering + acceptance)
   - Developer B: User Story 3 + Model Integration (memory + inference)
   - Developer C: User Story 4 (packaging)
3. Integrate after each story is independently testable
4. All team members join Phase 8: Polish and gate validation

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Constitution gates (<=30ms e2e, <=3ms retrieval, <=15ms TTFT, <=600MB RSS, idle CPU ~0%, >=85% coverage) are blocking for merge/release
- All tests touching daemon safety, scheduling, ranking, IPC, shell UX, latency, resource, compatibility, and install/uninstall are mandatory per constitution
