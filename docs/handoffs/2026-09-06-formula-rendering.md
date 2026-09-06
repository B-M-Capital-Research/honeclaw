# 2026-09-06 终稿里的公式渲染不出来

## 现象

用户截图：估值终稿里出现 `$$\text{目标价} = $38.00 \times 32\text{x} = \mathbf{$1,216.00}$$`、
`$$\frac{$881.255}{$33.17} = \mathbf{26.57\text{x}}$$`，客户端原样显示成 LaTeX 源码。
同 8 题的历史探针里，3–4/8 的回答含 LaTeX，单轮最多 92 处。

## 为什么不是「装一个数学引擎」

投研正文里 `$` 是货币符号。`$$…$$` 之间又出现 `$38.00`、`\mathbf{$1,216.00}`，
对 KaTeX 一样是语法错误（math mode 被内层 `$` 截断），装引擎只会把「显示源码」换成「显示报错」。
所以两层都朝「纯文本算式」走。

## 改了什么

- **渲染层**（`packages/ui/src/lib/math-text.ts`，`parseMarkdown` 与投资主线视图接入）：
  `normalizeMathToPlainText` 把实际出现过的 LaTeX 子集翻成一行算式——`$$…$$` / `\[ \]` / `\( \)` 剥分隔符，
  `\frac{a}{b}` → `a / b`（分子分母含运算符时补括号），`\text{}` / `\mathrm{}` 只留内容，
  `\mathbf{}` → Markdown 粗体，`\times`→`×`、`\approx`→`≈` 等符号表，`\begin{}/\end{}`、`\left/\right` 丢弃。
  安全边界：单个 `$` 永远是货币，只有 `$\command$` 这种不可能是金额的写法才当数学；
  代码块与行内代码整段跳过；数学区不跨空行；正文里没有 `$$` 也没有反斜杠时原样返回。
  这一层同时修好了**历史消息**——已经写进会话的那些公式不用重跑也能读。
- **生成层**（`prompt.rs` 的 `DEFAULT_FINANCE_DOMAIN_POLICY`，每轮必注入 + `valuation-audit`）：
  算式一律写成一行纯文本，给了例子，并点名禁止 `$$…$$`、`\frac{}{}`、`\text{}`、`\mathbf{}`，
  写清原因（渲染层没有数学引擎；正文里 `$` 只表示货币）。
- **仓库卫生**：`.gitignore` 的 `lib/` 规则把 `packages/ui/src/lib` 也吞了。`markdown.ts` 早已被跟踪所以改动看着正常，
  新建的 `math-text.ts` 被静默丢弃，第一次推送的 commit 引用了一个不存在的模块（前端会构建失败）。
  已加 `!packages/ui/src/lib/` 否定规则并补提交，且用 origin/main 的干净 worktree 实际跑通 `vite build` 复核。

## 验证

- 截图里那两条公式经真实渲染管线后：`目标价 = $38.00 × 32x = $1,216.00`、`$881.255 / $33.17 = 26.57x`。
- 前端 583 项测试通过（新增 3 项：公式渲染、分式、货币/反斜杠/代码不受影响）；`hone-channels` prompt 测试 27/27。
- 生产复测 `math1`（同 8 道估值题）：**0/8 含 LaTeX**（改前 3–4/8、57–92 处），算式行都是
  「基准 = FY+2 EPS 4.45 美元 × 22x P/E = 97.90 美元」这种纯文本。
- 发布：`031d057b`（镜像 `…@sha256:89e87800da0c371aa3176a930fcaa7930dc0f3cba0847bd925c90363a2acc377`），
  current → 031d057b、previous → 9f67f32b，NRestarts=0、无 error；harness 只换 `valuation-audit`；
  Pages 已带归一化逻辑。后续提交 `5a0af5ae` 只含前端文件与 .gitignore，后端与 harness 与 031d057b 逐字节一致。

## 没做

- 研究报告 / PDF 预览那条链路（`research-preview.tsx`）没有接入。AGENTS.md 明确限制在 renderer 链路上加处理，
  那条线的改动应该单独判断。若报告里也出现 LaTeX，接一行 `normalizeMathToPlainText` 即可。
