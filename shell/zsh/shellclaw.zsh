# ShellClaw — Zsh hook
#
# Non-blocking Ghost Text completion integration using zle -F for async response
# handling inside the zle event loop. Silently no-ops if the daemon socket is
# unavailable.

_ssc_complete_buffer() {
    local original="${_ssc_edit_originals[$WIDGET]:-}"
    [[ -n "$original" ]] || return 1
    _ssc_active_request_id=""
    _ssc_req_line=""
    _ssc_clear_suggestion
    zle "$original"
}

_ssc_wrap_completion_widget() {
    local widget="$1"
    local original="_ssc_original_${widget//-/_}"
    zle -A "$widget" "$original" 2>/dev/null || return 0
    _ssc_edit_originals[$widget]="$original"
    zle -N "$widget" _ssc_complete_buffer
}

_ssc_upgrade_loaded_hook() {
    local tab_binding
    tab_binding="$(bindkey '^I' 2>/dev/null)"
    if [[ "$tab_binding" == *" _ssc_accept_tab" ]]; then
        bindkey '^I' "${_ssc_original_tab:-expand-or-complete}"
    fi

    [[ -n "${_SHELLCLAW_ZSH_LOADED:-}" ]] || return 0

    typeset -gA _ssc_edit_originals
    local completion_widget definition
    for completion_widget in \
        expand-or-complete \
        complete-word \
        menu-complete \
        reverse-menu-complete; do
        definition="$(zle -l -L "$completion_widget" 2>/dev/null)"
        [[ "$definition" == *"_ssc_complete_buffer"* ]] || \
            _ssc_wrap_completion_widget "$completion_widget"
    done
}

# This must run before the load guard so re-sourcing upgrades an open shell
# without re-wrapping self-insert and the other already-initialized widgets.
_ssc_upgrade_loaded_hook

if [[ -n "${_SHELLCLAW_ZSH_LOADED:-}" ]]; then
    return 0
fi
typeset -g _SHELLCLAW_ZSH_LOADED=1

SSC_SOCKET="${SSC_SOCKET:-${SHELLCLAW_DATA_DIR:-$HOME/.shellclaw}/daemon.sock}"
SSC_DEADLINE_MS="${SSC_DEADLINE_MS:-1500}"

_ssc_now_ms() {
    zmodload zsh/datetime 2>/dev/null
    local now
    printf -v now '%.0f' "$(( EPOCHREALTIME * 1000 ))"
    print -r -- "$now"
}

# ---- 调试日志 ----
# SSC_DEBUG=1 时,把 hook 发出的每个请求和收到的每个响应都追加到
# $SSC_HOOK_LOG(默认 $PROJECT_ROOT/logs/hook.log),方便排查问题。
SSC_DEBUG="${SSC_DEBUG:-0}"
if [[ -z "$SSC_HOOK_LOG" ]]; then
    # 由 hook 脚本路径推导项目根目录(…/shell/zsh/xxx.zsh 向上两级)
    case "$0" in
        /*/*) _ssc_hook_root="${0%/shell/zsh/*}"; SSC_HOOK_LOG="$_ssc_hook_root/logs/hook.log" ;;
        *)    SSC_HOOK_LOG="$HOME/.shellclaw/hook.log" ;;
    esac
fi

# ---- 可选交互日志 ----
# 默认关闭，避免把命令中的令牌或私有路径持久化。仅在用户显式设置
# SSC_INTERACTION_LOG_ENABLED=1 时记录。
SSC_INTERACTION_LOG_ENABLED="${SSC_INTERACTION_LOG_ENABLED:-0}"
if [[ -z "$SSC_INTERACTION_LOG" ]]; then
    case "$0" in
        /*/*) _ssc_hook_root2="${0%/shell/zsh/*}"; SSC_INTERACTION_LOG="$_ssc_hook_root2/logs/interaction.log" ;;
        *)    SSC_INTERACTION_LOG="$HOME/.shellclaw/interaction.log" ;;
    esac
