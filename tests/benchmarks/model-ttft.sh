#!/usr/bin/env bash
set -euo pipefail

MODEL="${1:-models/qwen3-0.6b-base.gguf}"

echo "=== Model Warmed TTFT Benchmark ==="
echo "Model: $MODEL"
echo ""
echo "Status: SKIPPED (harness scaffold — no implementation yet)"
echo "Goal: <=15ms warmed prefill plus incremental decode TTFT"
