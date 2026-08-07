# Smart Shell Copilot — Zsh hook
#
# Non-blocking Ghost Text completion integration using zle -F for async response
# handling inside the zle event loop. Silently no-ops if the daemon socket is
# unavailable.

SSC_SOCKET="${SSC_SOCKET:-/tmp/smart-shell-copilot.sock}"
SSC_DEADLINE_MS="${SSC_DEADLINE_MS:-2000}"

# ---- 调试日志 ----
# SSC_DEBUG=1 时,把 hook 发出的每个请求和收到的每个响应都追加到
# $SSC_HOOK_LOG(默认 $PROJECT_ROOT/logs/hook.log),方便排查问题。
SSC_DEBUG="${SSC_DEBUG:-0}"
if [[ -z "$SSC_HOOK_LOG" ]]; then
    # 由 hook 脚本路径推导项目根目录(…/shell/zsh/xxx.zsh 向上两级)
    case "$0" in
        /*/*) _ssc_hook_root="${0%/shell/zsh/*}"; SSC_HOOK_LOG="$_ssc_hook_root/logs/hook.log" ;;
        *)    SSC_HOOK_LOG="$HOME/smart-shell-copilot-hook.log" ;;
    esac
fi

# ---- 交互日志(始终记录,不依赖 SSC_DEBUG)----
# 人类可读:一行 时间戳 + INPUT(用户命令行输入),一行 时间戳 + OUTPUT(模型建议)。
# 写到 $SSC_INTERACTION_LOG(默认项目根 logs/interaction.log)。
if [[ -z "$SSC_INTERACTION_LOG" ]]; then
    case "$0" in
        /*/*) _ssc_hook_root2="${0%/shell/zsh/*}"; SSC_INTERACTION_LOG="$_ssc_hook_root2/logs/interaction.log" ;;
        *)    SSC_INTERACTION_LOG="$HOME/smart-shell-copilot-interaction.log" ;;
    esac
fi

typeset -g _ssc_session_id="ssc-$$"
typeset -g _ssc_request_counter=0
typeset -g _ssc_active_request_id=""
typeset -g _ssc_req_line=""          # 最近一次请求对应的命令行(过期判断)
typeset -g _ssc_suggestion=""
typeset -g _ssc_last_key_ms=0
typeset -g _ssc_connected=0
typeset -gHi _ssc_fd=-1

# 追加一行到 hook 日志(仅 SSC_DEBUG=1 时)。
_ssc_log() {
    (( SSC_DEBUG )) || return
    local dir
    dir="$(dirname "$SSC_HOOK_LOG" 2>/dev/null)"
    [[ -n "$dir" ]] && mkdir -p "$dir" 2>/dev/null
    printf '%s\t%s\n' "$(date '+%Y-%m-%dT%H:%M:%S')" "$1" >> "$SSC_HOOK_LOG" 2>/dev/null
}

# 追加一行到交互日志(始终记录)。type 为 "INPUT" 或 "OUTPUT"。
_ssc_log_interaction() {
    local type="$1"
    local text="$2"
    local dir
    dir="$(dirname "$SSC_INTERACTION_LOG" 2>/dev/null)"
    [[ -n "$dir" ]] && mkdir -p "$dir" 2>/dev/null
    printf '%s  %-6s %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$type" "$text" >> "$SSC_INTERACTION_LOG" 2>/dev/null
}

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
    _ssc_req_line="${BUFFER:-}"      # 记录本次请求对应的命令行,过期判断用

    local line="${BUFFER:-}"
    local cursor="${CURSOR:-0}"
    local cwd="${PWD}"
    local now
    now="$(perl -e 'print time*1000' 2>/dev/null || echo $(( $(date +%s) * 1000 )))"

    local payload
    payload="{\"jsonrpc\":\"2.0\",\"id\":\"${req_id}\",\"method\":\"completion.request\",\"params\":{\"session_id\":\"${_ssc_session_id}\",\"shell_kind\":\"zsh\",\"line\":\"${line}\",\"cursor\":${cursor},\"cwd\":\"${cwd}\",\"deadline_ms\":${SSC_DEADLINE_MS},\"client_sent_at_ms\":${now}}}"

    # Write the request to the persistent socket fd.
    _ssc_log "REQ(hook) >>> ${payload}"
    _ssc_log_interaction "INPUT" "$line"
    print -u "$_ssc_fd" -r -- "$payload"
}

_ssc_handle_response() {
    # Called by zle -F when fd $_ssc_fd is readable. Reads one JSON line.
    local expected_id="${_ssc_active_request_id}"

    local response=""
    IFS= read -u "$_ssc_fd" -r response 2>/dev/null
    [[ -n "$response" ]] || return

    _ssc_log "RESP(hook) <<< ${response}"

    # 只处理最新请求的响应(id 不匹配 = 过期,丢弃)。
    # 注意:不要在 zle -F 异步回调里读 BUFFER 判断过期 —— 该回调里 BLUEFFER
    # 不一定是当前编辑行(经常为空),会导致建议被误判过期而清空、闪烁。
    # 过期判定完全交给 _ssc_active_request_id(id 唯一) + self-insert 时清空。
    [[ "$response" == *"\"id\":\"${expected_id}\""* ]] || return

    # "none" 也当作一次确定的覆盖:清空当前建议(而非保留旧的)。
    if [[ "$response" != *'"kind":"suggestion"'* ]]; then
        _ssc_clear_suggestion
        return
    fi

    local suffix
    suffix="$(printf '%s' "$response" | grep -o '"suffix":"[^"]*"' | sed 's/"suffix":"//;s/"$//')"
    [[ -z "$suffix" ]] && { _ssc_clear_suggestion; return; }
    [[ "$suffix" == *$'\n'* || "$suffix" == *$'\r'* ]] && { _ssc_clear_suggestion; return; }

    _ssc_suggestion="$suffix"
    _ssc_log "RENDER(hook) suffix=${suffix}"
    _ssc_log_interaction "OUTPUT" "$suffix"
    _ssc_render_suggestion
}

_ssc_render_suggestion() {
    local n=${#_ssc_suggestion}
    if (( n > 0 )) && [[ -n "$_ssc_suggestion" ]]; then
        # 渲染用"请求时的行"(_ssc_req_line),而非 zle -F 异步回调里的 $BUFFER
        # (后者在异步上下文常为空/不正确,是闪烁和无输出的大根因)。
        local base="${_ssc_req_line}"
        region_highlight+=("${#base} $(( ${#base} + n )) fg=8")
        zle -R -- "${base}${_ssc_suggestion}"
    fi
}

_ssc_clear_suggestion() {
    if [[ -n "$_ssc_suggestion" ]]; then
        _ssc_suggestion=""
        zle -R -- "$BUFFER" 2>/dev/null || true
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
