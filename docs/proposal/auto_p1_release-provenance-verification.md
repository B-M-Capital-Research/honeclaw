# Proposal: Release Provenance and Installer Verification Chain

status: proposed
priority: P1
created_at: 2026-05-26 02:06:42 +0800
owner: automation
related_files:
- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_update-compatibility-center.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p1_redacted-support-bundle.md`
- `docs/proposal/auto_p0_secrets-vault-rotation.md`
- `.github/workflows/release.yml`
- `.github/workflows/release-cache-warm.yml`
- `.github/workflows/secret-scan.yml`
- `scripts/install_hone_cli.sh`
- `scripts/update_homebrew_formula.sh`
- `scripts/prepare_release_notes.sh`
- `scripts/install_gitleaks.sh`
- `make_dmg_release.sh`
- `scripts/prepare_tauri_sidecar.mjs`
- `bins/hone-cli/src/main.rs`
- `bins/hone-cli/src/reports.rs`
- `bins/hone-cli/src/cleanup.rs`
- `bins/hone-desktop/src/sidecar.rs`
- `crates/hone-web-api/src/types.rs`
- `crates/hone-web-api/src/lib.rs`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Honeclaw 已经有一条可用的正式发布链路：

- `README.md` 给用户提供 `curl | bash`、Homebrew、源码运行和桌面端多种入口。
- `.github/workflows/release.yml` 在 `v*` tag 上创建 GitHub Release，分别构建 Linux/macOS CLI bundle，打包 `hone-cli`、`hone-console-page`、渠道二进制、`hone-mcp`、Web 产物、`skills/`、`config.example.yaml` 和 `soul.md`。
- release workflow 的 Homebrew job 会下载三个 `honeclaw-*.tar.gz` 产物，计算 `SHASUMS256.txt`，生成 Homebrew formula 并上传 checksum 文件。
- `scripts/update_homebrew_formula.sh` 把每个平台 tarball 的 SHA-256 写进公式，所以 Homebrew 安装路径已经具备基础完整性校验。
- `scripts/install_hone_cli.sh` 会按平台下载对应 tarball，检查 archive layout 是否只有一个顶层目录，拒绝 `../` / 绝对路径等危险条目，并检查必要 bundle 文件。
- `docs/proposal/auto_p1_update-compatibility-center.md` 已经提出 release/build manifest、安装状态、升级与回滚能力。

这些基础让 Hone 可以发版和安装，但它们还没有形成一条用户可验证的 release trust chain。尤其是 `curl | bash` 路径目前只验证 archive 结构，不验证下载内容是否匹配 release checksum，也不验证产物是否由官方 GitHub Actions workflow、目标 tag、目标 commit 构建。`SHASUMS256.txt` 本身也是普通 release asset，没有签名、attestation 或 bundle 内证据包。对一个会启动本地后台进程、IM listener、MCP server、agent runner 和桌面 sidecar 的产品来说，发布产物可信度是核心体验的一部分。

本轮参考的外部实践：

- GitHub Artifact Attestations 支持为 workflow 产物生成带 provenance 的签名声明，并可用 GitHub CLI 验证。
- GitHub 文档说明生成 attestation 需要 workflow 配置 `id-token: write`、`attestations: write` 等权限，并使用官方 artifact attestation action。
- SLSA 的验证模型强调：provenance 只有被消费者或监控系统按预期 builder、签名、subject digest、predicate 和参数检查时才真正产生安全价值。
- OpenSSF Scorecard 代表了开源项目把 supply-chain health 自动化成可持续信号的方向。

相关官方链接见文末 `## 外部参考`。

## 问题或机会

1. **`curl | bash` 安装路径没有强制 checksum 验证。**  
   installer 下载 tarball 后会做 layout safety 检查，但没有同时下载 `SHASUMS256.txt` 并比对 SHA-256。网络劫持、缓存污染、误传 asset 或镜像问题不能被第一时间发现。

2. **checksum 没有绑定构建身份。**  
   `SHASUMS256.txt` 能证明本地文件与某个发布页面上的 hash 一致，但不能证明 tarball 是由指定 GitHub workflow、指定 tag、指定 commit 构建。release asset 被替换、workflow context 被污染或人工覆盖时，用户缺少独立验证步骤。

