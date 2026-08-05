#!/usr/bin/env bash
set -euo pipefail

SHELL="${1:-zsh}"
SAMPLES="${2:-1000}"

echo "=== End-to-End Latency Benchmark ==="
echo "Shell: $SHELL"
echo "Samples: $SAMPLES"
echo ""
echo "Status: SKIPPED (harness scaffold — no implementation yet)"
echo "Goal: <=30ms end-to-end keystroke-to-Ghost-Text latency"
echo "Goal: 0 skipped characters in any sample"
echo "Goal: no visible stutter in >=99% of samples"
