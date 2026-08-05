#!/usr/bin/env bash
set -euo pipefail

echo "=== Coverage Gate ==="
echo ""
echo "Status: SKIPPED (harness scaffold — no implementation yet)"
echo "Goal: core daemon coverage >=85% for scheduling, debounce/cancellation, ranking, IPC, failure handling"
echo ""
echo "Run with: cargo llvm-cov --workspace --lcov --output-path coverage.lcov"
