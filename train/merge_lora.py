#!/usr/bin/env python3
"""
把 LoRA adapter 合并回基座模型,得到可直接转 GGUF 的完整 HF 模型。

用法:
    python3 train/merge_lora.py \
        --base models/qwen2.5-coder-instruct \
        --adapter train/lora-coder-full \
        --output models/qwen2.5-coder-merged
"""

import argparse
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", default="models/qwen2.5-coder-instruct",
                        help="基座模型路径")
    parser.add_argument("--adapter", required=True,
                        help="LoRA 输出目录(或 checkpoint 目录)")
    parser.add_argument("--output", default="models/qwen2.5-coder-merged",
                        help="合并后模型保存路径")
    args = parser.parse_args()

    print(f"[1/3] 加载基座模型 {args.base} ...")
    tokenizer = AutoTokenizer.from_pretrained(args.base)
    base = AutoModelForCausalLM.from_pretrained(args.base, dtype=torch.float16)

    print(f"[2/3] 加载并合并 LoRA adapter {args.adapter} ...")
    model = PeftModel.from_pretrained(base, args.adapter)
    merged = model.merge_and_unload()
    merged = merged.half()

    print(f"[3/3] 保存合并模型到 {args.output} ...")
    merged.save_pretrained(args.output)
    tokenizer.save_pretrained(args.output)
    print("完成!该目录可直接转 GGUF。")


if __name__ == "__main__":
    main()
