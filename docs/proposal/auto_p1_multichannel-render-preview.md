# Proposal: Multichannel Render Contract and Preview Harness

status: proposed
priority: P1
created_at: 2026-05-08 11:04:22 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `crates/hone-channels/src/outbound.rs`
- `crates/hone-channels/src/response_finalizer.rs`
- `crates/hone-channels/src/attachments/ingest.rs`
- `bins/hone-feishu/src/markdown.rs`
- `bins/hone-feishu/src/outbound.rs`
- `bins/hone-telegram/src/markdown_v2.rs`
- `bins/hone-telegram/src/listener.rs`
- `bins/hone-discord/src/utils.rs`
- `packages/app/src/lib/messages.ts`
- `packages/app/src/lib/messages.test.ts`
- `crates/hone-web-api/src/routes/files.rs`
- `tests/fixtures/local_image_markers.json`

## 背景与现状

Hone 的产品承诺不是只在 Web chat 里生成一段好文本，而是把同一个投资研究助手接入 Web、桌面、Feishu、Telegram、Discord、iMessage 与未来公共 API。仓库当前已经有一条共享出站基础设施：

- `crates/hone-channels/src/outbound.rs` 定义 `OutboundAdapter`、placeholder/progress/final response 生命周期、reasoning 可见性、`ResponseContentSegment`、本地图表 `file://` marker 拆分，以及 `PlatformMessageSplitter`。
- `response_finalizer.rs` 会在送达前清理内部输出、处理空成功和过渡计划句，并把 actor sandbox 内生成的图片稳定到 `gen_images`，再继续使用 `file://` 作为 canonical 本地图像契约。
- Web 端 `packages/app/src/lib/messages.ts` 解析同一类 `file://` marker，通过 `/api/image` 本地文件代理渲染图片；`messages.test.ts` 已复用 `tests/fixtures/local_image_markers.json`，说明前后端已经开始共享媒体解析 fixture。
- Feishu、Telegram、Discord 都实现了自己的最终渲染与发送逻辑：Feishu 会把部分 Markdown 表格转换为 interactive card table；Telegram 走 HTML parse mode 并做 Markdown-ish 转换；Discord 走 Markdown 分段和附件上传。
- `crates/hone-channels/src/attachments/ingest.rs` 已经为多渠道附件建立了准入边界，按大小、图片尺寸、压缩包展开等规则避免危险输入进入 prompt。

这说明 Hone 已经把“模型输出”与“渠道投递”抽出了一部分共享层，但渠道最终渲染仍然主要靠各 bin 的局部代码和少量单元测试维护。现在的系统可以保证大体能发出去，却还不能产品化地回答几个关键问题：

- 同一段包含标题、表格、引用、代码块、长列表、中文标点和图表的回答，在 Web / Feishu / Telegram / Discord 中分别会变成什么？
- 哪些格式会被保留、降级、拆分、转义、丢失或变成附件？
- 一条 answer 在 channel A 看起来结构清楚，在 channel B 是否会因为 HTML 标签、Markdown fence、表格片段、图片失败或长度限制变得难读？
- 运营或开发如何在不真实发送 IM 消息的情况下预览一次输出在多渠道里的用户可见效果？

当前活跃工作已经覆盖了安全门禁、自动化控制面、run trace、送达决策、技能与 ACP 语义等高价值链路，但“最终消息渲染契约”仍是一个独立缺口。它直接影响用户体验和多渠道可信度，尤其是 Hone 常输出投资研究表格、图表、证据列表和长篇解释。

## 问题或机会

这是 P1 级问题，因为 Hone 的用户经常不是在 Web 控制台里消费结果，而是在 IM 或桌面通知里接收主动推送。投资研究内容如果在渠道渲染时损坏，会带来几类后果：

1. **同一结论跨渠道理解成本不同。**
   Feishu 可能把表格渲染成卡片，Telegram 可能改写为 HTML，Discord 可能只保留 Markdown 文本，Web 又会内联图片。用户切换渠道时看到的是不同信息密度和不同层级结构，容易误解“这是同一份研究”。

2. **格式降级不可见。**
   代码里已经有不少 fallback，例如 Telegram HTML 发送失败回退纯文本、Feishu 损坏 table tag 降级为文本、图片发送失败时插入说明。但这些降级不进入统一 preview / audit / task-health 视图，问题通常要等用户反馈“消息很乱”才发现。

3. **富媒体契约仍然偏脆弱。**
   `file://` marker 已被确认为 canonical 本地图像契约，但 Web 使用 TypeScript regex，Rust outbound 使用 `split_response_content_segments`。虽然已经共享 fixture，一旦未来支持 PDF preview、CSV 附件、chart alt text、research artifact link 或多图 caption，前后端和各渠道仍可能再次分叉。

