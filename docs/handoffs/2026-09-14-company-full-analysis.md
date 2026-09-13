# Company full analysis handoff

- title: 公司完整分析原工作流迁移、持久进度与完整 PDF
- status: done / deployed
- created_at: 2026-09-14
- updated_at: 2026-09-14
- owner: Codex
- related_files: crates/hone-web-api/src/routes/company_analysis/; packages/app/src/components/company-analysis-dialog.tsx; skills/earnings-research/scripts/; scripts/ci/check_fmt_changed.sh
- related_docs: docs/runbooks/company-full-analysis.md; docs/decisions.md#d-2026-09-14-01-company-full-analysis-as-a-durable-staged-task; crates/hone-web-api/src/routes/company_analysis/original/README.md
- related_prs: https://github.com/B-M-Capital-Research/honeclaw/commit/9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d

## Summary

管理员在聊天输入框上方「工具 → 公司完整分析」输入公司名/代码，启动独立任务。按原 Dify `全跑完-美` DAG 与 9 个原节点 Prompt 执行，原研究 Prompt 未编辑。服务端 UUID 与真实阶段进度支持关闭弹窗后恢复查看、技术失败后继续，最终交付完整 PDF。

## What Changed

- 读取用户指定主工作流与财务、搜索子工作流的可见节点，逐字留存 system/user Prompt 与变量；只迁移美股完整链路。原数据接口换成 HONE 财务/搜索工具，外部鉴权码、上传、进度服务换成 HONE 管理员授权、actor 任务与 OSS。
- 每次 API 请求检查数据库管理员标志。任务读写按 actor 隔离；20 秒续期/两分钟租约，恢复分配新 attempt token。原创建时间与完成节点复用。每次尝试使用独立 PDF key，完成前上传并回读 SHA-256；旧进程不能覆写新进程产物。
- PDF 只做排版与技术完整性检查。保留原完整段落、quote 假设、图和代码；只去掉匹配的外层 Dify 三单引号包装。离线 Mermaid/KaTeX、严格 CSP、中文字体、表格换行与技术失败降级源文；无机械内容门禁。
- 修复 changed-file rustfmt 脚本递归检查未改动子模块的问题，按既有“仅改动文件”契约设置 skip_children，并用临时 Git 仓库回归证明。

## Verification

证据目录：`/Users/bytedance/Desktop/honeclaw-artifacts/company-analysis-20260914/`，含完整 Markdown/PDF、canary JSON、权限与重启哈希证据和测试日志。无账号凭据。

- Web API 完整单元测试 386 passed / 2 ignored / 0 failed。
- Rust workspace check 通过；公司任务专项 8/8，通过 actor 隔离、DDL 初始化、执行权围栏、检查点恢复、完整产物要求及原 Prompt 绑定测试。
- Web 单元测试 649/649，typecheck/public build 通过；浏览器 E2E 5/5，覆盖公司单输入、任务恢复/不重复启动、PDF 下载与非管理员入口隐藏；Worker 45/45 与 typecheck 通过。
- PDF 回归通过，包括进程挂起回收、原子发布、图表完成信号、失败保留源文、HTML 安全、数学公式和美元价格区分；changed-file fmt 与其新增回归通过。
- 真实 HTTP 权限：匿名 401；普通用户对 list/start/status/resume/pdf 均 403；未知/非所属任务不存在。PG 单元测试证明另一 actor 无法读取或恢复任务。
- 首轮 TEM task `9a791709-4a88-4ca5-8eab-274b517db6d3`：9 个模型节点完成。隔离 canary 的 R2 provider 配置遗漏在保存阶段失败；修正后以新 attempt 恢复，所有模型用量完全不变，成功交付 20 页 PDF，服务重启后下载 SHA-256 相同。
- 最终 TEM task `f7dacef1-4280-4866-9f8c-730c687be990`：Gemini 3.1 Pro，202.5 秒，一次完成 9 节点；输入 165,933 tokens、输出 34,388 tokens；PDF 4,667,458 bytes / 20 页，SHA-256 `ef55537fce4f7a079fbf6e39c409f746233163a01ff2747efacf67a7a9862b05`，无渲染 warnings。逐页检查所有中文、表格、图解与假设。
- 相同最终正文在生产 Linux Chromium 渲染为 19 页，完整逐页检查通过；页数差异来自字体排版。独立 Linux 数学 smoke 验证箭头、分数、上标及普通美元价格，不联网加载字体或脚本。
- 全仓 `cargo test --no-fail-fast` 的历史失败仍是 hone-agent 两个 streaming mock `unreachable`（`deferred_prefix_ignores_structurally_invalid_datafetch_activation`、`first_batch_identity_route_limit_executes_only_six_valid_routes`）与 hone-core `config_example_avoids_stale_config_knobs` 的 README_EN api_key/api_keys 断言。其他 targets 通过。CI-safe 总脚本停在旧 finance acceptance 的 40/50，失败案例与上一轮一致（18/22/40/42/23/24/28/29/37/43）；不宣称全仓全绿。