3. **bundle 内没有 release evidence pack。**  
   用户安装后可以运行 `hone-cli doctor/status` 看运行健康，但无法离线回答“这个 bundle 包含哪些二进制、Web build、skills snapshot、源 commit、release notes、checksum、SBOM、attestation bundle”。支持人员排障也只能从版本号、日志和 release notes 拼。

4. **缺少 SBOM 和依赖可视性。**  
   Hone 发布包同时包含 Rust binaries、Bun-built Web assets、skills 和配置模板。现在没有一个伴随 release 的 SPDX/CycloneDX 或最小 dependency inventory。遇到依赖漏洞、许可证问题或用户安全审查时，无法把某个已安装 bundle 与依赖快照快速对应。

5. **桌面 DMG 与 CLI tarball 的信任链不统一。**  
   repo map 记录了 `make_dmg_release.sh` 和 Tauri sidecar 打包路径，但当前主要 tag release workflow 只覆盖 CLI bundle。桌面用户最终安装的 `.app` / `.dmg` 也需要同一套 manifest、checksum、attestation 和后续 notarization/signing 状态展示。

6. **已有 Update Compatibility Center 需要 provenance 信号作为输入。**  
   兼容中心可以告诉用户版本是否匹配、是否需要升级和如何回滚；但如果本地 release dir 没有经过完整性和来源验证，兼容判断只能说明“版本看起来对”，不能说明“安装物可信”。

这是 P1：它不是当前核心聊天链路的直接功能 bug，但会显著影响开源安装信任、桌面端留存、企业/团队自托管审查、支持效率和未来商业化可信度。随着 Hone 从源码项目走向可安装 agent 工作台，release provenance 应成为基础产品能力。

## 方案概述

新增 **Release Provenance and Installer Verification Chain**，把发布产物从“可下载 tarball + checksum”升级成“可验证来源、完整性、组成和安装状态”的产品与运维层。

第一版目标：

- 每个 release asset 生成并上传 build manifest。
- 每个 tarball 生成 SHA-256，并让 `curl | bash` installer 强制校验。
- 每个 tarball 生成 GitHub artifact attestation；支持用户用 `gh attestation verify` 或 `hone-cli release verify` 检查。
- 为 release bundle 生成最小 SBOM 或 dependency inventory，并对 SBOM 产物做 attestation。
- bundle 内保留 `release-evidence.json`，把 version、commit、target、asset digest、workflow、release notes、manifest、SBOM 路径和验证指令串起来。
- `hone-cli doctor/status` 与未来 Update Compatibility Center 消费 evidence，展示 `verified` / `checksum_only` / `unverified` / `unknown`。

第一版不要求立刻实现自动更新、不要求自建 PKI、不要求把所有 GitHub Actions pin 到 commit，也不把 macOS notarization 一次性做完。重点是先把 CLI bundle 和 installer 的验证链闭上。

## 用户体验变化

### 用户端

- `curl | bash` 安装时显示：
  - 下载的 release tag 与 asset 名称。
  - SHA-256 校验通过。
  - 如果本机有 `gh` 且启用 provenance 验证，则显示 attestation 通过或跳过原因。
  - 如果 checksum 下载失败或不匹配，默认停止安装。
- 安装后运行 `hone-cli doctor` 可看到：
  - `release integrity: verified` / `checksum_only` / `unverified`
  - `source commit`
  - `built by workflow`
  - `SBOM present`
  - `release notes`
- 普通用户不需要理解 SLSA，但能看到“这个安装包来自官方 release workflow”的明确状态。

### 管理端

- Settings / Update 面板展示 release provenance 摘要：
  - 当前 backend build manifest。
  - 当前 Web bundle / skills snapshot / channel binary 是否来自同一 release evidence。
  - 检测到 `unverified` 时给出修复动作：重新安装、通过 Homebrew 升级、运行 `hone-cli release verify`。
- Redacted support bundle 可以包含 evidence 摘要和验证结果，不包含用户密钥或私有数据。

### 桌面端

- Bundled desktop 启动时读取 sidecar / resource manifest，展示 shell、backend、channel binaries、Web build 是否同源。
- 后续 DMG 打包接入同一 evidence schema。macOS code signing / notarization 状态可作为 `desktop_signature` 字段加入，但不阻塞第一版 CLI bundle。
- Remote desktop mode 只展示远端 backend 返回的 provenance 摘要，不尝试校验本机不存在的远端 tarball。

### 多渠道

