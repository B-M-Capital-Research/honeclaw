# 财报工作流重构与 TEM 验收

- title: 财报工作流重构与 TEM 验收
- status: done
- created_at: 2026-09-13
- updated_at: 2026-09-13
- owner: Codex
- related_files: `skills/earnings-research/`; `crates/hone-channels/src/earnings_materials.rs`; `crates/hone-channels/src/runners/opencode_acp.rs`; `crates/hone-tools/src/public_page.rs`; `crates/hone-tools/src/web_search.rs`; `packages/app/src/pages/chat.tsx`
- related_docs: `docs/archive/plans/earnings-workflow-refactor-20260913.md`; `skills/earnings-research/references/workflow-source.md`; `docs/decisions.md#d-2026-09-13-01-company-only-earnings-workflows-and-reliable-pdf-completion`; `docs/runbooks/backend-deployment.md`

## Summary

用户要求两个弹窗只填公司、自动取得财报/电话会、遵从原 Dify Prompt、取消额外内容门禁、提高 PDF 稳定性，TEM 实测后上线。原工作目录含用户未提交改动和本地提交，本任务从 origin/main bbdc828c 建独立 worktree，未混入原目录改动。

## What Changed

- 移除弹窗材料上传，保留结构化 kind/company 请求与后端管理员/actor 边界。
- 逐节点只读对照用户给出的 Dify 与新闻子流程，恢复原 Prompt 案例、示例、措辞、查询 USER 与新闻查询 Prompt，移除后来添加的发布/单位自检；不把 Prompt 的章节/条数示例变成代码门禁。
- 自动取材先搜索发布财报及电话会候选，读取公开页面原文并注入当前轮；检索综合摘要与原文分开。缺失材料留作研究缺口，不拒绝报告。上下文与恢复保持当前轮隔离。
- 搜索默认仍为轻量摘要；可显式请求原文/更多结果或读取公开 URL。公网 DNS 固定、逐跳校验、禁止 ambient proxy、标准端口、时限/体积界限阻止私网读取；页面不执行脚本。
- 专用财报系统提示不再叠加通用投研 soul 和其他技能推荐，避免真实 canary 误入 hari-invest/company-thesis-ratings；原 Prompt 独立拥有内容与流程。
- 财报专用模型对照：同一原 Prompt、同一读取实现下，Gemini 3.1 Pro 即便取得原文仍过度归因于经营增长，且把 15 亿美元并购误写为 1.5 亿。配置切换至 OpenRouter `anthropic/claude-opus-4.8` 后，分析正确解释 9,850 万美元证券浮盈、非 GAAP 经营仍亏损及 15 亿美元并购，风险与建议同步收敛；没有追加内容 gate 或针对 TEM 改 Prompt。仅财报专用配置改变，通用聊天模型不变。
- 原文读取真实 canary 发现官方 IR CDN 会重置不兼容客户端标识的请求；使用明确标识 HONE 的 HTTP 客户端兼容标识后，成功读取 33,641 字符、36 个链接，包含此前摘要遗漏的 98.5 million 投资收益说明。没有绕过登录、付费墙或 TLS 验证。
- 显式 OpenRouter 路由覆盖 OpenCode 全局 provider 过滤器，修复本地 canary `model not found`。
- PDF 使用两次 40 秒技术尝试、独立 Chromium profile、完整落盘检测和进程组回收、原子发布。修复 Chrome 完成打印后进程不退出而丢弃成品的问题；分享图片内嵌，重复表头、行分页、链接属性转义保留正文。

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` 通过。
- 定向 Rust：channels 879、tools 235、web-api 378 项覆盖（tools 并行回归期间一次 PG 连接失败，单独复跑通过）。
- Web 单测 646 项通过；Public Community Edge Worker 类型检查及 45 项单测通过；`bun run build:web:public` 通过。
- Playwright 财报弹窗/请求与 PDF 命名下载：3 项通过。
- PDF CI 技术回归通过，包括生成后不退出、进程回收、截断产物拒收、原文件保护、临时目录清理；真实 Chrome 手工回归通过，2 页 A4。
- GitHub 主干 CI run `34743671556` 复现了下述同两项 hone-agent streaming mock 失败；Runtime Image 与 Secret Scan 均通过。相关财报定向验证通过，不宣称完整 CI 全绿。
- 全仓历史失败：hone-agent 两项 streaming mock 测试在 `agents/function_calling/src/lib.rs:8265` 触发 unreachable；hone-core `config_example_avoids_stale_config_knobs` 因 README_EN 既有 api_key/api_keys 文案断言失败；finance automation acceptance 40/50，失败项 18/22/40/42/23/24/28/29/37/43。相关实现未在本次修改。CI runner 在该项停止后，其余脚本单独逐项执行通过。
- TEM 初轮发现本地未接通生产 loopback 搜索代理、全局 provider 过滤器、摘要替代原文及 Markdown 参数被模型预转义的问题；这些轮次作为诊断，不能用 `success=true` 冒充最终内容验收。
- 新增公开原文手工网络 canary：`HONE_PUBLIC_SOURCE_CANARY_URL=https://investors.tempus.com/news-releases/news-release-details/tempus-reports-second-quarter-2026-results HONE_PUBLIC_SOURCE_CANARY_TEXT=98.5 cargo test -p hone-tools reads_real_public_source_body -- --ignored --nocapture` 通过。测试默认忽略，不把外部网络或报告内容检查提升为 CI/生产门禁。
- Linux 原文兼容修复前双模式均生成并下载了 PDF，但分析没有解释投资收益对 GAAP 净利的影响，因此只记为诊断轮；修复取材后重跑，不添加任何内容拒绝逻辑。
- 最终 TEM 双模式（同一 `161fe4b3` Linux 镜像、Opus 4.8、独立 PostgreSQL、真实搜索/FMP）：分析 run `a11c6460-e3aa-4ace-868b-3de8f3ff0cab`，339.9 秒（含下载），5 页，725,287 bytes，SHA-256 `185f9121268c8fb4b15c53bcdb51cb529bc757f4c622d01e667a2f21e6652c80`；前瞻 run `c89a85b4-d520-4c5b-bdc6-48d9f6a86eaf`，394.8 秒（含下载），6 页，937,682 bytes，SHA-256 `83c87e775636a7764ec835c1c6bc6d449f1fc4fbbbc6439e79e47084c8aad686`。两轮各调用一次 renderer，全部页面已视觉检查，重启旁路后经登录态下载哈希完全相同。供应商账单/token 明细未单独核验，不能据此推断单次成本。
- 双模式首次正式 canary 的分享页显示 fallback，定位到技能进程清空环境后读不到显式分享图变量。部署改为 `<release>/skills/earnings-research` + `<release>/packages/app/public/membership_zsxq.jpg` 的仓库布局，逐文件哈希不变；使用相同 Markdown、服务账号和 `env -i` 重新技术渲染，已确认图片自动定位及分享页正确。没有重新研究或改写报告。
- 生产浏览器从「工具 → 财报分析」只填 TEM 启动，271 秒完成，显示可下载 PDF，附件路径已是 actor 归属的 `oss://...`；浏览器实际下载 771,992 bytes / 6 页 PDF，SHA-256 `6d9673379d7e635a3bad4ef559e22d275e68b49c1899051757c6444640aabddb`；与 OSS 直接下载、同版本重启后再次下载逐字节一致。生产 6 页已逐页检查，分享图正常，盈利质量与来源限制有自然披露。

