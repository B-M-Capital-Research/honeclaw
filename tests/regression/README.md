# Regression Tests

这个目录分成两类脚本：

- `ci/`：CI-safe 回归脚本
- `manual/`：依赖外部 CLI、账号或本机状态的手工回归脚本

## 运行方式

- `bash tests/regression/run_ci.sh`
- `bash tests/regression/run_manual.sh`
- `bash tests/regression/manual/test_<topic>.sh`

## 约定

- CI-safe 脚本必须无交互、可重复、可判定，失败时返回非 0
- 手工脚本可以依赖外部账号或本机环境，但不要把它们当成默认 CI 门禁
- 新增脚本时优先选择明确的主题命名，例如 `test_secret_scan.sh`
