# 训练日志 — 2026-08-10

> 本轮在集群（Brown OSCAR, NVIDIA RTX A6000）上复刻并验证 `train_aug_10.md` 训练计划。
> 关键变更：基座从 Qwen3-0.6B-Base **换为 Qwen2.5-Coder-0.5B-Instruct**（文档更新后的推荐）。

## 一、目标

微调 Qwen2.5-Coder-0.5B-Instruct，学会 Unix 命令补全（`(已输入前缀, 剩余后缀)` 配对），产出 GGUF 接回 daemon 的 llama.cpp 推理。

## 二、硬件与训练环境（集群）

| 项 | 值 |
|---|---|
| 调度器 | SLURM（partition: gpu, gres: 1×GPU） |
| GPU | NVIDIA RTX A6000（训练/评估/合并/转换均跑在 GPU 节点） |
| 工具链 | `.venv`: torch 2.6.0+cu124, transformers 5.14.1, peft 0.20.0, datasets 5.0.1 |
| 数据 | `train/commands-macos.jsonl`（tldr 提取，27217 条，common + osx） |
| 队列 | `squeue` 提交，日志在 `train/logs/` |

## 三、基座模型选型（本轮核心决策）

- **✅ Qwen2.5-Coder-0.5B-Instruct**（HF 下载 safetensors 988MB → `models/qwen2.5-coder-0.5b-instruct/`）
- **❌ 弃用 Qwen3-0.6B-Base** —— 上一轮本集群实测失败：
  - loss 卡 **3.0** 不降，评估输出 `zwłaszc`（波兰语）、`najczęście` 等乱码 token
  - 根因：Base 模型要连"对话格式"+"命令先验"两层一起学，0.5B 学不动；Qwen3 模板自带 `<think>` 污染
  - 教训与 `train_log_aug_9.md` 完全一致：**选对基座 > 调参**

## 四、训练执行

### 阶段 1：快速验证（job 4822685, ~1min）

```
--max-samples 3000 --epochs 2 --batch-size 48
```

loss 曲线（64 步）：
```
3.971 → 2.073 → 1.809 → 1.35 → 1.202 → 1.167
```
✅ 收敛趋势明显（对比 Qwen3 卡死 3.0），批准全量训练。

### 阶段 2：全量训练（job 4822713, 约 7 min）

```
--data train/commands-macos.jsonl
--model models/qwen2.5-coder-0.5b-instruct
--output train/lora-macos-coder
--epochs 3 --batch-size 48 --grad-accum 2 --max-length 48
--rank 32 --alpha 64 --lr 3e-4 --warmup-ratio 0.1
--weight-decay 0.01 --truncations 2 --device cuda --seed 42
```

- 有效 batch = 48×2 = 96，5.4 万训练对，83 次 loss 记录
- loss：`6.445 → 0.8167`，train_loss = **1.492**（与 `train_log_aug_9.md` 全量 0.754 量级一致）

### 阶段 3：评估（job 4822839）

| 前缀 | 补全 | 评价 |
|---|---|---|
| `git` | `diff-files --summary` | ✅ 合法 |
| `git che` | `--help` | ✅ 合法（未选 checkout，属 greedy 次级高频） |
| `docker` | `compose up --build` | ✅ 合理 |
| `pip inst` | `--user` | ✅ 合法 |
| `npm r` | `--json` | ✅ 合法 |
| `systemctl enable` | `--now` | ✅ 合理（文档实测一致） |
| `brew upd` | `--all` | ✅ 合理 |
| `kubectl get` | `all` | ✅ 合理（文档实测一致） |
| `pip` | `install` | ✅ 短前缀泛化正确 |
| 全部 | 无乱码 / 无换行 / 无重复循环 | ✅ 相比 Qwen3 彻底修复 |

## 五、GGUF 转换链路（LoRA → daemon 可用）

```
LoRA adapter (train/lora-macos-coder)
  → merge_lora.py 合并进基座 (models/qwen2.5-coder-0.5b-instruct-merged/)
  → convert_hf_to_gguf.py 转 q8_0
  → models/qwen2.5-coder-0.5b-instruct-finetuned.gguf (525MB, 290 张量)
```

### 踩坑与修复（本轮新增）

| 坑 | 修复 |
|---|---|
| PyPI `gguf==0.19.0` 缺 `DFLASH` 属性 | 从 llama.cpp 仓库（master）复制配套 `gguf-py/` 到 `tools/llama.cpp/gguf-py/`（convert 脚本优先从该目录导入） |
| 缺 `sentencepiece` | `.venv/bin/pip install sentencepiece` |
| 合并后缺 `vocab.json` + `merges.txt`（BPE） | 从基座目录 `cp` 补上（与 aug_9 文档记录一致） |

转换验证：dry-run 通过后真实转换，`Model successfully exported`，chat template 已嵌入。

## 六、daemon 接入（阶段 5）

- `crates/daemon/src/config/mod.rs`：默认 `model_path` 改为 `models/qwen2.5-coder-0.5b-instruct-finetuned.gguf`
- system prompt 对齐确认：Qwen2.5-Coder 的 `apply_chat_template` **自动注入**默认 system
  `"You are Qwen, created by Alibaba Cloud. You are a helpful assistant."`
  与 `adapter.rs:194-196` 完全一致 —— 训练/推理无需额外改动
- 同步更新：systemd / launchd / homebrew / `internal_readme.md` / `models/README.md` / 基准脚本 / quickstart 中的模型路径
- ⚠️ 集群无 Rust 工具链，daemon 二进制需在本地机重新 `cargo build --release` 后验证 Ghost Text

## 七、产物清单

| 路径 | 说明 |
|---|---|
| `models/qwen2.5-coder-0.5b-instruct/` | 基座 safetensors（988MB, 可重下） |
| `train/lora-macos-coder/` | 最终 LoRA adapter（3 epoch） |
| `train/lora-macos-coder-test/` | 快速验证 adapter（3000 样本） |
| `models/qwen2.5-coder-0.5b-instruct-merged/` | 合并后 HF 模型（958MB, 含补齐 vocab） |
| `models/qwen2.5-coder-0.5b-instruct-finetuned.gguf` | **最终模型**（525MB, q8_0, 已 gitignore） |
| `tools/llama.cpp/gguf-py/` | 配套 gguf-py（修复 DFLASH） |
| `train/run_{test,full,eval,merge_gguf,gguf}.sbatch` | 复现用 SLURM 脚本 |

## 八、复现步骤

```bash
# 1. 训练（测试 + 全量）
sbatch train/run_test.sbatch
sbatch train/run_full.sbatch

# 2. 评估
sbatch train/run_eval.sbatch

# 3. 合并 + 转 GGUF
sbatch train/run_merge_gguf.sbatch   # 已含 merge_lora.py + convert_hf_to_gguf.py
```

## 九、经验教训补充

1. **基座选型是决定性因素**：同一套数据/脚本/超参，Coder-Instruct loss 到 0.8，Base 卡 3.0 —— 印证 `train_log_aug_9.md`
2. **gguf 包版本要与转换代码同源**：PyPI 版缺 `DFLASH`，必须用 llama.cpp 仓库的 `gguf-py/`
3. **合并产物要补 BPE vocab**：`save_pretrained` 不写 `vocab.json`/`merges.txt`，转换前 `cp` 补齐
4. **Qwen2.5 模板自动注入默认 system**，与推理端手动添加的保持一致即可，无需重训
