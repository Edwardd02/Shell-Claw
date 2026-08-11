<div align="center">

# ⚡ ShellClaw

**Privacy-first smart terminal completion**

### Ghost text completions that follow your command line, like GitHub Copilot

> A Rust daemon + shell hook that gives Zsh / Bash instant gray next-word
> completions. Local SQLite command memory + optional local model inference.
> Data never leaves your machine. Zero-touch install.

**Local-first &nbsp;·&nbsp; Private &nbsp;·&nbsp; Zero-touch &nbsp;·&nbsp; Self-hostable**

[![License](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-✓-000000?style=flat-square&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Linux](https://img.shields.io/badge/Linux-开发中-8A8A8A?style=flat-square&logo=linux&logoColor=white)](https://www.linux.org/)

**[Quick Start](#-quick-start)** &nbsp;·&nbsp; **[Features](#-features)** &nbsp;·&nbsp; **[Architecture](#-architecture)** &nbsp;·&nbsp; **[CLI](#-cli)** &nbsp;·&nbsp; **[Install](#-install)** &nbsp;·&nbsp; **[Privacy](#-privacy--data-safety)** &nbsp;·&nbsp; **[FAQ](#-faq)**

</div>

---

## 🚀 Quick Start

Open your terminal and start typing a command:

```
$ git che【cursor here, gray hint: ckout main】
```

If ShellClaw is installed, the gray completion appears right of the cursor. Press **Tab** or **→** to accept, or just keep typing to ignore it.

```bash
# 1. Install (see Install section below)
# 2. Start the daemon
shellclaw start

# 3. Check status
shellclaw status
# → shellclaw: running

# 4. Open a new terminal and start using it!
```

> Command memory is accumulated automatically as you run commands — the more
> you use ShellClaw, the better it learns your habits.

---

## ✨ Features

| Capability | Description |
|------|------|
| **Ghost Text completion** | A gray single-line hint right after the cursor, never distrupting your typing |
| **Accept keys** | `Tab` or `→` accepts instantly; keep typing to replace or clear seamlessly |
| **Non-blocking** | Completions run asynchronously — even if the daemon hangs, your shell input is unaffected |
| **Local command memory** | SQLite + FTS5 hybrid ranking (BM25 + cwd relevance + frequency + recency) learns your common commands |
| **Local model inference** | Optional local GGUF model for smart guesses on commands outside memory |
| **Silent degradation** | When the daemon is missing, slow, or errors, the shell falls back to native behavior — zero errors, zero interruption |
| **Privacy** | Everything is stored locally; your memory never leaves the machine |
| **Zsh / Bash** | Out-of-the-box support for both major shells |

---

## 🏗️ Architecture

```
Keystrokes → Shell Hook (zle) → Unix Socket → Rust Daemon
                                              ↓
                              SQLite command memory (FTS5)
                                              ↓
                              Local model inference (llama.cpp, optional)
                                              ↓
                        Return completion suffix → Hook renders Ghost Text
```

Three layers with clear separation of concerns:

- **Shell Hook**: listens to keys, debounces, sends requests, renders gray completions — never touches main shell input
- **Rust Daemon**: resident background process, handles retrieval/inference, talks to the hook over a Unix socket
- **Storage/Inference**: SQLite memory + optional local model, all local

**Data flow (one completion)**:

```
You type "git che" in the terminal
   ↓ pause briefly (debounce)
Hook sends JSON-RPC completion.request
   ↓
Daemon checks local memory → hit: subtract prefix, get suffix "ckout main"
                              miss: fall back to the local model
   ↓
Returns {"kind":"suggestion","suffix":"ckout main"}
   ↓
Hook renders gray "ckout main" right of the cursor
  press Tab/→ to accept, or keep typing to clear
```

---

## 🛠️ CLI

`shellclaw` is a single self-contained binary with subcommands:

```bash
shellclaw daemon          Run daemon in the foreground (for service managers)
shellclaw start           Start the daemon in the background
shellclaw stop            Stop the daemon
shellclaw status          Show running status
shellclaw log on|off      Enable/disable file logging (persisted)
shellclaw help            Show help
```

```bash
# Logging is off by default (clean). Enable it for diagnostics:
shellclaw log on
shellclaw start
# → starts logging to ~/.shellclaw/daemon.log

# Keep it off in daily use
shellclaw log off
```

---

## 📦 Install

### Homebrew (recommended)

```bash
brew tap Edwardd02/homebrew-shellclaw
brew install shellclaw
```

### Build from source

```bash
# Requires Rust 1.80+
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
```

### Configuration

```bash
# ShellClaw data directory (default ~/.shellclaw)
export SHELLCLAW_DATA_DIR=~/your/custom/dir

# Model path (if using a local model)
export SHELLCLAW_MODEL_PATH=/path/to/your/model.gguf
```

---

## 🔒 Privacy & Data Safety

- **100% local**: command memory (SQLite) and model inference all happen on your machine; nothing is ever sent out
- **No telemetry**: we do not collect any usage data
- **Fully removable**: delete `~/.shellclaw/` to clear all data and config (zero residue)

---

## ❓ FAQ

**Will completions disrupt my shell?**
No. Completions only show as gray text right of the cursor and never affect what you've typed. Even when the daemon is completely unavailable, the shell keeps working normally with zero errors.

**How does ShellClaw learn my commands?**
Whenever you press Enter on a command, ShellClaw records it into local memory in the background. When you later type the same or a similar prefix, it prioritizes what you've actually used. It purely learns your own habits — private and precise.

**Why is no completion shown sometimes?**
- The command is already complete (e.g. you already typed the full `git commit`)
- Neither memory nor the model is confident → it stays silent (better nothing than wrong)
- The daemon isn't running → the hook degrades silently

**How is this different from zsh-autosuggestions?**
zsh-autosuggestions matches mechanically against shell history. ShellClaw additionally combines local memory + an optional model, and uses a separate Rust daemon, enabling richer ranking and reasoning.

---

## 📄 License

[MIT License](LICENSE) © 2026 Edwardd02

---

Thanks for checking out ShellClaw! If it makes your terminal nicer to use, a star is the best support.

**[⭐ Star on GitHub](https://github.com/Edwardd02/Shell-Claw)** &nbsp;·&nbsp; **[Report an issue](https://github.com/Edwardd02/Shell-Claw/issues)**

---

<div align="center">

[English](README.md) &nbsp;·&nbsp; [简体中文](README.zh-CN.md) &nbsp;·&nbsp; [日本語](README.ja.md)

</div>
