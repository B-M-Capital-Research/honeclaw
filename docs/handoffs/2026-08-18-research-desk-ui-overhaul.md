# 研究台 UI 重构 Handoff

- title: 研究台信息架构分层、面板排版重做与设计 token 收敛
- status: done
- created_at: 2026-08-18
- updated_at: 2026-08-18
- owner: Claude
- related_files:
  - `packages/app/src/pages/public-foundation.css`
  - `packages/app/src/pages/public-research.tsx`
  - `packages/app/src/pages/public-research.css`
  - `packages/app/src/components/research/research.css`
  - `packages/app/src/components/research/research-panel-contract.test.ts`
  - `packages/app/src/components/daily-signal-dashboard.tsx`
  - `packages/app/src/components/*-dashboard.css`（七个面板）
- related_docs:
  - `AGENTS.md`
  - `docs/invariants.md`

## Summary

研究台首页从「八张同质卡片的网格」改为三层信息架构：外层直接陈述结论，明细退到 panel。
同时修掉一个会导致弹窗塌缩的架构缺陷——七个研究面板都在和共享 modal 外壳抢同一个节点的
几何声明——并把七个面板的 CSS 收敛到 foundation token。

改动全部位于 `packages/app/src`，未触及 Rust crates。

## What Changed

### 1. 弹窗几何归共享外壳独占（架构缺陷，非样式偏好）

七个面板各自把 `backdropClass` / `dialogClass` 传进 `ResearchPanel`，于是 `.X-dialog` 与
`.research-panel` 落在**同一个 DOM 节点**上，两侧都声明 width / height / border-radius /
background / box-shadow。单类选择器特异性打平，胜负只由 CSS import 顺序决定——线上表现为
弹窗宽度塌缩成其副标题的 max-content 宽度。

现在 `.X-backdrop` / `.X-dialog` 只允许保留主题性声明（`color`、`--signal-color` 一类自定义
属性），几何全部由 `components/research/research.css` 拥有。

### 2. 信息架构分层

首页三层，每层排版语言不同，避免退回「一堆同质容器」：

| 层 | 内容 | 形态 |
|---|---|---|
| 判断 | 有 `signal` 的红绿灯 | 直接印分数 + 灯 + 一句结论 |
| 洞察 | 有产出的模块 | 一行一句话（标签 / 结论 / 量） |
| 待办 | `waiting` / `empty` | 收成一排虚线胶囊 |

配套：`stateLabel` 反转为**只标异常**（原来六成卡片挂着「今日已更新」，重复即无信息量）；
`leadSentence` 在句边界截断而非硬 clamp，分号结尾会去掉分号。

### 3. 面板排版

十个维度原本是十张独立圆角卡片（二十条边框、二十个圆角），实为一张同构的表。改为固定四列
（点 / 名称+趋势 / 走势 / 分数）+ 每行一条 hairline，行高 64px → 44px，十项同屏，且左右两列
的分数首次落在同一条竖线上。hero 去框，半圆仪表盘换成横向刻度条（那个 34px 数字是 head 里
同一个数字的第二次印刷，还占着固定 240px）。告警块从红底改为左侧一条红线。

### 4. Token 收敛

foundation 新增：七档字号阶梯（`--hone-text-2xs`…`2xl`，**11px 是地板**）、`--hone-radius-xs`
/ `-pill`、以及 `--hone-signal-*-ink` 一组文字专用色。

七个面板清零 174 处硬编码色、47 条 ≤10px 规则（含三处 8px 中文）。

## 可复用的结论

- **`-ink` 与显示色必须分家。** `--hone-signal-*` 是调给「圆点」看的，当字用时黄 3.39:1、
  橙 3.94:1、红 4.43:1 三个都不过 AA。新增的 `-ink` 变体在各自 `-soft` 底上 ≥4.6:1、白底
  ≥5.2:1；深色下 `-soft` 是 16% alpha，实测 4.91–5.96:1，因此直接复用显示色。