fi

typeset -g _ssc_session_id="ssc-$$"
typeset -g _ssc_request_counter=0
typeset -g _ssc_active_request_id=""
typeset -g _ssc_req_line=""          # 最近一次请求对应的命令行(过期判断)
typeset -g _ssc_suggestion=""
typeset -g _ssc_connected=0
typeset -gHi _ssc_fd=-1
typeset -g _ssc_original_self_insert="_ssc_original_self_insert"
typeset -g _ssc_original_right="forward-char"
typeset -gA _ssc_edit_originals

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
    (( SSC_INTERACTION_LOG_ENABLED )) || return
    local type="$1"
    local text="$2"
    local dir
    dir="$(dirname "$SSC_INTERACTION_LOG" 2>/dev/null)"
    [[ -n "$dir" ]] && mkdir -p "$dir" 2>/dev/null
    printf '%s  %-6s %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$type" "$text" >> "$SSC_INTERACTION_LOG" 2>/dev/null
}

_ssc_json_escape() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//\"/\\\"}"
    value="${value//$'\t'/\\t}"
    value="${value//$'\b'/\\b}"
    value="${value//$'\f'/\\f}"
    value="${value//$'\r'/\\r}"
    value="${value//$'\n'/\\n}"
    print -r -- "$value"
}

