<div align="center">

# ShellClaw

### Local LLM autocomplete for your terminal

**ShellClaw predicts the rest of your Zsh command and renders it as native
ghost text, powered by a coding model running entirely on your Mac.**

No API key. No cloud round-trip. No command history leaving your machine.

[![Release](https://img.shields.io/github/v/release/Edwardd02/Shell-Claw?style=flat-square)](https://github.com/Edwardd02/Shell-Claw/releases/latest)
[![Stars](https://img.shields.io/github/stars/Edwardd02/Shell-Claw?style=flat-square)](https://github.com/Edwardd02/Shell-Claw/stargazers)
[![Apple Silicon](https://img.shields.io/badge/macOS-Apple%20Silicon-black?style=flat-square&logo=apple)](https://www.apple.com/mac/)
[![Rust](https://img.shields.io/badge/built%20with-Rust-DEA584?style=flat-square&logo=rust)](https://www.rust-lang.org/)

**[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md)**

</div>

## Install

ShellClaw currently supports **macOS on Apple Silicon** with **Zsh**.

```bash
brew tap edwardd02/shellclaw
brew trust edwardd02/shellclaw
brew install shellclaw
```

That installation downloads the local model, adds a managed ShellClaw block to
`~/.zshrc`, and starts the daemon. The first install can take a little longer
while the model is downloaded from the faster of Hugging Face or ModelScope.

Open a new terminal and start typing:

```text
$ git che[ckout main]
         └───────── gray ghost text
```

Press **Right Arrow** to accept the suggestion. Keep typing to replace it or
ignore it. ShellClaw leaves `Tab` to your existing shell completion.

```bash
shellclaw status
# shellclaw: running
```

If the model download was interrupted, resume it with:

```bash
brew postinstall shellclaw
```

## Why ShellClaw

- **A real local LLM, not a static completion table.** A fine-tuned
  Qwen2.5-Coder 0.5B model can infer useful command continuations that are not
  already in your history.
- **Gets personal without sending data away.** SQLite FTS5 memory recalls the
  commands you actually use, with prefix and working-directory context.
- **Feels like part of the shell.** Suggestions are rendered through Zsh's
  native `POSTDISPLAY` ghost-text path and only enter the command buffer when
  you accept them.
- **Stays out of the way.** Requests are asynchronous, stale work is cancelled,
  and failures degrade silently without blocking terminal input.
- **Made for local use.** Inference uses llama.cpp with Metal acceleration;
  logs are off by default and the model unloads after 30 idle seconds.

## How It Works

```text
Zsh input
   │
   ▼
ShellClaw ZLE hook ── JSON-RPC over a local Unix socket ──▶ Rust daemon
                                                           │
                                      ┌────────────────────┴──────────────────┐
                                      ▼                                       ▼
                            SQLite FTS5 memory                     Local Qwen2.5-Coder
                            fast personal recall                    llama.cpp + Metal
                                      └────────────────────┬──────────────────┘
                                                           ▼
                                              validated completion suffix
                                                           │
                                                           ▼
                                              gray Zsh ghost text
```

The daemon checks local command memory first. When memory has no valid match,
the model generates a suffix. The hook accepts only the newest response for the
current command line, validates it, and keeps the suggestion separate from your
real input until you press `Right Arrow`.

## What You Get

| Area | Implementation |
|---|---|
| Model | Fine-tuned Qwen2.5-Coder 0.5B in GGUF format |
| Inference | llama.cpp with Metal acceleration on Apple Silicon |
| Personal memory | Local SQLite database with FTS5 retrieval |
| Interface | Native inline Zsh ghost text; `Right Arrow` to accept |
| Runtime | Rust daemon communicating over a local Unix socket |
| Privacy | On-device inference, no telemetry, file logging disabled by default |
| Resource use | Heavy model state unloads after 30 idle seconds |
| Current support | macOS Apple Silicon + Zsh |
| Experimental | Bash hook; Linux and Intel macOS are not released yet |

## CLI

```text
shellclaw status          Show daemon status
shellclaw start           Start the daemon
shellclaw stop            Stop the daemon
shellclaw log on|off      Enable or disable persistent file logging
shellclaw setup PATH      Install or refresh the managed Zsh hook
shellclaw --version       Show the installed version
shellclaw help            Show all commands
```

Useful environment variables:

```bash
# Store the model, database, socket, and config somewhere else
export SHELLCLAW_DATA_DIR=/your/data/directory

# Use a different compatible GGUF model
export SHELLCLAW_MODEL_PATH=/path/to/model.gguf
```

## Build From Source

You need Rust 1.80 or newer. Metal acceleration is enabled automatically on
macOS.

```bash
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
cargo test --workspace
```

The release installation also needs the packaged Zsh hook and a compatible
GGUF model. Homebrew remains the simplest complete setup.

## Troubleshooting

**No suggestion appears**

```bash
shellclaw status
ls ~/.shellclaw/models/*.gguf
```

Open a new Zsh terminal after installation. ShellClaw also intentionally stays
silent when the command is already complete, the response is stale, or no safe
suffix is available.

**The model download stopped**

```bash
brew postinstall shellclaw
```

Downloads resume from the partial file and automatically fall back between
Hugging Face and ModelScope.

**Tab still accepts ShellClaw suggestions after upgrading**

The previous hook used `Tab`. Open a new terminal after upgrading so Zsh loads
the new hook; `Tab` will then remain exclusively available to native completion.

**Remove ShellClaw completely**

```bash
shellclaw stop
brew uninstall shellclaw
rm -rf ~/.shellclaw
```

The last command removes the downloaded model and local command memory.

## Privacy

ShellClaw is local-first by construction:

- command completion and model inference run on your Mac;
- command memory stays in `~/.shellclaw/memory.db`;
- no telemetry or hosted API is used;
- interaction and daemon file logging are disabled unless you enable them.

Commands you execute are stored in the local memory database so future
suggestions can match your habits. Delete `~/.shellclaw` to erase that memory
and all ShellClaw data.

If ShellClaw makes your terminal more useful, consider giving the project a
[star](https://github.com/Edwardd02/Shell-Claw) or opening an
[issue](https://github.com/Edwardd02/Shell-Claw/issues) with real-world
feedback.
