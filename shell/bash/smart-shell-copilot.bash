# Smart Shell Copilot — Bash hook
#
# Non-blocking Ghost Text completion integration.
# Silently no-ops if the daemon socket is unavailable.

SSC_SOCKET="${SSC_SOCKET:-/tmp/smart-shell-copilot.sock}"
SSC_DEADLINE_MS="${SSC_DEADLINE_MS:-25}"

_ssc_session_id="ssc-$$"
_ssc_request_counter=0
_ssc_active_request_id=""
_ssc_suggestion=""
_ssc_last_key_sec=0
_ssc_debounce_sec=0.030

_ssc_probe() {
    [[ -S "$SSC_SOCKET" ]] || return 1
    return 0
}

_ssc_debounce_should_send() {
    local now
    now="$(python3 -c 'import time; print(time.time())' 2>/dev/null || python -c 'import time; print(time.time())' 2>/dev/null || echo 0)"
    if command -v bc >/dev/null 2>&1; then
        local diff
        diff="$(echo "$now - $_ssc_last_key_sec" | bc 2>/dev/null || echo 1)"
        if (( $(echo "$diff >= $_ssc_debounce_sec" | bc -l 2>/dev/null || echo 1) )); then
            _ssc_last_key_sec="$now"
            return 0
        fi
    fi
    return 1
}

_ssc_request() {
    _ssc_probe || return

    _ssc_request_counter=$(( _ssc_request_counter + 1 ))
    local req_id="${_ssc_session_id}:${_ssc_request_counter}"
    _ssc_active_request_id="$req_id"

    local line="${READLINE_LINE}"
    local cursor="${READLINE_POINT}"
    local cwd="${PWD}"
    local now
    now="$(python3 -c 'import time; print(int(time.time()*1000))' 2>/dev/null || python -c 'import time; print(int(time.time()*1000))' 2>/dev/null || echo 0)"

    local payload="{\"jsonrpc\":\"2.0\",\"id\":\"${req_id}\",\"method\":\"completion.request\",\"params\":{\"session_id\":\"${_ssc_session_id}\",\"shell_kind\":\"bash\",\"line\":\"${line}\",\"cursor\":${cursor},\"cwd\":\"${cwd}\",\"deadline_ms\":${SSC_DEADLINE_MS},\"client_sent_at_ms\":${now}}}"

    printf '%s\n' "$payload" | nc -U -w 1 "$SSC_SOCKET" 2>/dev/null | {
        while IFS= read -r response; do
            _ssc_handle_response "$response" "$req_id"
        done
    } &
}

_ssc_handle_response() {
    local response="$1"
    local expected_id="$2"

    [[ "$response" =~ '"kind":"suggestion"' ]] || return
    [[ "$response" =~ "\"id\":\"${expected_id}\"" ]] || return

    local suffix
    suffix="$(echo "$response" | sed -n 's/.*"suffix":"\([^"]*\)".*/\1/p')"
    [[ -n "$suffix" ]] || return
    [[ "$suffix" =~ [$'\n\r'] ]] && return

    _ssc_suggestion="$suffix"
}

_ssc_clear_suggestion() {
    _ssc_suggestion=""
    _ssc_active_request_id=""
}

_ssc_accept_tab() {
    if [[ -n "$_ssc_suggestion" ]]; then
        READLINE_LINE="${READLINE_LINE}${_ssc_suggestion}"
        READLINE_POINT=${#READLINE_LINE}
        _ssc_clear_suggestion
    else
        bind '"\C-i": complete' 2>/dev/null
    fi
}

_ssc_accept_right() {
    if [[ -n "$_ssc_suggestion" ]]; then
        READLINE_LINE="${READLINE_LINE}${_ssc_suggestion}"
        READLINE_POINT=${#READLINE_LINE}
        _ssc_clear_suggestion
    else
        bind '"\e[C": forward-char' 2>/dev/null
    fi
}

_ssc_bind_key_handler() {
    local line="${READLINE_LINE}"
    local point="${READLINE_POINT}"

    _ssc_clear_suggestion

    if [[ -n "$line" ]] && [[ "$line" =~ [^[:space:]] ]]; then
        _ssc_request
    fi

    if [[ -n "$_ssc_suggestion" ]]; then
        echo -ne "\e[2m${_ssc_suggestion}\e[0m"
    fi
}

if _ssc_probe; then
    bind -x '"\C-i": _ssc_accept_tab' 2>/dev/null
    bind -x '"\e[C": _ssc_accept_right' 2>/dev/null
    bind 'set show-all-if-ambiguous on' 2>/dev/null

    _ssc_record_command() {
        _ssc_probe || return
        local cmd="$BASH_COMMAND"
        local cwd="${PWD}"
        local payload="{\"jsonrpc\":\"2.0\",\"id\":\"${_ssc_session_id}:rec\",\"method\":\"memory.record\",\"params\":{\"session_id\":\"${_ssc_session_id}\",\"cwd\":\"${cwd}\",\"command\":\"${cmd}\"}}"
        printf '%s\n' "$payload" | nc -U -w 1 "$SSC_SOCKET" >/dev/null 2>&1 &
    }
    if declare -F _ssc_probe >/dev/null; then
        trap '_ssc_record_command' DEBUG 2>/dev/null
    fi
fi
