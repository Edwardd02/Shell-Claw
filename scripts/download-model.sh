#!/bin/sh
# ShellClaw — 自动下载模型(双源测速,选快源,失败自动切换)
#
# 轻量实现:纯 sh + curl,零外部依赖。
# 在 brew install (post_install) 或首次使用时调用。
#
# 用法: sh download-model.sh

set -e

MODEL="qwen2.5-coder-0.5b-instruct-finetuned.gguf"

# 双源地址
HF_REPO="aidebug/Qwen2.5-Coder-0.5B-Instruct-Shell"
MS_REPO="IrenSnow/Qwen2.5-Coder-0.5B-Instruct-Shell"

HF_URL="https://huggingface.co/${HF_REPO}/resolve/main/${MODEL}"
MS_URL="https://modelscope.cn/models/${MS_REPO}/resolve/master/${MODEL}"

# 输出目录(ShellClaw 数据目录)
DATA_DIR="${SHELLCLAW_DATA_DIR:-$HOME/.shellclaw}"
OUT_DIR="${DATA_DIR}/models"
DEST="${OUT_DIR}/${MODEL}"

# 测速用的临时字节数(1MB)
SPEED_PROBE=1048576

log() { echo "[shellclaw] $*"; }
die() { log "ERROR: $*"; exit 1; }

# 测速: 用 Range 拉前 1MB,返回下载速率(字节/秒)。失败返回 0。
speed_of() {
    url="$1"
    curl -sSL --max-time 10 -r 0-$((SPEED_PROBE - 1)) \
        -o /dev/null \
        -w '%{speed_download}' \
        "$url" 2>/dev/null
    # 失败时 curl 返回 0 或空
}

# 试下载整个文件。成功返回 0。
download_from() {
    url="$1"
    target="$2"
    log "Downloading $url"
    if curl -fL --retry 3 --retry-delay 2 \
        --output "$target" \
        "$url"; then
        return 0
    fi
    return 1
}

main() {
    mkdir -p "$OUT_DIR"

    # 已存在则直接跳过
    if [ -f "$DEST" ]; then
        sz=$(wc -c < "$DEST" 2>/dev/null || echo 0)
        if [ "$sz" -gt 1000000 ]; then
            log "Model already present: $DEST"
            exit 0
        fi
    fi

    log "Probing download sources..."

    # 测速两个源
    hf_speed=$(speed_of "$HF_URL")
    ms_speed=$(speed_of "$MS_URL")
    log "HF speed: ${hf_speed:-0} B/s | ModelScope speed: ${ms_speed:-0} B/s"

    # 选快的源先下载;失败则切另一源
    TMP="${DEST}.part"

    if [ "$(echo "$hf_speed >= $ms_speed" | bc 2>/dev/null || echo 1)" -ne 0 ]; then
        primary="$HF_URL"; backup="$MS_URL"
    else
        primary="$MS_URL"; backup="$HF_URL"
    fi

    if download_from "$primary" "$TMP"; then
        mv "$TMP" "$DEST"
    elif download_from "$backup" "$TMP"; then
        mv "$TMP" "$DEST"
    else
        rm -f "$TMP"
        die "Failed to download model from both sources."
    fi

    sz=$(wc -c < "$DEST" 2>/dev/null || echo 0)
    log "Model downloaded: $DEST ($((sz / 1024 / 1024)) MB)"
}

main "$@"
