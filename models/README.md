# Model

ShellClaw 使用一个本地边缘模型做命令补全推理。

## 推荐模型

**Qwen/Qwen2.5-Coder-0.5B-Instruct（微调用于 shell 补全）**

- 仓库：https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct
- 量化后约 500MB
- 加速：Metal（macOS）、AVX2（Linux）

## 获取

1. 下载 GGUF 量化文件：
   - 参考文件：`qwen2.5-coder-0.5b-instruct-finetuned.gguf`
2. 放到 `~/.shellclaw/models/`（或任意路径，用 `SHELLCLAW_MODEL_PATH` 指定）。
3. 重启 ShellClaw 使配置生效。

## 模型路径

默认模型路径可在环境变量中覆盖：

```bash
export SHELLCLAW_MODEL_PATH=/path/to/model.gguf
```

## 许可

分发/再分发前请遵守模型自身的许可证。
