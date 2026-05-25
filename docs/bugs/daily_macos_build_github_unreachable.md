# Daily macOS build cannot fetch latest main from GitHub

- 发现时间：2026-05-25 04:06 CST
- Bug Type：Daily macOS build verification / source sync
- 严重等级：P3
- 状态：Later
- GitHub Issue：未创建
- 证据来源：`honeclaw-mac` 每日 macOS 完整打包验证

## 端到端链路

1. 自动化开始时工作区干净，当前分支为 `main`。
2. 按要求先执行 `git fetch origin`，远端为 `git@github.com:B-M-Capital-Research/honeclaw.git`。
3. `git fetch origin` 在连接 `github.com:22` 时超时，未能读取远端仓库。
4. 备用验证不改 remote，分别尝试：
   - `GIT_SSH_COMMAND='ssh -o BatchMode=yes -o ConnectTimeout=20 -p 443' git ls-remote ssh://git@ssh.github.com/B-M-Capital-Research/honeclaw.git HEAD`
   - `GIT_TERMINAL_PROMPT=0 git ls-remote https://github.com/B-M-Capital-Research/honeclaw.git HEAD`
5. SSH-over-443 与 HTTPS 同样无法连接 GitHub，因此本轮不能确认本地 `main` 已更新到远端最新提交。
6. 2026-05-26 04:08 CST 每日自动化再次复现同一阻塞：本地 `main` 已收口上一轮遗留 rebase，工作区干净，但 `git fetch origin` 仍无法连接 GitHub。

## 期望效果

- 每日 macOS 打包验证能够先拉取远端最新 `main`。
- 拉取成功后再执行完整 `build:desktop`、确认 `.app` / `.dmg`，并用隔离配置启动 `.app/Contents/MacOS/hone-desktop` 做 smoke 验证。

## 当前实现效果

- 拉取阶段失败：
  - `ssh: connect to host github.com port 22: Operation timed out`
  - `ssh: connect to host ssh.github.com port 443: Operation timed out`
  - `Failed to connect to github.com port 443 after 75004 ms`
- 2026-05-26 04:08 CST 复现：
  - `git fetch origin`：`ssh: connect to host github.com port 22: Operation timed out`
  - SSH-over-443 `git ls-remote`：`ssh: connect to host ssh.github.com port 443: Operation timed out`
  - HTTPS `git ls-remote`：`Failed to connect to github.com port 443 after 75007 ms`
- 因为无法确认远端最新代码，本轮没有继续声称完成基于最新代码的打包与启动验证。

## 用户影响

- 本轮每日 macOS 完整打包验证未闭环，不能证明最新远端代码可以打出 `.app` 与 `.dmg`。
- 现有本地代码和历史产物未被修改；没有启动真实 Feishu / Telegram / Discord / iMessage 渠道。

## 根因判断

- 当前证据更像本机到 GitHub 的网络连通性或出口策略问题，而不是仓库代码问题。
- 因为 SSH 22、SSH 443 和 HTTPS 443 均失败，本轮暂按 P3 外部依赖 / 网络阻塞记录。
- 2026-05-25 04:10 CST `bug-2` 复核：该缺陷当前没有可本地代码修复的通用根因；按缺陷修复规则，不为单次 GitHub 网络不可达写特殊兼容逻辑，状态从 `New` 调整为 `Later`，待同机网络恢复后重跑每日 macOS 打包验证。
- 2026-05-26 04:12 CST `bug-2` 复核：04:08 CST 的复现仍集中在当前机器到 GitHub 的 SSH 22、SSH-over-443 与 HTTPS 443 连通性，属于外部网络/出口阻塞；不为该问题写代码特判，状态保持 `Later`，待同机网络恢复后重跑每日 macOS 打包验证。

## 下一步建议

1. 在同一机器上复查 GitHub 网络连通性、VPN / 代理 / 防火墙策略。
2. 网络恢复后重新运行 `honeclaw-mac`，优先确认 `git fetch origin` 与 `git pull --rebase` 能正常完成。
3. 若 GitHub 连通性恢复后打包或 smoke 验证失败，再按新的失败阶段另行更新或新增缺陷文档。

## 验证结果

- `git status --short`：通过，开始时无输出。
- `git fetch origin`：失败，GitHub SSH 22 超时。
- SSH-over-443 `git ls-remote`：失败，`ssh.github.com:443` 超时。
- HTTPS `git ls-remote`：失败，`github.com:443` 连接失败。
- `build:desktop`：未执行，因为拉取远端最新代码失败。
- `.app` / `.dmg` 产物确认：未执行。
- `.app/Contents/MacOS/hone-desktop` 隔离 smoke：未执行。
- 渠道隔离：未启动任何本轮验证 runtime 或真实 IM sidecar。
- 2026-05-25 04:10 CST `bug-2` 复核：未运行代码测试；本次只做外部网络因素状态归类。
- 2026-05-26 04:08 CST 每日自动化复现：
  - `git status --short --branch`：工作区干净，`main...origin/main [ahead 13]`。
  - `git fetch origin`：失败，GitHub SSH 22 超时。
  - SSH-over-443 `git ls-remote`：失败，`ssh.github.com:443` 超时。
  - HTTPS `git ls-remote`：失败，`github.com:443` 连接失败。
  - `build:desktop`：未执行，因为无法拉取/确认远端最新 `main`。
  - `.app` / `.dmg` 产物确认：未执行。
  - `.app/Contents/MacOS/hone-desktop` 隔离 smoke：未执行。
  - 渠道隔离：未启动任何本轮验证 runtime 或真实 IM sidecar。
- 2026-05-26 04:12 CST `bug-2` 复核：未运行代码测试；本次只保留外部网络复现证据，缺陷状态仍为 `Later`。
