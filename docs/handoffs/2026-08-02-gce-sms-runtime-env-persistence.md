# GCE 短信运行环境持久化修复交接

- title: GCE 短信运行环境持久化修复交接
- status: done
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - .env.example
  - scripts/check_backend_runtime_env.sh
  - tests/regression/ci/test_backend_runtime_env_contract.sh
  - docs/runbooks/backend-deployment.md
- related_docs:
  - docs/archive/plans/gce-sms-runtime-env-persistence.md
  - docs/handoffs/2026-08-02-public-admin-usage-analytics.md
- related_prs: none

## Summary

生产手机号验证码接口会先返回防枚举的通用成功文案，再异步调用阿里云。管理员重新登录未收到短信时，生产 journal 在北京时间 22:37:40 明确记录异步发送失败：GCE Web systemd 环境缺少阿里云 AccessKey ID。账号、管理员角色、唯一有效会话、手机号格式和应用限流均不是原因。

本次将本机忽略环境中已有的 canonical `ALIBABA_CLOUD_*` 凭证安全同步进托管后端的持久运行环境，并新增仓库级校验器、CI 回归和 systemd 启动前门禁。以后环境缺失、空值或占位值会在服务启动前失败，不再出现“服务健康但第一次真实登录才发现短信不可用”。

## What Changed

- `.env.example` 补齐短信认证所需的两个 canonical 变量，仅保留空模板。
- `scripts/check_backend_runtime_env.sh` 不执行/输出环境文件，只解析支持的三组变量名并拒绝缺失、空值、常见占位符；成功输出只有命中的变量名。
- 新增 CI-safe 回归覆盖缺文件、空文件、单边凭证、占位符、canonical、兼容别名和混合别名，且验证输出不泄露 secret。
- `docs/runbooks/backend-deployment.md` 增加每次启动前校验、`root:root 0600`、原子更新、systemd `ExecStartPre` 和异步短信 canary 契约。
- 托管后端持久环境已原子更新；root-owned 校验器安装到稳定系统路径，Web systemd drop-in 在每次启动前强制检查精确环境文件。

## Verification

- 定向环境契约回归通过；完整 `bash tests/regression/run_ci.sh` 全部通过。
- 本机真实忽略 `.env` 通过校验，输出只包含 canonical 变量名，不包含值。
- 远端无效环境探针被拒绝，真实 `/etc/hone/runtime.env` 通过；文件为 `root:root 0600`，校验器 `root:root 0755`，drop-in `root:root 0644`。
- 两次 active-chat 检查均为 0；重启后 Web 第 2 次探测就绪，Feishu active，运行 revision 保持 `39ce9ce54f5cbfea26e664459cb70edf3fd97292`。
- `/api/meta` 确认 PostgreSQL/R2 健康、云存储权威且本地 durable dependency 为 0；systemd journal 记录 `ExecStartPre` 校验成功。
- 北京时间 22:52:47 经公网对指定管理员手机号发出一次 canary，HTTP 返回通用 accepted；随后异步窗口无 `SMS verification send failed after generic acceptance`，Web/Feishu 仍 active、会话数为 0。
- 两端临时凭证副本与 staging/backup 均已删除；OS Login 2FA 恢复为 `TRUE`，临时 gcloud 配置删除。

## Risks / Follow-ups

- 当前成功证据证明应用完整链路拿到了阿里云 `Code=OK && Success=true`，因为任何配置、传输或 provider 非 OK 都会写同一 warning；它不等同于运营商最终送达回执。若需要独立证明终端送达，应由收件人确认本次短信，或后续配置阿里云短信状态报告接收。
- HTTP 通用成功是防止枚举白名单的既有安全行为，不能改为同步返回 provider 错误。运维验收必须结合指定号码和后台日志。
- systemd 门禁只保护托管后端的下一次启动；新增其它短信后端宿主时，必须安装同一校验器/drop-in 或等价 supervisor 前置检查。

## Next Entry Point

环境变量集合或托管启动方式有变化时，先更新 `scripts/check_backend_runtime_env.sh` 与对应回归，再按 `docs/runbooks/backend-deployment.md` 做 staging 校验、原子安装、零会话重启和单号码 canary。不要把真实 AccessKey 写入仓库、命令行或交接文档。
