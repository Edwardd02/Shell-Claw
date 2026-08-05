#!/usr/bin/env python3
"""
模拟 Shell Hook → Daemon 的完整请求/响应往返测试。

这个脚本扮演 shell hook 的角色,向守护进程发送真实的 JSON-RPC 请求,
并打印出它返回的每一个响应,方便对照预期。

预备条件: daemon 已启动且 socket 存在于 /tmp/smart-shell-copilot.sock

直接运行:
    python3 tests/integration/simulate_hook.py
"""

import json
import socket
import sys
import time
import os

SOCKET_PATH = os.environ.get("SSC_SOCKET_PATH", "/tmp/smart-shell-copilot.sock")
SESSION_ID = "sim-test"


def send_request(session_id, line, cursor, cwd="/tmp", deadline_ms=2000):
    """打开一个临时 socket 连接,发送单个 JSON-RPC completion.request,读回响应。"""
    req = {
        "jsonrpc": "2.0",
        "id": f"{session_id}:{int(time.time()*1000)}",
        "method": "completion.request",
        "params": {
            "session_id": session_id,
            "shell_kind": "zsh",
            "line": line,
            "cursor": cursor,
            "cwd": cwd,
            "deadline_ms": deadline_ms,
            "client_sent_at_ms": int(time.time() * 1000),
        },
    }
    payload = json.dumps(req) + "\n"

    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCKET_PATH)
    s.sendall(payload.encode("utf-8"))
    s.settimeout(deadline_ms / 1000 + 2.0)
    data = b""
    try:
        data = s.recv(65536)
    except socket.timeout:
        print("  !! 超时,daemon 未返回")
    s.close()
    return payload, data


def parse_response(raw):
    """把原始字节解析成 JSON 对象,并整理成可读格式。"""
    text = raw.decode("utf-8", errors="replace").strip()
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return {"_raw": text}


def render_result(result):
    """把 CompletionResult 变成一眼能看懂的人类可读描述。"""
    kind = result.get("kind", "?")
    if kind == "suggestion":
        return (
            "type=SUGGESTION(建议)\n"
            f"            suffix             = {result['suffix']!r}   <- 就是这段灰色补全\n"
            f"            replacement_start  = {result['replacement_start']}\n"
            f"            valid_for_line_hash= {result['valid_for_line_hash']}\n"
            f"            source             = {result['source']}   <- model=模型, memory=记忆\n"
            f"            daemon_latency_ms  = {result['daemon_latency_ms']} ms"
        )
    elif kind == "none":
        return "type=NONE(无建议)  <- 没匹配到/静默降级,shell 回退原生行为"
    else:
        return f"type=UNKNOWN: {result}"


def run_single(name, line, cursor, cwd="/tmp"):
    print("=" * 70)
    print(f"[场景] {name}")
    print(f"       输入命令: {line!r}  (光标在 {cursor},目录 {cwd})")
    print()
    payload, raw = send_request(SESSION_ID, line, cursor, cwd)
    print(f"  [发出请求] {payload.rstrip()}")
    print()
    resp = parse_response(raw)
    result = resp.get("result", {})
    print(f"  [收到响应] kind = {result.get('kind','?')}")
    if result.get("kind") == "suggestion":
        full = line + result["suffix"]
        print(f"   ┌ 补全结果: {line}{result['suffix']}")
        print(f"   │         ↑ 补全为完整命令: {full}")
        print(f"   └ 验证:")
        print(f"       . suffix 是否单行(无\\n\\r): {'✓ 是' if not any(c in result['suffix'] for c in '\\n\\r\\0') else '✗ 否'}")
    print()
    print("  [详情]")
    print(f"    {render_result(result)}")
    print()


def main():
    if not os.path.exists(SOCKET_PATH):
        print(f"✗ 找不到 socket: {SOCKET_PATH}")
        print("  请先启动 daemon:")
        print("  cd <项目根目录> && ./target/release/daemon")
        sys.exit(1)

    print(f"使用 socket: {SOCKET_PATH}")
    print(f"daemon 在线 ✓\n")

    # ---- 探测模型能力边界 ----
    # 注意:下面这些命令在 /tmp 的记忆里基本没有,大概率走模型推理,
    # 因此主要观察"模型能否生成语法正确的命令"。

    # A. ls 系列(你提到的)
    run_single("ls 部分输入", "ls -", 4)              # 建议 ls 的常用参数
    run_single("ls 完整+参数", "ls -la", 6)            # 已打全,看要不要继续补

    # B. 命令 + 参数(易出空格问题)
    run_single("ps 系列", "ps -", 5)
    run_single("git 带参数", "git commit -", 12)       # 记忆里有 'git commit -m fix'
    run_single("rm 系列", "rm -", 4)

    # C. 常见单条命令补全
    run_single("长短命令混合", "pip install", 11)
    run_single("嵌套/子命令", "systemctl enable", 18)
    run_single("相对常见", "mvn", 3)
    run_single("字符串内容", "python3 -m pyt", 14)     # 后面是模块名

    # D. 危险/需要小心处理的
    run_single("危险命令(应只给参数不给危险后缀)", "sudo rm -rf", 12)

    # E. 空结果/边界
    run_single("空命令(空白行,应无建议)", "   ", 3)
    run_single("无匹配前缀(应无建议)", "zzzxy_unknown", 6)

    print("=" * 70)
    print("测试完成。")


if __name__ == "__main__":
    main()