## Deployment

- 已推送 `main` 的代码 revision：`161fe4b30e84860a7da4f6d536a89d7d4c379d13`。原工作目录保持未修改。本次没有创建 release tag。
- GitHub Runtime Image run `34742221105`；不可变 digest `sha256:a757d6b48804fc9f2736a012b9bdccceed4b485eaf18a7655ff7de05149efab4`；镜像来源 `ghcr_linux_oci`，服务回读二进制 SHA-256 `63151672bf0003b602efe694f1d757c1c708ff859f70a9d6b225252e8f43afc1`。
- `/opt/hone/current` 指向该 revision；`/srv/honeclaw/skills/earnings-research` 指向 `/srv/honeclaw/earnings-releases/<revision>/skills/earnings-research`；前端 fallback 指向 `/opt/hone/public-web/releases/<revision>/dist-public`。全部 677 个部署文件按 manifest 验证大小、模式与 SHA。
- 生产仅把 `agent.earnings_workflow.model` 从 Gemini 3.1 Pro 改为 `anthropic/claude-opus-4.8`；保留其余配置、云 PG/OSS、runtime role `all`。切换前两次确认无活跃聊天，保留原配置与旧技能，原子交换后重启 `hone-web.service`。服务回读云存储权威、PG/OSS healthy、local durable dependencies = 0。
- 浏览器公司输入弹窗与真实执行通过。`/api/skills` 回读 `earnings-research` 为 enabled、loaded_from=system。公网站点安全响应头齐全；未登录 public auth 返回 401。
- 本地 build/origin fallback entry `/assets/index-BSPFD5kj.js`，SHA `2f1fd4b9140ba6c5b587b2036b8766ebb2c6632f61233f3dd2b025651c17fdfd`；Pages 从旧 `index-SI3XfZm6.js` 更新为 `index-DLa8E4Gb.js`，SHA `fced6f8b8757c73599e43ece791648e61c13d6d1f794baf130c96424a3113edf`；chat chunk `chat-Bm4UMEH_.js`，SHA `6c4ed2856a6f3066651a91c8795dda2403fc1032e2870ff54bdef29afd92f1c4`。Pages 与本机构建 entry 字节不同，已按 runbook 核对新 UI 文案和 `attachments:[] + earningsWorkflow:{kind,company}` 请求路径，并由真实线上 TEM 成功执行证明。origin 公网 hostname 禁止直接匿名访问；fallback 验证使用 loopback 8088。
- 回滚：旧 runtime `93818046a83c3c2b47ed02fc6aac007d875407ab-ghcr-runtime`；旧前端 `00ea36e01b8d35d4fbd0912fd6a836ce83c1b09b`；旧技能 `/srv/honeclaw/skill-rollbacks/earnings-research-before-<revision>`；配置备份 `/etc/hone/config.before-earnings-<revision>` 与 `/etc/hone/runtime.env.before-earnings-<revision>`（root 0600）。恢复时先确认无活跃聊天，成组恢复 symlink/模型配置/环境并重启服务。

