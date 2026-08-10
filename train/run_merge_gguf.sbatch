#!/bin/bash
#SBATCH --job-name=ssc-merge-gguf
#SBATCH --partition=gpu
#SBATCH --gres=gpu:1
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=32G
#SBATCH --time=02:00:00
#SBATCH --output=train/logs/merge_%j.log

set -e
export HF_HOME=/oscar/home/ryao21/.cache/huggingface
mkdir -p train/logs

PY=/oscar/home/ryao21/Shell-Claw/.venv/bin/python

srun $PY train/merge_lora.py \
    --base models/qwen2.5-coder-0.5b-instruct \
    --adapter train/lora-macos-coder \
    --output models/qwen2.5-coder-0.5b-instruct-merged

srun $PY tools/llama.cpp/convert_hf_to_gguf.py \
    models/qwen2.5-coder-0.5b-instruct-merged \
    --outfile models/qwen2.5-coder-0.5b-instruct-finetuned.gguf \
    --outtype q8_0

echo "MERGE_GGUF_DONE"
