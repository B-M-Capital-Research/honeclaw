# Session SQLite Migration Plan

状态：提案  
更新时间：2026-03-25  
范围：会话持久化、消息生命周期一致性、Web/API 读取一致性、历史数据迁移

补充进展（2026-03-26）：

- 已有独立影子库 backfill / diagnose 脚本
- Rust 运行时已接入 `json | sqlite` session backend 切换能力
- 本机运行时已切到 SQLite 主读，JSON 继续双写作回退镜像

## 1. 背景与问题定义

当前会话数据存储在 `data/sessions/*.json`，核心实现位于 `memory/src/session.rs`。这套方案在单进程、小规模读写时足够简单，但随着多渠道消息接入、Web 页面展示、桌面内嵌后端、计划任务和多进程并发增多，JSON 明盘已经成为一致性问题的主要来源。

当前已知症状：

- 同一个 session 的读写缺少跨进程事务语义，消息到达、页面展示、摘要压缩、重放恢复之间容易出现时序不一致。
- `SessionStorage` 已经是抽象入口，但 Web API 仍有直接扫描 `sessions_dir` 读取 JSON 的逻辑，导致“写路径走抽象，读路径绕过抽象”，页面展示不一定等于运行时真实状态。
- JSON 文件的整文件读改写天然存在 last-writer-wins 风险；当前 `SESSION_LOCKS` 只解决单进程互斥，不能覆盖多进程。
- 会话生命周期状态分散在消息数组、`summary`、`metadata`、`runtime.prompt.frozen_time_beijing` 等字段内，没有数据库级约束，页面列表、历史接口、消息压缩后的最终状态难以保证原子一致。
- 后续如果要做可靠去重、增量拉取、后台恢复、跨端排障或审计，JSON 文件结构的查询能力和校验能力都不够。

目标不是“把 JSON 换个壳”，而是把 session 变成一套有明确真相源、事务边界、迁移路径和回滚策略的持久化系统。

## 2. 现状梳理

### 2.1 当前真相源与调用路径

- 写路径主入口：`memory/src/session.rs`
- 共享装配：`crates/hone-channels/src/core.rs`
- 历史消息接口：`crates/hone-web-api/src/routes/history.rs`
- 用户 / 会话列表接口：`crates/hone-web-api/src/routes/users.rs`
- 运行时目录初始化：`crates/hone-web-api/src/runtime.rs`

当前 `Session` 主要字段：

- `id`
- `actor`
- `session_identity`
- `created_at`
- `updated_at`
- `messages`
- `metadata`
- `runtime.prompt.frozen_time_beijing`
- `summary`

当前能力与问题并存：

- 优点：结构直观、易于调试、无数据库依赖。
- 缺点：整文件覆写、跨进程无锁、查询靠全量扫描、接口层存在绕过抽象直读文件、摘要替换与消息追加难以原子化。

### 2.2 已有 SQLite 能力

仓库已经有成熟的 SQLite 持久化样板：`memory/src/llm_audit.rs`。其中已经采用了适合本地应用场景的配置：

- `WAL`
- `synchronous = NORMAL`
- `busy_timeout`
- 初始化 schema 与索引
- 只读打开能力

这说明把 session 迁移到 SQLite 不需要从零搭基础设施，重点在于：

- 定义正确的 schema
- 保持 `SessionStorage` 抽象平滑演进
- 把所有读写统一到同一真相源

## 3. 迁移目标

本次迁移的目标如下：

1. 让 SQLite 成为 session 的唯一真相源。
2. 保证消息写入、摘要压缩、metadata 更新、页面读取遵循同一套事务边界。
3. 消除 Web/API 对 `data/sessions/*.json` 的直接扫描依赖。
4. 支持历史 JSON 数据无损迁移，保证用户无感。
5. 支持渐进式切换、对账、回滚，避免一次性全量切换引入新问题。
6. 为后续消息去重、增量同步、审计排障、后台恢复提供稳定基础。

非目标：

- 本方案不改变渠道协议、不改变消息渲染协议、不改变前端页面结构。
- 本方案不要求把所有其它存储立刻迁移到 SQLite。
- 本方案不在第一阶段引入全文检索、复杂分析查询或分布式数据库。

## 4. 目标架构

### 4.1 真相源原则