4. **主动推送没有发送前视觉验收。**
   Scheduler、global digest、heartbeat、portfolio monitoring 这类内容会主动打到 IM。即使 Delivery Decision Loop 判断“应该发”，也不代表“发出去后在该渠道可读”。长表格、超长段落、深层 heading、图表穿插和引用来源很容易触发渠道限制。

5. **商业化和公共体验受影响。**
   Public Web / Hone Cloud 用户可能先在 Web 试用，再迁移到 IM 或桌面。如果多渠道体验割裂，用户会把它理解为产品不稳定，而不是某个平台格式限制。

机会是：Hone 已有足够共享层，第一版不需要重写渠道适配器。只需要把“最终 answer -> canonical render plan -> platform render plan -> preview / validation / audit”做成一等产品与工程契约，就能显著降低多渠道体验回归。

## 方案概述

新增 `MultichannelRenderContract`：把每次用户可见输出先规范化为一个渠道无关的 render plan，再由各平台 renderer 转成平台消息计划。该计划可在发送前预览、在发送后记录，并能用 fixture 做无账号回归。

核心对象：

- `CanonicalMessagePlan`
  - 由 final assistant text、附件、图片 marker、source refs、reasoning visibility 和 origin metadata 生成。
  - 基础 segment 类型先保持克制：`text_markdown`、`local_image`、`attachment_link`、`table_candidate`、`fallback_notice`。
  - 不要求模型直接输出 JSON；第一版仍从现有 final text 解析，避免破坏 runner 契约。

- `PlatformRenderPlan`
  - 每个目标渠道生成一组 `PlatformMessagePart`：text/card/image/file/reply/update/error。
  - 明确记录 `transformations`：table converted、markdown downgraded、html escaped、image uploaded、image unavailable、split_count、placeholder_reused、fallback_plain_text。
  - 明确记录 `warnings`：超过平台推荐长度、表格无法保真、图片不可访问、潜在 HTML parse 风险、caption 丢失。

- `RenderCapabilityMatrix`
  - 描述每个渠道支持什么：Markdown/HTML/card/table/image/file/thread reply/edit placeholder/reasoning/progress/maximum length。
  - 由代码常量或一处配置生成，供 admin UI、测试和 renderer 复用，不再只散落在每个 bin 的 `*_HARD_MAX_CHARS` 与局部 sanitizer 中。

- `RenderPreview`
  - Admin / desktop 可调用的 dry-run API：输入一段 Markdown 或选择一次历史 run，输出 Web/Feishu/Telegram/Discord 的平台计划和可读预览。
  - 不真实发送外部消息，不依赖 IM 账号；图片可以显示为 local proxy 或 unavailable badge。

- `RenderFixture`
  - 长期保留的无账号回归样本，覆盖多标题、长表格、损坏 raw table、代码块、引用、中文列表、多个本地图表、图片失败、超长段落、mixed text-image-text。

第一版重点不是做一个复杂富文本编辑器，而是让 Hone 知道“最终用户会看到什么”，并把格式损坏从隐性 bug 变成可观察、可回归、可灰度的产品边界。

## 用户体验变化

### 用户端

- 用户收到的 IM 消息结构更稳定：标题、段落、表格、图片和补充说明按渠道能力明确降级，而不是随机被某个平台的 parser 打乱。
- 当图表或附件不能在某渠道发送时，用户看到简短明确的替代说明，例如“图表已生成，但当前渠道无法直接展示，请在 Web 打开本次会话查看”，而不是裸露本地路径或静默丢图。
- 长篇投资研究在 IM 中按语义分段，避免一句话被硬切断在表格、代码块或 HTML tag 中间。
- Web 和桌面可以显示“本条内容在 IM 中已降级”的轻量状态，帮助用户理解为什么 IM 版本比 Web 版本短。

### 管理端

- 在 Run Trace / Task Health / Notifications 的详情中增加 render tab：
  - canonical plan
  - 每个渠道的 platform plan
  - split count、fallback count、warning reason
  - 发送成功/失败与渲染降级是否相关
- 管理员可把一次用户反馈的坏消息复制进 render preview，快速定位是模型输出问题、finalizer 问题、平台 renderer 问题还是外部 API 发送问题。
- 针对定时任务，可以在保存或启用前预览“下一次输出模板如果包含表格/图表，在目标渠道会怎么显示”。

### 桌面端

- Desktop bundled runtime 的 channel status 不只显示进程是否活着，还可以显示最近 N 次 render warning：例如 Telegram HTML fallback 频繁、Discord split 过多、Feishu card table 降级。
- 本地用户在设置 Feishu/Telegram/Discord 前，可以用 sample preview 验证目标渠道对投资研究表格和图表的呈现效果。

### 多渠道

