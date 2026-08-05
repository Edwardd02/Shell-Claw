# Feature Specification: Smart Shell Copilot

**Feature Branch**: `001-smart-shell-copilot`

**Created**: 2026-08-04

**Status**: Draft

**Input**: User description: "Technology stack, dual-process workflow, zero-configuration installation, SQLite Fast RAG design, constrained single-line inference, and phased roadmap for Smart Shell Copilot."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Receive Non-Blocking Shell Completion (Priority: P1)

A developer types a command in an interactive shell and receives a gray single-line completion suggestion to the right of the cursor without any skipped characters, visible pause, popup, or chat interface.

**Why this priority**: This is the core product promise. If typing is disrupted or the completion UI does not feel native, the copilot is not usable.

**Independent Test**: Start the local background service, open a supported shell, type a partial command, and verify the shell remains responsive while a single-line ghost suggestion appears and can be ignored by continuing to type.

**Acceptance Scenarios**:

1. **Given** the background service is running and the user is typing in a supported shell, **When** the user pauses briefly after entering a partial command, **Then** the system shows at most one gray single-line completion suffix to the right of the cursor.
2. **Given** a suggestion is visible, **When** the user continues typing characters that diverge from the suggestion, **Then** the visible suggestion is replaced or cleared without changing the user's typed command.
3. **Given** a completion request is still pending, **When** the user types another character before the request completes, **Then** the stale request is cancelled or ignored and cannot render an obsolete suggestion.

---

### User Story 2 - Accept Completion With Native Keystrokes (Priority: P1)

A developer can accept the visible suggestion using familiar shell keystrokes while all unrelated shell shortcuts continue to behave normally.

**Why this priority**: Completion acceptance must fit existing terminal muscle memory and must not steal unrelated keybindings.

**Independent Test**: Produce a visible suggestion, press `Tab` and `Right Arrow` in separate runs, and verify each accepts the suggestion instantly while unrelated native shortcuts still work.

**Acceptance Scenarios**:

1. **Given** a ghost suggestion is visible, **When** the user presses `Tab`, **Then** the suggestion is inserted into the command line immediately.
2. **Given** a ghost suggestion is visible, **When** the user presses `Right Arrow`, **Then** the suggestion is inserted into the command line immediately.
3. **Given** no ghost suggestion is visible, **When** the user presses native shell shortcuts, **Then** those shortcuts retain their normal shell behavior.

---

### User Story 3 - Improve Suggestions From Local Command Memory (Priority: P2)

A developer receives suggestions that account for the current working directory and frequently or recently used commands while preserving strict latency and local-only behavior.

**Why this priority**: Local memory makes suggestions useful for real projects, but it must never compromise speed or privacy expectations.

**Independent Test**: Seed command history for multiple directories, type matching prefixes in each directory, and verify suggestions favor relevant local commands while meeting retrieval and full-path latency targets.

**Acceptance Scenarios**:

1. **Given** matching command history exists for the current directory, **When** the user types a related prefix, **Then** the suggestion prioritizes commands associated with that directory over unrelated paths.
2. **Given** commands vary by frequency and recency, **When** multiple matches are possible, **Then** the chosen suggestion reflects text similarity, path relevance, usage frequency, and recency.
3. **Given** the local memory store is locked, unavailable, or empty, **When** the user types, **Then** completion gracefully degrades without printing an error or blocking shell input.

---

### User Story 4 - Install, Start, and Uninstall Without Manual Configuration (Priority: P2)

A developer installs the product through the package manager, opens a new terminal, and the service plus shell hook are active automatically. When uninstalling, service and shell integration are removed cleanly.

**Why this priority**: Zero-touch setup and zero-residue uninstall are constitution-level requirements and central to trust in a shell tool.

**Independent Test**: Install on each supported platform/shell combination, open a new terminal without editing shell files manually, verify completion activation or silent native fallback, then uninstall and verify no service or hook residue remains.

**Acceptance Scenarios**:

1. **Given** the user installs the package through the supported package manager, **When** installation completes, **Then** the background service is registered and starts automatically without manual user configuration.
2. **Given** the user opens a new supported shell after installation, **When** the shell loads, **Then** the shell hook activates automatically if the service is available or silently falls back to native behavior if unavailable.
3. **Given** the user uninstalls the package through the supported package manager, **When** uninstall completes, **Then** the background service, startup registration, and shell hook integration are removed without residue.

### Edge Cases

