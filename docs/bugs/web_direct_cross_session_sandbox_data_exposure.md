# Bug: Web direct can read and summarize other web sessions' sandbox data

- **发现时间**: 2026-05-13 19:03 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#41](https://github.com/B-M-Capital-Research/honeclaw/issues/41)

## 证据来源

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
