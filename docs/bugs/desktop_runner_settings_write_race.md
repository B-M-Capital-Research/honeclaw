# Bug: Desktop 设置页多入口保存共用同一份配置文件但缺少串行写保护，可能造成 runner 配置被并发保存静默覆盖

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 2026-04-15 当前源码复核
  - 代码证据:
    - `crates/hone-core/src/config.rs:940-980`
    - `bins/hone-desktop/src/sidecar.rs:845-981`
    - `bins/hone-desktop/src/sidecar.rs:1285-1452`
    - `packages/app/src/pages/settings.tsx:332-344`
    - `packages/app/src/pages/settings.tsx:996-1077`

## 端到端链路

1. Desktop 设置页上存在多个彼此独立的保存入口：Agent Settings、FMP、Tavily、以及其它运行时设置。
2. 这些保存命令都会修改同一份 canonical `config.yaml` / runtime effective config。
3. 底层 `apply_config_mutations(...)` 会先整文件读入 YAML、在内存中修改、再整文件原子写回；但这一过程本身没有跨命令的文件级串行保护。
4. Desktop sidecar 中 `set_agent_settings_impl(...)`、`set_openrouter_settings_impl(...)`、`set_fmp_settings_impl(...)`、`set_tavily_settings_impl(...)` 都直接调用 `apply_setting_updates(...)`，没有在写配置前使用统一互斥锁把“读-改-写”包起来。
5. 一旦用户在设置页快速连续保存多个区块，或者桌面里存在多个并发设置动作，后一个保存很可能基于较旧的文件快照重写配置，把前一个保存结果静默冲掉。

## 期望效果

- 所有写入同一份 Desktop runtime 配置的命令，应共享一个串行化写锁，保证每次保存都基于最新配置快照进行合并。
- runner 设置不应因为同时保存 FMP / Tavily / OpenRouter 等其它设置而被回滚或部分丢失。
- 用户连续保存多个区块后，最终配置应等于这些修改的合并结果，而不是“最后一次写谁赢”。

## 当前实现效果（问题发现时）

- `apply_config_mutations(...)` 在 `crates/hone-core/src/config.rs:940-980` 中是典型的“读当前文件 -> 修改内存结构 -> 原子写回”流程，但不自带跨调用锁。
- `set_agent_settings_impl(...)` 在 `bins/hone-desktop/src/sidecar.rs:845-981` 中直接写 runner 相关字段。
- `set_openrouter_settings_impl(...)`、`set_fmp_settings_impl(...)`、`set_tavily_settings_impl(...)` 在 `bins/hone-desktop/src/sidecar.rs:1285-1452` 中同样直接写同一份配置文件。
- 这些命令虽然会在后续 bundled backend 重启时使用 `transition_lock` 串行化 runtime 切换，但“配置文件读改写”本身并没有纳入同一把锁的保护范围。
- 前端页面本身也存在多个独立保存入口，例如 Agent Settings 提交位于 `packages/app/src/pages/settings.tsx:332-344`，FMP / Tavily 保存位于 `packages/app/src/pages/settings.tsx:996-1077`；因此用户完全可能在短时间内连续触发多个写命令。

## 当前实现效果（2026-04-15 HEAD 复核）

- 当前 `HEAD` 仍保留 `crates/hone-core/src/config.rs:940-980` 的整文件“读 -> 改 -> 原子写回”模式，没有新增跨调用共享的配置写锁。
- `bins/hone-desktop/src/sidecar.rs` 中的 `set_agent_settings_impl(...)`、`set_openrouter_settings_impl(...)`、`set_fmp_settings_impl(...)`、`set_tavily_settings_impl(...)` 仍分别直写同一份配置文件，未见统一串行化入口。
- 本轮巡检未发现把配置写入阶段纳入 `transition_lock` 或其它互斥机制的提交，因此该缺陷继续保持 `New`。

## 用户影响

- 用户可能先保存了 runner / 模型设置，随后再保存搜索或数据源 key，结果前一次 runner 改动被后一次写配置静默冲掉。
- 这类故障表现通常不是直接报错，而是“某一块设置莫名恢复旧值”或“保存 A 后 B 生效了，但 A 不见了”，非常难排查。
- 对依赖设置页集中配置运行时的 desktop 用户来说，这是配置一致性层面的严重风险。

## 根因判断

- runtime 配置写入的竞争面被拆散在多个 sidecar 命令中，但没有统一的配置写锁。
- `transition_lock` 只覆盖 backend 连接/重启阶段，没有覆盖配置文件的读改写阶段，因此不能防止 lost update。
- 底层配置写入是整文件重写，只要两个保存请求拿到不同时间点的快照，就天然存在最后写入覆盖前一写入的风险。

## 下一步建议

- 为所有会修改 desktop runtime 配置的 sidecar 命令补统一的配置写锁，把“读-改-写 canonical config + 写 effective config”纳入同一临界区。
- runner 设置相关排查不应只关注单次保存成功，还要验证与 FMP / Tavily / OpenRouter 等保存动作交错时是否会丢配置。
- 若短期内无法重构，应至少补一条并发回归测试，证明两个设置命令交错执行时最终配置仍能保留双方改动。

## 修复情况（2026-04-16）

- `bins/hone-desktop/src/sidecar.rs` 已新增共享的 `config_write_lock`
- 以下会修改 desktop runtime 配置的保存链路现在都会先拿这把锁，再执行 canonical/effective config 的“读-改-写”：
  - `set_agent_settings_impl(...)`
  - `set_channel_settings_impl(...)`
  - `set_openrouter_settings_impl(...)`
  - `set_fmp_settings_impl(...)`
  - `set_tavily_settings_impl(...)`
- 这次修复没有改变原有的 backend `transition_lock` 职责；配置写锁只负责串行化配置文件更新，bundled backend 重连仍走原来的 transition 串行化链路
- 新增并发回归测试：
  - `sidecar::tests::config_write_lock_serializes_concurrent_calls`
  - `sidecar::tests::config_write_lock_preserves_updates_from_concurrent_saves`
- 验证命令：
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop config_write_lock_ -- --nocapture`
