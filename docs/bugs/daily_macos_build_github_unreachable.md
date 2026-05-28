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
7. 2026-05-27 04:06 CST 每日自动化再次复现同一阻塞：工作区干净，当前 `HEAD` 与本地 upstream tracking ref 同为 `d67275276a15049d238ea528ec08b29328145e68`，但无法 fetch 远端确认最新 `main`。
8. 2026-05-29 04:06 CST 每日自动化再次复现默认 SSH remote 阻塞：工作区干净，当前 `HEAD=f2d3204b0587b57b65d4e42a19f51ceab2bafe31`、本地 upstream tracking ref 为 `ccf20a21adb99cd00eb3519d786eb3286072fcd7`，默认 SSH fetch 无法确认最新 `main`；随后发现系统代理 `127.0.0.1:1082` 可通过 HTTPS 读取 GitHub，并用 HTTPS fetch/rebase 恢复到远端 `51bad5b21bc8c8318597c03c5eaa2993a82f96f2` 后继续打包验证。

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
- 2026-05-27 04:06 CST 复现：
  - `GIT_SSH_COMMAND='ssh -o ConnectTimeout=20' git fetch origin`：`ssh: connect to host github.com port 22: Operation timed out`
  - `GIT_SSH_COMMAND='ssh -o ConnectTimeout=20 -p 443' git fetch ssh://git@ssh.github.com/B-M-Capital-Research/honeclaw.git`：`ssh: connect to host ssh.github.com port 443: Operation timed out`
  - `git ls-remote https://github.com/B-M-Capital-Research/honeclaw.git HEAD`：`Failed to connect to github.com port 443 after 75003 ms`
  - 本机当前 shell 未配置 `HTTP_PROXY` / `HTTPS_PROXY` / `ALL_PROXY`，且 `gh` CLI 不存在，未发现可用的备用认证传输路径。
- 2026-05-29 04:06 CST 复现：
  - `git fetch origin && git pull --rebase origin main`：`ssh: connect to host github.com port 22: Operation timed out`
  - SSH-over-443 `git ls-remote`：`ssh: connect to host ssh.github.com port 443: No route to host`
  - HTTPS `git ls-remote`：`Failed to connect to github.com port 443 after 75003 ms`
  - 当前 shell 未配置 `HTTP_PROXY` / `HTTPS_PROXY` / `ALL_PROXY`，远端仍为 `git@github.com:B-M-Capital-Research/honeclaw.git`。
  - `scutil --proxy` 显示系统 HTTP/HTTPS 代理为 `127.0.0.1:1082`，显式设置 `http.proxy=http://127.0.0.1:1082` 后，HTTPS `ls-remote` 返回远端 `HEAD=51bad5b21bc8c8318597c03c5eaa2993a82f96f2`。
- 2026-05-29 本轮最终已通过系统代理走 HTTPS fetch/rebase 到远端最新后继续完成打包与启动 smoke；该缺陷只记录默认 SSH remote / 未继承系统代理的拉取风险。

## 用户影响

- 本轮每日 macOS 完整打包验证未闭环，不能证明最新远端代码可以打出 `.app` 与 `.dmg`。
- 现有本地代码和历史产物未被修改；没有启动真实 Feishu / Telegram / Discord / iMessage 渠道。

## 根因判断

- 当前证据更像本机到 GitHub 的网络连通性或出口策略问题，而不是仓库代码问题。
- 因为 SSH 22、SSH 443 和 HTTPS 443 均失败，本轮暂按 P3 外部依赖 / 网络阻塞记录。
- 2026-05-25 04:10 CST `bug-2` 复核：该缺陷当前没有可本地代码修复的通用根因；按缺陷修复规则，不为单次 GitHub 网络不可达写特殊兼容逻辑，状态从 `New` 调整为 `Later`，待同机网络恢复后重跑每日 macOS 打包验证。
- 2026-05-26 04:12 CST `bug-2` 复核：04:08 CST 的复现仍集中在当前机器到 GitHub 的 SSH 22、SSH-over-443 与 HTTPS 443 连通性，属于外部网络/出口阻塞；不为该问题写代码特判，状态保持 `Later`，待同机网络恢复后重跑每日 macOS 打包验证。
- 2026-05-27 04:06 CST 复现仍集中在当前机器到 GitHub 的 SSH 22、SSH-over-443 与 HTTPS 443 连通性，未进入构建或 runtime 阶段；状态保持 `Later`，等待同机网络/代理/传输方式恢复。
- 2026-05-29 04:06 CST 复现集中在当前 shell 默认 SSH remote 与未继承系统代理的 HTTPS 连通性；显式使用系统代理后可恢复 HTTPS fetch。因此状态保持 `Later`，下一步应让每日自动化默认使用可用传输方式或修复 SSH 出口策略。

