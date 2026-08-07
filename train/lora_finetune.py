#!/usr/bin/env python3
"""
用 tldr 提取的命令数据 LoRA 微调 Qwen3-0.6B-Base,使其学会命令补全。

方案 B:输入"命令名/前缀",模型续出"完整命令"。
训练样例(因果 LM):
    input:  "git"
    target: "git push"
通过 Qwen chat template 包成:
    <|im_start|>user
    git
    <|im_end|>
    <|im_start|>assistant
    git push
    <|im_end|>

用法:
    python3 train/lora_finetune.py --data train/commands.jsonl \
        --model Qwen/Qwen3-0.6B-Base \
        --output train/lora-out
"""

import argparse
import json
import re
from pathlib import Path

import torch
from datasets import Dataset
from peft import LoraConfig, get_peft_model
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
    DataCollatorForLanguageModeling,
)

# 命令名(完整命令的第一个词)vs 完整命令的切分
# input=命令的第一个 token 片段,  target=完整命令
def split_prefix(cmd: str):
    """把完整命令切成 '命令名' 和 '完整命令'。输入用完整命令本身,
    让模型学习 '命令名 + 其余' 的联想。这里 input 直接给完整命令的前缀
    (第一个词), target 是完整命令。"""
    first_word = cmd.split()[0] if cmd.split() else cmd
    return first_word, cmd


def clean_cmd(cmd: str) -> str:
    """清理:去首尾空白,避免奇怪的不可见字符污染 tokenizer。"""
    return cmd.strip()


def build_dataset(jsonl: Path):
    """从 commands.jsonl 构建训练样本列表。"""
    samples = []
    with open(jsonl, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            cmd = clean_cmd(obj.get("cmd", ""))
            if not cmd:
                continue
            # 输入=命令名(第一个词), 目标=完整命令
            prefix, full = split_prefix(cmd)
            if not prefix or not full:
                continue
            samples.append((prefix, full))
    return samples


def tokenize_sample(tokenizer, prefix: str, full: str):
    """构造 Qwen chat 格式: user=prefix, assistant=full, 返回 input_ids 等。
    注意:不手写 labels —— DataCollatorForLanguageModeling(mlm=False) 会
    自动把 labels 设成 input_ids,并对 pad_token 置 -100 忽略。"""
    messages = [
        {"role": "user", "content": prefix},
        {"role": "assistant", "content": full},
    ]
    text = tokenizer.apply_chat_template(
        messages, tokenize=False, add_generation_prompt=False
    )
    enc = tokenizer(
        text,
        truncation=True,
        max_length=512,
        padding=False,
    )
    # 只返回 input_ids/attention_mask;labels 交给 data collator 生成
    return {
        "input_ids": enc["input_ids"],
        "attention_mask": enc["attention_mask"],
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data", default="train/commands.jsonl")
    parser.add_argument("--model", default="Qwen/Qwen3-0.6B-Base")
    parser.add_argument("--output", default="train/lora-out")
    parser.add_argument("--epochs", type=int, default=3)
    parser.add_argument("--lr", type=float, default=2e-4)
    parser.add_argument("--batch-size", type=int, default=8)
    parser.add_argument("--max-samples", type=int, default=0,
                        help="0=全部,否则只取前 N 条(快速实验)")
    parser.add_argument("--device", default="mps")
    args = parser.parse_args()

    # 1. 加载 tokenizer + model (0.6B 直接用 fp16,无需量化)
    print(f"[1/5] 加载模型 {args.model} ...")
    tokenizer = AutoTokenizer.from_pretrained(args.model)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    model = AutoModelForCausalLM.from_pretrained(
        args.model,
        dtype=torch.float16,
    )
    if args.device != "cpu" and torch.backends.mps.is_available():
        model = model.to("mps")

    # 2. 构建数据
    print("[2/5] 加载数据 ...")
    samples = build_dataset(Path(args.data))
    if args.max_samples and args.max_samples > 0:
        samples = samples[: args.max_samples]
    print(f"      共 {len(samples)} 条训练样本")

    # 3. tokenize
    print("[3/5] tokenize ...")
    tokenized = [tokenize_sample(tokenizer, p, f) for p, f in samples]
    dataset = Dataset.from_list(tokenized)

    # 4. LoRA 配置
    print("[4/5] 配置 LoRA ...")
    lora_config = LoraConfig(
        r=16,
        lora_alpha=32,
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"],
        lora_dropout=0.1,
        bias="none",
    )
    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    # 5. 训练
    print("[5/5] 开始训练 ...")
    train_args = TrainingArguments(
        output_dir=args.output,
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        learning_rate=args.lr,
        logging_steps=20,
        save_steps=500,
        save_total_limit=2,
        remove_unused_columns=False,
        report_to=[],
    )

    collator = DataCollatorForLanguageModeling(
        tokenizer=tokenizer, mlm=False
    )

    trainer = Trainer(
        model=model,
        args=train_args,
        train_dataset=dataset,
        data_collator=collator,
    )
    trainer.train()
    trainer.model.save_pretrained(args.output)
    tokenizer.save_pretrained(args.output)
    print(f"训练完成,LoRA 保存到 {args.output}")


if __name__ == "__main__":
    main()
