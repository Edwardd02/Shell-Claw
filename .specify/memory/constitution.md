<!--
Sync Impact Report
Version change: unratified template -> 1.0.0
Modified principles:
- Template principle 1 -> Code Quality Principles
- Template principle 2 -> Testing Standards
- Template principle 3 -> User Experience Consistency
- Template principle 4 -> Performance Requirements
- Template principle 5 -> removed; consolidated into four supplied principles
Added sections:
- Architecture Boundaries and Safety Constraints
- Development Workflow and Release Gates
Removed sections:
- Placeholder SECTION_2_NAME / SECTION_3_NAME template text
Templates requiring updates:
- ✅ .specify/templates/plan-template.md
- ✅ .specify/templates/spec-template.md
- ✅ .specify/templates/tasks-template.md
- ⚠ .agents/skills/speckit-tasks/SKILL.md (pending in current environment; still says tests are optional in command guidance)
- ✅ .agents/skills/speckit-specify/SKILL.md (inspected; no update required)
Follow-up TODOs:
- Update .agents/skills/speckit-tasks/SKILL.md when agent skill edits are permitted so command guidance states that constitution-governed tests are mandatory.
-->

# Smart Shell Copilot Constitution

## Core Principles

### I. Code Quality Principles
The Rust daemon MUST be memory-safe and concurrency-safe. `unsafe` blocks are
prohibited except where llama.cpp C++ FFI binding calls are strictly necessary.
Any permitted FFI usage MUST be isolated behind a safe Rust abstraction that
validates null pointers, ownership, lifetimes, buffer sizes, and boundary
conditions at the abstraction entry point.

The system MUST preserve strict layer boundaries between the Shell Hook frontend
communication layer, Rust Daemon task scheduling and IPC layer, SQLite Fast RAG
memory retrieval layer, and llama.cpp FFI inference layer. Layers MUST interact
only through explicit Rust traits or JSON-RPC packets. Direct cross-layer
dependencies, shared mutable shortcuts, or bypass APIs are forbidden.

The terminal experience MUST silently degrade on non-fatal failures. SQLite lock
conflicts, inference timeouts, socket interruptions, daemon restarts, and other
recoverable conditions MUST fall back to native shell behavior without printing
panic stacks, diagnostic logs, or AI error prompts into the user terminal.

Rationale: The terminal is both a high-frequency input surface and a safety
boundary. Correctness failures must not become typing latency, corrupted shell
state, or visible developer distraction.

### II. Testing Standards
The CI/CD pipeline MUST include automated end-to-end latency benchmarks covering
the full path from simulated keystroke trigger, socket transmission, FTS5
retrieval, and model incremental inference to Ghost Text rendering. Any change
whose measured end-to-end latency exceeds 30ms MUST NOT be merged.

The Rust daemon's task scheduling logic, debounce/cancellation mechanisms, and
SQLite hybrid ranking algorithms combining BM25, path association, and time
decay MUST have comprehensive unit and integration tests. Core code line
coverage MUST remain at or above 85%.

Every release MUST pass automated compatibility validation for Zsh and Bash on
macOS Intel, macOS Apple Silicon, and mainstream Linux distributions including
Ubuntu and Arch Linux. Release validation MUST verify automatic service
registration after package-manager install and zero-residue cleanup after
package-manager uninstall.

Rationale: This project competes with muscle memory. Regressions in latency,
ranking, shell compatibility, or install/uninstall behavior are product failures,
not merely implementation defects.

### III. User Experience Consistency
Smart completion MUST be rendered only as gray Ghost Text to the right of the
cursor. Suggestions MUST be limited to the single-line remainder of the current
command. Multi-line output, Markdown code blocks, popups, command palettes,
chat panes, or any AI chat interface are prohibited in the shell interaction.

Acceptance MUST follow standard shell typing habits. Pressing `Tab` or the
`Right Arrow` key MUST instantly accept the completion. Continuing to type MUST
seamlessly replace or invalidate the suggestion. The system MUST NOT alter,
shadow, or intercept unrelated native shell shortcuts.

