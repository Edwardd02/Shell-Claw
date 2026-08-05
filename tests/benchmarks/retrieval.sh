#!/usr/bin/env bash
set -euo pipefail

FIXTURE="${1:-tests/fixtures/command-history-large.sqlite}"

echo "=== Retrieval Benchmark ==="
echo "Fixture: $FIXTURE"
echo ""
echo "Status: SKIPPED (harness scaffold — no implementation yet)"
echo "Goal: <=3ms hybrid SQLite FTS5 retrieval"
echo "Goal: current-directory, frequent, and recent commands ranked above unrelated"
