<div align="center">

# ⚡ ShellClaw

**大语言模型驱动的智能终端补全（LLM-powered Smart Shell Completion）**

### 本地大语言模型 + 命令行 Ghost Text 补全，像 GitHub Copilot 一样、但完全在本地运行

> 一个本地 LLM + Rust 守护进程 + Shell 钩子，边输入边补全你的 Zsh 命令。
> 轻量 SQLite 命令记忆学习你的习惯、实现秒级召回；本地语言模型（llama.cpp）
> 在历史之外还能智能联想。数据不出机器。

**本地 LLM &nbsp;·&nbsp; 天生私密 &nbsp;·&nbsp; 零触感 &nbsp;·&nbsp; 记忆增强**

[![License](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-✓-000000?style=flat-square&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Linux](https://img.shields.io/badge/Linux-开发中-8A8A8A?style=flat-square&logo=linux&logoColor=white)](https://www.linux.org/)

**[English](README.md)** &nbsp;·&nbsp; **[简体中文](README.zh-CN.md)** &nbsp;·&nbsp; **[日本語](README.ja.md)**

**[安装](#-安装)** &nbsp;·&nbsp; **[使用](#-使用)** &nbsp;·&nbsp; **[快速开始](#-快速开始)** &nbsp;·&nbsp; **[功能特性](#-功能特性)** &nbsp;·&nbsp; **[架构](#-架构)** &nbsp;·&nbsp; **[CLI](#-cli)** &nbsp;·&nbsp; **[隐私](#-隐私--数据安全)** &nbsp;·&nbsp; **[FAQ](#-faq)**

</div>

---
## 📦 安装

### 方式一:Homebrew(推荐)

```bash
brew tap edwardd02/shellclaw
brew trust edwardd02/shellclaw && brew install shellclaw
```

> Homebrew 默认拒绝执行未受信任 tap 的公式。上面的 `brew trust` 一次即可
> 解除。或者用环境变量绕过单次检查:
> ```bash
> HOMEBREW_NO_REQUIRE_TAP_TRUST=1 brew install shellclaw
> ```

`brew install` 会:
1. 安装 `shellclaw` 二进制
2. 自动下载模型到 `~/.shellclaw/models/` —— 同时探测 **Hugging Face 和 ModelScope**,
   选择较快的源;若其中一个失败会自动切换到另一个
3. 幂等写入 `~/.zshrc` 的 ShellClaw 标记块，并启动 daemon

若模型下载被打断,重跑 `brew postinstall shellclaw` 即可续传。

> **要求**: macOS Apple Silicon(ARM)。Linux 和 Intel macOS 开发中。

### 方式二:从源码构建

```bash
# 需要 Rust 1.80+
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
```

---

---

## 🚗 使用

安装后 Homebrew 会启动 daemon 并配置 Zsh hook。**新开一个 Zsh 终端**即可获得补全。

```bash
# 查看状态
shellclaw status
# → shellclaw: running
```

然后在 shell 里输入命令:

```
$ git che【光标在此,灰色提示 ckout main】
```

按 **Tab** 或 **→** 接受,或继续打字忽略。

### 常用命令

```bash
shellclaw start           # 后台启动守护进程
shellclaw stop            # 停止守护进程
shellclaw status          # 查看是否在运行
shellclaw log on|off      # 开启/关闭文件日志(持久化)
shellclaw help            # 列出所有命令
```

### 自动加载

安装后,**新开终端即自动加载 shell 钩子**,无需手动编辑 `.zshrc`。如需在当前
shell 手动加载:

```bash
source /path/to/shellclaw.zsh     # Zsh
# 或
source /path/to/shellclaw.bash    # Bash（实验性）
```

### 配置

```bash
# ShellClaw 数据目录(默认 ~/.shellclaw)
export SHELLCLAW_DATA_DIR=~/your/custom/dir

# 模型路径(如模型下到了别处)
export SHELLCLAW_MODEL_PATH=/path/to/your/model.gguf
```

### 卸载

```bash
brew uninstall shellclaw
rm -rf ~/.shellclaw    # 清空全部数据和模型(零残留)
```

---

---

## 🚀 快速开始

打开你的终端，试着输入一个命令的开头：

```
$ git che【光标在此，灰色提示 ckout main】
```

如果 ShellClaw 已经安装，灰色补全会浮现在光标右侧。按 **Tab** 或 **右箭头** 接受，或者无视它继续打字。

```bash
# 1. 安装(见下方安装章节)
# 2. 状态确认
shellclaw status
# → shellclaw: running

# 3. 开一个新的 Zsh 终端,开始使用!
```

> 命令记忆会在你执行命令时自动积累——越用，补全越懂你的习惯。

---

---

## ✨ 功能特性

| 能力 | 说明 |
|------|------|
| **本地 LLM 补全** | 一个本地语言模型（llama.cpp）负责生成补全,而非固定规则——它理解 shell 命令并推断下一个词 |
| **记忆增强** | SQLite 命令记忆按你自己真正用过的命令重排,让 LLM 更快更贴合——频率、时效、目录相关性 |
| **Ghost Text 体验** | 灰色单行提示紧跟光标,绝不打断你的输入 |
| **接受键** | `Tab` 或 `右箭头` 立即接受;继续打字则无缝替换或清除 |
| **非阻塞** | 补全请求异步进行,即便 daemon 卡死,shell 输入也完全不受影响 |
| **静默降级** | daemon 缺失/超时/异常时,shell 自动回退原生行为,零报错、零打断 |
| **隐私安全** | LLM + 记忆 100% 本地运行,数据绝不出机器 |
| **Shell 支持** | Zsh 完整支持并自动配置；Bash 为实验性手动支持 |

---

---

## 🏗️ 架构

```
用户键盘 → Shell Hook(zle) → Unix Socket → Rust Daemon
                                        ↓
              SQLite 命令记忆(FTS5) — 重排 + 个人先验
                                        ↓
              本地 LLM(llama.cpp) — 补全大脑
                                        ↓
                            返回补全后缀 → Hook 渲染 Ghost Text
```

四层显式解耦:

- **Shell Hook**: 监听按键、去抖、发请求、渲染灰色补全,绝不影响 shell 主输入
- **Rust Daemon / scheduler**: 负责 JSON-RPC、deadline、取消和 worker 调度
- **SQLite 记忆**: 通过 `MemoryStore` Trait 提供 FTS5 快路径
- **本地模型**: 通过 `CompletionModel` Trait 提供 llama.cpp fallback

**数据流(单次补全)**:

```
你在终端输入 "git che"
   ↓ 停止片刻(防抖)
Hook 发送 JSON-RPC completion.request
   ↓
Daemon 从记忆检索相关命令(快速、个性化)
   ↓
记忆没有有效后缀时，本地 LLM 生成 fallback
   ↓
返回后缀 "ckout main" → Hook 在光标右侧渲染灰色 "ckout main"
  按 Tab/→ 接受, 或继续打字 清除
```

---

---

## 🛠️ CLI

`shellclaw` 是一个自包含二进制,支持子命令:

```bash
shellclaw daemon          前台运行 daemon(供服务管理器调用)
shellclaw start           后台启动 daemon
shellclaw stop            停止 daemon
shellclaw status          查看运行状态
shellclaw log on|off      开启/关闭文件日志(持久化)
shellclaw setup PATH      幂等配置 Zsh hook
shellclaw --version       显示安装版本
shellclaw help            帮助
```

```bash
# 日志默认关闭(干净);诊断问题时开启
shellclaw log on
shellclaw start
# → ~/.shellclaw/daemon.log 开始记录

# 平时保持关闭
shellclaw log off
```

---

---

## 🔒 隐私 & 数据安全

- **纯本地**: 命令记忆(SQLite) 和 模型推理全部在机器内完成,数据绝不外传
- **无遥测**: 不收集任何使用数据
- **默认隐私**: 交互日志和 daemon 文件日志默认关闭，只有显式开启才记录
- **低空闲占用**: 模型运行时空闲 30 秒后卸载，轻量 daemon 保持可用
- **可卸载**: 删除 `~/.shellclaw/` 即可清空全部数据和配置(零残留)

---

---

## ❓ FAQ

**ShellClaw 到底是什么?**
它是一个运行在你机器上的**本地大语言模型**,在你输入时补全 shell 命令。你的 SQLite 命令历史用来微调它的建议到你的习惯——既快又个人。

**补全会干扰我的 shell 吗?**
不会。补全只在光标右侧显示灰色文字,不影响已输入内容;且 daemon 完全不可用时,shell 依旧正常工作、零报错。

**补全怎么学会我的命令?**
每当你按回车执行一条命令,ShellClaw 后台记入本地记忆。LLM 会借这些记录把补全偏好到你的真实习惯——既智能(LLM) 又个人(你的历史)。一切都留在本机。

**为什么有时候不显示补全?**
- 当前命令已经完整(如已输入完整 `git commit`)
- LLM 没把握 → 静默不提示(宁缺毋滥)
- daemon 未运行 → hook 自动静默降级

**和 zsh-autosuggestions 有什么区别?**
zsh-autosuggestions 机械地从 shell 历史里回显单词。**ShellClaw 用一个真正的 LLM 生成补全**,它理解 shell 语义,而你的历史只用于让它更快、更贴个人。它能建议你从未输入过的命令。

---

---

## 📄 License

[MIT License](LICENSE) © 2026 Edwardd02

---

欢迎使用 ShellClaw!如果它让终端用起来更顺手,给个 ⭐ 就是最好的支持。

**[⭐ Star on GitHub](https://github.com/Edwardd02/Shell-Claw)** &nbsp;·&nbsp; **[报告问题](https://github.com/Edwardd02/Shell-Claw/issues)**
