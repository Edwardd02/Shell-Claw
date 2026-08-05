# Smart Shell Copilot — 内部文档

## 项目结构总览

```
Shell Claw/
├── Cargo.toml                    # Rust workspace 根：包含 3 个子 crate
├── rustfmt.toml                  # 代码格式化配置（max_width=100）
├── .cargo/config.toml            # 编译优化（target-cpu=native）
├── .gitignore                    # 忽略 target/, *.gguf, *.sock, *.db, *.log
├── .github/workflows/ci.yml     # CI 流水线
│
├── crates/
│   ├── daemon/                   # ★ 核心：Rust 异步守护进程（唯一二进制产物）
│   │   └── src/
│   │       ├── main.rs           # 入口：加载配置 → 初始化日志 → 启动 IPC + 调度器
│   │       ├── config/           # 从环境变量读取配置（socket 路径、模型路径、数据库路径等）
│   │       ├── diagnostics/      # tracing 日志系统（写文件，绝不输出到终端）
│   │       ├── ipc/
│   │       │   ├── server.rs     # Unix domain socket 监听、连接管理、行分帧
│   │       │   └── handler.rs    # JSON-RPC 消息解析和方法分发
│   │       ├── scheduler/
│   │       │   ├── mod.rs        # 全局单例调度器：请求提交、取消、请求 ID 生成
│   │       │   ├── validate.rs   # 请求校验：行长度、光标、工作目录、非法字符
│   │       │   ├── deadline.rs   # 截止时间追踪器
│   │       │   └── noop.rs       # "无建议"响应生成
│   │       ├── memory/
│   │       │   ├── mod.rs        # MemoryStore trait 定义 + 数据类型
│   │       │   ├── schema.rs     # SQL DDL：command_history 表 + command_fts 虚拟表
│   │       │   ├── db.rs         # SQLite 数据库封装（WAL 模式、FTS5）
│   │       │   ├── record.rs     # 命令录入（插入/更新使用次数）
│   │       │   ├── retrieve.rs   # ★ 混合排序核心：BM25 + 目录相关性 + 频率 + 时间衰减
│   │       │   └── store.rs      # MemoryStore trait 的具体实现
│   │       └── model/
│   │           ├── mod.rs        # CompletionModel trait 定义
│   │           ├── adapter.rs    # llama.cpp 适配器实现
│   │           ├── context.rs    # 模型上下文构建器（限 5 个候选、512 tokens）
│   │           ├── grammar.rs    # GBNF 单行语法限制 + 输出校验
│   │           ├── validate.rs   # 后缀校验（拒绝多行、Markdown、解释性文本）
│   │           ├── safe_wrapper.rs # FFI 安全边界（空指针检查、缓冲区校验）
│   │           └── warmup.rs     # 模型预热标志
│   ├── protocol/                 # JSON-RPC 2.0 协议类型（serde 序列化）
│   └── bench-harness/            # 基准测试 crate 骨架
│
├── shell/
│   ├── zsh/smart-shell-copilot.zsh   # Zsh 钩子脚本（Ghost Text + Tab/方向键接受）
│   └── bash/smart-shell-copilot.bash # Bash 钩子脚本
│
├── packaging/
│   ├── homebrew/smart-shell-copilot.rb   # Homebrew Formula（安装/卸载/服务管理）
│   ├── launchd/com.smart-shell-copilot.daemon.plist  # macOS 守护配置
│   └── systemd/smart-shell-copilot.service  # Linux systemd 用户服务
│
├── models/
│   └── README.md                 # 模型获取说明
│
├── tests/
│   ├── unit/daemon/              # 12 个 Rust 单元测试文件
│   ├── unit/zsh/                 # 2 个 Zsh 行为测试脚本
│   ├── integration/              # 5 个 Rust + 1 个 shell 集成测试
│   ├── compat/                   # Ghost Text UI 验证
│   ├── benchmarks/               # 5 个性能门控脚本
│   ├── packaging/                # 安装/卸载验证脚本
│   └── fixtures/                 # 测试数据生成脚本
│
├── specs/001-smart-shell-copilot/
│   ├── spec.md                   # 功能规格说明
│   ├── plan.md                   # 实现计划
│   ├── tasks.md                  # 131 个任务分解
│   ├── research.md               # 技术选型决策记录
│   ├── data-model.md             # 数据模型定义
│   ├── quickstart.md             # 快速验证指南
│   └── contracts/                # 5 个接口契约文件
│
├── .specify/memory/constitution.md  # 项目宪章（治理约束）
└── .agents/skills/                   # Speckit 开发工作流技能
```

---

## 数据流（请求生命周期的完整路径）

