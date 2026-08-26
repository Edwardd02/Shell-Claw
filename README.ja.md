<div align="center">

# ShellClaw

### ターミナルで動くローカル LLM 補完

**ShellClaw は Zsh コマンドの続きを予測し、インラインのゴーストテキスト
として表示します。コードモデルはすべてあなたの Mac 上で動作します。**

API キー不要。クラウド推論の待ち時間なし。コマンド履歴が外部に送信される
こともありません。

[![Release](https://img.shields.io/github/v/release/Edwardd02/Shell-Claw?style=flat-square)](https://github.com/Edwardd02/Shell-Claw/releases/latest)
[![Stars](https://img.shields.io/github/stars/Edwardd02/Shell-Claw?style=flat-square)](https://github.com/Edwardd02/Shell-Claw/stargazers)
[![Apple Silicon](https://img.shields.io/badge/macOS-Apple%20Silicon-black?style=flat-square&logo=apple)](https://www.apple.com/mac/)
[![Rust](https://img.shields.io/badge/built%20with-Rust-DEA584?style=flat-square&logo=rust)](https://www.rust-lang.org/)

**[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md)**

</div>

## インストール

現在、ShellClaw が正式に対応している環境は **Apple Silicon Mac** と
**Zsh** です。

```bash
brew tap edwardd02/shellclaw
brew trust edwardd02/shellclaw
brew install shellclaw
```

インストール時にローカルモデルをダウンロードし、`~/.zshrc` に管理可能な
ShellClaw ブロックを追加して daemon を起動します。初回はモデルのダウンロード
に少し時間がかかります。Hugging Face と ModelScope を測定し、速い方を優先します。

新しいターミナルを開いて入力を始めてください：

```text
$ git che[ckout main]
         └───────── 灰色のゴーストテキスト
```

**右矢印キー** で候補を確定します。そのまま入力を続ければ候補は置き換わるか
無視されます。ShellClaw は `Tab` を占有しないため、既存のシェル補完はそのまま
使えます。

```bash
shellclaw status
# shellclaw: running
```

モデルのダウンロードが中断した場合：

```bash
brew postinstall shellclaw
```

## ShellClaw を選ぶ理由

- **固定の補完表ではなく、本物のローカル LLM。** ファインチューニングした
  Qwen2.5-Coder 0.5B が、履歴にないコマンドの続きを推論できます。
- **データを外に出さずに個人化。** SQLite FTS5 メモリが、入力中の接頭辞と
  カレントディレクトリを使って実際によく使うコマンドを高速に呼び戻します。
- **シェル本来の機能のような操作感。** Zsh の `POSTDISPLAY` 経由で候補を
  インライン表示し、確定するまで実際のコマンドバッファには入りません。
- **入力を邪魔しない。** 非同期処理、古いリクエストのキャンセル、障害時の
  サイレントなフォールバックにより、ターミナル入力を止めません。
- **ローカル実行向けの設計。** llama.cpp と Apple Silicon Metal を使用し、
  ログはデフォルトで無効、モデルは 30 秒のアイドル後にアンロードされます。

## 仕組み

```text
Zsh 入力
   │
   ▼
ShellClaw ZLE Hook ── ローカル Unix Socket 上の JSON-RPC ──▶ Rust daemon
                                                            │
                                       ┌────────────────────┴──────────────────┐
                                       ▼                                       ▼
                             SQLite FTS5 メモリ                     ローカル Qwen2.5-Coder
                             高速な個人化検索                       llama.cpp + Metal
                                       └────────────────────┬──────────────────┘
                                                            ▼
                                                    検証済みの補完 suffix
                                                            │
                                                            ▼
                                                  Zsh の灰色ゴーストテキスト
```

daemon は最初にローカルのコマンドメモリを検索し、有効な一致がない場合にモデル
が suffix を生成します。Hook は現在のコマンド行に対する最新の応答だけを受け入れ、
`右矢印キー` を押すまでは候補と実入力を分離します。

## 現在の実装

| 項目 | 実装 |
|---|---|
| モデル | ファインチューニング済み Qwen2.5-Coder 0.5B、GGUF 形式 |
| 推論 | llama.cpp、Apple Silicon では Metal アクセラレーション |
| 個人メモリ | ローカル SQLite データベース + FTS5 検索 |
| UI | Zsh ネイティブのインライン表示、`右矢印キー` で確定 |
| ランタイム | ローカル Unix Socket で通信する Rust daemon |
| プライバシー | オンデバイス推論、テレメトリなし、ファイルログはデフォルト無効 |
| リソース | モデルは 30 秒のアイドル後にアンロード |
| 正式対応 | macOS Apple Silicon + Zsh |
| 実験的 | Bash Hook。Linux と Intel macOS は未リリース |

## CLI

```text
shellclaw status          daemon の状態を表示
shellclaw start           daemon を起動
shellclaw stop            daemon を停止
shellclaw log on|off      永続ファイルログを有効・無効化
shellclaw setup PATH      管理対象の Zsh Hook を設定・更新
shellclaw --version       インストール済みバージョンを表示
shellclaw help            全コマンドを表示
```

主な環境変数：

```bash
# モデル、DB、Socket、設定の保存先を変更
export SHELLCLAW_DATA_DIR=/your/data/directory

# 別の互換 GGUF モデルを使用
export SHELLCLAW_MODEL_PATH=/path/to/model.gguf
```

## ソースからビルド

Rust 1.80 以降が必要です。macOS では Metal が自動的に有効になります。

```bash
git clone https://github.com/Edwardd02/Shell-Claw.git
cd Shell-Claw
cargo build --release
cargo test --workspace
```

完全なインストールには、パッケージされた Zsh Hook と互換 GGUF モデルも必要です。
一式を導入する最も簡単な方法は Homebrew です。

## トラブルシューティング

**候補が表示されない**

```bash
shellclaw status
ls ~/.shellclaw/models/*.gguf
```

インストール後は新しい Zsh ターミナルを開いてください。コマンドがすでに完成
している、応答が古い、安全な suffix がない場合も、ShellClaw は意図的に何も
表示しません。

**モデルのダウンロードが止まった**

```bash
brew postinstall shellclaw
```

一時ファイルから再開し、Hugging Face と ModelScope を自動的に切り替えます。

**アップグレード後も Tab で ShellClaw の候補が確定される**

旧バージョンの Hook は `Tab` を使用していました。アップグレード後に新しい
ターミナルを開いて Zsh に新しい Hook を読み込ませると、`Tab` はネイティブ補完
専用になります。

**完全にアンインストールする**

```bash
shellclaw stop
brew uninstall shellclaw
rm -rf ~/.shellclaw
```

最後のコマンドで、ダウンロードしたモデルとローカルコマンドメモリを削除します。

## プライバシー

ShellClaw はアーキテクチャからローカルファーストです：

- コマンド補完とモデル推論は Mac 上で実行されます。
- コマンドメモリは `~/.shellclaw/memory.db` に保存されます。
- テレメトリやホスト型 API は使用しません。
- 対話ログと daemon のファイルログは、明示的に有効にした場合だけ記録されます。

実行したコマンドは、今後の候補を利用習慣に合わせるためローカルメモリ DB に
保存されます。`~/.shellclaw` を削除すれば、メモリを含む全データを消去できます。

ShellClaw がターミナルを便利にしたら、プロジェクトへの
[Star](https://github.com/Edwardd02/Shell-Claw) や、実際の利用例を含む
[Issue](https://github.com/Edwardd02/Shell-Claw/issues) を歓迎します。
