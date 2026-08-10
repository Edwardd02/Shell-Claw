#!/usr/bin/env python3
"""
用 tldr 提取的命令数据 LoRA 微调 Qwen3-0.6B-Base,使其学会命令补全。

训练样例(因果 LM,只对 assistant 回复计算 loss):
    user:      "git"
    assistant: "git push"
通过 Qwen chat template 包成:
    <|im_start|>user
    git
    <|im_end|>
    <|im_start|>assistant
    git push
    <|im_end|>

用法:
    python3 train/lora_finetune.py --data train/commands.jsonl \
        --model models/qwen2.5-coder-0.5b-instruct --output train/lora-checkpoint \
        --device cuda
"""

import argparse
import json
import re
from pathlib import Path
from typing import Any, Mapping

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
from transformers.data.data_collator import pad_without_fast_tokenizer_warning


class LabelMaskedDataCollator(DataCollatorForLanguageModeling):
    """尊重 tokenize_sample 里预设好的 labels(-100 标记 prompt token)。

    v5 的 DataCollatorForLanguageModeling 在 mlm=False 时会把 labels 覆盖成
    input_ids(丢掉我们的掩码);且 tokenizer.pad() 不会填充 labels 这个未知
    字段(长度不一致直接报错)。这里先去掉 labels 再 pad,最后手动补上。
    """

    def torch_call(self, examples):
        exs = [{k: v for k, v in ex.items() if k != "labels"} for ex in examples]
        raw_labels = [ex["labels"] for ex in examples]
        batch = pad_without_fast_tokenizer_warning(
            self.tokenizer,
            exs,
            return_tensors="pt",
            pad_to_multiple_of=self.pad_to_multiple_of,
        )
        max_len = batch["input_ids"].shape[1]
        labels_t = torch.full((len(raw_labels), max_len), -100, dtype=torch.long)
        for i, lab in enumerate(raw_labels):
            lab = lab[:max_len]
            labels_t[i, : len(lab)] = torch.tensor(lab, dtype=torch.long)
        batch["labels"] = labels_t
        return batch

# 命令名(完整命令的第一个词)vs 完整命令的切分
def split_pairs(cmd: str, truncations: int = 2):
    """生成 (输入前缀, 完整命令) 训练对。

    关键思路变更:模型只学"联想完整命令",不管衔接点 ——
    assistant 恒为【完整命令】(如 'git status')。
    输入只覆盖"完整命令名 ± 尾随空格"两种:
        ('git',    'git status')
        ('git ',   'git status')
    之后 daemon 推理做减法:补全 = 完整命令 - 用户当前输入。
    这样模型不需要精确理解"用户打到哪",只学"该命令长什么样",
    避免之前 'brew upgrad' -> '--all' 这种跳中间词的问题。

    truncations 参数保留用于兼容,不再影响切分(detail:始终两种前缀)。
    """
    words = cmd.split()
    if not words:
        return []
    first = words[0]          # 命令名,如 'git'
    samples = [
        (first, cmd),         # 无空格: git      -> git status
        (f"{first} ", cmd),   # 尾随空格: git  -> git status
    ]
    return samples


def clean_cmd(cmd: str) -> str:
    """清理:去首尾空白,避免奇怪的不可见字符污染 tokenizer。"""
    return cmd.strip()


def build_dataset(jsonl: Path, truncations: int = 2):
    """从 commands.jsonl 构建 (输入前缀, 完整命令) 训练样本列表。"""
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
            for prefix, full_cmd in split_pairs(cmd, truncations):
                if not prefix or not full_cmd:
                    continue
                samples.append((prefix, full_cmd))
    return samples