- **`--hone-ink-400` 与 `--hone-ink-500` 在浅色下是同一个值 `#68736f`。** 两个 token 名、
  一个颜色，阶梯是假的。本次未动，见「未决项」。
- **sparkline 画的是原始水平值，`trend_label` / `score` 来自同比动量。** 二者在正常情况下
  就会分歧（实际可支配收入可以月月上升而同比增速放缓），这正是「走弱 + 一路上扬的线」看起来
  像渲染 bug 的原因。线已改中性色并加末端点，注释写明它不是标签的图示；各自归一化是对的
  （不同量纲无法共用刻度），不要「修」成共用 y 轴域。

## 非显然的验证步骤

本地**跑不出**有数据的研究台：后端未编译（`target/` 无 hone 二进制），且 `data/` 最新到
5 月 22 日，而报告日是 8 月 17 日——登录进去只会看到八张「等待数据」。

因此视觉验收走的是 Playwright + `page.route("**/api/**")` 全量 mock，用真实组件与真实 CSS
渲染，只替换 HTTP 响应（模式同 `packages/app/e2e/public-mobile-overlays.spec.ts`）。对比度不靠
推算，在渲染后的页面里读 `getComputedStyle` 的实际 `color` 与逐级向上找到的实际背景色计算：

```
9px → 11px   3.31:1 → 4.92:1   指标副标
     12px    3.39:1 → 4.65:1   黄灯 chip
     13px    ——     → 5.47:1   首页结论行
```

该脚本是一次性验收工具，未留在仓库；要复现按上述方式重建即可。

## 验证结论

- `bun run test:web`：**520 pass / 0 fail**（基线 516 + 新增 4 条契约）
- `bun run typecheck:web`：0
- `tests/regression/ci/test_design_system_contract.sh`：PASS
- `test_research_curation_contract.sh` / `test_company_research_dialogue_contract.sh`：PASS
- `tests/regression/run_ci.sh` 在 `test_billing_http_e2e.sh` 中断：本机无 `psql`，环境缺失
- `test_navigation_responsiveness_contract.sh` 5 个失败：全部为
  `PostgreSQL 测试需要 HONE_POSTGRES_*`，环境缺失（AGENTS.md 已记载），本次未改任何 Rust

## 未决项与风险

- **`--hone-ink-400` / `--hone-ink-500` 同值未修。** 有 47 处用量散在 `public-home` /
  `public-terms` / `public-privacy` / `community-forum` 等与研究台无关的页面，改灰阶需要单独
  一个 commit 加全站回归，塞进本次会失控。注意：ink-400 现值 4.84:1 勉强达标，若要让它「真的
  更浅」，必须同时把所有 ≤14px 的文本用法迁到 ink-500，否则会引入新的对比度违规。
- **只有 daily-signal 面板做了排版重设计**，另外六个面板仅完成 token 收敛与几何拆除，内部
  仍是卡片套卡片。同样的「同构数据 → 表」判断大概率适用于 company-rating 与
  position-management。
- **`research-panel-contract.test.ts` 与 AGENTS.md 第 200 条存在张力**（「页面层不为静态 UI
  细节追求覆盖率」）。保留的理由是它锁的是一个已在生产出过故障的架构缺陷，属于第 203 条的
  bugfix 回归证明；其中字号/圆角/hex 的断言确属风格约束，若日后认为过重，可只保留
  `geometry` 一项。
- **仓库中途出现过 8 个带冲突标记的文件**（`config.example.yaml`、若干 event-engine Rust 文件、
  `public-content.ts`），来自 `pull --autostash` 的 pop 失败，`reflog` 另有 4 次非本任务发起的
  `reset`。本任务一个都没碰，后由并发会话解决。`stash@{0}` / `stash@{1}` 两份 event-engine
  WIP 仍在 stash 列表中，需确认是否仍需要。
