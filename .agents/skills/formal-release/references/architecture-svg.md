# Architecture SVG 更新指南

`resources/architecture.svg` 是仓库的对外展示海报。正式发版时如果架构面有变化，必须把它一起刷新；否则它会很快和真实代码脱节。

## 文件位置

- 海报本体：`resources/architecture.svg`
- 配套交互页：`resources/architecture.html`（颜色、层次和 SVG 保持一致）
- 在 `README.md` / `README_ZH.md` 中通过相对链接引用

## 什么时候需要刷新

**必须刷新**：

- 新增 / 删除 poller、EventKind、agent runner、LLM tool、channel binary、Web API 顶层路由
- 新增 / 删除 crate，或 crate 之间职责边界发生明显移动
- Agent runner 默认模型变化
- Storage 路径布局变化（`data/events.db`、`data/notif_prefs/...`、agent-sandboxes 等）
- Frontend dual-surface（admin / public）结构调整

**可以跳过**：

- 纯文档 / release-notes 修复
- bugfix release，没有触达上述结构
- 仅参数调整、阈值调整、prompt 内部改动

跳过时在 release notes 里注明“架构 SVG 不需要更新”，避免之后回看时怀疑遗漏。

## 海报里硬编码的事实清单

下面这些数字/名字直接写死在 SVG 里。每次发版都要逐项核对，并和当前代码对照。

### Hero 区

- 版本号卡片（如 `v0.6.0` 与日期）
- Tagline、副标题
- Stat 徽章组（数量必须和实际代码一致）：
  - **channels**：`bins/hone-*` 中作为 channel 的 bin 数
  - **pollers**：`crates/hone-event-engine/src/pollers/` 下实际注册的 poller
  - **EventKinds**：`crates/hone-event-engine` 中 `EventKind` enum 变体数
  - **agent runners**：`crates/hone-llm/src/agents/` 下注册的 runner
  - **LLM tools**：`crates/hone-tools/src/` 中实际注册到 tool registry 的工具
  - 站点链接（hone-claw.com）

### Layer 卡片区

- **Sources**：外部数据源类别
- **Pollers**：每个 poller 的文件名、轮询间隔、产出 EventKind、event id 模板
- **Event Engine**：classify / policy / route / digest 的具体实现位置
- **Brain**：5 类 agent runners 的默认模型
- **Channels Hub**：`HoneBotCore` + 核心 trait（compaction、tool routing 等）
- **Sinks**：每个 channel 的 outbound 适配器

### Control Plane 带

- Web API 路由清单（参考 `crates/hone-web-api/src/router.rs` 等）
- Storage 路径布局
- Frontend 关键页面（admin + public）

### Recent Milestones

- 最右侧/底部的 milestone 卡片要把当前发版加入，并把最旧那条删掉，保持 3–4 张

## 核对脚本

下列命令是“快速核对”，不能替代肉眼读代码：

```bash
# pollers
ls crates/hone-event-engine/src/pollers/

# EventKind 变体（数量）
rg -n 'enum EventKind' -A 200 crates/hone-event-engine | rg -c '^\s+[A-Z][A-Za-z0-9]+\s*[\{,(]'

# agent runners
ls crates/hone-llm/src/agents/

# LLM tools
ls crates/hone-tools/src/

# channel bins
ls bins/ | rg '^hone-'
```

如果数字对不上，**先改代码或改 SVG，不要打 tag**。

## 颜色与排版规范

- 配色和 `architecture.html` 保持一致：暖羊皮纸底（`#f5efe2`），墨色（`#1a1a2e`），品牌橙（`#e85d04`，来自 `resources/logo.svg`）
- 各 layer 的 accent bar：sources=`#5d8b73` / pollers=`#c8962a` / engine=`#c1633a` / brain=`#6b4f9c` / hub=`#3a5b9c` / sinks=`#a83a5a`
- 字体：`Georgia` 衬线用于 hero 与数字，无衬线用于正文，等宽用于路径与 ID
- 新增内容遵循已有 card grid（同列对齐、同 padding）

## 渲染校验

正式提交前一定要本地渲染一次，确认没有越界、错位、字体回退：

```bash
# macOS 自带 QuickLook
qlmanage -t -s 1800 -o /tmp /Users/$(whoami)/Workspace/honeclaw/resources/architecture.svg
open /tmp/architecture.svg.png
```

或在浏览器直接打开 SVG / `architecture.html`。常见问题：

- 文字超出卡片：缩短文案或增大 card 宽度
- 图标错位：检查 `<defs>` 中 marker 的 `refX` / `refY`
- 字体回退：核对 CSS class 是否在 `<defs>` 中定义，并使用本机已有字体栈

## 提交方式

- `resources/architecture.svg` 的更新和 release commit **同一个 PR / 同一个原子 commit** 一起进 `main`
- commit 信息里点出“refresh architecture SVG for vX.Y.Z”，方便后续追溯
- 不要单独再 push 一个“fix svg”的 follow-up commit 在 tag 之后；那样海报就和 tag 对不上了
