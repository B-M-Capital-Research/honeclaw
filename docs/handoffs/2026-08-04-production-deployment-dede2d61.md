# Production Deployment dede2d61

- title: 远端十提交审查与生产部署
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - packages/app/src/lib/public-content.ts
  - packages/app/src/pages/chat.tsx
  - crates/hone-channels/src/investment_response_guard.rs
  - skills/earnings-research/SKILL.md
  - skills/earnings-research/scripts/render_report_pdf.py
- related_docs:
  - docs/archive/plans/production-deployment-dede2d61.md
  - docs/runbooks/backend-deployment.md
  - docs/handoffs/2026-08-04-earnings-research-chat-entry.md
- related_prs: none; direct `main` commits and production deployment

## Summary

`ee7024b6..dede2d61` 的十个远端提交已完成逐模块审查；阻断项修复后形成并推送精确 revision `3b01aa2c4567f80ebe2c77fc096887d46b4b634f`。GitHub CI、runtime-image、Cloudflare Pages 与 secret scan 均成功，生产后端已从旧 release 原子切换到该 revision 的 GHCR digest `sha256:dd04fad942cf9c7c223eaea6bb24c830f1b4c39633f2a3ffcf75a20fb02f05a4`，旧 release 保留用于回滚。此次是普通 push/deploy，没有创建 `v*` tag 或正式 release。

## What Changed

- 审查修复了扩展时段预载使用陈旧/未核验分钟线、i18n 模块级内容冻结与硬编码中文日期、财报工作流客户端/服务端英文状态不一致，以及 macOS 系统代理导致 loopback billing E2E 超时四类问题。
- 私有 GHCR 包需要最小 `read:packages` 权限；经用户确认后，GitHub CLI OAuth scope 已扩展为只读包权限。镜像凭据仅通过标准输入进入 GCE 临时 `DOCKER_CONFIG`，暂存完成即删除，主机未留下 registry token。
- 生产 `HONE_SKILLS_DIR` 指向独立于 executable-only GHCR bundle 的外置技能树。目标 checkout 有大量既有未提交运维变更，未执行 pull 或覆盖；仅将原先不存在的 `earnings-research` 目录按三文件 SHA-256 manifest 原子安装，并通过 `/api/skills` 读回为 `enabled=true`、`loaded_from=system`。
- GCE 缺少官方 PDF 渲染器所需的 Chromium 与中文字体；安装 Debian `chromium 151.0.7922.71` 和 `fonts-noto-cjk` 后，以真实 `hone` 服务用户执行官方 renderer。首次视觉 QA 准确发现中文 tofu 方框，补字体后第二次两页 A4 PDF 逐页复验通过：中文正常、`知识星球：巴芒科技` 水印可见、知识星球分享图和免责声明完整。
- Cloudflare Pages 生产入口为 `/assets/index-C_8fsYF_.js`，聊天 chunk 为 `/assets/chat-CCc3iT0S.js`；在线 bundle 含 `earningsWorkflow`、`onStartEarnings`、中英文财报入口与 loading markers。当前已登录 Chrome 会话不是管理员，页面不显示两个管理员按钮符合权限边界；本轮没有在生产发起真实 SNDK 研究回合或创建业务 PDF。

## Verification

- 本地完整门禁：Rust workspace check/test、Web 362 tests、Worker typecheck/test、Public build、44/44 CI-safe regression 全部通过。
- GitHub：CI run `30915979016`、runtime image run `30915978081`、Cloudflare Pages、Secret Scan 全部成功；镜像 bundle 验证器确认 exact SHA 和所有 payload checksums。
- 切换前：runtime env validator 通过；连续两次 `/api/runtime/active-chat-runs` 均为 `{"count":0}`；目标 release 独立验证通过。
- 切换后：`hone-web.service=active`、`NRestarts=0`，真实 executable 位于目标 immutable release；`8077`/`8088` 监听正常，active chats 为 0。
- `/api/meta`：`git_sha=3b01aa2c...`、`source=ghcr_linux_oci`、`cloud_mode=cloud`、`cloud_storage_authoritative=true`、PostgreSQL/S3 health 均为 true、`local_durable_dependency_count=0`。
- loopback `8088` 与 `https://hone-claw.com/api/public/auth/me` 均返回应用 JSON `401`；Public API 响应经过 GCE/Caddy 路径。
- 生产官方 renderer smoke：PDF 1.4、A4、2 页、315847 bytes；Poppler 120-DPI 全页渲染人工检查无乱码、裁切、重叠或分享图缺失。

