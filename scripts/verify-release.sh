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
SHELLCLAW_ZSH_TESTING=1 zsh -f -c '
    source "$1"
    now="$(_ssc_now_ms)"
    [[ "$now" == <-> ]]
    [[ "$SSC_DEADLINE_MS" == <-> && "$SSC_DEADLINE_MS" -ge 1000 ]]
    BUFFER="shellclaw st"
    CURSOR=${#BUFFER}
    _ssc_req_line="$BUFFER"
    _ssc_suggestion="atus"
    _ssc_render_suggestion
    [[ "$POSTDISPLAY" == "atus" ]]
    _ssc_request() { test_request_line="$BUFFER"; }
    BUFFER="shellclaw s"
    CURSOR=${#BUFFER}
    _ssc_suggestion="stale"
    POSTDISPLAY="stale"
    _ssc_after_buffer_edit
    [[ -z "$_ssc_suggestion" && -z "$POSTDISPLAY" ]]
    [[ "$test_request_line" == "shellclaw s" ]]
    _ssc_open_socket() { return 0; }
    _ssc_init
    [[ "$(bindkey "^@")" == *" _ssc_accept_ctrl_space" ]]
    [[ "$(bindkey "^I")" == *" expand-or-complete" ]]
    [[ "$(bindkey "^[[C")" == *" forward-char" ]]
' sh "$TMP/shellclaw.zsh"
bash -n "$TMP/shellclaw.bash"
sh -n "$TMP/scripts/download-model.sh"
echo "release archive verified"
