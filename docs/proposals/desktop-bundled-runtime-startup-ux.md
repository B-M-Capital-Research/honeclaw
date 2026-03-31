# Proposal: Desktop Bundled Runtime Startup UX

日期：2026-03-29
状态：草案

## 背景

当前 Desktop bundled 模式为了保证一致性，在启动前会预检 `hone-console-page` 和已启用 channel listener 的启动锁。只要任一锁不可获取，桌面就直接报错并拒绝继续启动。

这个策略在“防止双实例并发”上是有效的，但在用户体验上存在明显问题：

- 用户只看到“检测到旧进程占用启动锁”，却不知道旧进程是否真的还活着。
- 用户被迫理解多进程架构、锁文件和清理动作，这不符合桌面产品的心智模型。
- 在频繁重启、崩溃恢复、热更新或部分子进程退出后，系统容易落入“其实可以自动修复，但当前只会阻断”的状态。

## 问题定义

这不是单纯的“锁设计是否正确”问题，而是“启动一致性策略过于保守，且把恢复责任暴露给了用户”。

用户真正想要的是：

- 点开应用就能恢复到一个一致可用的状态。
- 如果系统必须清理旧进程，应由系统自动判断并执行。
- 如果无法自动恢复，也应该给出更可理解的降级路径，而不是只展示锁文件路径。

## 目标

方案目标不是取消锁，而是把启动体验从“冲突即阻断”升级为“优先自动接管，失败时再有层次地降级”。

应满足：

1. 保留单实例 / 单 ownership 的一致性保证。
2. 允许桌面在大多数“旧实例残留”场景下自动恢复。
3. 只有在真正存在高风险冲突时才阻断用户。
4. 阻断时的提示必须面向用户语义，而不是面向实现语义。

## 推荐总策略

推荐采用三层启动策略，而不是当前的一刀切拒绝：

### 第一层：自动接管优先

Desktop 在发现 bundled runtime 锁冲突时，不要立刻报错，而是先判断“锁的持有者是否仍然健康且属于当前桌面可接管的旧实例”。

推荐判断顺序：

1. 读取锁元信息
   - pid
   - process start time
   - instance/session id
   - component name
   - lock 创建时间 / 最近心跳时间
2. 判断对应 pid 是否仍然存在
3. 若存在，再判断它是否真的健康
   - backend 是否能响应 `/api/meta`
   - channel 是否仍在向 backend 上报 heartbeat
4. 若不健康或明显已失联，则由新的 desktop 自动清理并接管

推荐默认行为：

- 旧 pid 不存在：自动删除 stale lock 并继续启动
- 旧 pid 存在但健康检查失败：先发温和终止信号，超时后强制回收，再继续启动
- 旧 pid 存在且健康：进入第二层策略，而不是直接报错

### 第二层：组件级恢复，而不是整套拒绝

当前 bundled 模式只要任一组件锁冲突就整体拒绝启动，这对用户来说过于脆弱。

更推荐的策略是：

- `hone-desktop` 自己先成功起来
- 再对 bundled runtime 做组件级 reconcile

推荐把组件分成两类：

- 核心组件：`hone-console-page`
  - 若不能恢复，桌面应降级为“界面已启动，但本地服务恢复中 / 不可用”
- 附属组件：`hone-discord`、`hone-feishu`、`hone-telegram`、`hone-imessage`
  - 某个 listener 恢复失败，不应该阻止整个桌面启动
  - 应在 channel status 中显示“恢复失败，点击重试/清理”

这意味着启动语义应从：

- “整套 bundled runtime 全成功，否则 desktop 不启动”

调整为：

- “desktop 必须先成功；runtime 由 desktop 接管并逐项恢复”

### 第三层：只有真正的活跃冲突才阻断

真正需要阻断的场景应该非常少，只保留给以下情况：

- 检测到另一套健康的 desktop-bundled 实例仍在运行，且属于不同 runtime ownership
- 当前实例无法安全判断旧进程是否可接管
- 自动清理失败，且继续启动会导致双 backend / 双 listener 并发

即使阻断，提示也应变成：

- “检测到另一套 Hone 正在运行，本次不会接管以避免重复收消息”
- 提供按钮：
  - `接管旧实例`
  - `仅打开界面（不启动本地服务）`
  - `退出`

而不是直接展示锁文件路径和让用户自己去处理。

## 关键设计点

## 1. 锁从“排它文件”升级为“ownership 记录”

当前 lock 更像“是否占用”的二值开关，但要做优雅接管，锁必须承载更多上下文。

推荐锁文件至少包含：

- `component`
- `pid`
- `started_at`
- `instance_id`
- `owner_mode`
  - `desktop_bundled`
  - `standalone`
  - `launch_sh`
- `backend_url`
- `last_heartbeat_at`

这样新的 desktop 才能判断：

- 这是我自己的旧实例
- 这是另一种模式启动的 standalone 进程
- 这是已经死掉但没清锁的残留

## 2. 用 heartbeat / health check 决定是否接管

仅仅看到 pid 存活并不等于服务真的可用。

推荐统一使用：

- `hone-console-page`
  - `/api/meta`
  - `/api/channels`
- listener
  - 最近 heartbeat 是否新鲜
  - 是否挂在当前 backend 上

判定原则：

