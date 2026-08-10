#!/bin/bash
#SBATCH --job-name=ssc-eval
#SBATCH --partition=gpu
#SBATCH --gres=gpu:1
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=24G
#SBATCH --time=00:30:00
#SBATCH --output=train/logs/eval_%j.log

set -e
export HF_HOME=/oscar/home/ryao21/.cache/huggingface

srun /oscar/home/ryao21/Shell-Claw/.venv/bin/python train/evaluate.py \
    --adapter train/lora-macos-coder

echo "EVAL_DONE"
