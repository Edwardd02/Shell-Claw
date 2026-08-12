#!/bin/sh

set -eu

VERSION="${1:-}"
case "$VERSION" in
    [0-9]*.[0-9]*.[0-9]*) ;;
    *) echo "usage: $0 X.Y.Z" >&2; exit 2 ;;
esac

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
OUT="$TARGET/release-package"
STAGE="$OUT/shellclaw-aarch64-apple-darwin"
ARCHIVE="$OUT/shellclaw-aarch64-apple-darwin.tar.gz"

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
    echo "release packaging currently requires macOS arm64" >&2
    exit 1
fi

cd "$ROOT"
cargo test --workspace
cargo build --release -p shellclaw

rm -rf "$OUT"
mkdir -p "$STAGE/scripts"
install -m 0755 "$TARGET/release/shellclaw" "$STAGE/shellclaw"
install -m 0644 "$ROOT/shell/zsh/shellclaw.zsh" "$STAGE/shellclaw.zsh"
install -m 0644 "$ROOT/shell/bash/shellclaw.bash" "$STAGE/shellclaw.bash"
install -m 0755 "$ROOT/scripts/download-model.sh" "$STAGE/scripts/download-model.sh"

tar -C "$STAGE" -czf "$ARCHIVE" .
shasum -a 256 "$ARCHIVE" > "$ARCHIVE.sha256"

echo "$ARCHIVE"
cat "$ARCHIVE.sha256"
