#!/usr/bin/env bash
set -euo pipefail

SHELL="${1:-zsh}"

echo "=== Ghost Text UX Validation ==="
echo "Shell: $SHELL"
echo ""
echo "Status: SKIPPED (harness scaffold — no implementation yet)"
echo "Expected outcomes:"
echo "  - Suggestions render only as gray same-line Ghost Text"
echo "  - Tab accepts visible suggestion"
echo "  - Right Arrow accepts visible suggestion"
echo "  - Continuing to type clears or replaces suggestion"
echo "  - No popup, chat UI, Markdown block, or multiline text"
echo "  - Native shortcuts fall through when no suggestion active"