The product MUST remain zero-touch after package-manager installation. Service
registration and shell hook mounting MUST complete automatically. Users MUST NOT
be required to manually edit `.zshrc`, modify configuration files, or run extra
initialization commands for the default experience.

Rationale: The copilot is useful only while it feels like the shell itself. Any
new UI mode, extra setup ritual, or keyboard surprise erodes trust immediately.

### IV. Performance Requirements
The resident Rust daemon RSS MUST stay within 600MB, including locked physical
memory for the 0.5B model. When idle with no keystrokes, daemon CPU utilization
MUST approach 0% and MUST NOT perform continuous polling or busy waiting.

SQLite FTS5 hybrid retrieval MUST complete within 3ms. The llama.cpp inference
engine MUST use resident KV-cache warmup and strict GBNF grammar sampling
constraints. Prefill plus incremental decode time to first token MUST remain
within 15ms.

All IPC and scheduling between the Shell Hook and daemon MUST run on independent
asynchronous execution paths. The shell interaction main thread MUST never block
on retrieval, model inference, socket activity, filesystem access, or daemon
state. The product MUST preserve zero skipped characters and zero visible
stutter during normal typing.

Rationale: Completion quality is irrelevant if the terminal hesitates. The main
thread exists to protect keystrokes first and suggestions second.

## Architecture Boundaries and Safety Constraints

The canonical architecture is Shell Hook -> Rust Daemon -> SQLite Fast RAG ->
llama.cpp FFI. Feature plans MUST identify which layer owns each change and MUST
declare the trait or JSON-RPC contract used at every layer boundary.

Any feature touching FFI MUST include a safe-wrapper design, explicit invalid
input behavior, and tests covering null, empty, oversized, timeout, and
cancelled-call cases. Any feature touching the shell hook MUST document how
native shell behavior is preserved when the daemon is absent, slow, or failing.

Any feature touching install, uninstall, service registration, or shell mounting
MUST preserve the zero-touch and zero-residue guarantees across the supported
platform and shell matrix.

## Development Workflow and Release Gates

Specifications MUST include measurable requirements for latency, resource use,
degradation behavior, Ghost Text UX, and installation impact whenever the feature
can affect those areas. Implementation plans MUST complete the Constitution
Check before research and repeat it after design.

Task lists MUST include test and benchmark work for constitution-governed
behavior. For affected areas, tasks MUST include unit tests, integration tests,
end-to-end latency benchmark updates, shell compatibility validation, and
install/uninstall validation. These tasks are mandatory unless the plan proves
the feature cannot affect the governed surface.

Code review MUST reject changes that introduce uncontrolled `unsafe`, bypass
layer interfaces, print non-fatal daemon errors into the terminal, exceed the
latency or resource budgets, weaken Ghost Text-only interaction, or require
manual setup for the default package-manager path.

Release candidates MUST pass the full automated gate set: latency <=30ms,
retrieval <=3ms, prefill plus decode TTFT <=15ms, daemon RSS <=600MB, idle CPU
approximately 0%, core coverage >=85%, and compatibility across the declared
shell/platform matrix.

## Governance

This constitution supersedes conflicting feature specs, implementation plans,
task lists, code review preferences, and informal development habits. Any
conflict MUST be resolved by changing the lower-level artifact or by explicitly
amending this constitution before implementation proceeds.

Amendments MUST be proposed with the exact text change, rationale, migration
impact, affected templates or skills, and a semantic version bump. MAJOR version
bumps apply to backward-incompatible governance changes or principle removals;
MINOR bumps apply to new principles, new mandatory sections, or materially
expanded guidance; PATCH bumps apply to clarifications and non-semantic wording
fixes.

Every `/speckit-plan`, `/speckit-tasks`, `/speckit-analyze`, and code review
MUST treat the constitution as a mandatory compliance source. Constitution
violations are blocking unless documented in the plan's Complexity Tracking with
a specific mitigation and reviewer approval.

**Version**: 1.0.0 | **Ratified**: 2026-08-04 | **Last Amended**: 2026-08-04
