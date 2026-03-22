# Large Files Physical Split Refactor

## Goal

把几个长期膨胀的大文件拆成“薄 façade + 兄弟模块”，保持行为、配置 schema、wire protocol、外部导出路径不变。

## Scope

- `bins/hone-feishu/src/main.rs`
- `bins/hone-feishu/src/{card,handler,listener,markdown,types}.rs`
- `bins/hone-telegram/src/main.rs`
- `bins/hone-telegram/src/{handler,listener,markdown_v2,types}.rs`
- `bins/hone-desktop/src/main.rs`
- `bins/hone-desktop/src/{commands,sidecar,tray}.rs`
- `crates/hone-core/src/config.rs`
- `crates/hone-core/src/config/{agent,channels,server}.rs`
- `crates/hone-channels/src/attachments.rs`
- `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`

## Progress

- Feishu：已拆分
- Telegram：已拆分
- Desktop：已拆分
- Config：已拆分
- Attachments：已拆分
- Docs：进行中
- Verification：进行中
- Handoff：待写

## Validation

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `bash tests/regression/run_ci.sh`

## Notes

- `crates/hone-channels/src/agent_session.rs` 不在本轮范围内。
- 完成后需要补一份 handoff，方便下次接手快速定位模块边界变化和验证结果。