## Risks / Follow-ups

- 公开来源可能限制访问或只提供音频；缺失原文由原研究流程披露。预取候选不证明公司/季度对应，模型需依据来源时间选择；未恢复任何内容 validator。
- 生产二进制、skills 与 public fallback 为三个独立产物；需要精确 SHA、文件模式及分享图片检查，不能只重启二进制。
- 真实 PDF 和下载截图保留在本地 `honeclaw-artifacts/earnings-20260913/`；最终产物哈希与下载结果已在本页与同目录 `canary-summary.json` / `production-pdf-verification.json` 留存。诊断轮不能作为投资报告交付。

临时旁路服务、反向数据库连接、搜索代理转发与本次启动的本地 PostgreSQL 已关闭，测试配置和登录态文件已移除；生产服务保持运行。

## Next Entry Point

已上线并归档；后续如出现特定公司的原文获取或报告质量偏差，沿原研究流程复现取数/搜索/模型偏差，保留真实 canary，不加内容门禁。原用户 worktree 未改动。生产报告保留在原聊天及 OSS；两份可交付 PDF 为 `honeclaw-artifacts/earnings-20260913/TEM-财报分析.pdf`（线上原产物）和 `TEM-财报前瞻.pdf`（旁路原文不变、修正分享图部署布局后的技术重渲染）。
