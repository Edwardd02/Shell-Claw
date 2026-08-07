# Smart Shell Copilot — LoRA 微调中文指南

在 **Linux + NVIDIA 3080ti** 上微调 Qwen3-0.6B-Base,让模型学会 shell 命令补全。

---

## 一、当前进度（已完成的）

| 内容 | 文件 | 状态 |
|---|---|---|
| tldr 命令提取脚本 | `train/extract.py` | ✅ 用 |
| 提取出的命令数据（39364 条） | `train/commands.jsonl` | ✅ 已生成 |
| LoRA 微调脚本（transformers 5.x） | `train/lora_finetune.py` | ✅ 已写好并跑通 smoke test |
| 评估脚本 | `train/evaluate.py` | ✅ 已写好 |
| Mac 上快速验证（2000条） | `train/lora-2000-test/` | ⚠️ 仅 Mac 上试跑过，效果不佳（数据量小） |

**为什么转 Linux？** Mac (M4/16GB) 用 MPS 训练 0.6B 虽能跑，但**很慢**。3080ti 有 12GB 显存 + CUDA，训练快得多、还能吃下更大 batch。

---

## 二、下一步计划

1. **在 Linux/3080ti 上装环境**（见第三节）
2. **用全部 39364 条数据微调**（3 epoch，3080ti 预计 20-60 分钟）
3. **评估**是否学会了命令补全
4. **转 GGUF** 接回 smart-shell-copilot 的 llama.cpp 推理

---

## 三、Linux 上需要安装的东西

### 1. 基础环境（系统/驱动）

```bash
# NVIDIA 驱动 + CUDA 已装好（nvidia-smi 有输出即可）
nvidia-smi          # 确认显示 3080ti、CUDA 版本

# Python 3.10-3.12
python3 --version
```

### 2. 创建虚拟环境（推荐）

```bash
python3 -m venv .venv
source .venv/bin/activate
```

### 3. 安装 PyTorch（CUDA 版）

根据你的 CUDA 版本选。通常官方命令：

```bash
# CUDA 12.x 一般用这个（3080ti 支持）
pip install torch --index-url https://download.pytorch.org/whl/cu121
```

验证 CUDA 可用：
```bash
python3 -c "import torch; print(torch.cuda.is_available(), torch.version.cuda)"
# 应输出 True 12.x
```

### 4. 安装训练相关库

```bash
pip install transformers peft datasets accelerate
```

验证：
```bash
python3 -c "from transformers import Trainer; from peft import LoraConfig; print('ok')"
```

---

## 四、开始训练

### 同步代码和数据

把你的 `train/` 目录（含 `commands.jsonl`、`lora_finetune.py`、`evaluate.py`）弄到 Linux 机器上（git clone 或 scp）。

### 用全部数据训练

```bash
cd <你的项目>/train
python3 lora_finetune.py \
    --data commands.jsonl \
    --model Qwen/Qwen3-0.6B-Base \
    --output lora-checkpoint \
    --epochs 3 \
    --batch-size 8 \
    --device cuda
```

> `--device cuda`：脚本会用 CUDA（3080ti）。如果 12GB 显存 OOM，把 `--batch-size` 降到 4。

**期望输出**：
- 每 20 步打印 loss，应**逐渐下降**（从 3-4 → 降到 1 以下）
- 完成后 `train/lora-checkpoint` 生成 LoRA adapter（`adapter_model.safetensors`）

### 先做小规模快速验证（可选，推荐）

怕全量太久，可先 3000 条确认能收敛：
```bash
python3 lora_finetune.py --data commands.jsonl \
    --max-samples 3000 --epochs 3 --batch-size 8 \
    --output lora-test --device cuda
```

---

## 五、评估

训练完（用 `lora-checkpoint` 或 `lora-test`）：

```bash
python3 evaluate.py --adapter lora-checkpoint
```

**看结果**：
- ✅ 好：`git`→`push`、`git che`→`ckout...`、`pip inst`→`all requests` 等
- ⚠️ 差/胡言：说明数据切分或训练有问题，回来调

---

## 六、常见问题

| 问题 | 解决 |
|---|---|
| `torch.cuda.is_available()` = False | 装了 CPU 版 torch；重装 CUDA 版 `--index-url .../cu121` |
| OOM（显存不足） | 减小 `--batch-size`（8→4）、减小 `--max-length` |
| transformers 版本报错 | 确认 ≥5.0；脚本按 5.x 写的 |
| 模型下载慢 | 先 `huggingface-cli download Qwen/Qwen3-0.6B-Base` 预热 |
| CUDA 版本不匹配 | 看 `nvidia-smi` 顶部 CUDA 版本,换对应 `cu11x/cu12x` 索引 |

---

## 七、后续（微调成功后）

转 GGUF 接回 llama.cpp（smart-shell-copilot 用的推理后端）：
```bash
# 需先合并 LoRA 权重,再转 GGUF
# 1. 合并:transformers 加载 base + peft,保存合并后 safetensors
# 2. 转 GGUF:python llama.cpp/convert_hf_to_gguf.py ...
# 3. 替换 models/ 下的模型文件,daemon 会自动加载
```
（此步等微调效果确认后详细补充）
