---
name: "zsh-ghost-text-render"
description: "Rendering gray Ghost Text suggestions in zsh ZLE. Records the fix for `zle -R` special-char crashes and, more importantly, the debugging lessons learned from over-engineering a one-line fix."
metadata:
  author: "shell-claw-dev"
  context: "Smart Shell Copilot — Zsh hook rendering"
  related: "bash/zsh autosuggestions, POSTDISPLAY, region_highlight, zle -F async"
---

# Zsh Ghost Text 渲染 —— 经验记录

## TL;DR（结论先行）

**渲染建议字符串到 zsh 命令行,用 `zle -R` 必须加 `--` 分隔符:**

```zsh
# ✅ 正确:建议含 `-`、`|`、`>`、`&` 等特殊字符时不会崩溃
zle -R -- "${BUFFER}${_ssc_suggestion}"

# ❌ 错误:建议以 `-`/`|` 开头时,zle 把它当命令行参数解析 → 崩溃
zle -R "${BUFFER}${_ssc_suggestion}"
# → 报错: zle:4: bad option: -|
```

**修这个 bug 只需要加 `--`,不需要任何架构改动。**

---

## 这次问题的完整复盘

### 现象

用户输入 `ls -`(或建议含 `|` 时),终端崩溃:

```
_ssc_render_suggestion:zle:4: bad option: -|
```

崩溃甚至打乱了终端本身的显示——模型输出的垃圾建议影响了 shell 的核心体验。

### 真正根因(仅此而已)

`zle -R string` 会把 `string` 当作 zle 的子命令参数来解析。当 `string` 以 `-` 开头(如 `-a| grep...`)时,zle 强行把它当选项,于是报 `bad option`。**就这么简单。**

### 正确修复(一行)

给 `zle -R` 传 `--`,告诉它"后面全是显示内容,不是选项":

```zsh
zle -R -- "${BUFFER}${_ssc_suggestion}"
```

### 我实际绕的远路(要引以为戒)

我没有第一时间看代码确认 `zle -R` 的用法,而是陷入了系统性误判的漩涡:

1. **误判为"渲染机制选错"** → 从 `zle -R` 换成 POSTDISPLAY
2. **误判为"异步时机不行"** → 花十几轮实验证明 `zle -F` 异步回调不立即渲染
3. **争论"引擎边界"** → 一度得出"zsh 无法停住凭空悬浮补全"的错误结论
4. **换架构** → 讨论用 fzf/peco 另开面板、讨论用 zsh-autosuggestions

其实**渲染机制、异步时机、引擎边界全都不是问题**。问题只是 `zle -R` 缺了一个 `--`。

---

## 教训:为什么会绕这么远

### 1. 没先看文档/验证 API 入参,先下架构结论

`zle -R` 的 `--` 语义、以及 zsh 命令解析的通用规则,是本可以用一条 `man`/`zsh -c` 快速确认的。我却用十几轮实验去猜行为,还被"PTY 模拟显示不出"误导去做机制级改造。

> **规则**:遇到 `某命令报 bad option / parse error`,**第一件事看向该命令的参数解析**,而不是怀疑架构。

### 2. 症状漂移 —— 把两个不同的问题混为一谈

这次其实有**两个独立问题**,我却当成一个在解决:

| 问题 | 表现 | 根因 |
|---|---|---|
| 渲染崩溃 | `bad option: -\|` | `zle -R` 缺 `--`(特殊字符被当参数) |
| 不显示 | 停住不按键时建议不出现 | zsh 异步重绘时机限制(与渲染无关) |

我把"崩溃"当成主要矛盾,结果所有改造(换 POSTDISPLAY、改异步)都在解决"不显示",而"崩溃"其实一行就能修。

> **规则**:一个 bug 现象,先拆是否其实有多个独立根因,逐一最小复现,别用一个方案想覆盖全部。

### 3. 过早抽象 / 过度工程化

我画架构分离"控制层/渲染层"、移植 zsh-autosuggestions 的全套异步——这些都有其价值,但**在没确认最小修复前就做,是浪费**。最小可复现 + 最小修复应该先落地。

> **规则**:调试遵循"先找最小修复,DONE 后再考虑优雅重构"。优雅化永远排在正确之后。

### 4. 被工具输出误导,叠加了不确定性

我的 PTY 实验一直显示"不渲染",让我误以为"异步永远不行"。但用户在真实终端能显示。**测试环境的结论未必等于生产环境**,尤其涉及 zle 交互时。

> **规则**:涉及交互式 shell 渲染,要以"真实终端用户复现"为最高权威,不要单凭模拟器判死刑。

---

## 可复用的调试流程(这次学到,下次直接用)

遇到 zsh 渲染/报错类 bug,按此顺序,不要跳:

1. **看报错字面**:`bad option: -|` → 这是参数解析 —— 先查 `man zle` 或 `zsh -c 'zle -R'` 看 `-R` 的用法和选项。
2. **看渲染那一行**:`zle -R "<...>"` —— 字符串可能被当参数。加 `--` 是标准的"内容转义"手法。
3. **最小复现**:构造一个 `suggestion=" -a| x"` 的字符串,在最小 zsh 脚本里调 `zle -R --` 验证。
4. **只修根因**:确认 `--` 能解决崩溃,先落地这个最小修复。
5. **再验证表现**:真实终端让用户测,不要只信模拟器。
6. **最后才重构**:确认功能稳定后,再谈架构分离、复用 zsh-autosuggestions 等。

---

## 附:本项目的 zsh 渲染知识速查

### `zle -R` 的两种用途

- `zle -R`(无参):触发完一次重绘(让 zle 按当前 BUFFER + region_highlight 重画)。
- `zle -R string`:在提示符位置显示 `string`(字符串会被当作 zle 参数解析 → **必须加 `--` 或确认不含 `-` 开头**)。

### region_highlight 灰色后缀

```zsh
region_highlight+=("$CURSOR $(( CURSOR + n )) fg=8")   # fg=8 是灰(亮黑)
```

### POSTDISPLAY(光标右侧提示)

```zsh
POSTDISPLAY="${suggestion#$BUFFER}"   # 显示"减去已输入"的后缀部分
```

POSTDISPLAY 是 zsh 原生"光标右侧悬浮"机制,不污染 BUFFER,适合悬浮提示。但它在**异步 `zle -F` 回调里不保证立即重绘**——显示仍需发生在 ZLE 重绘点。

### `zle -F` 异步回调

`zle -F fd handler` 在 fd 可读时调用 handler,但**该回调中设置的显示(zle -R / POSTDISPLAY / region_highlight)未必即时生效**。生产上多数方案让"渲染点落在按键后的 ZLE 重绘",异步只负责预取数据。

---

## 结论一句话

**`zle -R` 缺一个 `--` 就能修复渲染崩溃。** 更大的收获是:遇到报错先看参数解析、拆清多个根因、先最小修复再架构优化、以真实终端为准。别让"架构幻想"遮蔽"一行修复"。
