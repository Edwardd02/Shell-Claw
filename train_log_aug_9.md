# 训练日志 — 2026-08-09

## 一、目标

让 Qwen 系列本地小模型学会 shell 命令补全：给定"已输入的命令前缀"，续出"剩余后缀"，供 Smart Shell Copilot daemon 做 Ghost Text 推理。

## 二、硬件与训练环境

| 项 | 值 |
|---|---|
| 机器 | ASUS ROG 笔记本（Ubuntu 24.04, kernel 6.17.0-14） |
| GPU | NVIDIA RTX 3080 Mobile / Max-Q **16GB**（驱动 570.211.01, CUDA 12.8） |
| CPU / RAM | AMD Ryzen 7 6800HS / 16GB |
| 工具链 | uv venv, torch 2.11.0+cu128, transformers 5.14.1, peft 0.20.0 |

### 关键教训 1：Secure Boot 阻塞 NVIDIA 驱动

驱动装好但模块不加载 → 根因是 **Secure Boot 启用**，nvidia 模块签名密钥未注册进 MOK。
修复：`sudo mokutil --import /var/lib/shim-signed/mok/MOK.der` → 重启 → 蓝屏 Enroll MOK。

### 关键教训 2：笔记本风扇在 Linux 下失效

训练中 GPU 103°C、**GPU 风扇 0 RPM**（`hwmon5/pwm2_enable=0`），热崩溃两次并导致 GPU 从 PCIe 总线掉电（需彻底断电重启恢复）。
修复：`powerprofilesctl set performance`（`platform_profile` 由 quiet 切到 performance）+ `sudo sh -c 'echo 2 > /sys/class/hwmon/hwmon5/pwm2_enable'` 使 GPU 风扇自动控制。**重启后会失效，需开机自启。**

## 三、训练脚本改造（`train/lora_finetune.py`）

### 1. 设备适配
- `--device` 默认 `mps` → `cuda`；`evaluate.py` 的硬编码 `.to("mps")` 改为自动探测。

### 2. 训练数据格式（核心改动）

初始格式 `user=命令名, assistant=整条命令` 与推理任务不符（推理是"打到一半的前缀"），模型学不会。

改为 **(已输入前缀, 剩余后缀)** 对，与 daemon 推理任务完全一致：
```
'git push origin main' -> ('git', ' push origin main'), ('git push', ' origin main')
```
- 后缀带前导空格，与 Ghost Text 渲染衔接一致
- 每条命令按 1 / 中间 / 末尾 取多个切分点（`--truncations 3` → 42,579 条样本）
- 全部 39,364 条 tldr 命令，95% 序列 ≤ 24 token，max_length=48 覆盖 99.9%

### 3. labels 掩码

手工设置 labels：prompt（user 部分）全置 `-100`，**只对 assistant 后缀计算 loss**，避免模型学"复读用户输入"。

### 4. 自定义 DataCollator（`LabelMaskedDataCollator`）

transformers 5.x 两个坑：
- `tokenizer.pad()` 不会 pad `labels` 这个未知字段，长度不一会直接崩
- v5 的 `DataCollatorForLanguageModeling` 在 `mlm=False` 时会用 `input_ids` **覆盖**自定义 labels

解决：先剥掉 labels 再 pad，最后手动按 batch 最大长度补齐 labels。

### 5. 激进训练配置

```
batch-size 48 × grad-accum 2 = 有效 batch 96
max-length 48, rank 32, alpha 64, lr 3e-4 (cosine + 10% warmup)
weight-decay 0.01, grad-clip 1.0, fp16
```
显存实测：batch 48 @ seq 34 峰值 9.1GB，最坏(seq 80) 13.4GB，16GB 余量充足。
注意：注意力 O(n²) 在长序列会爆显存，max-length 需压到 ~48-64。

### 6. 续训支持

新增 `--resume <checkpoint目录>`（HF Trainer 原生 `resume_from_checkpoint`），中断后可从断点继续。

## 四、基座模型选型（最重要的决策）

### ❌ Qwen3-0.6B-Base 失败

| 症状 | 数值 |
|---|---|
| loss | 卡在 **3.0** 不降 |
| 输出 | `zwłaszc`（波兰语）、`敏感度高的地方`（中文）等乱码 token |
| 训练集复现 | 完全复现不了（对 `git` 永远吐同一段垃圾） |

