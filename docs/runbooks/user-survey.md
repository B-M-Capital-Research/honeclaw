# 用户调研问卷 — 运维与部署

面向接手部署的人：这份文档说明**要不要跑迁移**、**数据落在哪**、**怎么取数**、
以及**上线前必须确认的一件事**。

---

## 1. 数据库迁移：不需要手工执行

`survey_responses` 表写在 `CloudPgRuntime::ensure_schema()` 的 `batch_execute`
里，全部是 `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF NOT EXISTS`，
**服务启动时自动建表，幂等**。

- 位置：`crates/hone-core/src/cloud_runtime.rs`，紧邻 `community_spaces` 之前。
- 部署动作：**没有额外动作**。正常发布、服务起来即完成建表。
- 不需要往 `cloud_schema_migrations` 里补版本号 —— 该表只服务于少数需要
  数据回填的历史迁移，纯建表不走它。

表结构：

```sql
CREATE TABLE IF NOT EXISTS survey_responses (
  response_id   BIGSERIAL PRIMARY KEY,
  survey_id     TEXT NOT NULL,
  locale        TEXT NOT NULL DEFAULT 'zh',
  answers       JSONB NOT NULL DEFAULT '{}'::jsonb,
  contact       TEXT,
  client_digest TEXT,          -- 盐化摘要，不是 IP
  submitted_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_survey_responses_recent
  ON survey_responses(survey_id, submitted_at DESC, response_id DESC);
```

**若要提前手工建表**（比如想在发布前先建好），直接把上面这段跑一遍即可，
与 `ensure_schema` 完全一致，重复执行无副作用。

### 回滚

表是新增的，旧版本代码不读它，所以**回滚不需要删表**。真要清理：

```sql
DROP TABLE IF EXISTS survey_responses;
```

---

## 2. 上线前必须确认的一件事：`HONE_SURVEY_DIGEST_SALT`

问卷无登录，靠 `client_digest` 判断"同一台设备今天是不是已经提交过"。
这个摘要 = `sha256(salt + ":" + 客户端标识)`。

盐的来源，按优先级：

1. 环境变量 `HONE_SURVEY_DIGEST_SALT`（**多副本部署必须设置**）
2. 落在 `<sandbox_base_dir>/survey/.digest-salt` 的自动生成值（单机可用）

**为什么多副本必须设**：每个副本会各自生成一个不同的盐，同一个人打到不同副本
会算出不同摘要，24 小时去重就形同虚设。设置方式：

```bash
HONE_SURVEY_DIGEST_SALT="$(openssl rand -hex 32)"
```

一旦设定**不要再改**。改盐等于把历史摘要全部作废，去重窗口重新开始
（不影响已回收的答案本身）。

> 盐必须保密。无盐的 IP 哈希不构成匿名化 —— 整个 IPv4 空间几秒钟就能枚举完。

---

## 3. 数据落在哪

| 模式 | 位置 |
|---|---|
| cloud（配了 PG） | `survey_responses` 表 |
| local（没配 PG） | `<sandbox_base_dir>/survey/survey_<survey_id>.jsonl`，逐行追加 |

`sandbox_base_dir` 受 `HONE_AGENT_SANDBOX_DIR` 控制。**local 模式的 JSONL 要纳入
备份**，否则重装即丢。

---

## 4. 接口

### 提交（公开，无鉴权）

```
POST /api/public/survey
{
  "locale": "zh",
  "answers": { "q1": "weekly", "q2": ["fundamentals","sector"], "q11": "……" },
  "contact": "可选"
}
```

限流三层，任何一层触发都返回 `429`：

| 层 | 规则 | 说明 |
|---|---|---|
| 冷却 | 同 IP 20 秒一次 | 挡连点 |
| 进程内 | 同 IP 每小时 20 次 | 重启清零、不跨副本 |
| 落库 | 同摘要 24 小时 3 次 | **真正兜底的一层**，跨副本有效 |

落库计数失败时（例如 PG 抖动）会**放行并记 warn**，不会因为计数不了就把一个
正常用户的回答丢掉。

### 读取（管理员）

```
GET /api/public/admin/survey?limit=1000&survey_id=hone-user-research-2026-08
```

需要已登录且 `is_web_admin`。返回：

```jsonc
{
  "survey_id": "...",
  "total": 128,
  "summary": {
    "choice_counts":     { "q2": { "fundamentals": 61, "sector": 44 } },
    "text_answer_counts":{ "q11": 87 }          // 开放题只计数，不做词频
  },
  "responses": [ { "response_id", "locale", "answers", "contact", "submitted_at" } ]
}
```

`client_digest` **不在返回结构里**，`SurveyResponse` 根本没有这个字段
（`memory/src/survey.rs` 有专门的测试锁住这一点），所以任何读取路径都不可能
把它带到前端。

导出 CSV 目前没做，需要的话直接对 `responses` 处理。

---

## 5. 换一版问卷

改 `memory/src/survey.rs` 的 `ACTIVE_SURVEY_ID`（当前 `hone-user-research-2026-08`），
同时改前端 `CONTENT.survey.questions`。旧数据按旧 `survey_id` 留在同一张表里，
统计时天然隔离，不会混。

题库只在前端定义（`packages/app/src/lib/public-content.ts` 的 `survey` 块，
中英各一份）。后端只做结构校验，不认识任何选项文本，**改题不需要动后端和数据库**。

后端的结构上限（`crates/hone-web-api/src/routes/public_survey.rs`）：

- 最多 40 题；题目 key 只允许 `[A-Za-z0-9_-]`，≤48 字符
- 单题最多 20 个选项，每个选项 ≤120 字符
- 开放题 ≤2000 字符（**超长截断而不是拒绝**，避免一段认真写的长回答整条丢失）
- 联系方式 ≤120 字符
- 单选/开放题为字符串，多选为字符串数组；其它类型一律拒绝

---

## 6. 隐私

- 不需要登录，不写 cookie
- **不存原始 IP，不存 User-Agent**，只存盐化摘要
- 联系方式完全可选，页面明说"只有你愿意被回访时才填"
- 页面上的承诺（`CONTENT.survey.privacy`）与实现一致：改实现时必须同步改文案

---

## 7. 入口

- 独立页：`/survey`
- 首页入口：`.survey-home-card`，位于「Plan 预告」之前
- 中英文跟随全站 locale，随语言切换即时生效

## 8. 验收

```bash
cargo test -p hone-core -p hone-memory -p hone-web-api
cd packages/app && bunx tsc --noEmit && bun test src/lib/survey-model.test.ts
```
