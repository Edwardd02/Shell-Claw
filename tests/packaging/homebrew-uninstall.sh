#!/usr/bin/env bash
set -euo pipefail

PLATFORM="${1:-current}"
SHELL="${2:-zsh}"

echo "=== Homebrew Uninstall Validation ==="
echo "Platform: $PLATFORM"
echo "Shell: $SHELL"
echo ""
echo "Status: SKIPPED (harness scaffold — no implementation yet)"
echo "Expected outcomes:"
echo "  - Uninstall stops daemon, unregisters service, removes hook integration"
echo "  - No running daemon process remains"
echo "  - No active service socket remains"
echo "  - No unrelated user shell configuration deleted"
