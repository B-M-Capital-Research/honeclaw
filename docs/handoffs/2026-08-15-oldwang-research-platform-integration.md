# 2026-08-15 oldwang 投研功能整合（研究台）

## 交付了什么

把 `oldwang` 分支的 10 个投研功能（每日信号、公司评级、估值实验室、关键事件链、持仓新闻、
仓位管理、大V速报、周报、社区论坛、研究文库）按"深度投研平台"标准收敛后合入 main。
设计与诊断全文见 `docs/archive/plans/oldwang-research-platform-integration.md`。

### 信息架构
- 新一级区块「研究」`/research`：研究产品的家。总览网格一次聚合请求
  （`GET /api/public/research-overview`）画首屏；点卡片打开面板，面板状态在
  `?panel=<key>`，可分享、返回键关闭。估值实验室 / 研究文库保持独立路由并归入
  research 导航态。
- 聊天页移除 7 个仪表盘与 506 行 `!important` 导轨覆盖层，替换为纯导航入口条
  （不取数、不弹窗）；移动端 composer 上方死留白 218px→172px。
- 导航 4→5 区块：投资助手 / 研究 / 洞察 / 推送 / 我的。点击当前区块只回到底部，
  不再误触发对话截断（开新分段专属「新对话」按钮）。

### 前端结构
- 共享层：`components/research/research-panel.tsx`（ESC/aria-modal/滚动锁/backdrop）、
  `research-state.tsx`（加载/错误/空态）、`lib/saved-report-prompt.ts`（HONE_SAVED_* 提问
  信封唯一实现，marker 与 payload 逐字保持协议不变）、`lib/research-ask.ts`
  （面板→聊天的 sessionStorage 转交，`/chat?ask=research` 一次性标记）。
- 7 个仪表盘全部迁为受控面板（`XxxPanel { onClose, onAsk? }`），打开才取数；
  daily-signal 历史只在切"历史"页签时拉。
- CSS 回归 `--hone-*` 令牌；`public-foundation.css` 新增红绿灯语义令牌
  `--hone-signal-green/yellow/orange/red(-soft)`（信号黄合法存在，装饰性琥珀继续废弃，
  周报卡等主题强调改珊瑚）；5 个手写压缩 CSS 全部展开；valuation-lab v2 补丁层并回主文件。
- `window.prompt`/4 处 `confirm` 替换为行内确认与面板内 textarea；研究文库
  提交/审核改局部合并（接口回传条目，不再全量重拉）。
- 契约测试重写为钉新设计（研究台 URL 同步、令牌使用、聊天页禁止再长出取数仪表盘），
  新增 `public-research-style-contract.test.ts`。

### 后端
- `routes/research_overview.rs`：聚合端点，各模块自持 `overview_card()` 投影，单卡失败
  降级 waiting 卡，fail-closed。
- `routes/research_store.rs`：数据根推导 / 原子写 JSON / 北京时间 next_refresh 的唯一实现；
  8 个模块的本地拷贝全部迁移删除。portfolio_news / position_management 的存储根从
  `portfolio_dir.parent()` 统一到 `session_sqlite_db_path.parent()`——默认部署两者同为
  `./data` 无感；自定义 `portfolio_dir` 到别处的部署需一次性挪动
  `portfolio_news/`、`position_management/` 两个目录。
- P0 修复：`/research-library` POST 挂 21MB `DefaultBodyLimit`（原 axum 默认 2MB 使
  20MB 上限成死代码）；聊天热路径 `chat_context_for_user` 改 spawn_blocking
  （`chat_context_for_user_async`，public.rs 三处调用）；weekly-brief 增加 19:10 预生成
  worker，handler 优先读快照（消除请求内最多 52 次 FMP 扇出）；CORS 补 DELETE。
- `hone-core` 邮件令牌解析：默认 env 名读不到时回退旧名 `CLOUDFLARE_API_TOKEN` 并
  `warn!` 提示改名——旧部署不再静默停发邮件。
- 修一枚时间炸弹测试：position_management 的 `macro_context` 测试助手硬编码
  report_date "2026-08-11"，而 `advise_position` 要求宏观报告是当天才算 supportive，
  08-12 起必挂；改为动态当天日期。
- `COHR` 中文别名 `相干`→`相干公司`（裸 contains 下"不相干/相干光"必误触发研究基线注入）。

## 门禁结果（2026-08-15）
- `cargo test --workspace --exclude hone-desktop`：全绿（hone-web-api 303，全 workspace 零失败；
  hone-desktop 本机缺 Tauri 打包资源为既有环境问题）。
- `cd packages/app && bun test --preload ./happydom.ts ./src`：483 pass / 0 fail；`tsc` 干净。
- `tests/regression/ci/`：23/23 PASS。
- 浏览器可视验证：研究台网格/面板/URL 同步、聊天入口条（本地 dev-login）。

## 后续（建议另立任务）
- 6+1 个 worker 迁入 hone-scheduler，纳入 /admin/task-runs 可观测。
- 论坛"删除"仍是软删除且作者/admin 可见、附件永不清理；history 目录（尤其
  portfolio_news/position_management 每用户×每天）无保留策略；多实例 last-writer-wins。
- 公开路由器缺统一鉴权中间件（24 个端点手写 `require_public_user`，全对但无结构保障）。
- 新界面 i18n（EN locale 下全中文，被 chat-accessibility 契约锁定为中文优先）。
- skill_runtime 反向短语匹配的泛词误触发（`行业`/`估值`等 2 字词命中即加载投研 Skill）。
- 大 payload 端点（company-ratings / valuation-lab / forum 全量评论内联、50 帖硬顶无分页）
  的分页与 ETag。

## 生产部署记录（2026-08-15 20:0x 北京时间）

- 后端 origin（GCE `instance-20260731-081043`，us-central1-c）已切换到精确
  revision `e996fc875e0a9dcfbedf96ecaa020164572fec09`（GHCR digest
  `sha256:4baaf9125480248d32ca3ccae26a51db35634142f6205564761dc46f7b4c174a`，
  Runtime Image run #74）。流程：raw 脚本按本地同 revision SHA-256 比对 →
  `stage_ghcr_runtime.sh` 按 digest 暂存 `[PASS]` → 环境校验 OK → 两次
  active-chat-runs 空闲读数 `{"count":0}` → 原子符号链接切换 + systemd 重启 →
  loopback `/api/meta` 精确 sha、重启后日志零 error/panic。
- 保留回滚：`e0d53464`（上一版，含研究台整合）与 `e4e1e3e9`。
- 边缘验收：`hone-claw.com` 200；`/api/public/auth/dev-login/config` 返回
  `{"enabled":false}`（本地测试账号在生产关闭）；`/api/public/research-overview`
  401（鉴权门生效，端点已上线）。
- **注意：该实例 `HONE_RUNTIME_ROLE=web`，主机上没有任何进程承担研究
  worker——生产研究台目前无快照数据，所有卡片将显示等待态。**是否把该实例
  改为 `all`（或另起 worker 角色进程）属运营决策：worker 启动即做冷启动刷新，
  会消耗 FMP/搜索/模型配额，且 PM 的 QA 记录显示这些 provider 凭证配置状态
  待确认。等待产品负责人拍板后执行。
