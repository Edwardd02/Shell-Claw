#!/usr/bin/env python3
"""
从 tldr 仓库提取 shell 命令,生成逐行 JSON 训练数据。

用法:
    python3 extract.py [--pages-dir 路径] [--output 路径]

特性:
    - 递归扫描 pages/ 下所有英文 .md(common/linux/osx/windows/android)
    - 提取每个命令块中反引号 `cmd` 包裹的命令行
    - 剥离占位符 {{...}}(含选项式 {{[-t|--tag]}}, 环境变量 {{$VAR}} 等)
    - 逐行写入 JSONL:每条命令作为一个 JSON 对象立即写入并 flush,
      因此中途退出已提取的命令不会丢失
    - 用 tqdm 显示进度条
"""

import argparse
import json
import re
import sys
from pathlib import Path

try:
    from tqdm import tqdm
except ImportError:
    raise SystemExit("需要 tqdm:  pip install tqdm")


# 反引号包裹的命令行: `cmd...(非反引号)...`
CMD_RE = re.compile(r"`([^`]+)`")

# 占位符块: {{...}}, 惰性匹配到第一个 }}
PLACEHOLDER_RE = re.compile(r"\{\{.*?\}\}")


def strip_placeholders(cmd: str) -> str:
    """去掉命令里的 {{...}} 占位符,并把多余空格压缩为单个空格。"""
    cmd = PLACEHOLDER_RE.sub(" ", cmd)
    cmd = re.sub(r"\s+", " ", cmd)  # 压缩连续空白
    return cmd.strip()


# 合法的 shell 命令首字符(排除元字符/操作符/符号开头的脏数据)
# 允许:字母、数字、_、./、- 等合法命令开头
VALID_START = re.compile(r"^[a-zA-Z0-9_./\\~-]")


def is_valid_command(cmd: str) -> bool:
    """过滤掉不可用于前缀补全的脏数据(特殊符号页面/shell 操作符)。"""
    if not cmd:
        return False
    # 单字符纯符号或残缺片段直接淘汰
    if len(cmd) < 2:
        return False
    # 必须以合法字符开头(排除 ! $ % ^ , ( 等符号命令)
    if not VALID_START.match(cmd):
        return False
    # 含有悬空 shell 元字符但无语义(如 "sudo !!") —— 排除依赖 !! 的
    if '!!' in cmd or cmd.startswith('!') or cmd.endswith('!'):
        return False
    # 长度控制:太长的(可能是一整段脚本)或太短的,难做前缀补全
    if len(cmd) > 120:
        return False
    return True


def extract_from_md(md_path: Path):
    """从一个 .md 文件提取所有命令行。"""
    try:
        text = md_path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return
    for m in CMD_RE.finditer(text):
        raw = m.group(1)
        cmd = strip_placeholders(raw)
        if cmd:
            yield cmd


def collect_md_files(pages_dir: Path):
    """收集所有英文页面的 .md 文件(排除多语言 pages.xx)。"""
    md_files = []
    # pages/common, pages/linux, pages/osx, pages/windows, pages/android
    for d in ["common", "linux", "osx", "windows", "android"]:
        sub = pages_dir / d
        if sub.is_dir():
            md_files.extend(sorted(sub.glob("*.md")))
    return md_files


def main():
    parser = argparse.ArgumentParser(description="从 tldr 提取命令训练数据")
    parser.add_argument("--pages-dir", type=str,
                        default="tldr/pages",
                        help="tldr 的 pages 目录")
    parser.add_argument("--output", type=str,
                        default="train/commands.jsonl",
                        help="输出 JSONL 路径")
    args = parser.parse_args()

    pages_dir = Path(args.pages_dir)
    output_path = Path(args.output)

    if not pages_dir.is_dir():
        sys.exit(f"错误: pages 目录不存在 {pages_dir}")

    output_path.parent.mkdir(parents=True, exist_ok=True)

    md_files = collect_md_files(pages_dir)
    print(f"扫描到 {len(md_files)} 个英文 .md 文件")

    total = 0
    skipped = 0
    with open(output_path, "w", encoding="utf-8") as f:
        for md in tqdm(md_files, desc="提取命令", unit="file"):
            for cmd in extract_from_md(md):
                if not is_valid_command(cmd):
                    skipped += 1
                    continue
                # 逐行写入并立即 flush,中断不丢已写入数据
                f.write(json.dumps({"cmd": cmd}, ensure_ascii=False) + "\n")
                f.flush()
                total += 1

    print(f"\n完成! 共提取 {total} 条命令,过滤 {skipped} 条 -> {output_path}")


if __name__ == "__main__":
    main()
