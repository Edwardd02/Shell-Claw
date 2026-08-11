<div align="center">

# ⚡ ShellClaw

**プライバシー優先のスマート端末補完（Smart Terminal Completion）**

### カーソルに沿って灰色の次の候補を表示。GitHub Copilot のような補完体験

> Rust のデーモン + シェルフックで、Zsh / Bash に即時の次ワード補完を提供。
> ローカル SQLite のコマンド記憶 + オプションのローカルモデル推論。
> データはマシンから一切出ません。ゼロタッチインストール。

**ローカルファースト &nbsp;·&nbsp; プライバシー &nbsp;·&nbsp; ゼロタッチ &nbsp;·&nbsp; セルフホスト**

[![License](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-✓-000000?style=flat-square&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Linux](https://img.shields.io/badge/Linux-開発中-8A8A8A?style=flat-square&logo=linux&logoColor=white)](https://www.linux.org/)

**[クイックスタート](#-クイックスタート)** &nbsp;·&nbsp; **[機能](#-機能)** &nbsp;·&nbsp; **[アーキテクチャ](#-アーキテクチャ)** &nbsp;·&nbsp; **[CLI](#-cli)** &nbsp;·&nbsp; **[インストール](#-インストール)** &nbsp;·&nbsp; **[プライバシー](#-プライバシー--データ安全性)** &nbsp;·&nbsp; **[FAQ](#-faq)**

</div>

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

## ✨ 機能

| 機能 | 説明 |
|------|------|
| **ゴーストテキスト補完** | カーソルの直後に灰色の一行ヒントを表示。入力は妨げません |
| **確定キー** | `Tab` または `→` で即時に確定。打ち続けると自然に置き換わります |
| **非ブロッキング** | 補完は非同期実行。デーモンが固まってもシェル入力は影響を受けません |
| **ローカルコマンド記憶** | SQLite + FTS5 のハイブリッドランキング（BM25 + cwd 関連性 + 頻度 + 最近性）であなたの習慣を学習 |
| **ローカルモデル推論** | 記憶にないコマンドはオプションの GGUF モデルがスマートに推測 |
| **サイレントダウングレード** | デーモンが不在・遅延・異常時もシェルはネイティブ動作に自動フォールバック。エラー・中断は一切なし |
| **プライバシー** | すべてローカル。あなたの記憶がマシンから出ることはありません |
| **Zsh / Bash** | 両メジャーシェルを標準サポート |

---

## 🏗️ アーキテクチャ

```
キー入力 → Shell Hook(zle) → Unix Socket → Rust Daemon
                                        ↓
                              SQLite コマンド記憶(FTS5)
                                        ↓
                              ローカルモデル推論(llama.cpp, オプション)
                                        ↓
                            補完サフィックスを返す → Hook がゴーストテキストを描画
```

関心の分離が明確な3層構造:

- **Shell Hook**: キー監視・デバウンス・リクエスト送信・灰色補完の描画。シェルの主入力には一切触れません
- **Rust Daemon**: 常駐バックグラウンドプロセス。検索/推論を処理し、Unix Socket 経由でフックと通信
- **ストレージ/推論**: SQLite 記憶 + オプションのローカルモデル。すべてローカル

**データフロー（1回の補完）**:

```
ターミナルで "git che" と入力
   ↓ 少し停止（デバウンス）
Hook が JSON-RPC completion.request を送信
   ↓
Daemon がローカル記憶を検索 → ヒット: プレフィックスを引いてサフィックス "ckout main" を取得
                                ミス: ローカルモデルにフォールバック
   ↓
{"kind":"suggestion","suffix":"ckout main"} を返す
   ↓
Hook がカーソル右に灰色 "ckout main" を描画
  Tab/→ で確定、または打ち続けてクリア
```

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

## 📦 インストール

### Homebrew（推奨）

```bash
brew tap Edwardd02/homebrew-shellclaw
brew install shellclaw
```

### ソースからビルド

```bash
# Rust 1.80+ が必要
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
```

### 設定

```bash
# ShellClaw データディレクトリ（デフォルト ~/.shellclaw）
export SHELLCLAW_DATA_DIR=~/your/custom/dir

# モデルパス（ローカルモデルを使う場合）
export SHELLCLAW_MODEL_PATH=/path/to/your/model.gguf
```

---

## 🔒 プライバシー & データ安全性

- **完全ローカル**: コマンド記憶（SQLite）とモデル推論はすべてマシン内で完結。外部送信は一切なし
- **テレメトリなし**: 使用データは収集しません
- **完全削除可能**: `~/.shellclaw/` を削除すれば全データと設定がクリアされます（残骸ゼロ）

---

## ❓ FAQ

**補完はシェルを妨げますか?**
いいえ。補完はカーソル右のグレー文字としてのみ表示され、入力内容には影響しません。デーモンが完全に使えなくても、シェルはエラーなく正常動作します。

**ShellClaw はどうやってコマンドを学習するの?**
コマンドを実行して Enter を押すたびに、バックグラウンドでローカル記憶に記録されます。同じ・似たプレフィックスを後で入力すると、実際に使ったコマンドを優先します。純粋に自分の習慣だけを学習します。

**なぜ補完が表示されないことがあるの?**
- コマンドがすでに完了している場合（例: `git commit` を完全に入力済み）
- 記憶にもモデルにも確信がない場合 → 静かに非表示（間違いより無表示を優先）
- デーモンが起動していない場合 → フックが自動でサイレントダウングレード

**zsh-autosuggestions との違いは?**
zsh-autosuggestions はシェル履歴に機械的にマッチします。ShellClaw は加えてローカル記憶 + オプションのモデルを組み合わせ、独立した Rust デーモンを使うことで、より高度なランキングと推論を可能にします。

---

## 📄 ライセンス

[MIT License](LICENSE) © 2026 Edwardd02

---

ShellClaw をありがとうございます。ターミナルが使いやすくなったら、スターが最高の応援になります。

**[⭐ Star on GitHub](https://github.com/Edwardd02/Shell-Claw)** &nbsp;·&nbsp; **[Issue を報告](https://github.com/Edwardd02/Shell-Claw/issues)**

---

<div align="center">

[English](README.md) &nbsp;·&nbsp; [简体中文](README.zh-CN.md) &nbsp;·&nbsp; [日本語](README.ja.md)

</div>