## 下一步建议

1. 在同一机器上复查 GitHub 网络连通性、VPN / 代理 / 防火墙策略。
2. 若默认 SSH remote 继续不可达，让每日自动化显式使用系统 HTTP(S) 代理的 HTTPS fetch/push，或把 remote 改为可用传输方式。
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
  - 本轮缺陷文档已本地提交到当前 `main`，但 `git push origin main` 因同一 `github.com:22` 超时失败；按规则再次 `git fetch origin` 仍失败，无法完成 rebase 后重试。
- 2026-05-26 04:12 CST `bug-2` 复核：未运行代码测试；本次只保留外部网络复现证据，缺陷状态仍为 `Later`。
- 2026-05-27 04:06 CST 每日自动化复现：
  - `git status --short`：工作区干净。
  - `git rev-parse HEAD` 与 `git rev-parse @{u}`：均为 `d67275276a15049d238ea528ec08b29328145e68`，但 upstream ref 未刷新，不能证明远端无新增提交。
  - `git fetch origin`：失败，GitHub SSH 22 超时。
  - SSH-over-443 fetch：失败，`ssh.github.com:443` 超时。
  - HTTPS `ls-remote`：失败，`github.com:443` 连接失败。
  - `build:desktop`：未执行，因为无法拉取/确认远端最新 `main`。
  - `.app` / `.dmg` 产物确认：未执行。
  - `.app/Contents/MacOS/hone-desktop` 隔离 smoke：未执行。
  - 渠道隔离：未启动任何本轮验证 runtime 或真实 IM sidecar。
- 2026-05-29 04:06 CST 每日自动化复现：
  - `git status --short`：工作区干净。
  - `git rev-parse HEAD`：`f2d3204b0587b57b65d4e42a19f51ceab2bafe31`；`git rev-parse @{u}`：`ccf20a21adb99cd00eb3519d786eb3286072fcd7`，但 upstream ref 未刷新，不能证明远端最新 `main`。
  - `git fetch origin && git pull --rebase origin main`：失败，GitHub SSH 22 超时。
  - SSH-over-443 `git ls-remote`：失败，`ssh.github.com:443` 无路由。
  - HTTPS `ls-remote`：失败，`github.com:443` 连接失败。
  - 系统代理检查：`scutil --proxy` 显示 HTTP/HTTPS proxy 为 `127.0.0.1:1082`；`git -c http.proxy=http://127.0.0.1:1082 fetch https://github.com/B-M-Capital-Research/honeclaw.git +refs/heads/main:refs/remotes/origin/main` 成功，`origin/main=51bad5b21bc8c8318597c03c5eaa2993a82f96f2`。
  - `build:desktop`：通过，生成 `.app` 与 `.dmg`。
  - `.app`：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`，mtime `2026-05-29 04:24:59 CST`。
  - `.dmg`：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/Hone Financial_0.12.4_aarch64.dmg`，mtime `2026-05-29 04:25 CST`。
  - `.app/Contents/MacOS/hone-desktop` 隔离 smoke：通过，`HONE_DESKTOP_SMOKE_SERVER=1` 下 `/api/meta` 返回 `version=0.12.4`，`http://127.0.0.1:18088/` 返回 `200 text/html`。
  - 渠道隔离：`/api/channels` 显示 web running，iMessage / Discord / Feishu / Telegram 均 `enabled=false`、`status=disabled`；未启动真实 IM sidecar。
