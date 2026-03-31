# Session SQLite Shadow Backfill

## 目的

在不改动现网 session 读写路径的前提下，把 `data/sessions/*.json` 增量镜像到独立 SQLite，便于：

- 检查 session / message 数据是否完整
- 为后续真正切换到 SQLite 提前做数据基座和对账
- 在线上先低风险演练迁移，不影响当前 JSON 运行时逻辑

当前这套脚本是“影子库”方案：

- 运行时仍然只读写 `data/sessions/*.json`
- SQLite 只是额外镜像，不参与线上请求

当前 Rust 运行时已经支持两种模式：

- 配置项：`storage.session_sqlite_shadow_write_enabled`
- 目标库：`storage.session_sqlite_db_path`
- 运行时后端：`storage.session_runtime_backend`

语义：

- `session_runtime_backend: "json"`：JSON 主读，SQLite 可选 shadow write
- `session_runtime_backend: "sqlite"`：SQLite 主读，JSON 继续双写作为回退镜像

## 默认路径

- 源目录：`./data/sessions`
- 影子库：`./data/sessions.sqlite3`

## 迁移脚本

脚本：`scripts/migrate_sessions_to_sqlite.py`

特性：

- 默认 dry-run
- `--write` 后才真正写库
- 可重复运行
- 以文件内容 `sha256` 判断是否变化
- 未变化的 session 会跳过
- 新增或变更的 session 会整 session 原子重灌
- 不会改写源 JSON 文件

### 先看计划

```bash
python3 scripts/migrate_sessions_to_sqlite.py
```

### 实际写入

```bash
python3 scripts/migrate_sessions_to_sqlite.py --write
```

## 运行时影子写入开关

如果要让线上运行时在写 JSON 的同时自动镜像到 SQLite，可在运行时配置中打开：

```yaml
storage:
  session_sqlite_db_path: "./data/sessions.sqlite3"
  session_sqlite_shadow_write_enabled: true
```

注意：

- 这不会切换 runtime 读路径
- SQLite 写入失败只会记日志，不会阻断 JSON 主写
- 真正切 runtime 前，仍需继续完成 Web/API 读路径改造和对账验证

## 切换到 SQLite runtime

当对账和回归都通过后，可把运行时切到 SQLite：

```yaml
storage:
  session_sqlite_db_path: "./data/sessions.sqlite3"
  session_sqlite_shadow_write_enabled: true
  session_runtime_backend: "sqlite"
```

建议顺序：

1. 先跑一次最新 backfill
2. 再切 `session_runtime_backend: "sqlite"`
3. 保持 JSON 双写一段时间
4. 观察 `/api/users`、`/api/history` 和渠道新消息读写是否稳定

### 指定路径

```bash
python3 scripts/migrate_sessions_to_sqlite.py \
  --sessions-dir ./data/sessions \
  --db-path ./data/sessions.sqlite3 \
  --write
```

## 查看脚本

脚本：`scripts/diagnose_session_sqlite.py`

### 看总览

```bash
python3 scripts/diagnose_session_sqlite.py summary
```

### 看最近 session

```bash
python3 scripts/diagnose_session_sqlite.py sessions --limit 20
```

### 按渠道筛

```bash
python3 scripts/diagnose_session_sqlite.py sessions --channel feishu --limit 20
```

### 看某个 session 的消息

```bash
python3 scripts/diagnose_session_sqlite.py \
  messages \
  --session-id 'Actor_feishu__direct__alice'
```

### 输出 JSON

```bash
python3 scripts/diagnose_session_sqlite.py --json summary
```

## 幂等与增量语义

- 如果某个 JSON 文件内容没有变化，脚本会直接 `SKIP`
- 如果某个 JSON 文件新增了消息或其它字段变化，脚本只会重灌这一个 session
- 如果连续运行两次且源数据没变化，第二次应全部 `SKIP`

## 风险控制

- 脚本会先稳定读取文件；如果读取过程中源文件发生变化，会报错并跳过该文件
- 默认不做“缺失文件反向删库”，避免误删历史镜像
- 当前影子库不是线上真相源，异常不会影响现网服务

## 推荐操作顺序

1. 先 dry-run 看 summary
2. 再 `--write` 建库
3. 用 `diagnose_session_sqlite.py` 抽查 session 和消息
4. 后续按需重复执行，作为增量灌库任务