- 生产 `https://hone-claw.com/chat` 实际管理员浏览器 TEM task `f9ba885f-ec35-40cc-87d5-ffdfc5ac2460`：原 9 个节点完成，进度 10→20→25→50→100；关闭重开仍为同一任务，页面下载成功，PDF 1,517,386 bytes / 20 页，SHA-256 `f7ecf78f6549c5633c08703d5a4b3af584b80a1908ab7a6322d4ac6c6c3f6db9`。全 20 页排版检查通过，文件为证据目录 `TEM-production-company-analysis.pdf`，页面状态见本任务浏览器验收记录。

## Risks / Follow-ups

- 保留原生成逻辑意味着来源/模型仍影响研究质量；canary 复核了任务与排版完整性，不等于逐条审计投资主张。搜索服务目前可返回 Gemini grounding 综合结果，原 Prompt 仍可能混淆收购推进与整合完成等时态；后续应通过来源与上下文改进，不能加机械门禁或改写核心 Prompt。
- 页面关闭不取消任务；进度是阶段权重而非耗时比例。进程中断后须等待租约过期或任务标记失败，再点继续。
- 原 qwen-plus 标题节点适配为同一个配置模型。记录节点用量但不承诺每次相同成本或页数。
- PDF 页眉沿用服务器本地日期，研究日期/标题使用原 Asia/Shanghai 时间；跨 UTC 日期边界可相差一天。
- 回滚恢复旧 runtime/frontend/earnings renderer symlinks；数据库表是新增，保留任务和 OSS 对象即可，不需要删除数据。

## Next Entry Point

`docs/runbooks/company-full-analysis.md` 与 `routes/company_analysis/original/README.md`。运行构建为 `9a2f27c7`；后续 CI 文档提交不改变应用产物。

## Deployment

- Runtime Image build: https://github.com/B-M-Capital-Research/honeclaw/actions/runs/34769281131 (success).
- Application revision `9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d`; immutable image `sha256:bd144bcdcc7b7879bac2ff54239223a5d58fdf34dd70e5d485974be3dc9dc73f`; service binary SHA-256 `5be7b6bbc467c03e0c069ed65a760640c807d476167e336a7ba9fe7cc5a6cf74`.
- GCE `instance-20260731-081043` / `us-central1-c`: staged runtime, frontend and complete renderer assets; checked 683 asset hashes/modes; two zero-active-run checks before restart. Health confirmed exact revision, PostgreSQL/R2 healthy, cloud authoritative, local durable dependency count 0, role all.
- Current runtime `/opt/hone/releases/9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d-ghcr-runtime`; frontend `/opt/hone/public-web/releases/9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d/dist-public`; renderer `/srv/honeclaw/earnings-releases/9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d/skills/earnings-research`.
- Rollback: restore all three symlinks to the corresponding `161fe4b30e84860a7da4f6d536a89d7d4c379d13` directories and restart hone-web after draining. No production config or environment was changed; prior earnings Opus setting remains. No release tag created.

- Cloudflare Pages 主站已跟随 main 更新，线上管理员入口与单公司输入弹窗已实际验收；生产完整 TEM canary 与下载通过。
- 已停止本任务本地 canary 服务、搜索 SSH 隧道和临时 PostgreSQL，删除临时配置、cookie 与登录响应；不影响生产运行。
