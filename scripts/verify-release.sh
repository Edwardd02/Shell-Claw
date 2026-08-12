#!/bin/sh

set -eu

ARCHIVE="${1:-}"
[ -f "$ARCHIVE" ] || { echo "usage: $0 /path/to/archive.tar.gz" >&2; exit 2; }

TMP=$(mktemp -d "${TMPDIR:-/tmp}/shellclaw-verify.XXXXXX")
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
tar -xzf "$ARCHIVE" -C "$TMP"

for path in shellclaw shellclaw.zsh shellclaw.bash scripts/download-model.sh; do
    [ -f "$TMP/$path" ] || { echo "missing release file: $path" >&2; exit 1; }
done

"$TMP/shellclaw" --version
zsh -n "$TMP/shellclaw.zsh"
bash -n "$TMP/shellclaw.bash"
sh -n "$TMP/scripts/download-model.sh"
echo "release archive verified"
