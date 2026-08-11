<div align="center">

# ⚡ ShellClaw

**本地优先的智能终端补全（Smart Shell Completion）**

### Ghost Text 补全，像 GitHub Copilot 一样贴合你的命令行

> Rust 守护进程 + Shell 钩子，为 Zsh / Bash 提供即时的灰色下一词补全。
> 本地 SQLite 命令记忆 + 可选本地模型推理，数据不出机器，零触感安装。

**本地优先 &nbsp;·&nbsp; 隐私安全 &nbsp;·&nbsp; 零触感 &nbsp;·&nbsp; 可自托管**

[![License](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-✓-000000?style=flat-square&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Linux](https://img.shields.io/badge/Linux-开发中-8A8A8A?style=flat-square&logo=linux&logoColor=white)](https://www.linux.org/)

**[快速开始](#-快速开始)** &nbsp;·&nbsp; **[功能特性](#-功能特性)** &nbsp;·&nbsp; **[架构](#-架构)** &nbsp;·&nbsp; **[CLI](#-cli)** &nbsp;·&nbsp; **[安装](#-安装)** &nbsp;·&nbsp; **[隐私](#-隐私--数据安全)** &nbsp;·&nbsp; **[FAQ](#-faq)**

</div>

---

## 🚀 快速开始

打开你的终端，试着输入一个命令的开头：

```
$ git che【光标在此，灰色提示 ckout main】
```

如果 ShellClaw 已经安装，灰色补全会浮现在光标右侧。按 **Tab** 或 **右箭头** 接受，或者无视它继续打字。

```bash
# 1. 安装(见下方安装章节)
# 2. 启动 daemon
shellclaw start

# 3. 状态确认
shellclaw status
# → shellclaw: running

# 4. 开一个新的终端,开始使用!
```

> 命令记忆会在你执行命令时自动积累——越用，补全越懂你的习惯。

---

## ✨ 功能特性

| 能力 | 说明 |
|------|------|
| **Ghost Text 补全** | 灰色单行提示紧跟光标，绝不打断你的输入 |
| **接受键** | `Tab` 或 `右箭头` 立即接受；继续打字则无缝替换或清除 |
| **非阻塞** | 补全请求异步进行,即便 daemon 卡死,shell 输入也完全不受影响 |
| **本地命令记忆** | SQLite + FTS5 混合排序(BM25 + 目录相关性 + 频率 + 时间衰减),自动学习你的常用命令 |
| **本地模型推理** | 可选接入本地 GGUF 模型,对记忆外的命令智能联想 |
| **静默降级** | daemon 缺失/超时/异常时,shell 自动回退原生行为,零报错、零打断 |
| **隐私安全** | 全部数据存本地,记忆不出机器 |
| **Zsh / Bash** | 两大主流 shell 原生支持 |

---

## 🏗️ 架构

```
用户键盘 → Shell Hook(zle) → Unix Socket → Rust Daemon
                                        ↓
                              SQLite 命令记忆(FTS5)
                                        ↓
                              本地模型推理(llama.cpp, 可选)
                                        ↓
                            返回补全后缀 → Hook 渲染 Ghost Text
```

三层关注点分离:

- **Shell Hook**: 监听按键、去抖、发请求、渲染灰色补全,绝不影响 shell 主输入
- **Rust Daemon**: 常驻后台,处理检索/推理,通过 Unix Socket 与 hook 通信
- **存储/推理**: SQLite 记忆 + 可选本地模型,全部本地

**数据流(单次补全)**:

```
你在终端输入 "git che"
   ↓ 停止片刻(防抖)
Hook 发送 JSON-RPC completion.request
   ↓
Daemon 查本地记忆 → 命中则减法得后缀 "ckout main"
                    未命中 → 交给本地模型联想
   ↓
返回 {"kind":"suggestion","suffix":"ckout main"}
   ↓
Hook 在光标右侧渲染灰色 "ckout main"
  按 Tab/→ 接受, 或继续打字 清除
```

---

## 🛠️ CLI

`shellclaw` 是一个自包含二进制,支持子命令:

```bash
shellclaw daemon          前台运行 daemon(供服务管理器调用)
shellclaw start           后台启动 daemon
shellclaw stop            停止 daemon
shellclaw status          查看运行状态
shellclaw log on|off      开启/关闭文件日志(持久化)
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

## 📦 安装

### 方式一:Homebrew(推荐)

```bash
brew tap Edwardd02/homebrew-shellclaw
brew install shellclaw
```

### 方式二:从源码构建

```bash
# 需要 Rust 1.80+
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
```

### 配置

```bash
# ShellClaw 数据目录(默认 ~/.shellclaw)
export SHELLCLAW_DATA_DIR=~/your/custom/dir

# 模型路径(如使用本地模型)
export SHELLCLAW_MODEL_PATH=/path/to/your/model.gguf
```

---

## 🔒 隐私 & 数据安全

- **纯本地**: 命令记忆(SQLite) 和 模型推理全部在机器内完成,数据绝不外传
- **无遥测**: 不收集任何使用数据
- **可卸载**: 删除 `~/.shellclaw/` 即可清空全部数据和配置(零残留)

---

## ❓ FAQ

**补全会干扰我的 shell 吗?**
不会。补全只在光标右侧显示灰色文字,不影响已输入内容;且 daemon 完全不可用时,shell 依旧正常工作、零报错。

**补全怎么学会我的命令?**
当你按回车执行一条命令时,ShellClaw 后台记入本地记忆。之后输入相同/相似开头的命令,它会优先给出你真正用过的。它纯粹学习你自己的习惯,私密且精准。

**为什么有时候不显示补全?**
- 当前命令已经完整(如已输入完整 `git commit`)
- 记忆里没有、模型也没把握 → 静默不提示(宁缺毋滥)
- daemon 未运行 → hook 自动静默降级

**和 zsh-autosuggestions 有什么区别?**
zsh-autosuggestions 基于 shell 历史机械匹配;ShellClaw 额外结合本地记忆 + 可选模型,并分离出独立 Rust daemon,支持更复杂的排序与推理。

---

## 📄 License

[MIT License](LICENSE) © 2026 Edwardd02

---

欢迎使用 ShellClaw!如果它让终端用起来更顺手,给个 ⭐ 就是最好的支持。

**[⭐ Star on GitHub](https://github.com/Edwardd02/Shell-Claw)** &nbsp;·&nbsp; **[报告问题](https://github.com/Edwardd02/Shell-Claw/issues)**

---

<div align="center">

[English](README.md) &nbsp;·&nbsp; [简体中文](README.zh-CN.md) &nbsp;·&nbsp; [日本語](README.ja.md)

</div>