_ssc_decode_hex() {
    local hex="$1"
    [[ -n "$hex" && "$hex" != *[^0-9a-fA-F]* && $(( ${#hex} % 2 )) -eq 0 ]] || return 1
    local escaped=""
    local i
    for (( i = 1; i <= ${#hex}; i += 2 )); do
        escaped+="\\x${hex[i,i+1]}"
    done
    printf '%b' "$escaped"
}

_ssc_probe() {
    [[ -S "$SSC_SOCKET" ]]
}

_ssc_open_socket() {
    if (( _ssc_connected )); then
        return 0
    fi
    zmodload zsh/net/socket 2>/dev/null
    if zsocket "$SSC_SOCKET" 2>/dev/null; then
        _ssc_fd="${REPLY}"
        _ssc_connected=1
        zle -F "$_ssc_fd" _ssc_handle_response 2>/dev/null || true
        return 0
    fi
    _ssc_fd=-1
    _ssc_connected=0
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
    (( CURSOR == ${#line} )) || return
    local cursor="${CURSOR:-0}"
    local cwd="${PWD}"
    local now
    now="$(_ssc_now_ms)"

    local payload
    local escaped_line escaped_cwd
    escaped_line="$(_ssc_json_escape "$line")"
    escaped_cwd="$(_ssc_json_escape "$cwd")"
    payload="{\"jsonrpc\":\"2.0\",\"id\":\"${req_id}\",\"method\":\"completion.request\",\"params\":{\"session_id\":\"${_ssc_session_id}\",\"shell_kind\":\"zsh\",\"line\":\"${escaped_line}\",\"cursor\":${cursor},\"cwd\":\"${escaped_cwd}\",\"deadline_ms\":${SSC_DEADLINE_MS},\"client_sent_at_ms\":${now}}}"

    # Write the request to the persistent socket fd.
    _ssc_log "REQ(hook) >>> ${payload}"
    _ssc_log_interaction "INPUT" "$line"
    # 静默写入:若 fd 已失效(daemon 重启/socket 断开),写会 failed 并可能打印
    # broken pipe 污染终端。捕获错误:不打印、不报错,仅重置连接状态,下次请求
    # 重新连接。shell 输入绝不能因 daemon 问题被打断。
    if ! { print -u "$_ssc_fd" -r -- "$payload"; } 2>/dev/null; then
        _ssc_connected=0
        zle -F "$_ssc_fd" 2>/dev/null || true
        builtin exec {_ssc_fd}>&- 2>/dev/null || true
        _ssc_fd=-1
        _ssc_log "write failed (broken pipe); reconnecting next request"
    fi
}

_ssc_handle_response() {
    # Called by zle -F when fd $_ssc_fd is readable. Reads one JSON line.
    local expected_id="${_ssc_active_request_id}"

    local response=""
    IFS= read -u "$_ssc_fd" -r response 2>/dev/null
    if [[ -z "$response" ]]; then
        zle -F "$_ssc_fd" 2>/dev/null || true
        builtin exec {_ssc_fd}>&- 2>/dev/null || true
        _ssc_fd=-1
        _ssc_connected=0
        return
    fi

    _ssc_log "RESP(hook) <<< ${response}"

    # 只处理最新请求的响应(id 不匹配 = 过期,丢弃)。
    # 注意:不要仅在 zle -F 异步回调里读 BUFFER 判断过期 —— 回调到达时编辑行
    # 可能已变化。先用 request id 丢弃过期响应，再在渲染时核对 BUFFER/CURSOR。
    [[ "$response" == *"\"id\":\"${expected_id}\""* ]] || return

    # "none" 也当作一次确定的覆盖:清空当前建议(而非保留旧的)。
    if [[ "$response" != *'"kind":"suggestion"'* ]]; then
        _ssc_clear_suggestion
        return
    fi

    local suffix_hex suffix
    suffix_hex="${${response#*\"suffix_hex\":\"}%%\"*}"
    [[ "$suffix_hex" != "$response" ]] || { _ssc_clear_suggestion; return; }
    suffix="$(_ssc_decode_hex "$suffix_hex")"
    [[ -z "$suffix" ]] && { _ssc_clear_suggestion; return; }
    [[ "$suffix" == *$'\n'* || "$suffix" == *$'\r'* ]] && { _ssc_clear_suggestion; return; }

    _ssc_suggestion="$suffix"
    _ssc_log "RENDER(hook) suffix=${suffix}"
    _ssc_log_interaction "OUTPUT" "$suffix"
    zle _ssc_apply_suggestion 2>/dev/null || _ssc_clear_suggestion
}

# 拼接 base + suggestion,避免双空格:
# 当 base 已以空格结尾(s"git ")且 suggestion 以空格开头(s" diff")
# 时,去掉 suggestion 的一个前导空格,否则会拼出 "git  diff" 双空格。
_ssc_join_suggestion() {
    local base="$1"
    local sugg="$2"
    if [[ "$base" == *" " ]] && [[ "$sugg" == " "* ]]; then
        sugg="${sugg# }"
    fi
    print -r -- "${base}${sugg}"
}

_ssc_render_suggestion() {
    local n=${#_ssc_suggestion}
    if (( n > 0 )) && [[ -n "$_ssc_suggestion" ]]; then
        [[ "$BUFFER" == "$_ssc_req_line" ]] && (( CURSOR == ${#BUFFER} )) || {
            _ssc_clear_suggestion
            return
        }
        local display="$_ssc_suggestion"
        if [[ "$BUFFER" == *" " && "$display" == " "* ]]; then
            display="${display# }"
        fi
        POSTDISPLAY="$display"
        region_highlight=("${(@)region_highlight:#*memo=shellclaw}")
        region_highlight+=("P${#BUFFER} $(( ${#BUFFER} + ${#display} )) fg=8 memo=shellclaw")
        zle -R
    fi
}

_ssc_apply_suggestion() {
    _ssc_render_suggestion
}

_ssc_clear_suggestion() {
    _ssc_suggestion=""
    POSTDISPLAY=""
    region_highlight=("${(@)region_highlight:#*memo=shellclaw}")
    zle -R 2>/dev/null || true
}

_ssc_after_buffer_edit() {
    _ssc_clear_suggestion
    if [[ -n "$BUFFER" && "$BUFFER" =~ [^[:space:]] ]] && (( CURSOR == ${#BUFFER} )); then
        _ssc_request
    else
        _ssc_active_request_id=""
        _ssc_req_line=""
    fi
}

_ssc_self_insert() {
    zle "$_ssc_original_self_insert"
    _ssc_after_buffer_edit
}

_ssc_edit_buffer() {
    local original="${_ssc_edit_originals[$WIDGET]:-}"
    [[ -n "$original" ]] || return 1
    zle "$original"
    _ssc_after_buffer_edit
}

_ssc_wrap_edit_widget() {
    local widget="$1"
    local original="_ssc_original_${widget//-/_}"
    zle -A "$widget" "$original" 2>/dev/null || return 0
    _ssc_edit_originals[$widget]="$original"
    zle -N "$widget" _ssc_edit_buffer
}

_ssc_accept() {
    if [[ -n "$_ssc_suggestion" && "$BUFFER" == "$_ssc_req_line" ]] && (( CURSOR == ${#BUFFER} )); then
        BUFFER="$(_ssc_join_suggestion "$BUFFER" "$_ssc_suggestion")"
        CURSOR=${#BUFFER}
        # 接受后失效所有在途请求:若之前请求的响应晚到,_ssc_active_request_id
        # 已被置空,响应 id 不匹配会丢弃,不会复活 _ssc_suggestion 造成叠加。
        _ssc_active_request_id=""
        _ssc_req_line=""
        _ssc_clear_suggestion
        _ssc_suggestion=""
        zle -R
    else
        zle "$1"
    fi
}

_ssc_accept_right_arrow() {
    _ssc_accept "$_ssc_original_right"
}

_ssc_init() {
    zle -A self-insert "$_ssc_original_self_insert" 2>/dev/null || return 1
    local right_binding
    right_binding="$(bindkey '^[[C' 2>/dev/null)"
    _ssc_original_right="${${(z)right_binding}[2]:-forward-char}"
    zle -N self-insert _ssc_self_insert
    zle -N _ssc_apply_suggestion _ssc_apply_suggestion
    zle -N _ssc_accept_right_arrow _ssc_accept_right_arrow
    bindkey '^[[C' _ssc_accept_right_arrow

    local edit_widget
    for edit_widget in \
        backward-delete-char \
        delete-char-or-list \
        backward-kill-word \
        kill-word \
        kill-line \
        kill-whole-line \
        vi-backward-delete-char \
        vi-delete-char \
        vi-backward-kill-word \
        vi-kill-line; do
        _ssc_wrap_edit_widget "$edit_widget"
    done

    local completion_widget
    for completion_widget in \
        expand-or-complete \
        complete-word \
        menu-complete \
        reverse-menu-complete; do
        _ssc_wrap_completion_widget "$completion_widget"
    done

    _ssc_open_socket || {
        command -v shellclaw >/dev/null 2>&1 && shellclaw start >/dev/null 2>&1 &!
    }

    autoload -Uz add-zsh-hook
    add-zsh-hook preexec _ssc_record_command
    return 0
}

_ssc_record_command() {
    _ssc_probe || return

    local cmd="$1"
    local cwd="${PWD}"

    local payload escaped_cwd escaped_cmd
    escaped_cwd="$(_ssc_json_escape "$cwd")"
    escaped_cmd="$(_ssc_json_escape "$cmd")"
    payload="{\"jsonrpc\":\"2.0\",\"id\":\"${_ssc_session_id}:rec\",\"method\":\"memory.record\",\"params\":{\"session_id\":\"${_ssc_session_id}\",\"cwd\":\"${escaped_cwd}\",\"command\":\"${escaped_cmd}\"}}"

    {
        printf '%s\n' "$payload" | nc -U -w 1 "$SSC_SOCKET" >/dev/null 2>&1
    } &!
}

if [[ "${SHELLCLAW_ZSH_TESTING:-0}" != "1" ]]; then
    _ssc_init
fi