- Feishu 保持卡片和 table 优势，但 table 转换失败必须形成结构化 warning。
- Telegram 保持 HTML parse mode，但 sanitizer/fallback 结果进入 plan，不再只是 `warn!` 日志。
- Discord 保持 Markdown 和附件发送，但 split/attachment fallback 进入 plan。
- iMessage 可以先声明为 low capability channel：text-first、image optional、少量格式保留，避免和卡片渠道使用同一预期。
- Public Web / Hone Cloud 可以暴露“best effort text + media refs”的稳定语义，让 API 客户端明确知道哪些输出需要自己渲染。

## 技术方案

### 1. 收敛 canonical render plan

在 `crates/hone-channels/src/outbound.rs` 附近新增渲染计划模块，或拆成 `crates/hone-channels/src/render_plan.rs`：

- 复用现有 `split_response_content_segments` 作为 v1 parser。
- 把本地图像 marker、纯文本段、潜在 Markdown table、代码块、引用块识别为 canonical segments。
- 保持 backward compatibility：`OutboundAdapter::send_response(&str)` 可以先继续存在；新 path 先在 dry-run 与少数渠道内部使用。
- 将 `replace_local_image_markers`、`collect_local_image_markers` 与 Web 解析 fixture 统一到更明确的 `CanonicalMessagePlan` 测试中。

### 2. 定义平台 capability 和 renderer trait

新增轻量 trait：

```rust
trait PlatformRenderer {
    fn platform(&self) -> &'static str;
    fn capabilities(&self) -> RenderCapabilities;
    fn render(&self, plan: &CanonicalMessagePlan) -> PlatformRenderPlan;
}
```

各渠道先实现 dry-run renderer，不马上替换真实发送：

- Feishu renderer 复用 `preprocess_markdown_for_feishu`、`split_into_segments`、table conversion 逻辑。
- Telegram renderer 复用 `sanitize_telegram_html_public`、`TelegramSplitter.split_html`。
- Discord renderer 复用 `DiscordSplitter.split_markdown`、图片附件 plan。
- Web renderer 复用 local file proxy 与 `parseMessageContent` fixture 对齐。

真实发送路径可第二阶段逐步改为“render plan -> send parts”，避免一次性重构所有 channel bin。

### 3. 增加 preview API 与 UI

在 `crates/hone-web-api` 增加 admin-only API：

- `POST /api/admin/render-preview`
  - input: raw markdown / session_id + message_id / task_run_id + channel list
  - output: canonical plan、platform plans、warnings、preview-safe text
- 不访问外部 IM API，不需要 channel token。
- 本地图片只返回 proxy URL 或 unavailable 状态，不泄漏 sandbox 外绝对路径。

前端可先在管理端加一个小型 preview drawer，不需要新增大型页面：

- 从 Run Trace / Task Health / Notifications / Sessions 的消息详情打开。
- 左侧 raw answer，右侧按渠道 tab 展示 plan 与预览。
- 对 `warnings` 做稳定 reason code 展示，便于复制到 bug / proposal / regression。

### 4. 建立 fixture 与 CI-safe 回归

新增或扩展 fixture：

- `tests/fixtures/render_contract/*.json`
- 覆盖 canonical parse、平台 split、Feishu table、Telegram HTML、Discord Markdown、本地图像、长文本、损坏 table、fallback notice。

测试策略：

- Rust unit tests 放在 `hone-channels` 和各渠道 renderer 附近。
- 前端继续用 Bun 测 `parseMessageContent`，并共享同一批 local image marker fixture。
- 增加一个 CI-safe regression 脚本 `tests/regression/ci/test_render_contract.sh`，只跑无账号 dry-run，不调用 Feishu/Telegram/Discord API。

### 5. 接入发送后观测

真实发送仍由各渠道完成，但发送后把 render plan 摘要写入现有运行记录或未来 Run Trace：

- `platform`
- `message_parts_count`
- `image_parts_count`
- `fallback_count`
- `warnings`
- `send_error_kind`

这样用户反馈“Telegram 那条晨报格式坏了”时，可以从 task run 反查当时到底是 render warning、send failure 还是模型输出本身不适合该渠道。

## 实施步骤

1. **Contract v1**
   - 新增 `CanonicalMessagePlan` / `PlatformRenderPlan` 类型。
   - 从现有 final text 解析 text/image segments。
   - 迁移或复用 `tests/fixtures/local_image_markers.json`。
   - 保持现有真实发送路径不变。

2. **Dry-run renderers**
   - 为 Web、Feishu、Telegram、Discord 实现 dry-run renderer。
   - 把现有 splitter / sanitizer / table conversion 包装为可测试函数。
   - 输出 transformations 和 warnings。

