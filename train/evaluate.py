#!/usr/bin/env python3
"""
评估微调后的 LoRA 模型,测试命令补全效果。

用法:
    python3 train/evaluate.py --adapter train/lora-checkpoint

内置一组测试命令,打印每个前缀的补全结果。
"""

import argparse
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

BASE = "models/qwen2.5-coder-0.5b-instruct"


def get_device():
    if torch.cuda.is_available():
        return "cuda"
    return "cpu"


DEVICE = get_device()


def load_model(base: str, adapter_path: str):
    print(f"加载基础模型 {base} ...")
    tok = AutoTokenizer.from_pretrained(base)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    model_base = AutoModelForCausalLM.from_pretrained(
        base, dtype=torch.float16
    ).to(DEVICE)
    print(f"加载 LoRA adapter: {adapter_path} ...")
    model = PeftModel.from_pretrained(model_base, adapter_path)
    model.eval()
    return model, tok


def complete(model, tok, prefix: str, max_new=40):
    msgs = [{"role": "user", "content": prefix}]
    txt = tok.apply_chat_template(msgs, tokenize=False, add_generation_prompt=True)
    ids = tok(txt, return_tensors="pt", truncation=True, max_length=512).input_ids.to(DEVICE)
    with torch.no_grad():
        out = model.generate(input_ids=ids, max_new_tokens=max_new, do_sample=False)
    return tok.decode(out[0][len(ids[0]):], skip_special_tokens=True).strip()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--adapter", required=True, help="LoRA 输出目录")
    parser.add_argument("--base", default=BASE, help="基础模型路径(本地目录)")
    args = parser.parse_args()

    model, tok = load_model(args.base, args.adapter)

    # 记忆里有的命令(应能正确补出)
    tests_memory = [
        "git", "git che", "git st", "docker", "pip inst", "npm r",
        "systemctl enable", "brew upd", "kubectl get",
    ]
    # 需要泛化的短前缀
    tests_short = ["g", "gi", "do", "pip", "sy"]

    print("\n=== 常用命令前缀 ===")
    for p in tests_memory:
        print(f"  [{p}] -> {complete(model, tok, p)!r}")

    print("\n=== 短前缀(泛化能力)===")
    for p in tests_short:
        print(f"  [{p}] -> {complete(model, tok, p)!r}")


if __name__ == "__main__":
    main()
