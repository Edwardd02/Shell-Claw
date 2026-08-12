<div align="center">

# ⚡ ShellClaw

**ローカルLLM駆動のスマート端末補完（LLM-powered Smart Terminal Completion）**

### ローカル言語モデル + GitHub Copilot ライクな Ghost Text 補完。完全オンデバイスで動作

> ローカル LLM + Rust デーモン + シェルフックで、Zsh / Bash のコマンド入力を補完。
> 軽量 SQLite のコマンド記憶があなたの習慣を学習して瞬間応答を実現。
> ローカル言語モデル（llama.cpp）が履歴を超えてスマートに推論。データは外に出ません。

**ローカルLLM &nbsp;·&nbsp; プライバシーバイデザイン &nbsp;·&nbsp; ゼロタッチ &nbsp;·&nbsp; 記憶拡張**

[![License](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-✓-000000?style=flat-square&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Linux](https://img.shields.io/badge/Linux-開発中-8A8A8A?style=flat-square&logo=linux&logoColor=white)](https://www.linux.org/)

**[English](README.md)** &nbsp;·&nbsp; **[简体中文](README.zh-CN.md)** &nbsp;·&nbsp; **[日本語](README.ja.md)**

**[インストール](#-インストール)** &nbsp;·&nbsp; **[使い方](#-使い方)** &nbsp;·&nbsp; **[クイックスタート](#-クイックスタート)** &nbsp;·&nbsp; **[機能](#-機能)** &nbsp;·&nbsp; **[アーキテクチャ](#-アーキテクチャ)** &nbsp;·&nbsp; **[CLI](#-cli)** &nbsp;·&nbsp; **[プライバシー](#-プライバシー--データ安全性)** &nbsp;·&nbsp; **[FAQ](#-faq)**

</div>

---
## 📦 インストール

### Homebrew（推奨）

```bash
brew tap --trusted Edwardd02/homebrew-shellclaw
brew install shellclaw
```

`brew install` は:
1. `shellclaw` バイナリをインストール
2. モデルを自動で `~/.shellclaw/models/` にダウンロード —— **Hugging Face と
   ModelScope** の両方を測速し、速い方を選択。片方が失敗しても他方に自動フェールバック

ダウンロードが中断されたら、`brew postinstall shellclaw` を再実行すれば再開します。

> **要件**: macOS Apple Silicon（ARM）。Linux / Intel macOS は開発中。

### ソースからビルド

```bash
# Rust 1.80+ が必要
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
```

---

---

## 🚗 使い方

インストール後、ShellClaw はバックグラウンドのデーモンとして動作します。起動して、
**新しいターミナル**を開けば補完が使えます。

```bash
# デーモンを起動（バックグラウンド）
shellclaw start

# 状態を確認
shellclaw status
# → shellclaw: running
```

シェルにコマンドを入力してください：

```
$ git che【カーソル位置、灰色のヒント: ckout main】
```

**Tab** または **→** で確定、打ち続けて無視します。

### 主なコマンド

```bash
shellclaw start           # デーモンをバックグラウンド起動
shellclaw stop            # デーモンを停止
shellclaw status          # 起動状態を確認
shellclaw log on|off      # ファイルログ有効/無効（永続化）
shellclaw help            # 全コマンドを表示
```

### 自動ロード

インストール後、**新しいターミナルで自動的にシェルフックが読み込まれ**、
手動で `.zshrc` を編集する必要はありません。現在のシェルに手動で読み込む場合:

```bash
source /path/to/shellclaw.zsh     # Zsh
# または
source /path/to/shellclaw.bash    # Bash
```

### 設定

```bash
# ShellClaw データディレクトリ（デフォルト ~/.shellclaw）
export SHELLCLAW_DATA_DIR=~/your/custom/dir

# モデルパス（別の場所にモデルを置いた場合）
export SHELLCLAW_MODEL_PATH=/path/to/your/model.gguf
```

### アンインストール

```bash
brew uninstall shellclaw
rm -rf ~/.shellclaw    # 全データ・モデルを削除（残骸ゼロ）
```

---

---

## 🚀 クイックスタート

ターミナルを開いて、コマンドの先頭を入力してみてください：

```
$ git che【カーソル位置、灰色のヒント: ckout main】
```

ShellClaw がインストールされていれば、カーソルの右に灰色の補完が表示されます。**Tab** または **→** で確定するか、そのまま打ち続けて無視します。

```bash
# 1. インストール（下記インストール節を参照）
# 2. デーモンを起動
shellclaw start

# 3. 状態を確認
shellclaw status
# → shellclaw: running

# 4. 新しいターミナルを開いて使い始めましょう！
```

> コマンドを実行するたびに記憶が自動で蓄積されます。使うほど補完があなたの習慣を理解します。

---

---

## ✨ 機能

| 機能 | 説明 |
|------|------|
| **ローカル LLM 補完** | ローカル言語モデル（llama.cpp）が補完を生成。固定ルールではなく、シェルコマンドを理解して次の語を推論 |
| **記憶拡張** | SQLite コマンド記憶があなたの実際の使用に基づいて提案を再ランキングし、LLM を高速・関連性高く保つ（頻度・最近性・cwd） |
| **ゴーストテキスト UX** | カーソルの直後に灰色の一行ヒントを表示。入力は妨げません |
| **確定キー** | `Tab` または `→` で即時に確定。打ち続けると自然に置き換わります |
| **非ブロッキング** | 補完は非同期実行。デーモンが固まってもシェル入力は影響を受けません |
| **サイレントダウングレード** | デーモンが不在・遅延・異常時もシェルはネイティブ動作に自動フォールバック。エラー・中断は一切なし |
| **プライバシー** | LLM + 記憶は 100% オンデバイス。データがマシンを出ることはありません |
| **Zsh / Bash** | 両メジャーシェルを標準サポート |

---

---

## 🏗️ アーキテクチャ

```
キー入力 → Shell Hook(zle) → Unix Socket → Rust Daemon
                                        ↓
                  SQLite コマンド記憶(FTS5) — 再ランキング + 個人先験
                                        ↓
                  ローカル LLM(llama.cpp) — 補完の頭脳
                                        ↓
                            補完サフィックスを返す → Hook がゴーストテキストを描画
```

関心の分離が明確な3層構造:

- **Shell Hook**: キー監視・デバウンス・リクエスト送信・灰色補完の描画。シェルの主入力には一切触れません
- **Rust Daemon**: 常駐バックグラウンドプロセス。ローカル LLM と記憶を駆動し、Unix Socket 経由でフックと通信
- **ローカル LLM + 記憶**: llama.cpp 推論 + SQLite 記憶。すべてローカル

**データフロー（1回の補完）**:

```
ターミナルで "git che" と入力
   ↓ 少し停止（デバウンス）
Hook が JSON-RPC completion.request を送信
   ↓
Daemon が記憶から関連コマンドを取得（高速・個人化）
   ↓
ローカル LLM がその記憶候補に基づいて補完を生成
   ↓
サフィックス "ckout main" を返す → Hook がカーソル右に灰色 "ckout main" を描画
  Tab/→ で確定、または打ち続けてクリア
```

---

---

## 🛠️ CLI

`shellclaw` は単一の自己完結バイナリで、サブコマンドを提供します:

```bash
shellclaw daemon          デーモンをフォアグラウンドで実行（サービス管理用）
shellclaw start           デーモンをバックグラウンドで起動
shellclaw stop            デーモンを停止
shellclaw status          実行状態を表示
shellclaw log on|off      ファイルログを有効/無効化（永続化）
shellclaw help            ヘルプを表示
```

```bash
# ログはデフォルトでオフ（クリーン）。診断時に有効化:
shellclaw log on
shellclaw start
# → ~/.shellclaw/daemon.log に記録開始

# 日常利用ではオフのまま
shellclaw log off
```

---

---

## 🔒 プライバシー & データ安全性

- **完全ローカル**: コマンド記憶（SQLite）とモデル推論はすべてマシン内で完結。外部送信は一切なし
- **テレメトリなし**: 使用データは収集しません
- **完全削除可能**: `~/.shellclaw/` を削除すれば全データと設定がクリアされます（残骸ゼロ）

---

---

## ❓ FAQ

**ShellClaw とは実際何?**
あなたのマシン上で動作する**ローカル言語モデル**が、入力時にシェルコマンドを補完します。あなたの SQLite コマンド履歴が LLM の提案をあなたの習慣に合わせ、高速かつ個人的にします。

**補完はシェルを妨げますか?**
いいえ。補完はカーソル右のグレー文字としてのみ表示され、入力内容には影響しません。デーモンが完全に使えなくても、シェルはエラーなく正常動作します。

**ShellClaw はどうやってコマンドを学習するの?**
実行したすべてのコマンドがローカル記憶に記録されます。LLM はその記録を使って、提案をあなたの実際の行動に寄せます。スマート（LLM）かつ個人的（あなたの履歴）です。すべてマシン内に留まります。

**なぜ補完が表示されないことがあるの?**
- コマンドがすでに完了している場合（例: `git commit` を完全に入力済み）
- LLM に確信がない場合 → 静かに非表示（間違いより無表示を優先）
- デーモンが起動していない場合 → フックが自動でサイレントダウングレード

**zsh-autosuggestions との違いは?**
zsh-autosuggestions はシェル履歴から機械的に単語をリピートします。**ShellClaw は本物の LLM で補完を生成**し、シェルの意味を理解します。あなたの履歴は LLM を高速・個人的にするためにのみ使われます。一度も入力したことのないコマンドも提案できます。

---

---

## 📄 ライセンス

[MIT License](LICENSE) © 2026 Edwardd02

---

ShellClaw をありがとうございます。ターミナルが使いやすくなったら、スターが最高の応援になります。

**[⭐ Star on GitHub](https://github.com/Edwardd02/Shell-Claw)** &nbsp;·&nbsp; **[Issue を報告](https://github.com/Edwardd02/Shell-Claw/issues)**

