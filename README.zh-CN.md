<div align="center">

# ShellClaw

### 在终端里运行的本地大语言模型补全

**ShellClaw 预测 Zsh 命令的后续内容，并以内联灰字显示；背后的代码模型
完全运行在你的 Mac 上。**

不需要 API Key，没有云端推理延迟，命令历史不会离开本机。

[![Release](https://img.shields.io/github/v/release/Edwardd02/Shell-Claw?style=flat-square)](https://github.com/Edwardd02/Shell-Claw/releases/latest)
[![Stars](https://img.shields.io/github/stars/Edwardd02/Shell-Claw?style=flat-square)](https://github.com/Edwardd02/Shell-Claw/stargazers)
[![Apple Silicon](https://img.shields.io/badge/macOS-Apple%20Silicon-black?style=flat-square&logo=apple)](https://www.apple.com/mac/)
[![Rust](https://img.shields.io/badge/built%20with-Rust-DEA584?style=flat-square&logo=rust)](https://www.rust-lang.org/)

**[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md)**

</div>

## 安装

ShellClaw 目前支持 **Apple Silicon Mac** 和 **Zsh**。

```bash
brew tap edwardd02/shellclaw
brew trust edwardd02/shellclaw
brew install shellclaw
```

安装过程会下载本地模型、向 `~/.zshrc` 写入一段可管理的 ShellClaw 配置，
并启动 daemon。首次安装需要等待模型下载；ShellClaw 会测速 Hugging Face
和 ModelScope，优先使用更快的源。

打开一个新终端，然后开始输入：

```text
$ git che[ckout main]
         └───────── 灰色 Ghost Text
```

按 **Ctrl+Space** 接受提示；继续输入则替换或忽略它。ShellClaw 不占用
`Tab` 和方向键，原有的 Shell 补全和快捷键可以照常使用。

```bash
shellclaw status
# shellclaw: running
```

如果模型下载中断，可以继续下载：

```bash
brew postinstall shellclaw
```

## 为什么选择 ShellClaw

- **真正的本地 LLM，而不是固定补全表。** 微调后的 Qwen2.5-Coder 0.5B
  能推断命令的后续内容，不局限于简单回放历史记录。
- **越用越贴合，同时数据不出本机。** SQLite FTS5 记忆会结合命令前缀
  和当前工作目录，快速召回你真正使用过的命令。
- **像 Shell 原生能力一样自然。** 提示通过 Zsh 的 `POSTDISPLAY` 路径
  渲染成紧跟输入的灰字，只有按下接受键后才会进入真实命令缓冲区。
- **不打断输入。** 请求异步处理，过期任务会被取消；daemon 异常时静默
  降级，不会卡住或污染终端。
- **为本地运行设计。** llama.cpp 使用 Apple Silicon Metal 加速；日志默认
  关闭，模型空闲 30 秒后自动卸载。

## 工作原理

```text
Zsh 输入
   │
   ▼
ShellClaw ZLE Hook ── 本地 Unix Socket 上的 JSON-RPC ──▶ Rust Daemon
                                                          │
                                     ┌────────────────────┴──────────────────┐
                                     ▼                                       ▼
                           SQLite FTS5 记忆                       本地 Qwen2.5-Coder
                           快速个性化召回                         llama.cpp + Metal
                                     └────────────────────┬──────────────────┘
                                                          ▼
                                                   校验补全后缀
                                                          │
                                                          ▼
                                                Zsh 内联灰色提示
```

daemon 会先查询本地命令记忆；没有有效匹配时，再由模型生成补全后缀。Hook
只接受当前命令行最新一次请求的结果，并在你按下 `Ctrl+Space` 前始终把提示
与真实输入分开。

## 项目实况

| 项目 | 当前实现 |
|---|---|
| 模型 | 微调 Qwen2.5-Coder 0.5B，GGUF 格式 |
| 推理 | llama.cpp，Apple Silicon 上使用 Metal 加速 |
| 个性化记忆 | 本地 SQLite 数据库 + FTS5 检索 |
| 交互 | Zsh 原生内联灰字；`Ctrl+Space` 接受 |
| 运行方式 | Rust daemon，通过本地 Unix Socket 通信 |
| 隐私 | 本地推理、无遥测、文件日志默认关闭 |
| 资源占用 | 模型空闲 30 秒后卸载，轻量 daemon 保持可用 |
| 当前完整支持 | macOS Apple Silicon + Zsh |
| 实验性支持 | Bash Hook；Linux 和 Intel macOS 尚未发布 |

## CLI

```text
shellclaw status          查看 daemon 状态
shellclaw start           启动 daemon
shellclaw stop            停止 daemon
shellclaw log on|off      开启或关闭持久化文件日志
shellclaw setup PATH      安装或刷新受管理的 Zsh Hook
shellclaw --version       查看已安装版本
shellclaw help            查看全部命令
```

常用环境变量：

```bash
# 自定义模型、数据库、Socket 和配置的存储目录
export SHELLCLAW_DATA_DIR=/your/data/directory

# 使用另一个兼容的 GGUF 模型
export SHELLCLAW_MODEL_PATH=/path/to/model.gguf
```

## 从源码构建

需要 Rust 1.80 或更高版本。在 macOS 上会自动启用 Metal 加速。

```bash
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
cargo test --workspace
```

完整安装还需要打包的 Zsh Hook 和兼容的 GGUF 模型。Homebrew 仍然是最简单
的完整安装方式。

## 排查问题

**没有出现补全提示**

```bash
shellclaw status
ls ~/.shellclaw/models/*.gguf
```

安装后需要打开新的 Zsh 终端。当命令已经完整、响应已经过期，或没有可用的
安全后缀时，ShellClaw 也会有意保持安静。

**模型下载中断**

```bash
brew postinstall shellclaw
```

下载会从临时文件继续，并在 Hugging Face 和 ModelScope 之间自动切换。

**Ctrl+Space 没有传到 Zsh**

部分 macOS 输入法切换或终端快捷键会占用 `Ctrl+Space`。取消或修改冲突的
系统快捷键，让终端能够把 `^@` 发送给 Zsh。

**彻底卸载**

```bash
shellclaw stop
brew uninstall shellclaw
rm -rf ~/.shellclaw
```

最后一条命令会删除下载的模型和本地命令记忆。

## 隐私

ShellClaw 从架构上坚持本地优先：

- 命令补全和模型推理在你的 Mac 上完成；
- 命令记忆保存在 `~/.shellclaw/memory.db`；
- 不使用遥测或托管 API；
- 交互日志和 daemon 文件日志只有在你主动开启后才会写入。

你执行过的命令会存入本地记忆数据库，用于匹配今后的使用习惯。删除
`~/.shellclaw` 即可清除这份记忆和全部 ShellClaw 数据。

如果 ShellClaw 让你的终端更好用，欢迎给项目一个
[Star](https://github.com/Edwardd02/Shell-Claw)，或提交包含真实使用场景的
[Issue](https://github.com/Edwardd02/Shell-Claw/issues)。
