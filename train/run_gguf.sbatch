#!/bin/bash
#SBATCH --job-name=ssc-to-gguf
#SBATCH --partition=gpu
#SBATCH --gres=gpu:1
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=32G
#SBATCH --time=01:00:00
#SBATCH --output=train/logs/gguf_%j.log

set -e
export HF_HOME=/oscar/home/ryao21/.cache/huggingface
mkdir -p train/logs

srun /oscar/home/ryao21/Shell-Claw/.venv/bin/python tools/llama.cpp/convert_hf_to_gguf.py \
    models/qwen2.5-coder-0.5b-instruct-merged \
    --outfile models/qwen2.5-coder-0.5b-instruct-finetuned.gguf \
    --outtype q8_0

echo "GGUF_DONE"
