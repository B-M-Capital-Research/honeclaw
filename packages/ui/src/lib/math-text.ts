/**
 * 把终稿里的 LaTeX 记法就地翻译成纯文本。
 *
 * 渲染层没有数学引擎，`$$\text{目标价} = $38.00 \times 32$$` 会原样显示成源码。
 * 装一个数学引擎并不能解决问题：投研正文里 `$` 是货币符号，`$$…$$` 里再出现
 * `$38.00`、`\mathbf{$1,216.00}` 会把公式从中间截断——KaTeX 同样渲染不出来。
 * 所以这里的选择是「翻译成人能读的一行算式」，而不是「渲染数学」：
 * `目标价 = $38.00 × 32x = **$1,216.00**`。
 *
 * 生成侧同时有规则要求直接写纯文本算式；这一层负责历史消息和偶发漏网的那部分。
 */

/** 只翻译成对的数学区，单个 `$` 一律当货币，绝不当分隔符。 */
const MATH_REGIONS: { open: RegExp; close: string }[] = [
  { open: /\$\$/g, close: "$$" },
  { open: /\\\[/g, close: "\\]" },
  { open: /\\\(/g, close: "\\)" },
];

const SYMBOLS: Record<string, string> = {
  times: "×",
  cdot: "·",
  div: "÷",
  approx: "≈",
  sim: "~",
  neq: "≠",
  ne: "≠",
  geq: "≥",
  ge: "≥",
  leq: "≤",
  le: "≤",
  pm: "±",
  mp: "∓",
  propto: "∝",
  infty: "∞",
  rightarrow: "→",
  to: "→",
  leftarrow: "←",
  Rightarrow: "⇒",
  Leftarrow: "⇐",
  leftrightarrow: "↔",
  cdots: "…",
  ldots: "…",
  dots: "…",
  quad: " ",
  qquad: "  ",
  alpha: "α",
  beta: "β",
  gamma: "γ",
  delta: "δ",
  Delta: "Δ",
  sigma: "σ",
  Sigma: "Σ",
  mu: "μ",
  pi: "π",
  sum: "Σ",
  prod: "Π",
  partial: "∂",
};

/** 只改变排版、本身不产出字符的命令。 */
const DROPPED = new Set([
  "left",
  "right",
  "displaystyle",
  "textstyle",
  "limits",
  "nolimits",
  "big",
  "Big",
  "bigg",
  "Bigg",
]);

/** 语义是「这段是文字」的包裹命令：只保留里面的内容。 */
const TEXT_WRAPPERS = new Set([
  "text",
  "textrm",
  "textnormal",
  "mathrm",
  "mathit",
  "mathsf",
  "mathtt",
  "operatorname",
  "mbox",
  "hbox",
]);

/** 加粗包裹命令：翻成 Markdown 粗体，保住原文的强调。 */
const BOLD_WRAPPERS = new Set(["mathbf", "bf", "boldsymbol", "textbf", "pmb"]);

const FRACTIONS = new Set(["frac", "dfrac", "tfrac", "cfrac"]);

/** 从 `{` 开始读一个配对的花括号参数，返回内容与右括号之后的位置。 */
function readBraceArgument(source: string, start: number) {
  if (source[start] !== "{") return undefined;
  let depth = 0;
  for (let index = start; index < source.length; index += 1) {
    const char = source[index];
    if (char === "\\") {
      index += 1;
      continue;
    }
    if (char === "{") depth += 1;
    else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return { body: source.slice(start + 1, index), end: index + 1 };
      }
    }
  }
  return undefined;
}

/** 分式的分子分母含运算符时补一层括号，`a + b / c` 这种歧义不能留给读者。 */
function parenthesizeIfCompound(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return trimmed;
  if (/^\(.*\)$/.test(trimmed)) return trimmed;
  return /[+\-×÷/]/.test(trimmed) ? `(${trimmed})` : trimmed;
}

