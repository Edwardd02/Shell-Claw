# Smart Shell Copilot — Zsh hook
#
# Non-blocking Ghost Text completion integration using zle -F for async response
# handling inside the zle event loop. Silently no-ops if the daemon socket is
# unavailable.

SSC_SOCKET="${SSC_SOCKET:-/tmp/smart-shell-copilot.sock}"
SSC_DEADLINE_MS="${SSC_DEADLINE_MS:-2000}"

typeset -g _ssc_session_id="ssc-$$"
typeset -g _ssc_request_counter=0
typeset -g _ssc_active_request_id=""
typeset -g _ssc_suggestion=""
typeset -g _ssc_last_key_ms=0
typeset -g _ssc_connected=0
typeset -gHi _ssc_fd=-1

_ssc_probe() {
    [[ -S "$SSC_SOCKET" ]] || return 1
    command -v nc >/dev/null 2>&1 && return 0
    command -v curl >/dev/null 2>&1 && return 0
    return 1
}

_ssc_open_socket() {
    if (( _ssc_connected )); then
        return 0
    fi
    zmodload zsh/net/socket 2>/dev/null
    if zsocket "$SSC_SOCKET" 2>/dev/null; then
        _ssc_fd="${REPLY}"
        _ssc_connected=1
        return 0
    fi
    _ssc_fd=-1
    _ssc_connected=0
    return 1
}

_ssc_debounce_should_send() {
    local now
    now="$(perl -e 'print time*1000' 2>/dev/null || echo $(( $(date +%s) * 1000 )))"
    if (( now - _ssc_last_key_ms >= 30 )); then
        _ssc_last_key_ms=$now
        return 0
    fi
    return 1
}

_ssc_request() {
    _ssc_probe || return
    _ssc_open_socket || return

    _ssc_request_counter=$(( _ssc_request_counter + 1 ))
    local req_id="${_ssc_session_id}:${_ssc_request_counter}"
    _ssc_active_request_id="$req_id"

    local line="${BUFFER:-}"
    local cursor="${CURSOR:-0}"
    local cwd="${PWD}"
    local now
    now="$(perl -e 'print time*1000' 2>/dev/null || echo $(( $(date +%s) * 1000 )))"

    local payload
    payload="{\"jsonrpc\":\"2.0\",\"id\":\"${req_id}\",\"method\":\"completion.request\",\"params\":{\"session_id\":\"${_ssc_session_id}\",\"shell_kind\":\"zsh\",\"line\":\"${line}\",\"cursor\":${cursor},\"cwd\":\"${cwd}\",\"deadline_ms\":${SSC_DEADLINE_MS},\"client_sent_at_ms\":${now}}}"

    # Write the request to the persistent socket fd.
    print -u "$_ssc_fd" -r -- "$payload"
}

_ssc_handle_response() {
    # Called by zle -F when fd $_ssc_fd is readable. Reads one JSON line.
    local expected_id="${_ssc_active_request_id}"

    local response=""
    IFS= read -u "$_ssc_fd" -r response 2>/dev/null
    [[ -n "$response" ]] || return

    # Only accept a suggestion for the latest request.
    [[ "$response" == *'"kind":"suggestion"'* ]] || return
    [[ "$response" == *"\"id\":\"${expected_id}\""* ]] || return

    local suffix
    suffix="$(printf '%s' "$response" | grep -o '"suffix":"[^"]*"' | sed 's/"suffix":"//;s/"$//')"
    [[ -z "$suffix" ]] && return
    [[ "$suffix" == *$'\n'* || "$suffix" == *$'\r'* ]] && return

    _ssc_suggestion="$suffix"
    _ssc_render_suggestion
}

_ssc_render_suggestion() {
    local n=${#_ssc_suggestion}
    if (( n > 0 )) && [[ -n "$_ssc_suggestion" ]]; then
        region_highlight+=("$CURSOR $(( CURSOR + n )) fg=8")
        zle -R "${BUFFER}${_ssc_suggestion}"
    fi
}

_ssc_clear_suggestion() {
    if [[ -n "$_ssc_suggestion" ]]; then
        _ssc_suggestion=""
        zle -R "$BUFFER" 2>/dev/null || true
    fi
}

_ssc_self_insert() {
    zle .self-insert
    _ssc_clear_suggestion

    if _ssc_debounce_should_send && [[ -n "$BUFFER" ]] && [[ "$BUFFER" =~ [^[:space:]] ]]; then
        _ssc_request
    fi
}

_ssc_accept() {
    if [[ -n "$_ssc_suggestion" ]]; then
        BUFFER="${BUFFER}${_ssc_suggestion}"
        CURSOR=${#BUFFER}
        _ssc_clear_suggestion
        zle -R
    else
        zle ."$1"
    fi
}

_ssc_accept_tab() {
    _ssc_accept expand-or-complete
}

_ssc_accept_right_arrow() {
    _ssc_accept forward-char
}

_ssc_init() {
    if _ssc_probe; then
        _ssc_open_socket

        zle -N self-insert _ssc_self_insert
        zle -N _ssc_accept_tab _ssc_accept_tab
        zle -N _ssc_accept_right_arrow _ssc_accept_right_arrow
        bindkey '^I' _ssc_accept_tab
        bindkey '^[[C' _ssc_accept_right_arrow

        if (( _ssc_connected )); then
            zle -F "$_ssc_fd" _ssc_handle_response
        fi

        autoload -Uz add-zsh-hook
        add-zsh-hook preexec _ssc_record_command
        return 0
    fi
    return 1
}

_ssc_record_command() {
    _ssc_probe || return

    local cmd="$1"
    local cwd="${PWD}"

    local payload
    payload="{\"jsonrpc\":\"2.0\",\"id\":\"${_ssc_session_id}:rec\",\"method\":\"memory.record\",\"params\":{\"session_id\":\"${_ssc_session_id}\",\"cwd\":\"${cwd}\",\"command\":\"${cmd}\"}}"

    {
        printf '%s\n' "$payload" | nc -U -w 1 "$SSC_SOCKET" >/dev/null 2>&1
    } &!
}

_ssc_init