- Channel heartbeat 可附带 `build_id` / `release_evidence_id`，让 `/api/channels` 区分“进程在线但来自未知构建”和“进程在线且与 backend 同 release”。
- 当用户在 IM 渠道遇到升级或维护提示时，系统可以输出稳定 reason code，不暴露内部路径或 workflow detail。

## 技术方案

### 1. Release evidence schema

在每个 tarball 内写入：

```text
share/honeclaw/release-evidence.json
share/honeclaw/SBOM.spdx.json
share/honeclaw/SHASUMS256.txt
```

建议 `release-evidence.json` 字段：

```json
{
  "schema_version": 1,
  "product": "honeclaw",
  "version": "0.12.2",
  "git_commit": "...",
  "git_ref": "refs/tags/v0.12.2",
  "target": "aarch64-apple-darwin",
  "asset_name": "honeclaw-darwin-aarch64.tar.gz",
  "asset_sha256": "...",
  "workflow": ".github/workflows/release.yml",
  "workflow_run_id": "...",
  "built_at": "2026-05-26T00:00:00Z",
  "release_notes": "docs/releases/v0.12.2.md",
  "sbom": "share/honeclaw/SBOM.spdx.json",
  "binaries": [
    {"name": "hone-cli", "sha256": "..."},
    {"name": "hone-console-page", "sha256": "..."},
    {"name": "hone-mcp", "sha256": "..."}
  ],
  "web_assets": {
    "admin": {"path": "share/honeclaw/web", "sha256_manifest": "..."},
    "public": {"path": "share/honeclaw/web-public", "sha256_manifest": "..."}
  },
  "skills_snapshot_sha256": "...",
  "attestation": {
    "provider": "github-artifact-attestations",
    "expected_repo": "B-M-Capital-Research/honeclaw",
    "expected_signer_workflow": ".github/workflows/release.yml",
    "predicate_type": "https://slsa.dev/provenance/v1"
  }
}
```

该 schema 与 Update Compatibility Center 的 `BuildManifest` 相邻，但职责不同：

- `BuildManifest` 回答版本、API、组件兼容。
- `release-evidence.json` 回答来源、digest、SBOM 和验证策略。

实际落地时可以合并成一个文件的两个 section，避免重复读写。

### 2. Release workflow 改造

建议调整 `.github/workflows/release.yml`：

1. 将 workflow permissions 从仅 `contents: write` 扩展为最小必要权限：
   - `contents: write`
   - `id-token: write`
   - `attestations: write`
2. 在每个 matrix build job 内：
   - 生成 bundle 文件级 hash manifest。
   - 生成 SBOM 或最小 dependency inventory。
   - 写入 `release-evidence.json`。
   - 打包 tarball。
   - 计算 tarball SHA-256。
   - 对 tarball 运行官方 build provenance attestation action。
   - 对 SBOM 运行 SBOM attestation。
3. 上传 tarball、checksum、evidence 和 SBOM。
4. 在 Homebrew job 中继续生成 formula checksum，但 checksum 来源改为 build job 产出的 digest artifact，而不是只在下载后重新计算。
5. 增加 release verify job：
   - 下载本次 release assets。
   - 逐个验证 SHA-256。
   - 运行 `gh attestation verify dist/<asset> -R B-M-Capital-Research/honeclaw --signer-workflow B-M-Capital-Research/honeclaw/.github/workflows/release.yml`.
   - 验证 evidence 中的 digest 与实际文件一致。

### 3. Installer verification

`scripts/install_hone_cli.sh` 增加强制 checksum 校验：

- 下载目标 tarball。
- 下载同 tag 的 `SHASUMS256.txt`。
- 从 checksum 文件中提取当前 `ASSET_NAME` 的 digest。
- 本地计算 SHA-256 并比对。
- 通过后再执行当前 archive layout 检查和解压。

Attestation 策略：

- 默认不要求用户安装 `gh`，避免破坏低摩擦安装。
- 如果本机存在 `gh`，且 `HONE_VERIFY_ATTESTATION=1` 或 installer 进入 strict mode，则运行 attestation 验证。
- 如果 strict mode 下 attestation 失败，停止安装。
- 非 strict mode 下，缺少 `gh` 只标为 `checksum_only`，安装完成后 `hone-cli doctor` 提醒可运行验证命令。

建议新增环境变量：

