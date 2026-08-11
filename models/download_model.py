"""
Download the local edge model for ShellClaw.

Usage: python download_model.py
"""

import os
import subprocess
import sys
import urllib.request

MODEL_DIR = os.path.dirname(os.path.abspath(__file__))
REPO = "Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF"
FILE = "qwen2.5-coder-0.5b-instruct-q4_k_m.gguf"
DEST = os.path.join(MODEL_DIR, "qwen2.5-coder-0.5b-instruct.gguf")

if os.path.exists(DEST):
    mb = os.path.getsize(DEST) / (1024 * 1024)
    print(f"Already exists: {DEST} ({mb:.0f} MB)")
    sys.exit(0)

# Try hf first, fallback to huggingface-cli
for cmd in (("hf",), ("huggingface-cli",)):
    if subprocess.run(["which", cmd[0]], capture_output=True).returncode == 0:
        subprocess.run(
            [*cmd, "download", REPO, FILE, "--local-dir", MODEL_DIR],
            check=True,
        )
        src = os.path.join(MODEL_DIR, FILE)
        if not os.path.exists(src):
            src = os.path.join(MODEL_DIR, "models--" + REPO.replace("/", "--"), "snapshots", FILE)
        if os.path.exists(src):
            os.rename(src, DEST)
        break
else:
    url = f"https://huggingface.co/{REPO}/resolve/main/{FILE}"
    print(f"Downloading {url} ...")
    urllib.request.urlretrieve(url, DEST)

mb = os.path.getsize(DEST) / (1024 * 1024) if os.path.exists(DEST) else 0
print(f"Done: {DEST} ({mb:.0f} MB)")
