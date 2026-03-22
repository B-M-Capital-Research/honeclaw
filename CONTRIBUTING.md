# 贡献指南

感谢你愿意参与这个项目。

## 开始前

- 安装 Rust 1.87 或更高版本
- 安装 Bun 1.3 或更高版本
- 先复制 `config.example.yaml` 再创建本地配置
- 不要提交任何真实 token、secret、cookie、session、SQLite 数据库或运行时日志

## 建议流程

1. 基于主干创建分支
2. 完成目标变更
3. 跑完必要验证
4. 提交 PR 并说明影响面

## 推荐验证

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `bash tests/regression/run_ci.sh`

如果变更影响桌面端、渠道运行时或外部集成，再补充对应的手工回归脚本。

## 提交建议

- 优先使用 Conventional Commits 风格的提交信息
- 单次 PR 只聚焦一类问题
- 涉及配置文件时，优先更新 `config.example.yaml` 和文档说明

## 报告问题

- 文档问题和一般缺陷可以直接提 Issue
- 涉及安全问题、密钥泄露或权限绕过，请优先走 `SECURITY.md` 里的私下披露流程
