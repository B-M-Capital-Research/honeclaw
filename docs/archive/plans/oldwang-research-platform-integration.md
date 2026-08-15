# oldwang 投研功能整合：信息架构与数据流收敛

状态：`done`（2026-08-15 合入 main：b65a7cc1）
分支：`integrate/oldwang`（基于 `origin/oldwang`，其为 `main` 的严格超集）
日期：2026-08-12

## 背景

`oldwang` 分支一次性带来 10 个投研功能（每日信号、公司评级、估值实验室、关键事件链、
持仓新闻、仓位管理、大V速报、周报、社区论坛、研究文库）+ 2 个技能包，约 2.95 万行。
功能价值成立，但交互与架构存在系统性问题，以"深度投研平台"标准整合进 main 前需收敛。

## 诊断结论（三路深查汇总）

前端：
- 7 个仪表盘没有路由，全部挤在聊天页 composer 上方一条 144px 芯片横轨里；
  不可分享、返回键失效、移动端一屏只见 2.8 张卡，靠盲滚发现。
- 聊天页首屏并发 ~9 个快照请求（daily-signal 单组件 4 连发），零缓存，用户可能一个都不点。
- 约 620 处硬编码色值绕开 `--hone-*` 令牌；官方珊瑚被复刻成 8 个近似值；废弃琥珀大面积回归；
  5 个 CSS 文件手写压缩成单行；506 行 `!important` 覆盖层把异构启动器硬压成统一外观。
- `window.prompt` 收集审核意见、4 处原生 confirm；3 个弹窗无 ESC；7 个弹窗无滚动锁。
- 契约测试反向钉死了坏设计（压缩 CSS 字面量、像素魔数、组件顺序、4 Tab）。
- `/valuation-lab` 高亮"洞察"、`/research-library` 高亮"我的"，导航态与位置不一致。

后端：
- 24 个新公开端点鉴权全部到位（`require_public_user`），fail-closed 纪律好。
- 但绕开了 hone-scheduler / event-engine / memory：6 个裸 `tokio::spawn` worker 运维不可见；
  路径推导 ×8、原子写 ×6、`next_refresh` ×7 等纯重复约 1500–2500 行。
- P0：`/research-library` POST 缺 `DefaultBodyLimit`（20MB 上传实际 413）；
  `/weekly-brief` 请求内最多 52 次 FMP 扇出；聊天热路径同步读盘（`chat_context_for_user`）。
- CORS 只放行 GET/POST/PUT，论坛两个 DELETE 路由 preflight 会被拒。

运行时：
- hari-invest 已成全渠道默认人格（每轮 +1.26KB system prompt）；`COHR → "相干"` 中文别名裸
  contains 会误触发；邮件令牌环境变量从 `CLOUDFLARE_API_TOKEN` 改名为
  `HONE_CLOUDFLARE_EMAIL_API_TOKEN`，已有部署会静默停发邮件。
- 产品经理自己的 QA 结论为 NO-GO：对话模型与数据 provider 未配置属部署项，不阻塞代码整合。

## 整合设计

### 1. 信息架构：新增一级区块「研究」

- 新路由 `/research`（研究台）：所有日更研究产品的家。
  - 首屏一张总览网格：每个板块一张摘要卡（信号灯/评分/日期/一句话），数据来自单个聚合接口。
  - 点卡片就地打开该板块的详情面板，面板状态同步进 URL `?panel=<key>`：可分享、可收藏、
    浏览器返回键关闭面板。
  - 估值实验室、研究文库保持独立路由，总览网格里以卡片入口链接过去。
- 聊天页移除数据仪表盘横轨，替换为一行纯导航入口（不取数、不弹窗），把 composer 上方
  预留的 218px 魔数还给对话。
- 工作台导航从 4 区块升为 5：投资助手 / 研究 / 洞察 / 推送 / 我的。
  `/valuation-lab`、`/research-library` 的 activeSection 归入 `research`。

### 2. 数据流：一次往返画首屏，打开才拉全量

- 新端点 `GET /api/public/research-overview`：读取各板块 latest 快照，投影成紧凑摘要卡数组，
  一次请求替代首屏 9 连发。各板块模块各自实现 `overview_card()`，聚合层不越界读文件。
- 各详情面板改为打开时才拉全量快照；daily-signal 的 history 只在切到"历史"页签时拉。
- 「发送到对话」的 6 份手写 `HONE_SAVED_*` JSON 注入模板收敛为共享
  `buildSavedReportPrompt()`，只投影回答所需字段。

### 3. 样式：回到 `--hone-*` 令牌体系

- `public-foundation.css` 新增红绿灯语义令牌（`--hone-signal-green/yellow/orange/red` 及
  soft 面色）：信号语义的黄与品牌琥珀是两回事，前者以令牌形式合法存在，后者继续废弃。
- 研究台新 CSS 全量使用令牌；存量仪表盘 CSS 解压回多行格式，字面色值机械映射回令牌，
  装饰性琥珀（周报卡主题等）改珊瑚/中性。
- 移除 `public-chat-accessibility.css` 中针对导轨的 `!important` 覆盖层。

### 4. 交互质量

- 共享 `ResearchPanel` 弹层：ESC 关闭、`aria-modal`、backdrop、body 滚动锁，7 个仪表盘统一接入。
- 共享加载/错误/空态组件，替代 13 个各自为政的 state class。
- `window.prompt`/`confirm` 替换为面板内联确认与文本域（研究文库审核意见、论坛删帖）。

### 5. 后端治理（本轮范围）

- 新增 `routes/research_store.rs`：统一数据根目录推导、原子写 JSON、北京时间 `next_refresh`、
  含幂等检查的通用 worker 循环；10 个模块机械迁移，消除 ①②③④ 类重复。
- P0 修复：`/research-library` POST 挂 `DefaultBodyLimit(21MB)`；`chat_context_for_user`
  改异步/spawn_blocking；weekly-brief 改 worker 预生成快照（对齐其余 9 个板块的模式）。
- CORS 补 DELETE；邮件令牌 env 加 `CLOUDFLARE_API_TOKEN` 旧名回退并告警；
  移除 `COHR → "相干"` 误触发别名。

### 6. 测试策略

- 重写钉死旧 IA 的契约（横轨、压缩 CSS 字面量、像素魔数、组件顺序、4 Tab）。
- 新增研究台契约：路由存在、panel 状态进 URL、新 CSS 引用令牌、弹层可 ESC。
- 门禁：`cargo test`（除本机缺资源的 hone-desktop）、`bun test --preload ./happydom.ts ./src`、
  `tests/regression/ci/` 契约脚本。

## 明确不做（记为后续任务）

- worker 迁移 hone-scheduler / 纳入 admin 可观测（架构迁移，单独立项）
- 论坛删除语义与附件 GC、history 目录保留策略、多实例安全
- 新界面 i18n（当前被契约测试锁定为中文优先）
- skill_runtime 反向短语匹配的泛词误触发收敛（需产品确认边界）