```
用户在终端打字 → 触发 Zle/Readline 回调
       │
       ▼
  shell/(zsh|bash)/smart-shell-copilot.zsh
       │
       ├── 30ms 去抖动防抖
       ├── 构建 JSON-RPC 请求包
       ├── nc -U /tmp/smart-shell-copilot.sock 非阻塞发送
       │   （&! 后台运行，不阻塞 shell 主线程）
       └── 收到响应后：
           ├── 如果 kind=suggestion → 渲染灰色 Ghost Text
           ├── 如果 kind=none → 什么都不做（静默降级）
           ├── Tab / 右箭头 → 接受建议并插入命令行
           └── 继续打字 → 清除旧建议
       │
       │  Unix Domain Socket
       ▼
  crates/daemon/src/ipc/server.rs
       │
       ├── 接受连接，拆分为读写半通道
       ├── 读取换行分隔的 JSON 帧
       └── 调用 handler::dispatch()
       │
       ▼
  crates/daemon/src/ipc/handler.rs
       │
       ├── 解析 JSON → JsonRpcMessage(Request | Cancel)
       ├── completion.request → scheduler::submit_completion()
       └── session.cancel → scheduler::cancel_request()
       │
       ▼
  crates/daemon/src/scheduler/mod.rs
       │
       ├── 1. RequestValidator::validate()
       │      - 行长度 ≤ 4096、光标合法、CWD 为绝对路径、无 NUL 字符
       │
       ├── 2. DeadlineTracker 检查是否超时
       │
       ├── 3. memory::retrieve::retrieve()  ← SQLite FTS5
       │      - 构建 FTS5 前缀查询
       │      - 混合排序 = BM25×0.40 + CWD匹配×0.25 + log(频率)×0.20 + 时间衰减×0.15
       │      - 返回排序后的候选列表
       │
       ├── 4. model::adapter::complete_suffix()  ← llama.cpp
       │      - 构建 ModelContext（限制 5 个候选、512 tokens）
       │      - 应用 GBNF 单行语法约束
       │      - 生成后缀
       │      - model::validate::validate_suffix() 后校验
       │
       ├── 5. 组装 CompletionResponse
       └── 6. 如果任何步骤失败 → kind=none（静默降级）
```

---

## 各模块详细说明

### `crates/daemon/` — 核心守护进程

| 文件 | 一句话说明 |
|------|-----------|
| `main.rs` | 加载配置 → 初始化文件日志 → 在 Unix Socket 上启动 IPC 服务器 + 调度器后台任务 |
| `config/mod.rs` | 从环境变量（`SSC_SOCKET_PATH`, `SSC_MODEL_PATH`, `SSC_DATA_DIR` 等）读取配置，带合理默认值 |
| `diagnostics/mod.rs` | tracing 日志初始化（写入文件，绝不打印到终端），定义 IPC/内存/模型三个指标结构体 |
| `ipc/server.rs` | Unix domain socket 监听、并发连接处理、JSON 帧读写 |
| `ipc/handler.rs` | 解析 JSON-RPC → 调度到 scheduler，返回响应给 shell hook |
| `scheduler/mod.rs` | 全局单例调度器（OnceLock），管理请求 ID、校验、截止时间、协调 memory 和 model |
| `scheduler/validate.rs` | 请求合法性校验：长度、光标、目录、非法字符 |
| `scheduler/deadline.rs` | 基于 Instant 的截止时间检查器 |
| `memory/schema.rs` | SQL 常量：建表、插入、FTS5 检索查询 |
| `memory/db.rs` | SQLite 封装（rusqlite + WAL 模式 + FTS5） |
| `memory/retrieve.rs` | **核心排序算法**——BM25 文本匹配 + 目录相关性 + 使用频率对数 + 指数时间衰减 |
| `memory/record.rs` | 执行命令后记录到本地历史 |
| `model/adapter.rs` | llama.cpp 的 Rust 适配器，处理加载、取消、超时、校验 |
| `model/grammar.rs` | GBNF 单行语法 `[^\r\n\x00]+`，防止模型输出多行/Markdown |
| `model/validate.rs` | 后缀输出后校验：拒绝空、重复、多行、解释性文本 |
| `model/safe_wrapper.rs` | FFI 安全边界：空指针检查、缓冲区大小校验 |

### 混合排序算法详解（`memory/retrieve.rs`）

```
Score = 0.40 × BM25(查询文本, 历史命令)
      + 0.25 × CWD匹配度（同一目录=1.0, 子目录=0.5, 父目录=0.3, 无关=0.0）
      + 0.20 × log(1 + 使用次数)
      + 0.15 × e^(-λ × Δtime / 86400)
```

按 final_score 降序排列返回。

### `crates/protocol/` — 协议定义