3. **Preview API**
   - 增加 admin-only render preview API。
   - 支持 raw markdown 和历史消息两种输入。
   - 确保本地文件代理仍遵守 `routes/files.rs` 的 allowlist root 约束。

4. **Admin / Desktop preview**
   - 先在 Sessions 或 Task Health 详情中增加 preview drawer。
   - Desktop 只消费同一 Web API，不另起本地实现。

5. **发送路径灰度接入**
   - 先在一个低风险渠道或 scheduled task dry-run 中记录 render plan 摘要。
   - 再逐步把真实发送改成 `PlatformRenderPlan` 驱动，减少渠道 bin 重复逻辑。

6. **CI-safe regression**
   - 固化 fixture 和 regression 脚本。
   - 每次修改 outbound、channel markdown、message parser、file marker 或 chart skill 时运行该子集。

## 验证方式

- 单元测试：
  - `split_response_content_segments` 与 Web `parseMessageContent` 对同一 fixture 产生一致 segment 类型。
  - Feishu renderer 对合法 Markdown table 转 table，对损坏 raw table 降级且产生 warning。
  - Telegram renderer 不输出未闭合 HTML tag，fallback 文案可读。
  - Discord renderer 不在 Markdown code fence 中间产生不可恢复断裂。

- CI-safe 回归：
  - `bash tests/regression/ci/test_render_contract.sh`
  - fixture 覆盖 text-image-text、multi-image、长表格、中文投资简报、损坏 `<table`、超长段落。

- 手工验收：
  - 在 Admin preview 输入同一段包含表格和图表的投资研究回答，确认 Web / Feishu / Telegram / Discord preview 均可读。
  - 对一次真实 scheduled digest，确认 task detail 能看到 render warnings 和 message part count。

- 产品指标：
  - 渲染 fallback 率按渠道下降。
  - “消息格式损坏 / 图表没看到 / IM 内容不完整”类用户反馈下降。
  - 定时任务 send failure 中由 parse mode / message too long 引起的占比下降。

## 风险与取舍

- 不在第一版要求模型输出结构化富文本 JSON。这样能降低风险，但 canonical parser 仍需处理模型自由文本的边界。
- 不在第一版追求四个渠道视觉完全一致。目标是语义一致、降级明确、可预览，而不是强行让 Feishu card、Telegram HTML、Discord Markdown 和 Web DOM 长得一样。
- Preview 不等于真实平台渲染 100% 一致。外部平台仍可能调整 parser 或 API 限制；因此 preview 应标注为 dry-run，并保留真实发送后的 warning/error 观测。
- 需要小心本地文件路径。Preview 和 API 不能把 sandbox 外绝对路径透给用户，必须继续走 `routes/files.rs` 的 allowlist 与 local proxy。
- 如果过早把真实发送路径全部改成 render plan，可能引入大范围渠道回归。应先 dry-run、再单渠道灰度、最后替换重复发送逻辑。
- 本提案不解决“是否应该发送”或“内容是否投资安全”。这些仍属于 Delivery Decision Loop 和 Investment Output Safety Gate。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案。结论：本提案不重复，重点差异如下：

- `auto_p0_investment_output_safety_gate.md` 关注投资内容是否安全、是否应降级或拦截；本提案关注已经允许送达的内容在不同渠道中的可读性、格式保真和渲染降级。
- `auto_p1_delivery_decision_loop.md` 关注通知何时发送、发送给谁、如何避免重复和无效打扰；本提案关注确定发送之后，每个渠道实际会展示哪些 message parts。
- `auto_p1_run_trace_workbench.md` 关注 agent 运行过程、工具调用、prompt/audit 与可靠性排障；本提案只把最终消息 render plan 作为一个可预览、可回归的子契约，可作为 Run Trace 的一个 tab，但不是完整 trace 系统。
- `auto_p1_research_artifact_library.md` 和 `auto_p1_investment_document_inbox.md` 关注研究材料的输入、沉淀和交接；本提案关注输出层的渠道渲染，不新增研究资产类型。
- `auto_p1_automation_intent_control_plane.md` 关注自动化创建/修改前的意图确认与影响预演；本提案可被它调用来预览自动化输出，但不改变 cron job intent 模型。
- `desktop-bundled-runtime-startup-ux.md` 关注桌面启动与 runtime 接管；本提案只要求桌面复用 render preview，不改变 sidecar 生命周期。
- `skill-runtime-multi-agent-alignment.md` 关注 skill 调用、runner、上下文和权限语义；本提案不改变 skill activation，只验证 skill 产出的图表/表格/文本能否跨渠道稳定呈现。

因此，本提案填补的是“多渠道最终渲染契约和预览验收”这一独立产品架构层，避免 Hone 的核心投资研究能力在最后一公里因为平台格式差异而变得不可信。