/** 翻译一段已经剥掉分隔符的数学内容。递归处理嵌套参数。 */
function convertMath(source: string): string {
  let out = "";
  let index = 0;
  while (index < source.length) {
    const char = source[index];
    if (char !== "\\") {
      // 对齐符与花括号是排版结构，不是内容。
      if (char === "&") {
        out += " ";
        index += 1;
        continue;
      }
      if (char === "{" || char === "}") {
        index += 1;
        continue;
      }
      out += char;
      index += 1;
      continue;
    }

    const command = /^\\([a-zA-Z]+)/.exec(source.slice(index));
    if (!command) {
      const next = source[index + 1];
      if (next === "\\") {
        out += "\n";
        index += 2;
        continue;
      }
      if (next && "$%{}_&#".includes(next)) {
        out += next;
        index += 2;
        continue;
      }
      if (next && ",;: ".includes(next)) {
        out += " ";
        index += 2;
        continue;
      }
      if (next === "!") {
        index += 2;
        continue;
      }
      out += char;
      index += 1;
      continue;
    }

    const name = command[1];
    let cursor = index + command[0].length;

    if (name === "begin" || name === "end") {
      const argument = readBraceArgument(source, cursor);
      index = argument ? argument.end : cursor;
      continue;
    }
    if (FRACTIONS.has(name)) {
      const numerator = readBraceArgument(source, cursor);
      const denominator = numerator
        ? readBraceArgument(source, numerator.end)
        : undefined;
      if (numerator && denominator) {
        out += `${parenthesizeIfCompound(convertMath(numerator.body))} / ${parenthesizeIfCompound(
          convertMath(denominator.body),
        )}`;
        index = denominator.end;
        continue;
      }
    }
    if (TEXT_WRAPPERS.has(name)) {
      const argument = readBraceArgument(source, cursor);
      if (argument) {
        out += convertMath(argument.body);
        index = argument.end;
        continue;
      }
    }
    if (BOLD_WRAPPERS.has(name)) {
      const argument = readBraceArgument(source, cursor);
      if (argument) {
        const inner = convertMath(argument.body).trim();
        // 已经含 `*` 时再包一层会破坏 Markdown 强调配对。
        out += inner && !inner.includes("*") ? `**${inner}**` : inner;
        index = argument.end;
        continue;
      }
    }
    if (DROPPED.has(name)) {
      index = cursor;
      continue;
    }
    if (name in SYMBOLS) {
      // 命令名后的空格在 LaTeX 里只是终止符，但在纯文本算式里正好是需要的间隔，
      // 所以保留它：`\times 32` 要读成「× 32」而不是「×32」。
      out += SYMBOLS[name];
      index = cursor;
      continue;
    }
    // 不认识的命令：留下名字本身，至少读得出来是什么。
    out += name;
    index = cursor;
  }
  return out;
}

/** 收掉翻译后多出来的空白，但不动换行。 */
function tidy(value: string) {
  return value
    .replace(/[ \t]{2,}/g, " ")
    .replace(/\s+([,.;:%)])/g, "$1")
    .replace(/\(\s+/g, "(")
    .trim();
}

/** 把一段不含代码的文本里的数学区翻成纯文本。 */
function convertRegions(segment: string): string {
  let result = segment;

  for (const region of MATH_REGIONS) {
    const delimiter = region.close;
    const opener = region.open.source;
    // 区间内不允许跨空行：少一个闭合符时，最坏只影响这一段，不会把后面整段吞掉。
    const pattern = new RegExp(
      `${opener}((?:(?!\\n\\s*\\n)[\\s\\S])*?)${delimiter.replace(/[\\$()[\]]/g, "\\$&")}`,
      "g",
    );
    result = result.replace(pattern, (_match, body: string) =>
      tidy(convertMath(body)),
    );
  }

  // `$\rightarrow$` 这类「单个命令」写法：内容不可能是货币，翻译是安全的。
  result = result.replace(/\$(\\[a-zA-Z]+)\$/g, (_match, body: string) =>
    convertMath(body),
  );

  return result;
}

/**
 * 只翻译代码块与行内代码之外的部分：代码块里的反斜杠是内容，不是排版。
 */
export function normalizeMathToPlainText(markdown: string): string {
  if (!markdown) return markdown;
  // 没有数学痕迹就原样返回：绝大多数消息不该为这条规则付出任何风险。
  if (!markdown.includes("$$") && !markdown.includes("\\")) return markdown;

  const segments = markdown.split(/(```[\s\S]*?```|~~~[\s\S]*?~~~|`[^`\n]*`)/g);
  return segments
    .map((segment, position) =>
      position % 2 === 1 ? segment : convertRegions(segment),
    )
    .join("");
}