定义所有 JSON-RPC 2.0 序列化类型：
- `CompletionRequest` / `CompletionResponse`（结果用 tagged enum: `suggestion` 或 `none`）
- `CancelRequest` / `CancelResponse`
- `SuggestionSource`（区分 model 还是 memory 来源）

### `shell/` — Shell 钩子

**Zsh 钩子** (`shell/zsh/smart-shell-copilot.zsh`)：
- 通过 `zle -N self-insert` 拦截每次按键
- 30ms 去抖后，通过 `nc -U` 发送 JSON-RPC 请求（`&!` 后台执行）
- 收到有效建议后通过 `zle -R` 渲染 Ghost Text
- Tab → 调用 `_ssc_accept expand-or-complete`（有建议接受，无建议走原生）
- 右箭头 → 调用 `_ssc_accept forward-char`
- 所有其他快捷键（Ctrl+C/D, Up/Down, Ctrl+A/E/U/K）全部保留原生行为

**Bash 钩子** (`shell/bash/smart-shell-copilot.bash`)：
- 使用 `READLINE_LINE` / `READLINE_POINT` 和 `bind -x`
- 逻辑与 Zsh 相同，适配 bash 的 readline 机制

### `packaging/` — 打包和部署

| 文件 | 作用 |
|------|------|
| `homebrew/smart-shell-copilot.rb` | Homebrew Formula：平台感知 URL、构建安装、服务启动、hook 注册、卸载清理 |
| `launchd/com.smart-shell-copilot.daemon.plist` | macOS 守护配置：登录启动、KeepAlive、Nice=10、日志重定向 |
| `systemd/smart-shell-copilot.service` | Linux systemd 用户服务：Restart=always、Nice=10、日志追加 |

---

## 如何部署和运行

### 1. 编译
```bash
# 前置条件：Rust 1.80+、SQLite（rusqlite 自带 bundled 版本）
cargo build --release
```

### 2. 下载模型
```bash
# 从 HuggingFace 下载 GGUF 格式模型文件，放到 models/ 目录
# 主模型：Qwen/Qwen3-0.6B-Base (~500MB)
# 备用：Qwen2.5-Coder-0.5B-Instruct
```

### 3. 手动启动守护进程
```bash
SSC_SOCKET_PATH=/tmp/smart-shell-copilot.sock \
SSC_MODEL_PATH=models/qwen3-0.6b-base.gguf \
SSC_DATA_DIR=~/.smart-shell-copilot \
cargo run --release -p daemon
```

### 4. 加载 Shell 钩子
```bash
# Zsh
source shell/zsh/smart-shell-copilot.zsh

# Bash
source shell/bash/smart-shell-copilot.bash
```
钩子会自动探测 socket，如果守护进程不可用则静默 no-op。

### 5. 通过 Homebrew 安装（生产环境）
```bash
brew install smart-shell-copilot
# 安装完成即：二进制就位、模型就位、hook 已注册、服务已启动
# 打开新终端即可使用

brew uninstall smart-shell-copilot
# 停止服务、注销注册、清理所有文件，零残留
```

---

## 如何运行测试

```bash
# 所有 Rust 单元测试 + 集成测试
cargo test --workspace

# Zsh 行为测试
zsh tests/unit/zsh/test_acceptance.zsh
zsh tests/unit/zsh/test_fallback.zsh

# 性能基准（骨架已建，完整体现在实现中）
./tests/benchmarks/e2e-latency.sh   # 端到端延迟 ≤30ms
./tests/benchmarks/retrieval.sh     # 检索延迟 ≤3ms
./tests/benchmarks/model-ttft.sh   # 模型首 token ≤15ms
./tests/benchmarks/daemon-resources.sh  # RSS ≤600MB, 空闲 CPU ~0%

# 覆盖率门控
cargo llvm-cov --workspace --fail-under-lines 85
```

---

## 关键架构原则

1. **层边界清晰**：Shell Hook ↔ JSON-RPC ↔ 守护进程 ↔ MemoryStore trait ↔ SQLite；守护进程 ↔ CompletionModel trait ↔ llama.cpp
2. **静默降级**：所有失败路径返回 `kind: none`，终端零错误输出，原生 shell 行为完全保留
3. **Ghost Text 唯一 UI**：单行灰色后缀，Tab/右箭头接受，无弹窗、无聊天 UI、无 Markdown
4. **零触感安装/卸载**：包管理器自动处理服务注册、hook 加载和完整清理，用户不需要编辑 shell 配置文件
5. **性能硬门控**：端到端 ≤30ms、检索 ≤3ms、首 token ≤15ms、内存 ≤600MB、空闲 CPU ≈0%、覆盖率 ≥85%