- The background service is not running, is slow to respond, crashes, or restarts while the user is typing.
- The local socket is missing, stale, permission-denied, interrupted, or already bound by a previous process.
- The user types continuously faster than completion requests can finish.
- The memory store is locked, corrupted, missing, empty, or contains no relevant command history.
- Inference produces no safe suffix, an empty suffix, a duplicate of already typed text, multi-line content, explanatory text, or non-command text.
- The current command already spans multiple shell continuations or contains unmatched quotes, pipes, escapes, or shell substitutions.
- The user changes directories between request creation and response rendering.
- Install or uninstall runs on a supported shell/platform where service registration succeeds but hook mounting fails, or vice versa.
- Multiple terminal windows issue completion requests concurrently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide smart command completion in supported interactive shells as a local, non-blocking assistance layer.
- **FR-002**: The system MUST render completions only as gray Ghost Text to the right of the cursor.
- **FR-003**: The system MUST limit each completion suggestion to a single-line suffix of the current command.
- **FR-004**: The system MUST NOT show Markdown, explanatory prose, popups, command palettes, chat panels, or multi-line suggestions in the shell interaction.
- **FR-005**: The user MUST be able to accept a visible suggestion with `Tab` or `Right Arrow`.
- **FR-006**: Continuing to type MUST replace, update, or clear the current suggestion without modifying the user's typed command unexpectedly.
- **FR-007**: The shell input path MUST remain responsive and MUST NOT block on completion generation, memory retrieval, service state, socket activity, or model execution.
- **FR-008**: The system MUST debounce rapid typing and discard, cancel, or ignore stale completion requests so obsolete suggestions cannot render.
- **FR-009**: The system MUST send completion requests using the current command line, current working directory, and relevant local context parameters.
- **FR-010**: The background service MUST cancel or supersede prior incomplete inference work when a newer request arrives for the same interactive shell session.
- **FR-011**: The system MUST maintain local command memory containing command text, execution path, last-used time, and usage frequency.
- **FR-012**: The system MUST retrieve local memory candidates using a hybrid ranking that accounts for text similarity, current-path relevance, frequency, and recency.
- **FR-013**: The system MUST keep command memory local to the user's machine unless a future specification explicitly adds a user-approved sync feature.
- **FR-014**: The inference result MUST be constrained so only a command suffix eligible for same-line insertion can be returned to the shell UI.
- **FR-015**: If completion generation fails, times out, returns unsafe content, or has no relevant suggestion, the system MUST silently fall back to native shell behavior.
- **FR-016**: Non-fatal failures MUST NOT print panic stacks, diagnostic logs, model errors, socket errors, or recovery prompts into the terminal input/output stream.
- **FR-017**: The background daemon MUST be memory-safe and concurrency-safe; any required native inference boundary MUST be hidden behind a safe abstraction with boundary-condition validation.
- **FR-018**: The Shell Hook, background daemon, local memory retrieval layer, and inference layer MUST interact only through explicit layer contracts or structured local request/response packets.
- **FR-019**: Package-manager installation MUST register and start the background service automatically on supported platforms.
- **FR-020**: New supported shell sessions MUST load the shell hook automatically after installation without requiring users to edit shell startup files manually.
- **FR-021**: Package-manager uninstall MUST stop the background service and remove service registration plus shell hook integration without residual files or active processes.
- **FR-022**: The system MUST support Zsh and Bash validation on macOS Intel, macOS Apple Silicon, Ubuntu Linux, and Arch Linux before release.

### Key Entities *(include if feature involves data)*

- **Completion Request**: A transient request generated from the current shell session, command line, working directory, timing information, and request identity used to prevent stale rendering.
- **Completion Suggestion**: A same-line command suffix returned to the shell for Ghost Text rendering, including enough metadata to verify it still applies to the current input state.
- **Command Memory Entry**: A locally stored historical command with command text, execution path, last-used timestamp, and usage count.
- **Shell Session**: An active interactive terminal context with its own command line, cursor state, current working directory, and pending completion state.
- **Service Registration**: The platform-specific background service startup configuration created during install and removed during uninstall.
- **Shell Hook Registration**: The automatically loaded shell integration that probes for the service and enables or silently disables completion per shell session.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: During automated typing simulations, 100% of typed characters appear in order with zero skipped characters while completion requests are active.
- **SC-002**: Completion suggestions appear, update, or clear without visible shell stutter in at least 99% of benchmarked interactive typing samples.
- **SC-003**: End-to-end completion latency from typed input trigger to Ghost Text render is no more than 30ms in the automated release benchmark.
- **SC-004**: Local memory retrieval completes in no more than 3ms in the automated release benchmark.
- **SC-005**: First-token suggestion generation remains no more than 15ms in the automated release benchmark for warmed resident operation.
- **SC-006**: Resident background service memory stays at or below 600MB during normal operation, including the local edge model footprint.
- **SC-007**: Idle background CPU usage remains approximately 0% when there are no keystroke-triggered requests.
- **SC-008**: Core daemon code coverage remains at or above 85% for scheduling, debounce/cancellation, memory retrieval ranking, and failure handling logic.
- **SC-009**: On each supported platform/shell combination, package-manager install enables service startup and shell completion without manual shell-file editing.
- **SC-010**: On each supported platform/shell combination, package-manager uninstall leaves no running service, startup registration, or shell hook residue.
- **SC-011**: In failure-injection tests for service absence, socket interruption, memory-store lock, inference timeout, and invalid suggestion output, 100% of cases fall back to native shell behavior without terminal error output.
- **SC-012**: In UX validation tests, 100% of rendered suggestions are single-line gray Ghost Text and 0 cases render Markdown, prose, popups, chat UI, or multi-line content.

## Assumptions

- The default supported shells for the first release are Zsh and Bash.
- The default supported package-manager path is Homebrew-based install and uninstall.
- The product runs local-first by default; no cloud inference, telemetry upload, or cross-device memory sync is included in this feature.
- Exact inference library, model variant, acceleration backend, storage schema details, and service manager files will be finalized in the implementation plan as long as they satisfy the measurable requirements and constitution.
- When technical input contains conflicting model names, planning will choose the smallest local code/shell-capable model that satisfies the 600MB resident memory and 15ms warmed first-token targets.
- Global shell hook loading must be implemented in a way that is reversible by uninstall and does not require the user to manually edit personal shell configuration for the default experience.
