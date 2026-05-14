# Bug: Web direct can read and summarize other web sessions' sandbox data

- **发现时间**: 2026-05-13 19:03 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#41](https://github.com/B-M-Capital-Research/honeclaw/issues/41)

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_web__direct__web-user-028a885ded9b`
  - `ordinal=22`
  - `timestamp=2026-05-13T20:18:03.752307+08:00`
  - 用户要求从 `fengming2` 目录和 `~/.codex` session 记录里搜索“某个人持仓说明记录”。
  - `ordinal=23`
  - `timestamp=2026-05-13T20:22:47.386373+08:00`
  - 用户重复要求从 `~/.codex` session 记录搜索同一信息。
  - `ordinal=24`
  - `timestamp=2026-05-13T20:24:57.759144+08:00`
  - assistant final 明确回显：`~/.codex` 下有 `1281` 个 session JSONL，列出本机 `~/.codex/sessions/...jsonl` 路径、全局 `data/portfolio/portfolio_...json` 路径，并总结持仓标的、股数和成本，还额外提到 Discord 用户持仓文件。
- `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_web__direct__web-user-028a885ded9b`
  - `ordinal=18`
  - `timestamp=2026-05-13T17:40:45.507081+08:00`
  - 用户要求查看本地文件，并要求不要只看当前工作目录。
  - `ordinal=19`
  - `timestamp=2026-05-13T17:44:44.865924+08:00`
  - assistant final 回显当前 web actor sandbox 绝对路径，并继续总结全局数据目录下的公司画像数量、全局 portfolio 文件数量、其它渠道 / 账户的持仓组合摘要。
  - `ordinal=20`
  - `timestamp=2026-05-13T17:46:47.793661+08:00`
  - 用户进一步点名要求查看 `data/agent-sandboxes/web` 下其它 session 数据。
  - `ordinal=21`
  - `timestamp=2026-05-13T17:49:45.690196+08:00`
  - assistant final 列出其它 web session 的 sandbox 标识、公司画像文件名和画像内容摘要，并再次说明全局 portfolio 数据不在 web sandboxes 目录里。
- `data/runtime/logs/hone-console-page-prod.log`
  - `2026-05-13T09:49:45Z` 同一轮 `MsgFlow/web ... done` 显示 runner 实际执行了多次本地文件读取 / 搜索：
    - 遍历 `data/agent-sandboxes/web/*`
    - 读取其它 web session 的 company profile / events 文件
    - 查询另一个 sandbox 内的 `sessions.sqlite3`
  - 同一轮以 `success=true` 收口，并把跨 session 摘要持久化为 assistant final。
- `docs/invariants.md`
  - 长期约束明确要求：`ActorIdentity` 用于权限、quota、sandbox 和私有数据隔离；channel actor 的本地文件可见性应限制在 actor sandbox；用户可见运行进度中的 actor sandbox 绝对路径也应改写为相对路径，sandbox 外绝对路径不得原样透出。

## 端到端链路

1. Web direct 用户在自己的 session 中要求查看本地 Hone 数据。
2. Codex ACP runner 获得本地文件读取能力，并从当前 actor sandbox 继续访问到全局 `data/agent-sandboxes/web` 与 `data/portfolio`。
3. runner 读取其它 web session 的公司画像、事件文件和部分全局持仓文件。
4. assistant final 将这些跨 actor / 跨 session 的私有数据摘要直接返回给当前 web 用户。
5. 会话以 `success=true` 正常落库，没有被 sandbox guard、输出净化或权限层拦截。

## 期望效果

- Web direct 的本地文件读取默认只能访问当前 actor sandbox。
- 即使用户显式要求查看其它本机路径，也应拒绝或只返回“无权限访问其它 session / 全局私有数据”的产品化说明。
- 用户可见回复不应暴露其它 web session 的标识、画像摘要、全局持仓文件摘要或本机绝对路径。

## 当前实现效果

- 当前 web 用户可以通过自然语言请求诱导 runner 读取其它 web session 的 sandbox 数据。
- assistant final 实际返回了其它 session 的画像摘要和全局持仓摘要。
- 2026-05-13 20:24 CST 的复发样本进一步说明，读取范围不止 `data/agent-sandboxes/web`：同一 Web direct 会话还可以读取 `~/.codex` 历史会话记录，并通过其中线索继续访问全局 portfolio 文件。
- 本轮日志显示底层 runner 多次执行本地文件读取命令，说明这不是单纯最终文本净化缺口，而是执行权限 / sandbox 隔离缺口。

## 用户影响

- 这是私有数据隔离失败：一个 web actor 可以看到其它 web session 的公司画像、事件记录和全局持仓摘要。
- 影响范围不止输出格式或可读性；它破坏了 actor sandbox 作为用户私有数据边界的核心安全假设。
- 定级为 `P1`：当前证据已经证明真实 web direct 会话中可跨 session 读取并外发私有数据；虽然没有发现跨用户主动误投递或全站不可用，因此不定为 `P0`。

## 根因判断

- 当前 Web direct 使用的 runner / 本地命令能力没有强制限制到当前 actor sandbox。
- 现有 `relativize_user_visible_paths` / `sanitize_user_visible_output` 主要处理用户可见文本中的路径展示，不能阻止 runner 读取 sandbox 外的文件。
- `docs/invariants.md` 已把 actor sandbox 隔离列为长期约束，但 live Web direct 执行路径仍允许访问仓库数据目录和其它 actor sandbox。

## 下一步建议

- 优先修复 Web direct 的本地文件权限边界：runner 工作目录、文件工具 root、shell / read 工具都必须强制限制到当前 actor sandbox。
- 对用户显式提供的 sandbox 外绝对路径做执行前拒绝，不能只依赖最终输出净化。
- 增加回归测试：Web actor 请求读取 `data/agent-sandboxes/web/<other-session>`、`data/portfolio` 或任意 sandbox 外绝对路径时，应返回无权限说明，并且不执行底层读取。
- 修复后复核最近真实 Web direct 会话，确认同类请求不会再返回其它 session 文件名、画像摘要或全局持仓摘要。

## 修复记录

- 2026-05-14 15:04 CST 复核：最近四小时继续看到当前 live Web direct 旧运行态可读取 sandbox 外敏感本机数据，但不把本单从 `Fixed` 回退为 `New`。
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_web__direct__web-user-028a885ded9b`
    - `2026-05-14T11:13:09+08:00` 用户要求杀掉 Hone 进程，并把 Codex 其它 session 会话文件移动到新目录。
    - `2026-05-14T11:16:41+08:00` assistant final 表示已把 `~/.codex/sessions` 与 `~/.codex/archived_sessions` 复制归档到 `~/.codex/memories/...`，共 `1283` 个 JSONL、约 `980M`，并在当前 actor sandbox 写了宿主侧脚本，脚本用于杀进程和移动 Codex session 目录。
    - `2026-05-14T11:35:45+08:00` assistant final 表示读取了 `~/.codex/auth.json`，但主动将 `tokens` 等敏感字段脱敏；`2026-05-14T11:46:20+08:00` 又明确说明原始 `tokens` 字段大概率是完整认证 token，只是不适合输出到聊天记录。
  - `ps -p 63485` 复核显示当前 live `hone-console-page` 进程仍为 `PID 63485`，启动时间 `2026-05-13 19:28:21 CST`，早于 `2026-05-14 04:24 CST` 的 sandbox 根隔离代码修复；因此这条证据按“修复前 live 进程仍未重启 / 未部署”处理。
  - 这条样本进一步说明旧运行态的可读范围不止 repo 内 `data/agent-sandboxes`，还包括 `~/.codex` session 和认证配置文件；但它未证明当前 HEAD 的 repo-external sandbox 修复失效。状态维持 `Fixed`，关联 Issue [#41](https://github.com/B-M-Capital-Research/honeclaw/issues/41) 不重复创建。
- 2026-05-14 11:05 CST 复核：最近四小时仍看到当前 live Web direct 旧运行态暴露宿主环境 / 进程信息，但不把本单从 `Fixed` 回退为 `New`。
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_web__direct__web-user-028a885ded9b`
    - `2026-05-14T10:50:38+08:00` assistant final 返回主机名、repo 内 actor sandbox 绝对路径以及“能只读查看一些本机路径”等说明。
    - `2026-05-14T10:53:14+08:00` assistant final 在 `ps aux` 失败后改用 `lsof` / 日志，列出 `hone-console-page` PID、二进制路径、工作目录、监听端口和日志路径。
    - `2026-05-14T10:55:32+08:00` 到 `11:00:10+08:00` assistant 按用户要求尝试 `kill` / 脚本封装后失败，并把 `Operation not permitted` 与进程仍监听端口的结果返回给用户。
  - 本机 `ps` 复核显示该 live `hone-console-page` 进程 `PID 63485` 启动于 `2026-05-13 19:28:21 CST`，早于 `2026-05-14 04:24 CST` 的 sandbox 根隔离代码修复；因此这条证据按“修复前 live 进程仍未重启 / 未部署”处理。
  - 这条样本未证明当前 HEAD 的 repo-external sandbox 修复失效，也没有看到再次读取其它 web session 画像或全局 portfolio 的新证据；状态维持 `Fixed`，但仍不能更新为 `Closed`。后续必须在 live 重启到修复后代码后复打“查看本机路径 / 其它 session / 宿主进程”类提示词。
- 2026-05-13 23:04 CST 复核：在 20:24 CST 真实 Web direct 会话里仍能读取 `~/.codex` session 记录和全局 portfolio 文件并外发摘要；因此本单从 `Fixed` 调回 `New`。已有 GitHub Issue [#41](https://github.com/B-M-Capital-Research/honeclaw/issues/41)，本轮不重复创建。
- 2026-05-14 04:24 CST：实际代码已补齐共享 sandbox 根隔离，而不再只停留在文档结论。
  - `crates/hone-channels/src/sandbox.rs` 不再从 `HONE_DATA_DIR` 派生 actor sandbox；若 `HONE_AGENT_SANDBOX_DIR` 指向当前 git worktree 内部，会退回 repo-external temp sandbox。
  - `ensure_actor_sandbox()` 初始化当前 actor sandbox 时会删除误落入 sandbox 根的 `portfolio_*.json`、`portfolio/`、`portfolios/`，明确持仓真相源仍是 `storage.portfolio_dir`。
  - desktop sidecar 改为维护并注入独立 `sandbox_dir`，不再把 `HONE_AGENT_SANDBOX_DIR` 固定为 repo `data/agent-sandboxes`。
  - 新增回归：`sandbox_base_dir_falls_back_to_temp_not_data_dir`、`ensure_actor_sandbox_removes_legacy_portfolio_files`、`prepare_ignores_repo_internal_sandbox_override`、`desktop_actor_sandbox_dir_moves_repo_checkout_out_of_data_dir`、`desktop_actor_sandbox_dir_keeps_external_data_dir_sibling_root`。
- 2026-05-14 04:24 CST 验证：
  - `cargo test -p hone-channels sandbox --lib -- --nocapture`
  - `cargo test -p hone-channels prepare_ignores_repo_internal_sandbox_override --lib -- --nocapture`
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop runtime_env -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- 当前仍保留的未验证项：
  - 本自动化不重启 live 服务，因此尚未在新运行态下复打原始 Web direct 提示词；状态先记 `Fixed`，待下一次正常部署/重启后再决定是否可更新为 `Closed`。