def tokenize_sample(tokenizer, prefix: str, full_cmd: str, max_length: int):
    """构造 Qwen chat 格式: user=输入前缀, assistant=【完整命令】。

    关键:assistant 是完整命令(如 'git status'),模型学的是"联想完整命令"。
    手工设置 labels: 只对 assistant(完整命令)部分计算 loss,
    user 输入前缀部分置 -100(不监督,避免干扰)。
    注意:assistant 包含复读的命令名前缀(如 'git'),这是应有的(模型要输出完整命令)。
    """
    messages = [
        {"role": "user", "content": prefix},
        {"role": "assistant", "content": full_cmd},
    ]
    text = tokenizer.apply_chat_template(
        messages, tokenize=False, add_generation_prompt=False
    )
    enc = tokenizer(text, truncation=True, max_length=max_length, padding=False)
    input_ids = enc["input_ids"]

    prompt_text = tokenizer.apply_chat_template(
        [{"role": "user", "content": prefix}],
        tokenize=False,
        add_generation_prompt=True,
    )
    prompt_ids = tokenizer(prompt_text, add_special_tokens=False)["input_ids"]

    labels = [-100] * len(input_ids)
    if len(prompt_ids) < len(input_ids):
        for i in range(len(prompt_ids), len(input_ids)):
            labels[i] = input_ids[i]

    return {
        "input_ids": input_ids,
        "attention_mask": enc["attention_mask"],
        "labels": labels,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data", default="train/commands.jsonl")
    parser.add_argument("--model", default="models/qwen2.5-coder-0.5b-instruct")
    parser.add_argument("--output", default="train/lora-out")
    parser.add_argument("--epochs", type=int, default=3)
    parser.add_argument("--lr", type=float, default=3e-4)
    parser.add_argument("--batch-size", type=int, default=48)
    parser.add_argument("--grad-accum", type=int, default=2,
                        help="梯度累积,有效 batch = batch-size * grad-accum")
    parser.add_argument("--max-length", type=int, default=48,
                        help="截断长度;数据 p99=34,48 已覆盖 99.9%")
    parser.add_argument("--rank", type=int, default=32)
    parser.add_argument("--alpha", type=int, default=64)
    parser.add_argument("--warmup-ratio", type=float, default=0.1)
    parser.add_argument("--weight-decay", type=float, default=0.01)
    parser.add_argument("--max-samples", type=int, default=0,
                        help="0=全部,否则只取前 N 条(快速实验)")
    parser.add_argument("--truncations", type=int, default=2,
                        help="每条命令生成的(前缀,后缀)训练对个数")
    parser.add_argument("--device", default="cuda",
                        help="cuda/mps/cpu(自动探测可用后端)")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--resume", default="",
                        help="从指定 checkpoint 目录续训(如 train/lora-coder-full/checkpoint-1000)")
    args = parser.parse_args()

    torch.manual_seed(args.seed)

    # 1. 加载 tokenizer + model (0.6B 直接用 fp16,无需量化)
    print(f"[1/5] 加载模型 {args.model} ...")
    tokenizer = AutoTokenizer.from_pretrained(args.model)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    model = AutoModelForCausalLM.from_pretrained(
        args.model,
        dtype=torch.float16,
    )
    if args.device == "cuda" and torch.cuda.is_available():
        model = model.to("cuda")
        print(f"     使用 CUDA: {torch.cuda.get_device_name(0)}")
    elif args.device == "mps" and torch.backends.mps.is_available():
        model = model.to("mps")

    # 2. 构建数据
    print("[2/5] 加载数据 ...")
    samples = build_dataset(Path(args.data), args.truncations)
    if args.max_samples and args.max_samples > 0:
        samples = samples[: args.max_samples]
    print(f"      共 {len(samples)} 条训练样本 (truncations={args.truncations})")

    # 3. tokenize
    print("[3/5] tokenize ...")
    tokenized = [
        tokenize_sample(tokenizer, p, f, args.max_length) for p, f in samples
    ]
    dataset = Dataset.from_list(tokenized)

    # 4. LoRA 配置
    print("[4/5] 配置 LoRA ...")
    lora_config = LoraConfig(
        r=args.rank,
        lora_alpha=args.alpha,
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"],
        lora_dropout=0.1,
        bias="none",
    )
    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    # 5. 训练
    effective_batch = args.batch_size * args.grad_accum
    total_steps = max(1, len(dataset) // effective_batch) * args.epochs
    warmup_steps = max(1, int(total_steps * args.warmup_ratio))
    print(f"[5/5] 开始训练 (per-device batch={args.batch_size} x grad-accum={args.grad_accum} = 有效 batch {effective_batch}, {total_steps} 步) ...")
    train_args = TrainingArguments(
        output_dir=args.output,
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch_size,
        gradient_accumulation_steps=args.grad_accum,
        learning_rate=args.lr,
        lr_scheduler_type="cosine",
        warmup_steps=warmup_steps,
        weight_decay=args.weight_decay,
        max_grad_norm=1.0,
        fp16=True,
        logging_steps=10,
        save_steps=1000,
        save_total_limit=2,
        remove_unused_columns=False,
        report_to=[],
        seed=args.seed,
    )

    collator = LabelMaskedDataCollator(
        tokenizer=tokenizer, mlm=False
    )

    trainer = Trainer(
        model=model,
        args=train_args,
        train_dataset=dataset,
        data_collator=collator,
    )
    trainer.train(resume_from_checkpoint=args.resume or None)
    trainer.model.save_pretrained(args.output)
    tokenizer.save_pretrained(args.output)
    print(f"训练完成,LoRA 保存到 {args.output}")


if __name__ == "__main__":
    main()