- “锁存在 + pid 存在 + 健康检查通过” 才算真正活跃
- 其它情况都优先视为可自动接管

## 3. Desktop 需要成为 bundled runtime 的唯一 owner

从产品体验角度，bundled 模式必须只有一个 owner，就是 `hone-desktop`。

推荐：

- `hone-console-page` 和各 listener 都标注它们的 owner 是哪个 desktop `instance_id`
- 新 desktop 启动时，只要发现旧组件 owner 与自己不同，就做一次 takeover 流程
- takeover 成功后重写 ownership

这样系统语义会更清晰：

- 不是“多个进程谁抢到锁算谁”
- 而是“desktop 统一管理 bundled runtime”

## 4. 组件恢复要支持温和终止 + 超时升级

推荐标准恢复顺序：

1. 请求旧 backend 正常退出
2. 等待短超时
3. 若仍未退出，发送 `SIGTERM`
4. 再超时则 `SIGKILL`
5. 清理 lock + heartbeat + 过期 pid state
6. 启动新实例

这样既能降低误杀概率，也能覆盖卡死状态。

## 关于“固定每个渠道端口”的建议

你的直觉是对的：固定端口在用户理解上更简单，也有利于更稳定的探活和接管。

但它不能单独解决当前问题，只能作为辅助设计。

### 推荐保留固定端口的部分

- `hone-console-page` 在 bundled 模式下优先使用固定 loopback 端口
  - 例如固定主端口 + 小范围备用端口池
- 需要从 desktop 访问的内部控制面使用固定地址更有价值
  - 健康检查
  - graceful shutdown
  - ownership 查询

理由：

- 新 desktop 能更快判断“旧 backend 是否真的还活着”
- 可以避免仅靠锁文件猜状态
- 对日志、调试和恢复都更可预测

### 不建议给所有 channel 都强制分配独立固定业务端口

原因：

- Discord / Telegram / Feishu listener 本身主要依赖长连接、轮询或 SDK，不一定天然需要独立端口
- 给每个组件都做固定端口只会增加配置面和冲突面
- 真正关键的是 ownership + heartbeat，而不是每个 listener 都占一个稳定端口

更好的做法是：

- backend 固定控制面端口
- listener 继续以“由 desktop 拉起 + 向 backend 回报 heartbeat”的模式存在

## 推荐实施顺序

推荐分三期做，而不是一次性大改。

### Phase 1：启动体验止痛

目标：尽快消除“用户看到锁冲突只能手工清理”的粗糙体验。

建议内容：

- 启动时自动识别 stale lock
- 对失活 backend / listener 自动清理并继续启动
- 只在检测到健康旧实例时才弹窗
- 弹窗文案改成用户语义

这是最值得优先做的部分，风险最低，收益最大。

### Phase 2：bundled runtime ownership 收口

目标：让 desktop 真正成为 bundled runtime 唯一 owner。

建议内容：

- lock 写入 `instance_id` 和 owner mode
- backend / listener 暴露可查询 ownership
- desktop 具备正式 takeover 流程
- 启动由“预检整套锁”变成“desktop 启动后自行 reconcile runtime”

这一步会显著改善一致性和恢复能力。

### Phase 3：固定 backend 控制面端口 + 明确 remote/bundled 边界

目标：让 runtime 探测和接管更可预测。

建议内容：

- bundled backend 使用固定 loopback 控制面端口
- remote mode / standalone mode / bundled mode 的 ownership 语义彻底分开
- 若检测到 standalone backend 正在运行，desktop 提供“切到 remote”或“接管为 bundled”的明确选择

## UI / 文案建议

当前报错过于工程化。推荐改为三种用户态文案：

### 1. 自动恢复中

- “正在恢复上一次未正常关闭的 Hone 本地服务…”

### 2. 部分恢复失败

- “Hone 已启动，但部分渠道尚未恢复。你仍可继续使用桌面和 Web 控制台。”

### 3. 检测到另一套活跃实例

- “检测到另一套 Hone 本地服务仍在运行。为避免重复收消息，本次没有直接接管。”

同时给出动作按钮，而不是只给锁文件路径。

## 不推荐的方案

以下方向不建议作为主方案：

- 单纯取消启动锁
  - 会重新引入双实例并发和重复收消息问题
- 遇到冲突一律强杀，不做健康判断
  - 会误杀仍在正常运行、但只是启动顺序错位的实例
- 所有组件都绑定固定端口并用端口替代锁
  - 端口冲突比锁冲突更难解释，也不能表达 ownership

## 最终建议

最推荐的方向是：

1. 保留锁，但把锁升级为 ownership + health-aware 的恢复入口
2. desktop 先启动，再去 reconcile bundled runtime，而不是启动前整套拒绝
3. 优先自动接管 stale / unhealthy 旧进程
4. backend 使用固定控制面端口辅助探活与接管
5. listener 不强制独立固定端口，继续依赖 heartbeat 回报

这样既能维持一致性，也能把用户体验从“自己处理锁文件”提升到“应用自己恢复”。

## 后续实现建议

如果进入实现，建议拆成三个独立任务：

- Task A：stale lock 自动清理 + 启动文案优化
- Task B：desktop-bundled ownership / takeover 协议
- Task C：bundled backend 固定控制面端口与 remote/bundled 语义收口