迁移完成后，session 相关所有读写都只经过 `SessionStorage` 抽象，再由其落到 SQLite。`data/sessions/*.json` 只在迁移期作为历史输入或回滚兜底，不再作为运行时真相源。

### 4.2 推荐组件结构

- 保留 `memory/src/session.rs` 作为对外抽象入口，但内部改为 SQLite 实现。
- 将当前 JSON 文件实现下沉为兼容层，例如 `JsonSessionStorage` 或迁移工具专用 reader。
- 新增 SQLite 实现，例如 `SqliteSessionStorage`，负责 schema、事务、查询和迁移辅助方法。
- `hone-channels`、`hone-web-api`、各渠道 handler 继续依赖统一的 `SessionStorage` API，不直接感知底层文件或数据库。

### 4.3 单一读取路径

必须改掉下面这类旁路读取：

- `crates/hone-web-api/src/routes/users.rs` 当前直接扫描 `sessions_dir`

迁移后 Web API 应只通过 `SessionStorage` 提供的查询接口取数据，例如：

- `list_sessions(...)`
- `get_session_preview(...)`
- `get_messages(...)`
- `load_session(...)`

这样可以保证页面顶部列表、历史页、消息详情、摘要状态都基于同一份数据库快照。

## 5. 目标数据模型

推荐拆分为“会话主表 + 消息明细表 + metadata 表”，避免把整个 session 继续塞成一整块 JSON。

### 5.1 `sessions` 主表

建议字段：