- `HONE_INSTALL_VERIFY=checksum` 默认值。
- `HONE_INSTALL_VERIFY=attestation` 强制 `gh attestation verify`。
- `HONE_INSTALL_VERIFY=none` 仅允许显式开发/应急绕过，并打印高风险警告。

### 4. CLI verification surface

在 `hone-cli` 增加只读命令：

```text
hone-cli release verify
hone-cli release verify --asset ./honeclaw-darwin-aarch64.tar.gz --tag v0.12.2
hone-cli release evidence --json
```

`hone-cli release verify` 默认验证当前安装：

- 读取 `$HONE_INSTALL_ROOT/share/honeclaw/release-evidence.json`。
- 验证当前二进制和 Web/skills snapshot digest 是否匹配 evidence。
- 如果可以访问 GitHub 且本机有 `gh`，验证对应 release asset attestation。
- 输出稳定状态：
  - `verified`
  - `checksum_only`
  - `local_digest_mismatch`
  - `attestation_missing`
  - `attestation_failed`
  - `evidence_missing`
  - `source_checkout`

`hone-cli doctor/status --json` 只消费摘要，不强制联网。

### 5. SBOM 与 dependency inventory

第一版可以采用渐进策略：

- Rust：从 `cargo metadata --locked` 生成 dependency inventory；后续可接 `cargo cyclonedx` 或 `cargo auditable`。
- Web：从 `bun.lock` / package metadata 生成 dependency inventory；后续可接 CycloneDX 工具。
- Bundle：记录打包进 release 的二进制、Web assets、skills snapshot、配置模板和 license files。

即使第一版不是完整 CycloneDX，也要让 evidence 明确 `sbom_kind=minimal_inventory`，避免误称完整 SBOM。等工具链稳定后再升级到 SPDX/CycloneDX 并生成 attestation。

### 6. Runtime / Web API consumption

后端 `/api/meta` 可逐步增加非敏感字段：

- `build.evidence_status`
- `build.version`
- `build.git_commit`
- `build.target`
- `build.workflow`
- `build.source="release" | "source_checkout" | "unknown"`

Public API 不暴露本地文件路径、install root 或完整 workflow run URL；admin/desktop surface 才展示更完整 evidence。

## 实施步骤

### Phase 1: Checksum and evidence baseline

- 在 release package step 生成 `release-evidence.json` 和文件级 digest manifest。
- 将 tarball SHA-256 从 build job 产出为 artifact，Homebrew job 消费该 digest。
- 修改 `scripts/install_hone_cli.sh`，默认强制下载并验证 `SHASUMS256.txt`。
- 增加 installer 单元/脚本验证：正确 checksum 通过，篡改 tarball 失败，缺少 checksum 失败。

### Phase 2: GitHub artifact attestations

- 给 release workflow 加 `id-token: write` 与 `attestations: write`。
- 对三个 CLI tarball 生成 provenance attestation。
- 增加 release verify job，用 `gh attestation verify` 校验每个 asset 的 repo、signer workflow、tag/source ref。
- 在 release notes 或 `docs/releases/README.md` 增加用户验证命令。

### Phase 3: CLI verify command and Update Center integration

- 新增 `hone-cli release verify/evidence`。
- `hone-cli doctor/status --json` 输出 evidence 摘要。
- Future Update Compatibility Center 展示 provenance status，并在 support bundle 中包含脱敏 evidence summary。

### Phase 4: SBOM and desktop alignment

- 生成最小 dependency inventory，并对 SBOM/inventory 做 attestation。
- 将桌面 `make_dmg_release.sh` / Tauri sidecar bundle 写入同类 evidence。
- 将 macOS code signing / notarization 状态加入 evidence，但不让第一版 release provenance 依赖它。

## 验证方式

- Release workflow 验证：
  - 每个 release tarball 都有 SHA-256、evidence 和 attestation。
  - `gh attestation verify` 对三个 tarball 成功，且 signer workflow 限定到 `.github/workflows/release.yml`。
  - SBOM/inventory 文件存在并与 evidence 引用一致。
- Installer 回归：
  - 正常 release asset 通过 checksum、layout 和 required file 检查。
  - 修改 tarball 任意字节后 checksum mismatch，installer 退出非 0。
  - `HONE_INSTALL_VERIFY=attestation` 且缺少 `gh` 时退出非 0，并给出明确安装/降级说明。
  - `HONE_INSTALL_VERIFY=none` 只在显式设置时允许继续，并打印 warning。
