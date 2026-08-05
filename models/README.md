# Model Files

Smart Shell Copilot uses a local edge model for command completion inference.

## Primary Model

**Qwen/Qwen3-0.6B-Base**

- Repository: https://huggingface.co/Qwen/Qwen3-0.6B-Base
- Expected memory: ~500MB
- Hardware acceleration: Metal (macOS), AVX2 (Linux)

## Fallback Model

**Qwen2.5-Coder-0.5B-Instruct**

- Repository: https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct
- Used if the primary model misses TTFT or memory gates

## Acquisition

1. Download the GGUF quantized file from the repository.
2. Place it in this directory as `qwen3-0.6b-base.gguf` (or `qwen2.5-coder-0.5b-instruct.gguf` for fallback).
3. Update the daemon config model path to point to this file.

## Conversion

If a GGUF file is not available, convert using:

```bash
python convert.py <model_dir> --outtype q8_0
```

## License Compliance

Ensure compliance with the model's license before distribution or redistribution.
