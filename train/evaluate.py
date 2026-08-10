#!/usr/bin/env python3
"""
评估微调后的 LoRA 模型,测试命令补全效果(完整命令联想 + 减法)。

用法:
    python3 train/evaluate.py --adapter train/lora-macos-coder

评估方式(与 daemon 推理契约一致):
    模型被训练成: 输入命令名±空格, 输出【完整命令】(如 'git' -> 'git status')
    这里展示: 模型输出的完整命令, 以及做减法后的实际后缀。

用法:
    python3 train/evaluate.py --adapter train/lora-macos-coder
"""

import argparse
import os
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

BASE = os.environ.get("SSC_BASE_MODEL", "models/qwen2.5-coder-0.5b-instruct")


def get_device():
    if torch.cuda.is_available():
        return "cuda"
    if torch.backends.mps.is_available():
        return "mps"
    return "cpu"


DEVICE = get_device()


def load_model(base: str, adapter_path: str):
    print(f"[1/4] 基础模型 {base}")
    tok = AutoTokenizer.from_pretrained(base)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    model_base = AutoModelForCausalLM.from_pretrained(base, dtype=torch.float16).to(DEVICE)
    print(f"[2/4] LoRA adapter {adapter_path}")
    model = PeftModel.from_pretrained(model_base, adapter_path)
    model.eval()
    return model, tok


def complete_full_cmd(model, tok, prefix: str, max_new=64):
    """让模型输出完整命令(新训练契约: user=命令名±空格, assistant=完整命令)。"""
    msgs = [{"role": "user", "content": prefix}]
    txt = tok.apply_chat_template(msgs, tokenize=False, add_generation_prompt=True)
    ids = tok(txt, return_tensors="pt", truncation=True, max_length=512).input_ids.to(DEVICE)
    with torch.no_grad():
        out = model.generate(input_ids=ids, max_new_tokens=max_new, do_sample=False)
    return tok.decode(out[0][len(ids[0]):], skip_special_tokens=True).strip()


def sub_suffix(full_cmd: str, prefix: str):
    """做减法: 完整命令 - 用户前缀 = 待插入后缀(与 daemon adapter.rs 一致)。"""
    if not full_cmd:
        return ""
    if full_cmd.startswith(prefix):
        return full_cmd[len(prefix):]
    return ""  # 模型没以用户前缀开头,视为无法可靠减法


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--adapter", required=True, help="LoRA 输出目录")
    parser.add_argument("--base", default=BASE)
    args = parser.parse_args()

    model, tok = load_model(args.base, args.adapter)

    # 测试:命令名(可能带尾随空格),这是新训练实际覆盖的输入
    tests = [
        "git", "git ", "docker", "docker ", "brew", "brew ",
        "ls", "ls ", "pip", "pip ", "npm", "npm ", "systemctl", "systemctl ",
        "kubectl", "kubectl ", "cd", "cd ",
    ]

    print(f"\n[4/4] 评估 ({DEVICE})")
    print("(模型应输出完整命令; suffix = 完整命令 - 前缀 = daemon 实际插入的补全)\n")
    for prefix in tests:
        full = complete_full_cmd(model, tok, prefix)
        suffix = sub_suffix(full, prefix)
        print(f"  输入[{prefix!r:12}] 完整[{full!r:30}] → suffix[{suffix!r}]")


if __name__ == "__main__":
    main()
