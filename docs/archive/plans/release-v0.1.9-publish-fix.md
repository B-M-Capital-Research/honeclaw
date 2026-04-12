# Plan

- title: v0.1.9 Release 失败修复与补发
- status: done
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: Codex
- related_files:
  - docs/releases/v0.1.9.md
  - docs/releases/README.md
  - docs/templates/release-notes.md
  - scripts/prepare_release_notes.sh
  - .github/workflows/release.yml
  - docs/current-plan.md
- related_docs:
  - docs/archive/index.md

## Goal

修复 `Release / ensure-release` 因缺失 release notes 模板而失败的问题，并成功补发 `v0.1.9`。

## Scope

- 补齐 `docs/releases/v0.1.9.md`
- 本地验证 release notes 生成脚本
- 提交修复并重推 `v0.1.9` tag
- 观察 GitHub Actions release workflow 是否重新启动

## Validation

- `bash scripts/prepare_release_notes.sh v0.1.9 /tmp/release-notes-v0.1.9.md`
- `git status --short`
- GitHub Actions `Release / ensure-release` 新 run 通过
- GitHub Actions `Release` workflow 全部 job 成功

## Documentation Sync

- 已更新 `docs/current-plan.md`
- 已归档本计划到 `docs/archive/plans/`
- 已在 `docs/archive/index.md` 增加发布修复记录

## Risks / Open Questions

- `v0.1.9` 已经推送到远端，需要重写 tag；默认假设当前尚未被外部消费
- 风险已解除：release notes、三套产物上传与 Homebrew formula 发布均已成功
