# Smart Shell Copilot — LoRA 微调训练计划

> 日期：2026-08-10
> 目标：微调 Qwen3-0.6B-Base，让它学会 Unix 命令补全，并接回 daemon 的 llama.cpp 推理。

---

## 一、训练目标（用什么格式做补全）

用 **`(已输入前缀, 剩余后缀)`** 的配对训练，贴合真实交互：

```
用户打到 "git che"  →  补 "ckout main"
用户打到 "git"      →  补 " push origin main"
```

具体地，`lora_finetune.py` 的 `split_pairs` 会把一条命令切成多个训练对：

```
'git push origin main'  →  ('git', ' push origin main')     # 只打了第1个词
                        →  ('git push', ' origin main')     # 打了前2个词
```

> 关键设计：**后缀带前导空格**（` push origin main`），和 daemon 渲染 Ghost Text 时的衔接一致。
> 且只监督 assistant 输出（prompt 部分 label 置 -100），避免模型"复读用户输入"。

---

## 二、提示词（Prompt）设计

### 训练时（tokenize_sample）
```
user:       git che
assistant:  ckout main
```
- 已通过 Qwen chat template 包成 `<|im_start|>user ... <|im_start|>assistant ...`
- **不额外加 system**（或只用训练时见过的 Qwen 默认 system）
- 用 `apply_chat_template` 确保和推理一致

### 推理时（daemon adapter.rs 的 build_instruct_prompt）
**必须和训练完全一致**，否则 0.5B 模型会退化（这是我们踩过的坑）：

```rust
let system = "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.";  // 训练用的 Qwen 默认
let user = prefix;   // 用户已输入的前缀
// apply_chat_template(add_ass=true) -> 末尾加 assistant 起始符,模型续后缀
```

> **红线**：推理的 system prompt 必须 = 训练时的 system prompt。
> 0.5B 模型对"没见过的指令"毫无鲁棒性 —— 换一个 system 就会随机吐 `#`/`..`。

---

## 三、数据准备

### 数据来源（tldr 提取）
用 `train/extract.py`，按平台生成两个数据集：

```bash
# macOS 目标（推荐）：common + osx（27117 条）
python3 train/extract.py --dirs common osx --output train/commands-macos.jsonl

# Linux 目标：common + linux（36462 条）
python3 train/extract.py --dirs common linux --output train/commands-linux.jsonl
```

> **为什么按平台分**：macOS 工具是 BSD、Linux 是 GNU，参数有差异；
> 且 tldr linux/ 页 97% 与 mac 不兼容。按目标平台选数据可避免污染。

### 每条数据的形态
```json
{"cmd": "git push origin main"}
{"cmd": "ls -la"}
```

---

## 四、微调配置（lora_finetune.py 参数）

脚本已支持：LoRA、切分训练对、手工 labels、梯度累积、续训。

**推荐配置（在 Linux/3080ti 上）**：

```bash
python3 train/lora_finetune.py \
    --data train/commands-macos.jsonl \
    --model models/qwen3-0.6b-base \
    --output train/lora-macos \
    --epochs 3 \
    --batch-size 48 \
    --grad-accum 2 \
    --max-length 48 \
    --rank 32 \
    --alpha 64 \
    --lr 3e-4 \
    --warmup-ratio 0.1 \
    --weight-decay 0.01 \
    --truncations 2 \
    --device cuda \
    --seed 42
```

### 参数详解

| 参数 | 推荐 | 说明 |
|---|---|---|
| `--data` | commands-macos.jsonl | 训练数据 |
| `--model` | models/qwen3-0.6b-base | 基础模型（本地路径，需先下载 safetensors）|
| `--epochs` | 3 | 遍历数据次数 |
| `--batch-size` | 48 | 单 batch；3080ti 显存大可以高 |
| `--grad-accum` | 2 | 有效 batch = 48×2 = 96 |
| `--max-length` | 48 | 截断长度；数据 p99=34，48 覆盖足够 |
| `--rank` / `--alpha` | 32 / 64 | LoRA 秩 / 缩放 |
| `--lr` | 3e-4 | 学习率 |
| `--truncations` | 2 | 每条命令切 2 个 (前缀,后缀) 对 |
| `--device` | cuda | 自动探测后端 |
| `--resume` | 空 | 续训某个 checkpoint |
| `--max-samples` | 0 | 0=全部；设小值快速实验 |

### 有效样本量估算
macos 数据 27217 条 × `truncations=2` ≈ **5.4 万训练对**。0.6B + LoRA，3080ti 上约 **20-40 分钟/epoch**，3 epoch 约 1-2 小时。

---

## 五、小规模快速验证（推荐先跑）

正式训练前，先用一小部分确认能收敛、不报错：

```bash
python3 train/lora_finetune.py \
    --data train/commands-macos.jsonl \
    --model models/qwen3-0.6b-base \
    --output train/lora-macos-test \
    --max-samples 3000 --epochs 2 --batch-size 48 \
    --device cuda
```

观察 loss：应从 ~2-3 逐渐降到 **<1**（说明学到东西）。

---

## 六、评估（train/evaluate.py）

训练完（用 `--output` 目录，如 `train/lora-macos`）：

```bash
python3 train/evaluate.py --adapter train/lora-macos
```

内置测试命令，看补全是否合理：
```bash
# 期望（grep 是检查命令是否正常）
git che    -> ckout ...
pip ins    -> tall requests...
# 短前缀泛化
git        -> 继续补 push/log/...
```

> 若仍乱，检查：(1) system prompt 是否对齐训练；(2) loss 是否真下降。

---

## 七、合并 LoRA 权重 + 转 GGUF（可参考 train/ 下工具链）

微调产出的是 LoRA adapter（PEFT）。要接回 llama.cpp 的 daemon，需要：

1. **合并权重**：把 LoRA adapter 合并进基础模型，导出完整 safetensors
```bash
python3 -c "
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel
base = AutoModelForCausalLM.from_pretrained('models/qwen3-0.6b-base', dtype='float16')
merged = PeftModel.from_pretrained(base, 'train/lora-macos').merged
merged.save_pretrained('models/qwen3-0.6b-base-merged')
"
```

2. **转 GGUF**：用 llama.cpp 的转换脚本（如果你的环境有）：
```bash
python3 llama.cpp/convert_hf_to_gguf.py \
    models/qwen3-0.6b-base-merged --outfile models/qwen3-0.6b-base-finetuned.gguf --outtype q8_0
```

3. **更新 daemon 模型路径**，重启测试：
```bash
SSC_MODEL_PATH=models/qwen3-0.6b-base-finetuned.gguf
```

---

## 八、常见问题

| 问题 | 解决 |
|---|---|
| loss 不降 / 一直 ~3 | 学习率太低、数据有问题 |
| 推理说 `#` / 乱 token | system prompt 和训练不一致（见第二节红线）|
| CUDA 不可用 | `nvidia-smi` 检查驱动；torch 换 cu12x 版 |
| GGUF 转换失败 | 先确认已合并权重；需要 llama.cpp repo |
| 命令补全带 Windows 风格 | 用了 linux/windows 混合数据；换成按平台数据集 |

---

## 九、本次训练验证 Checklist

- [ ] 用 macos 数据集（无 Windows 污染）
- [ ] 先 `--max-samples 3000` 快速验证，loss < 1
- [ ] 全量 macos 训练 3 epoch
- [ ] evaluate.py 补全合理（git/pip/ls...）
- [ ] 合并权重 + 转 GGUF
- [ ] daemon 用新模型补全正常，且 system prompt 对齐