- `session_id TEXT PRIMARY KEY`
- `version INTEGER NOT NULL`
- `actor_channel TEXT NULL`
- `actor_user_id TEXT NULL`
- `session_channel TEXT NULL`
- `session_target TEXT NULL`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`
- `message_count INTEGER NOT NULL`
- `last_message_at TEXT NULL`
- `last_message_preview TEXT NULL`
- `summary_json TEXT NULL`
- `frozen_time_beijing TEXT NULL`
- `title TEXT NULL`
- `deleted_at TEXT NULL`

说明：

- `actor_*` 与 `session_*` 必须继续分开，保持现有不变量。
- `summary_json` 继续承载当前 `SessionSummary`，但其更新必须和消息压缩在同一事务内。
- `message_count`、`last_message_at`、`last_message_preview` 作为列表页冗余字段，避免每次扫整段消息。

### 5.2 `session_messages` 表

建议字段：

- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `session_id TEXT NOT NULL`
- `ordinal INTEGER NOT NULL`
- `message_id TEXT NULL`
- `role TEXT NOT NULL`
- `content_json TEXT NOT NULL`
- `name TEXT NULL`
- `tool_call_id TEXT NULL`
- `created_at TEXT NOT NULL`
- `source_channel TEXT NULL`
- `source_event_id TEXT NULL`
- `dedupe_key TEXT NULL`
- `metadata_json TEXT NULL`
- `is_summary_replacement INTEGER NOT NULL DEFAULT 0`

索引建议：

- `UNIQUE(session_id, ordinal)`
- `INDEX(session_id, created_at)`
- `INDEX(session_id, dedupe_key)`
- `INDEX(source_channel, source_event_id)`

说明：

- `ordinal` 是 session 内的稳定顺序，页面展示和历史拉取都按它排序。
- `message_id` 可选，用于未来对接更细粒度消息定位。
- `dedupe_key` 为后续消息去重预留，不必在首期全部打通，但 schema 应提前留位。
- `content_json` 保持与现有 `SessionMessage` 可逆映射，避免第一阶段就做协议变形。

### 5.3 `session_metadata` 表

建议字段：

- `session_id TEXT NOT NULL`
- `key TEXT NOT NULL`
- `value_json TEXT NOT NULL`
- `updated_at TEXT NOT NULL`
- `PRIMARY KEY(session_id, key)`

说明：

- 当前 `metadata: HashMap<String, Value>` 可拆到独立表，避免频繁整对象重写。
- 若实现复杂度受限，第一阶段也可先把 metadata 继续放回 `sessions.metadata_json`，但长期建议拆表。

### 5.4 `session_events` 可选表

如果希望把“生命周期事件”和“用户可见消息”分离，建议预留事件表：

- `event_id TEXT PRIMARY KEY`
- `session_id TEXT NOT NULL`
- `event_type TEXT NOT NULL`
- `payload_json TEXT NOT NULL`
- `created_at TEXT NOT NULL`

首期不是必须，但它适合记录：

- session 创建
- prompt state 冻结
- 摘要压缩
- 外部消息入站
- 回复生成完成

如果首期不做，也应在方案里明确：后续若需要审计级生命周期追踪，可在不破坏主 schema 的前提下增量引入。

## 6. 事务与一致性设计

### 6.1 原子写入原则

以下操作必须使用单事务完成：

1. 创建 session 与写入第一条消息
2. 追加消息并更新 `sessions.updated_at / message_count / last_message_*`
3. `replace_messages` 与 `summary` 更新
4. `replace_messages_with_summary`
5. metadata 更新与影响列表展示字段的同步

### 6.2 并发控制

推荐策略：

- SQLite 开启 `WAL`
- 每次写操作采用 `BEGIN IMMEDIATE`
- 保留应用层按 `session_id` 的轻量互斥，减少热点 session 的竞争重试
- 但最终正确性以数据库事务为准，不再依赖进程内 `SESSION_LOCKS`

原因：

- 进程内锁只能限制本进程
- 事务锁可以覆盖 Web、desktop sidecar、channel worker 等跨进程并发

### 6.3 生命周期不变量

迁移后必须继续保证以下不变量：

1. `ActorIdentity` 与 `SessionIdentity` 语义分离，不能混用。
2. `frozen_time_beijing` 一旦为某个 session 生成，后续不得漂移。
3. `summary` 是显式字段，不允许靠消息内容隐式推断。
4. 历史消息展示顺序以 `ordinal` 为准，不能依赖文件数组顺序或文件修改时间。
5. 页面列表、历史拉取、压缩替换必须能在同一存储快照下得到一致结果。

## 7. API 与页面一致性方案

### 7.1 后端接口层改造原则

所有页面依赖的 session 数据都必须通过统一查询接口获取，禁止继续直接扫描 JSON 文件。

需要重点替换的读路径：

- `/api/users`：从“扫描目录 + 反序列化 JSON”切到 SQLite 查询
- `/api/history`：继续通过 `SessionStorage`，但底层改为 SQLite

推荐新增的查询能力：

- `list_recent_sessions(limit, cursor, filters)`
- `list_sessions_by_actor(actor)`
- `load_session_header(session_id)`
- `list_session_messages(session_id, limit, before_ordinal)`
- `get_session_summary(session_id)`

### 7.2 页面展示一致性要求

前端不应自己拼状态真相。页面层应只消费后端返回的统一视图：

- session 列表显示的标题、最后一条消息、更新时间、消息数
- 详情页显示的消息序列
- 若存在摘要压缩，前端应由后端返回“当前可展示消息集合 + summary”，而不是让前端猜测历史是否被替换

### 7.3 缓存策略

本地页面缓存只能作为性能优化，不能成为真相源。推荐：

- API 响应带 `updated_at` 或 `last_ordinal`
- 前端基于版本戳决定是否刷新
- 不再依赖文件扫描顺序或文件 mtime

## 8. 数据迁移方案

### 8.1 总体原则

迁移必须满足：

- 无损
- 可重入
- 可校验
- 可回滚
- 对最终用户无感

### 8.2 迁移输入

迁移输入来源：

- `data/sessions/*.json`

每个 JSON 文件都按当前 `Session` 结构反序列化，转换为：

- 1 行 `sessions`
- N 行 `session_messages`
- 0..N 行 `session_metadata`

### 8.3 迁移工具设计

建议新增独立迁移命令，例如：

- `hone-cli migrate sessions-to-sqlite`

或内部管理命令：

- `hone-cli sessions backfill-sqlite`

工具行为建议：

1. 扫描所有 JSON 文件
2. 逐个解析为内存 `Session`
3. 在单 session 事务中 upsert 到 SQLite
4. 为每个 session 生成校验摘要
5. 输出迁移报告

### 8.4 幂等策略

迁移工具必须可重复执行。推荐规则：

- 以 `session_id` 为主键 upsert
- 若 SQLite 中 session 不存在，则插入
- 若已存在，则比较校验摘要或 `updated_at`
- 默认只在“JSON 比 SQLite 更新”时覆盖
- 提供 `--force-rebuild-session <id>` 能力，便于单 session 修复

### 8.5 消息顺序迁移

消息顺序必须稳定映射：

- JSON `messages[0]` -> `ordinal = 0`
- JSON `messages[1]` -> `ordinal = 1`
- 以此类推

不要在迁移时按 `created_at` 重排，因为历史数据可能存在相同时间戳、缺失时间戳或后补写入。

### 8.6 脏数据处理

迁移工具需要识别并分级处理：

- 反序列化失败
- 缺失 `session_id`
- 消息内容格式异常
- `actor` / `session_identity` 不完整
- `updated_at < created_at`

处理策略：

- 能迁移的尽量迁移
- 不能安全自动修复的会话进入 quarantine 报告
- 不允许静默跳过

建议输出：

- 总 session 数
- 成功数
- 失败数
- quarantine 清单
- 统计校验结果

## 9. 灰度切换与兼容策略

不建议直接“一刀切停 JSON”。推荐四阶段迁移。

在进入真正的双写 / 切读之前，建议先增加一个更保守的预备阶段：

- 先用独立脚本把 `data/sessions/*.json` 镜像到影子 SQLite
- 这个影子库不参与线上读写
- 影子库阶段必须保留源 `session_id`、源文件边界和原始 JSON 语义
- 不应在这个阶段擅自把历史 `Session_*` / `Actor_*` / `User_*` 标识重新规范化，否则会把“数据迁移”变成“数据纠错”，放大风险

### 阶段 0：引入 SQLite schema 与读写抽象

- 新增 SQLite session storage 实现
- 保持现有 JSON 为生产真相源
- 提供 backfill 工具
- 补齐对账脚本

退出条件：

- schema 稳定
- 能完成全量 backfill
- 对账通过

### 阶段 1：双写，读仍以 JSON 为准

- 新消息同时写 JSON 和 SQLite
- 页面与运行时仍读取 JSON
- 开始积累线上对账样本

目标：

- 先验证 SQLite 写入稳定性，不让读路径一次性切换

补充建议：

- 双写前，先让影子库跑一段时间，用它验证历史数据形态、脏数据分布与增量灌入语义
- 影子库阶段若发现异常，应优先修脚本与数据报告，不要直接修改线上运行时代码

### 阶段 2：双写，读切到 SQLite

- `/api/users`、`/api/history`、运行时读取都切到 SQLite
- JSON 继续保留写入作为回滚兜底
- 增加“SQLite 与 JSON session 校验差异”报警

退出条件：

- 若干天无差异或差异都可解释并修复
- 页面和渠道行为稳定

### 阶段 3：SQLite 单写，JSON 停止实时写入

- SQLite 成为唯一运行时真相源
- JSON 改为停写
- 仅保留导出 / 回滚工具

### 阶段 4：归档 JSON

- 将 `data/sessions` 迁移为只读归档
- 设置删除窗口，例如 30 天或 60 天后手动清理

## 10. 校验方案

### 10.1 迁移前校验

在切换前先做基线统计：

- JSON session 总数
- 总消息数
- 最近 7 天活跃 session 数
- 读取失败文件数

### 10.2 迁移后对账

对每个 session 生成 parity 校验：

- `session_id`
- `created_at`
- `updated_at`
- `message_count`
- `last_message_preview`
- `summary` 是否存在
- `frozen_time_beijing`
- 消息内容哈希

推荐定义 session 级 hash：

- header hash：主表关键字段
- messages hash：按 `ordinal` 拼接后哈希

### 10.3 切换期运行时校验

在双写阶段，后台定时抽样比对：

- 新增 session 是否双端都存在
- 新增消息数量是否一致
- 历史接口返回数量是否一致
- 列表页预览字段是否一致

### 10.4 回归验证

切换必须覆盖以下用例：

1. 新建 session
2. 多轮消息追加
3. 摘要压缩后继续对话
4. Web 列表页刷新
5. Web 历史页分页
6. 多渠道并发写入不同 session
7. 同一 session 高频追加
8. 进程重启后恢复读取

建议新增：

- Rust 单元测试：SQLite session storage CRUD / replace / summary
- Rust 集成测试：Web API 读取一致性
- 手工回归脚本：双写对账、迁移演练

## 11. 回滚方案

回滚不能依赖人工即兴操作，必须提前定义。

### 11.1 阶段 1 / 2 回滚

如果 SQLite 读路径上线后有问题：

- 配置开关切回 JSON 读取
- 保留双写，避免切回期间丢增量数据
- 用 SQLite -> JSON 导出或 JSON 继续主写兜底

### 11.2 阶段 3 回滚

如果已经停掉 JSON 实时写入，再回滚会更复杂，因此需要：

- 在停写前确保具备 SQLite -> JSON 的导出工具
- 对关键活跃 session 做额外备份
- 保留近若干天的 SQLite 文件快照

### 11.3 数据库损坏兜底

本地 SQLite 虽然稳定，但仍需准备：

- WAL + busy timeout
- 启动时 `PRAGMA integrity_check`
- 定期备份
- 数据损坏时从最近快照恢复并重放增量

## 12. 配置与目录策略

当前配置只有：

- `storage.sessions_dir`
- `storage.llm_audit_db_path`

建议新增独立配置：

- `storage.session_db_path`

默认值建议：

- `./data/sessions.sqlite3`

兼容策略：

- 保留 `storage.sessions_dir` 一段时间，用于迁移输入、归档或导出目录
- 在最终切换完成前，不立即删除该配置项
- 文档中明确其角色从“运行时真相源”转为“迁移期兼容目录”

## 13. 风险与控制措施

### 风险 1：直接切读 SQLite 后页面字段不全

原因：

- 当前 `/api/users` 是文件扫描拼装字段，切换后若 SQL 查询没补齐预览字段，页面会退化。

控制：

- 在 `sessions` 主表冗余 `message_count / last_message_preview / last_message_at`
- 切换前做接口响应对比

### 风险 2：摘要压缩后历史顺序出错

原因：

- `replace_messages_with_summary` 语义复杂，如果迁移后仍按简单 delete + insert，但事务边界不正确，页面可能瞬间看到不完整历史。

控制：

- 将“删除旧消息 + 插入新消息 + 更新 summary + 更新 header”放入单事务

### 风险 3：双写期间 JSON 与 SQLite 漂移

原因：

- 一边成功一边失败

控制：

- 每次双写记录结果
- 对失败侧告警
- 后台定时对账
- 保留单 session 重放工具

### 风险 4：历史脏数据导致迁移中断

控制：

- quarantine 机制
- 分批迁移
- 迁移报告
- 不因单个坏文件阻断全量迁移

## 14. 推荐实施顺序

建议按以下顺序落地：

1. 设计并冻结 SQLite schema
2. 为 `SessionStorage` 抽象补齐列表和查询接口，消除 Web API 对 JSON 的旁路读取依赖
3. 实现 SQLite storage
4. 实现 JSON -> SQLite backfill 工具
5. 跑一次全量迁移演练和对账
6. 上线双写
7. 切换读路径到 SQLite
8. 稳定观察后停止 JSON 实时写入
9. 完成 JSON 归档

## 15. 建议的交付拆分

为了降低风险，建议拆成以下若干 PR：

### PR 1：抽象补齐

- 为 `SessionStorage` 增加列表 / 预览 / 分页查询接口
- Web API 不再直接扫 `sessions_dir`

### PR 2：SQLite session storage

- 新增 schema、CRUD、事务和测试

### PR 3：迁移工具与对账工具

- JSON backfill
- parity 校验
- quarantine 报告

### PR 4：双写与灰度开关

- 配置开关
- 运行时指标
- 回滚路径

### PR 5：SQLite 读路径切换

- Web/API/运行时统一改读 SQLite

### PR 6：停写 JSON 与归档

- 清理旧逻辑
- 更新文档与 runbook

## 16. 决策建议

建议采纳以下核心决策：

1. SQLite 成为 session 的唯一真相源，JSON 不再承担运行时读写职责。
2. 迁移以 `SessionStorage` 抽象为边界推进，先消灭旁路读取，再切换底层实现。
3. 采用“先双写、后切读、再停写”的渐进式迁移，而不是一次性替换。
4. 历史迁移必须提供可重入 backfill、差异对账和 quarantine 机制。
5. 页面一致性问题优先通过统一后端查询模型解决，而不是在前端追加补丁。

## 17. 本方案对应的后续文档更新建议

本次仅输出迁移方案，不改代码。真正实施时还应同步更新：

- `docs/repo-map.md`
- `docs/invariants.md`
- `config.example.yaml`
- 必要时补一份 ADR，记录“session 真相源从 JSON 切换到 SQLite”的长期决策
