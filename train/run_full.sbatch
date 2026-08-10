#!/bin/bash
#SBATCH --job-name=ssc-lora-full
#SBATCH --partition=gpu
#SBATCH --gres=gpu:1
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=32G
#SBATCH --time=12:00:00
#SBATCH --output=train/logs/full_%j.log

set -e
export HF_HOME=/oscar/home/ryao21/.cache/huggingface
mkdir -p train/logs

srun /oscar/home/ryao21/Shell-Claw/.venv/bin/python train/lora_finetune.py \
    --data train/commands-macos.jsonl \
    --model models/qwen2.5-coder-0.5b-instruct \
    --output train/lora-macos-coder \
    --epochs 3 --batch-size 48 --grad-accum 2 --max-length 48 \
    --rank 32 --alpha 64 --lr 3e-4 --warmup-ratio 0.1 \
    --weight-decay 0.01 --truncations 2 --device cuda --seed 42

echo "FULL_TRAIN_DONE"
