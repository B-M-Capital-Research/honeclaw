# Earnings Routing 与 MiniMax 协议泄漏生产热修复

- title: Earnings Routing 与 MiniMax 协议泄漏生产热修复
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-channels/src/agent_session/artifacts.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `agents/function_calling/src/lib.rs`
- related_docs:
  - `docs/archive/index.md`
  - `docs/handoffs/2026-08-04-production-deployment-dede2d61.md`

## Goal

恢复生产管理员财报入口：标准财报前瞻 / 财报分析请求无论来自结构化弹窗还是聊天框标准文案，都必须由服务端提升为 `earnings-research` 原生 Skill 并使用 Codex ACP；同时阻止 MiniMax 等普通 function-calling provider 的私有工具协议标签泄漏到用户正文。

## Scope

- 只识别 HONE 自己生成的中英文财报标准文案，不把宽泛自然语言误判为管理员工作流。
- 只有数据库复核后的管理员可以通过标准文案进入受信任 runner；非管理员保持普通问答与既有权限边界。
- function-calling 流式输出隐藏标准工具标签与带 provider namespace 的工具标签，包括拆分 chunk 和未闭合块。
- 财报 Skill 已写入附件标记时，仍需从本轮 actor sandbox 收集生成物并在 cloud-authoritative 模式上传 OSS；把脱敏占位引用改写为持久引用，不能因看到 `[附件: ...]` 就跳过持久化。
- 公开下载代理只放行当前已认证 actor 自己的 OSS 前缀；Agent 生成物与用户上传物都可下载，任何其它 actor 或 bucket 继续 fail closed。

## Validation

- `hone-web-api` 单元测试覆盖中英文 preview / analysis 标准文案、公司提取、普通文本不误判。
- `function_calling` 单元测试覆盖 `<minimax:tool_call>` 完整、拆分和未闭合输出，并保持标签前后正常正文。
- `hone-channels` 单元测试覆盖 Agent 自带 `<absolute-path>` 附件标记时仍收集文件、替换引用且不生成重复卡片。
- `hone-core` / Web API 回归覆盖 actor-owned OSS 生成物可下载，同时拒绝其它 actor 与其它 bucket。
- `hone-core` 136 tests、`hone-web-api` 185 tests（2 ignored）、此前完整 `hone-channels` 752 tests（1 ignored）、workspace check、Web 与 CI-safe 回归通过。
- GitHub CI `30934748021`、Runtime Image `30934748252`、Secret Scan `30934750399` 成功。
- 生产精确 revision `9d64c5967bf74a5126948c7b49f6b918128f951a` 健康；真实 CRCL PDF 在服务重启后仍显示并可从聊天卡片下载。

## Documentation Sync

- 已从 `docs/current-plan.md` 移除，并归档到本文件。
- 已更新生产 handoff 与 `docs/archive/index.md`。
- 本次不改变模块边界或长期架构约束，因此未修改 `docs/repo-map.md`、`docs/invariants.md`，也未新增 ADR。

## Risks / Open Questions

- 修复前生成的旧 CRCL PDF 只存在于已消失的临时目录，无法恢复；修复后新生成物才是持久附件。
- 标准文案识别继续 fail-closed，普通用户不能仅靠提示词触发管理员原生 runner。
- Skill 备份不能放在 `HONE_SKILLS_DIR` 的任何子目录中，否则会被发现为额外候选。
- ChatGPT 设备授权后若同一 `auth.json` 被复制并并发刷新，仍可能触发 refresh-token rotation 冲突。

## Result

生产标准财报请求现在稳定走 Codex ACP 与最新版 `earnings-research` Skill；真实 PDF 会在当前 actor 边界内持久化到 OSS，并可在服务重启后从历史聊天下载。此次没有创建 tag 或正式 release，旧 runtime release 保留用于回滚。