- CLI 验证：
  - source checkout 返回 `source_checkout`，不误报 release 缺失。
  - Homebrew 安装读取 formula checksum / local evidence，不暴露 Cellar 外敏感路径给 public API。
  - 当前安装二进制 hash 不匹配时 `hone-cli release verify` 返回非 0。
- API / UI 验证：
  - `/api/meta` public surface 只返回非敏感 build summary。
  - admin/desktop surface 能显示 `verified` / `checksum_only` / `unverified`。
- 手工验收：
  - 从 GitHub Release 下载 tarball，按 release notes 中的命令验证 checksum 和 attestation。
  - 通过 `curl | bash` 安装最新 release，确认 installer 输出校验状态。

## 风险与取舍

- 风险：过早强制 attestation 会让低摩擦安装失败。  
  取舍：第一版默认强制 checksum，attestation 作为 strict mode；企业/高级用户可开启强校验。

- 风险：GitHub Actions 仍是信任根，attestation 不防 GitHub 平台或 workflow 本身被信任内攻击。  
  取舍：先验证 repo、workflow、tag、commit 和 digest；后续再推动 pinned actions、two-person release、protected tags、Scorecard gate。

- 风险：SBOM 工具链增加 release 时间和维护负担。  
  取舍：第一版可以是 `minimal_inventory`，明确不冒充完整 CycloneDX/SPDX；待稳定后再升级。

- 风险：checksum/evidence/manifest 与 Update Compatibility Center 重叠。  
  取舍：provenance 只负责来源和完整性；compatibility 负责版本窗口、升级、回滚和 schema 支持。两者可以共享文件，但不混淆判定。

- 风险：桌面 DMG、macOS signing、notarization 范围较大。  
  取舍：第一版先闭合 CLI tarball 和 installer；桌面 evidence 在第四阶段接入，不阻塞主线。

- 风险：release workflow 权限扩大。  
  取舍：只增加 attestation 所需的 `id-token: write` 与 `attestations: write`，并把 release verify job 作为同一 workflow 的强校验。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点核对了 release、update、compatibility、secret、support、journey、security、supply-chain、checksum、attestation、SBOM、Homebrew、installer、desktop startup 相关主题。

- 不重复 `auto_p1_update-compatibility-center.md`：该提案回答“当前安装/组件/远端 API 是否兼容、如何升级或回滚”；本提案回答“当前 release asset 是否由官方 workflow 构建、内容是否被篡改、安装物是否有 SBOM/evidence 可验证”。
- 不重复 `auto_p1_user-journey-replay-lab.md`：该提案把产品旅程转成 release confidence 测试；本提案把发布产物转成可验证 supply-chain evidence。
- 不重复 `auto_p1_redacted-support-bundle.md`：该提案导出排障证据且脱敏；本提案提供 support bundle 可引用的非敏感 release provenance 摘要。
- 不重复 `auto_p0_secrets-vault-rotation.md`：该提案治理运行期凭证生命周期；本提案治理安装产物来源、完整性和 SBOM。
- 不重复 `docs/proposals/desktop-bundled-runtime-startup-ux.md`：该历史提案解决桌面 bundled runtime 启动体验；本提案只在后续阶段把 desktop bundle 纳入同一 release evidence schema。

差异结论：现有提案已经覆盖更新兼容、运行就绪、用户旅程测试、凭证安全和支持包，但还没有覆盖 **release artifact provenance + installer verification + SBOM/evidence pack** 这一层。该主题可独立落地，并能成为后续 Update Compatibility、desktop release、support bundle 和企业安全审查的基础输入。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，因此不更新 `docs/current-plan.md`、`docs/repo-map.md` 或 `docs/invariants.md`。如果后续实际落地该提案，需要新增/复用 current plan，并在改变 release workflow、installer 验证契约、release evidence schema 或 CLI verify 命令时同步更新 repo map、release runbook、installer 文档和必要的长期约束。

## 外部参考

- GitHub Docs: [Using artifact attestations to establish provenance for builds](https://docs.github.com/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds)
- GitHub CLI Manual: [gh attestation verify](https://cli.github.com/manual/gh_attestation_verify)
- SLSA: [Build: Verifying artifacts](https://slsa.dev/spec/v1.2/verifying-artifacts)
- OpenSSF: [Scorecard](https://openssf.org/scorecard/)
