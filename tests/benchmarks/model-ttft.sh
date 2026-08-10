#!/usr/bin/env bash
set -euo pipefail

MODEL="${1:-models/qwen2.5-coder-0.5b-instruct-finetuned.gguf}"

echo "=== Model Warmed TTFT Benchmark ==="
echo "Model: $MODEL"
echo ""
echo "Status: SKIPPED (harness scaffold — no implementation yet)"
echo "Goal: <=15ms warmed prefill plus incremental decode TTFT"