**根因**（三重叠加）：
1. **Base 模型不懂 chat 格式**——训练时模型要同时学"对话格式"+"命令任务"两层，0.6B 容量不够
2. **Base 先验是网文**——token 概率分布偏向多语言网页高频词，命令 token 概率不占优
3. **Qwen3 模板自带 `<think>` 思考块**——污染输出，且 loss 被易学的模板 token 拉低造成假象

loss 3.0 ≈ "git 后面该接什么"的信息熵（一对多歧义），说明模型学到分布极限但命令先验是乱的。

### ✅ Qwen2.5-Coder-0.5B-Instruct 成功

| 症状 | 数值 |
|---|---|
| loss（8000 条快测） | 4.86 → **1.04** |
| loss（全量 42,579×3 epoch） | 6.74 → **0.754**（1332 步） |
| 输出 | 全部合法命令续写，无乱码/换行/重复循环 |

评估效果：`pip→install`、`systemctl enable→--now`、`kubectl get→all`、`docker→compose down --rmi all` 等。

**结论**：Instruct/Coder 基座只差"任务"一层要学，Base 要连"格式+先验"两层一起学，小模型学不动。

## 五、GGUF 转换链路（LoRA → daemon 可用）

```
LoRA adapter (train/lora-coder-full)
  → merge_lora.py 合并进基座 (models/qwen2.5-coder-merged/)
  → llama.cpp convert_hf_to_gguf.py 转 q8_0
  → models/qwen2.5-coder-instruct.gguf (507MB)
```

关键点：
- 合并脚本：`train/merge_lora.py`（`merge_and_unload` + `.half()`）
- llama.cpp 转换代码只需要 `convert_hf_to_gguf.py` + `conversion/` 包（84 个 .py），已存到 `tools/llama.cpp/`
- **`gguf` pip 包必须和 llama.cpp 版本匹配**（PyPI 旧版缺 `DFLASH`）→ 从 llama.cpp 仓库 `gguf-py` 装 0.19.0
- Qwen2.5 是 BPE，合并后需补 `vocab.json` + `merges.txt`（`save_pretrained` 可能不保存）；还要装 `sentencepiece`（转换代码 import 顺序问题）
- GGUF 为 q8_0，290 张量，525MB，架构 Qwen2ForCausalLM，chat template 已嵌入

## 六、⚠️ 未解决：训练/推理 system prompt 不一致（后续必须处理）

**问题**：daemon（`crates/daemon/src/model/adapter.rs:191`）用自定义 system prompt：
```
"You are a shell command autocomplete engine. The user typed a shell command prefix..."
```
而训练/evaluate 用的是 Qwen 默认 system：
```
"You are Qwen, created by Alibaba Cloud. You are a helpful assistant."
```

**后果**：模型训练时从没见过 daemon 的 system prompt，0.5B 模型遇到陌生指令退化乱吐（Mac 上 `mv` 输出 `#` 被 `clean_suffix` 拒绝）。对照实验证明只换 system prompt，同一输入输出从 `-Force` 变成 `..`。

**修复方案（二选一，未实施）**：
1. 快速修：把 adapter.rs 的 system 改为与训练一致的 Qwen 默认（1 行，免重训）
2. 正确修：用 daemon 的 autocomplete system prompt **重训**（训练/推理完全对齐，效果最好）

## 七、产物清单

| 路径 | 说明 |
|---|---|
| `train/lora_finetune.py` | 训练脚本（数据格式/labels/collator/续训/激进配置） |
| `train/evaluate.py` | 评估脚本（CUDA 支持、本地模型路径） |
| `train/merge_lora.py` | LoRA 合并脚本 |
| `tools/llama.cpp/` | GGUF 转换最小工具集（convert_hf_to_gguf.py + conversion/） |
| `models/qwen2.5-coder-instruct.gguf` | **最终模型**（507MB, q8_0, 已 gitignore） |
| `train/lora-coder-full/` | 最终 LoRA adapter + checkpoint（已 gitignore） |

## 八、经验教训总结

1. **选对基座 > 调参**：instruct/coder 基座 + 匹配任务的数据格式，比任何超参都重要
2. **训练数据要贴合推理**：数据格式必须和实际调用完全一致（包括 system prompt！）
3. **笔记本训练要管散热**：Linux 下 ASUS 风扇失效是崩溃元凶，性能模式 + 手动风扇 + 温度监控缺一不可
4. **loss 卡住先算熵**：一对多任务 loss 停在 ~1 是信息熵下限，不是没学好；对比 base 崩在 3.0 才是真失败
