#!/bin/bash
#SBATCH --job-name=ssc-lora-test
#SBATCH --partition=gpu
#SBATCH --gres=gpu:1
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=32G
#SBATCH --time=02:00:00
#SBATCH --output=train/logs/test_%j.log

set -e
export HF_HOME=/oscar/home/ryao21/.cache/huggingface
mkdir -p train/logs

srun /oscar/home/ryao21/Shell-Claw/.venv/bin/python train/lora_finetune.py \
    --data train/commands-macos.jsonl \
    --model models/qwen2.5-coder-0.5b-instruct \
    --output train/lora-macos-coder-test \
    --max-samples 3000 --epochs 2 --batch-size 48 \
    --device cuda

echo "TEST_DONE"
