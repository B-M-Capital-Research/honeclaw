- title: Chart Visualization Skill And Multi-Channel PNG Delivery
- status: in_progress
- created_at: 2026-04-17
- updated_at: 2026-04-17
- owner: codex
- related_files:
  - skills/chart_visualization/SKILL.md
  - skills/chart_visualization/scripts/render_chart.py
  - crates/hone-tools/src/skill_tool.rs
  - crates/hone-channels/src/outbound.rs
  - crates/hone-web-api/src/routes/history.rs
  - bins/hone-feishu/src/{client.rs,outbound.rs}
  - bins/hone-telegram/src/listener.rs
  - bins/hone-discord/src/utils.rs
  - packages/app/src/lib/{messages.ts,messages.test.ts}
  - tests/regression/manual/test_chart_visualization_{web,feishu,telegram,discord}.sh
- related_docs:
  - docs/current-plan.md
  - docs/current-plans/chart-visualization-skill.md
  - docs/decisions.md
  - docs/repo-map.md
- related_prs:
  - N/A

## Summary

本轮把“本地图表从 skill 生成出来后，如何穿过 Hone runtime 一直到不同前端/通道显示”落实成一条统一契约。`chart_visualization` skill 现在可以调用本地 Python 脚本渲染 PNG；`skill_tool` 会解析结构化 JSON stdout，并把校验过的图片 artifacts 暴露给模型；模型需要在最终回答里插入精确的 `file:///abs/path/to/chart.png`；Web 继续从文本里内联渲染，而 Feishu / Telegram / Discord 会把同一段文本拆成 `text -> image -> text` 序列并按顺序发送真实图片。

## What Changed

- 新增 `skills/chart_visualization/`
  - `SKILL.md` 使用当前 schema，声明 `allowed-tools`、`script`、`shell` 与 `arguments`
  - `scripts/render_chart.py` 支持 `line / area / bar / scatter / histogram / horizontal_bar`
  - 默认输出到 `${HONE_DATA_DIR}/gen_images/${HONE_SESSION_ID}/...`
- 扩展 `crates/hone-tools/src/skill_tool.rs`
  - `execute_script=true` 现在解析结构化 JSON stdout
  - 稳定暴露 `artifacts`、`render_success`、`render_summary`、`render_error` 等字段
  - 限制 artifact 必须是绝对路径，且位于 Hone 允许根目录下
  - v1 仅接受常见图片扩展名：`png / jpg / jpeg / webp / gif`
- 共享本地图片 marker 契约
  - canonical marker: `file:///abs/path/to/image.png`
  - `crates/hone-channels/src/outbound.rs` 统一拆分 response segments
  - `crates/hone-web-api/src/routes/history.rs` 会把 inline `file://` marker 也提取成历史附件
- 各端投递行为
  - Web：保留最终 assistant 文本中的 `file://` marker，由 `packages/app/src/lib/messages.ts` 内联渲染
  - Telegram：用 `send_photo` 发送本地图片
  - Discord：把本地图片作为 message attachment 发送
  - Feishu：先上传图片拿到 `image_key`，再发 `image` 消息
  - 三个外部通道都不会把原始本地 `file://` 路径发给用户；图片失败时回退成不泄露路径的文本说明
- 契约兼容性修补
  - 真实 Telegram 坏样本表明模型有时会把本地图片写成 `<a href="file:///...png">file:///...png</a>`，而不是裸 URI
  - 共享 parser、Web parser 与 history extraction 现已兼容 HTML anchor 和 Markdown link 包裹形式，不再只依赖裸 `file:///...png`
  - 后续又发现另一类真实坏样本：模型会输出 `file:///...png<br>`，即本地图片 URI 后面紧跟 HTML 标签；共享 parser 现已在 `<` 之前正确截断，不再把 `<br>` 一起吞进 URI
  - `AgentSession` 现在会把 sandbox 内生成的图片复制进稳定的 `data/gen_images/<session>/...`，并在历史恢复与 session compact 时把旧 `file://` 本地图片 marker 折叠成 `（上文包含图表）` 占位，避免模型跨轮复述失效临时路径
  - 如果 assistant 最终回答引用了已经不存在的本地图片文件，运行时会降级成 `（图表文件不可用，请重新生成）`，不再把坏掉的 `file://` 绝对路径直接展示或继续喂回模型
- 相关 finance / research skills 已显式提示在趋势、对比、分布类回答中调用 `chart_visualization`

## Verification

- 自动化：
  - `cargo test -p hone-tools`
  - `cargo test -p hone-channels`
  - `cargo test -p hone-web-api`
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-telegram`
  - `cargo test -p hone-discord`
  - `bun run test:web`
- 手工回归脚本：
  - `bash tests/regression/manual/test_chart_visualization_web.sh`
  - `bash tests/regression/manual/test_chart_visualization_feishu.sh`
  - `bash tests/regression/manual/test_chart_visualization_telegram.sh`
  - `bash tests/regression/manual/test_chart_visualization_discord.sh`
  - 针对 anchor-wrapped local image 的回归：
    - `cargo test -p hone-channels response_content_segments_`
    - `cargo test -p hone-web-api history_attachments_`
    - `bun run test:web`
  - 针对 `<br>`-wrapped / stale-marker 回归：
    - `cargo test -p hone-channels response_content_segments_extract_bare_local_images_before_html_tags -- --nocapture`
    - `cargo test -p hone-channels sanitize_assistant_context_content_redacts_local_image_markers -- --nocapture`
    - `cargo test -p hone-channels normalize_local_image_references_replaces_missing_images_with_fallback_note -- --nocapture`
    - `cargo run -q -p hone-cli -- --config config.yaml probe --channel telegram --user-id probe_chart_fix --group --scope 'chat:-1002012381143' --show-events false --query '再试一次'`

## Risks / Follow-ups

- 真实 Feishu / Telegram / Discord 账号链路尚需手工验证；当前代码只保证本地路径不会直接外泄到外部通道。
- 当前 Web 仍依赖最终 assistant 文本里的 `file://` marker 渲染图片；如果后续改成独立媒体事件，需要重新审视兼容策略。
- 运行机缺少 `matplotlib` 时，skill 会结构化失败并要求模型回退成纯文本，但不会自动安装依赖。

## Next Entry Point

- `docs/current-plans/chart-visualization-skill.md`