## Risks / Follow-ups

- `origin.hone-claw.com` 仍返回旧 Sunny-Ngrok 的 `307 Tunnel not found`，而当前用户主站 API 实际通过受 token 保护的 GCE/Caddy origin 健康工作。该旧 alias 不影响本次主站发布，但仍是 community-edge legacy fallback 和运维文档中的风险；必须在启用相关 fallback 前单独梳理 DNS/Worker origin 约定，不能把此 `307` 误判为健康，也不要在未评审时直接改 DNS。
- GHCR bundle 目前不携带技能和分享图，生产依赖外置 `HONE_SKILLS_DIR`。本次已把精确同步、runtime readback、Chromium/CJK 视觉门禁写入 runbook；长期更稳妥的方向是将版本化技能资产纳入不可变 release，避免 executable revision 与 skill revision 漂移。
- 当前生产 Chrome smoke 只能证明普通用户看不到管理员入口和 live bundle markers 已上线；若要重复真实端到端验收，使用管理员账号从按钮发起一次附件型 `财报分析`，核对聊天正文、PDF 卡片和鉴权下载。不要用普通聊天命令替代结构化管理员入口。
- 实例级 OS Login 2FA 已按用户说明关闭。主机重建后需重新核对该实例级元数据，同时继续保留 IAP、最小 IAM/OS Login 权限和审计。

## Next Entry Point

生产回滚时将 `/opt/hone/current` 原子指回保留的 `edddfc5b890d124d76d8c6eddc9aa85f2e94b807-ghcr-runtime` 并重启 `hone-web.service`，随后重复云权威、active-chat、loopback/public `401` 与 Pages asset 检查。财报链路后续从 `docs/handoffs/2026-08-04-earnings-research-chat-entry.md` 和本 runbook 的 runtime skill/PDF 依赖段继续。

## 2026-08-05 财报链路生产复验与补丁

用户后续提交 `078b0883`、`cfb75481` 与 `50aa8b23` 分别补齐了标准财报文案到 Codex ACP 的强制路由、认证下载与安全路径恢复，以及近期新闻页密度。真实生产验收又发现两个通用缺口：当模型正文已经包含 `[附件: ...]` 时，附件收集会提前返回，导致临时文件没有上传 OSS；上传后，公开下载代理又只接受 public-upload 前缀，拒绝当前 actor 自己的 Agent 生成物前缀。`ee250d72` 和 `9d64c596` 分别修复这两处，并加入回归测试。

精确生产运行 revision 为 `9d64c5967bf74a5126948c7b49f6b918128f951a`，GHCR digest 为 `sha256:c05de8786317522a523f9754745b4ca509696074cae22a3815ad9e9d1bc2ee1d`。`/api/meta`、PostgreSQL、OSS、cloud-authoritative 与零本地持久依赖均健康，服务 `NRestarts=0`。外置 `earnings-research` Skill 使用 `50aa8b23` 内容；旧 Skill 备份已移出发现根目录到 `/opt/hone/skill-rollbacks/earnings-research-cfb75481`，避免被当成第二个同名候选加载。

管理员浏览器用标准原话“请为 CRCL 生成财报前瞻，并完成证据核验和可分享 PDF。”完成真实验收：请求被服务端提升为 `/earnings-research`，日志确认由 `codex-acp 1.1.7` 执行，生成 `CRCL_FY2026_Q2_20260805.pdf-451d7f44.pdf`，上传为当前 actor 的 OSS 对象并写入聊天历史。完整服务重启后卡片仍存在，点击状态变为“已开始下载”。修复前生成的旧 `CRCL_FY2026_Q2_-.pdf-b316b020.pdf` 仅存在于已消失的临时目录，无法恢复；修复后的新附件才具备持久性。

本阶段通过 `hone-core` 136 tests、`hone-web-api` 185 tests（2 ignored）、此前完整 `hone-channels` 752 tests（1 ignored）、workspace check、Web 与 CI-safe 回归；GitHub CI `30934748021`、Runtime Image `30934748252` 与 Secret Scan `30934750399` 成功。此次为普通 push 与生产部署，没有创建 tag 或正式 release。保留旧 runtime release 作为回滚点。
