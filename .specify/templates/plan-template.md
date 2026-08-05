# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]

**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Rust stable for daemon code, shell script for hooks, Python 3.11, Swift 5.9 or NEEDS CLARIFICATION]

**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]

**Storage**: [if applicable, e.g., SQLite FTS5 memory index or N/A]

**Testing**: [e.g., cargo test, shell compatibility harness, latency benchmark, coverage tooling or NEEDS CLARIFICATION]

**Target Platform**: [e.g., macOS Intel/Apple Silicon and Linux shells, Linux server, iOS 15+ or NEEDS CLARIFICATION]

**Project Type**: [e.g., library/cli/web-service/mobile-app/compiler/desktop-app or NEEDS CLARIFICATION]

**Performance Goals**: [domain-specific; for Smart Shell Copilot use <=30ms end-to-end latency, <=3ms SQLite retrieval, <=15ms prefill+decode TTFT, <=600MB daemon RSS, idle CPU approximately 0% or NEEDS CLARIFICATION]

**Constraints**: [domain-specific; include safe Rust/no uncontrolled unsafe, explicit layer boundaries, silent degradation, Ghost Text-only UX, non-blocking shell main thread, zero-touch install or NEEDS CLARIFICATION]

**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Memory and concurrency safety**: [Confirm daemon code remains safe Rust; any llama.cpp FFI is isolated behind safe abstractions with null/bounds/lifetime validation]
- **Layer boundaries**: [Confirm Shell Hook, Rust Daemon, SQLite Fast RAG, and llama.cpp FFI communicate only through explicit traits or JSON-RPC contracts]
- **Silent degradation**: [Confirm non-fatal daemon, SQLite, socket, timeout, and inference failures fall back to native shell behavior with no terminal panic/log output]
- **Testing gates**: [Confirm tasks will include unit/integration coverage for affected scheduling, cancellation, ranking, IPC, compatibility, and install/uninstall behavior; core coverage remains >=85%]
- **Latency/resource budgets**: [Confirm design can meet <=30ms end-to-end, <=3ms retrieval, <=15ms prefill+decode TTFT, <=600MB RSS, and idle CPU approximately 0%]
- **Ghost Text UX**: [Confirm suggestions remain single-line gray Ghost Text only, accepted by Tab/Right Arrow, and do not intercept unrelated shell shortcuts]
- **Zero-touch install**: [Confirm package-manager install/uninstall remains automatic and zero-residue across supported shells/platforms]

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
