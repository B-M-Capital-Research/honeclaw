# Release Checklist

## 目标

本仓库的正式 release 不是“本地打个 tag 就算完”。完整链路是：

1. 版本号与 release note 落库
2. 必要验证通过
3. release commit 推到 `main`
4. annotated `v*` tag 推到远端
5. GitHub Actions `release.yml` 被 tag 触发，生成 tarball、上传 release assets、发布 Homebrew formula

## 先检查什么

1. 当前工作区是否有用户已有改动；不要覆盖或回滚不属于本次 release 的内容。
2. 当前分支是否落后 `origin/main`。
3. 最近真实 tag 是什么：

```bash
git tag --sort=-version:refname | head -n 5
```

4. 自上次 release 到现在有哪些 commit 和未发布工作：

```bash
git log --oneline <last-tag>..HEAD
git log --oneline HEAD..origin/main
git diff --stat
```

## 版本号位置

正式发版至少同步这些位置：

- `Cargo.toml`
- `Cargo.lock`
- `bins/hone-desktop/tauri.conf.json`

常见做法：

1. 先改 `Cargo.toml` 与 `bins/hone-desktop/tauri.conf.json`
2. 再跑一次会刷新 lock 的命令，例如：

```bash
cargo check --workspace --all-targets --exclude hone-desktop
```

3. 确认 `Cargo.lock` 里的 workspace package version 已跟上

## Release Note 规则

先读：

- `docs/releases/README.md`
- `docs/templates/release-notes.md`

必须创建：

- `docs/releases/vX.Y.Z.md`

要求：

- 中文在前，英文在后
- 不写内部流水账
- 要覆盖“用户应该知道什么”
- compare 链接要指向上一个 tag 到当前 tag

本地可用下面命令做一次模板校验：

```bash
bash scripts/prepare_release_notes.sh vX.Y.Z /tmp/release-notes-vX.Y.Z.md
```

如果这个脚本失败，先补齐 `docs/releases/vX.Y.Z.md`，不要继续打 tag。

## 推荐验证基线

至少跑：

```bash
cargo check --workspace --all-targets --exclude hone-desktop
bash scripts/prepare_release_notes.sh vX.Y.Z /tmp/release-notes-vX.Y.Z.md
```

再根据本次改动面追加定向测试，例如：

```bash
cargo test -p hone-tools -p hone-channels
cargo test -p hone-telegram
bash tests/regression/manual/test_<topic>.sh
```

如果 release 吸收了文档治理、skill、runtime、desktop、channel 等跨模块变化，优先补对应模块的 targeted tests，而不是盲跑全仓所有重测试。

## 提交与推送顺序

1. 先审计暂存区：

```bash
git diff --staged
```

2. 再提交。不要用 `--no-verify`。
3. 如果本地落后远端，先 rebase：

```bash
git pull --rebase origin main
```

4. rebase 后如 commit 发生变化，至少重跑一轮快速验证。
5. 推 `main`：

```bash
git push origin main
```

6. 创建 annotated tag：

```bash
git tag -a vX.Y.Z -m "vX.Y.Z"
```

7. 推 tag：

```bash
git push origin vX.Y.Z
```

只有 tag 推送成功，GitHub release workflow 才会真正启动。

## Hook 与失败处理

### pre-push rustfmt / gitleaks

- 本仓库 `.githooks/pre-push` 会先检查即将推送的 Rust 变更是否 `rustfmt --check` 干净，再跑 `gitleaks`
- 失败时先修，再重新提交或重新推
- 不要用 `--no-verify`

### non-fast-forward

- 先 `fetch` / `pull --rebase`
- 确认没有把别人的新 commit 覆盖掉
- 再重推

### 远端 release 还没完成

tag 推上去只代表 workflow 被触发，不代表 tarball / Homebrew formula 已全部发布。汇报时应说明：

- `main` 是否已推送
- `vX.Y.Z` 是否已推送
- workflow 是否已触发
- 如未检查 GitHub Actions 结果，要明确写“尚未在本地确认远端 workflow 最终成功”

## 文档同步提醒

如果这次 release 合并了正在跟踪的活跃任务，别忘了同步：

- `docs/current-plan.md`
- `docs/current-plans/*.md`
- `docs/handoffs/*.md`
- `docs/archive/index.md`

不需要时也应明确说明为什么不更新，避免“代码发了但上下文资产没同步”。
