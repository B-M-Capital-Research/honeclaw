# Bug: Desktop runtime logs 接口曾因坏日志数据或 runtime overlay 漏读而失效，日志面板无法稳定恢复最近运行痕迹

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 直接修复提交: `d031f16 fix: harden desktop runtime log recovery`
  - 本次修复验证:
    - `crates/hone-core/src/config.rs:87-94`
    - `crates/hone-core/src/config.rs:1230-1260`
    - `crates/hone-web-api/src/routes/logs.rs:85-129`
    - `crates/hone-web-api/src/routes/logs.rs:159-202`
    - `crates/hone-web-api/src/routes/logs.rs:234-302`

## 端到端链路

1. Desktop 用户在 bundled runtime 里切换 runner、重启后端或排查故障时，会通过日志面板读取 `/api/logs` 返回的最近运行日志。
2. 旧实现从 runtime 日志文件读取内容时直接使用 `std::fs::read_to_string(...)`，并在解析纯文本日志行时按字节切片固定前 24 个字符。
3. 当 runtime 日志中混入无效 UTF-8、中文等多字节文本，或进程内日志缓冲区 mutex 已被 poison 时，这条日志恢复链路会在读取或拼装日志时直接 panic / 返回空。
4. 同时，`HoneConfig::from_file(...)` 之前没有合并 runtime overlay，导致日志接口在恢复 runtime 目录时可能按基础配置而不是当前生效配置去找日志文件。
5. 最终用户在最需要排障的时候，看到的结果会是日志面板空白、接口不稳定，或切换 runner 后无法从日志里确认当前 runtime 是否真的已恢复。

## 期望效果

- `/api/logs` 应在坏日志内容、poisoned buffer、混合编码文本存在时继续返回尽可能多的可解析日志，而不是把整个接口弄崩。
- Desktop runtime 日志恢复应基于当前生效配置，包括 runtime overlay，而不是只看基础 `config.yaml`。
- 即使日志内容部分异常，用户仍应能从日志面板看到最近的 runner 恢复、启动和报错痕迹。

## 当前实现效果（问题修复前）

- `crates/hone-web-api/src/routes/logs.rs` 旧实现直接 `read_to_string(...)` 读取日志文件，遇到无效 UTF-8 会整文件丢弃。
- 同文件旧实现按字节切片 `cleaned[..24]` / `cleaned[24..]` 解析时间戳，面对多字节纯文本日志存在 panic 风险。
- 旧版 `handle_logs(...)` 直接 `lock().unwrap()` 读取进程内日志缓冲；只要 buffer 被 poison，就可能把整个 `/api/logs` 路径一起拖垮。
- `HoneConfig::from_file(...)` 旧实现调用的是 `read_yaml_value(...)`，不会把 runtime overlay 合并进来，导致日志恢复路径可能落在过时目录。

## 当前实现效果（2026-04-15 HEAD 复核）

- `crates/hone-core/src/config.rs:87-94` 已改为通过 `read_merged_yaml_value(...)` 读取配置，`test_from_file_applies_runtime_overlay` 也覆盖了 runtime overlay 生效场景。
- `crates/hone-web-api/src/routes/logs.rs:122-129` 已改为按字节读取后使用 `String::from_utf8_lossy(...)`，坏 UTF-8 不再让整份日志直接失效。
- `crates/hone-web-api/src/routes/logs.rs:85-99` 已改为按字符而不是按字节切割时间戳与正文，多字节纯文本日志不会再因为切片越界把接口弄崩。
- `crates/hone-web-api/src/routes/logs.rs:159-202` 已把 buffer snapshot 和 runtime 文件收集都包进 `catch_unwind(...)`，并在 poisoned mutex 场景下回退到可恢复快照。

## 用户影响

- 该问题直接影响 Desktop 最核心的排障入口之一：当 bundled runtime 重启异常、runner 切换失效或日志格式混杂时，用户可能拿不到任何可用日志。
- 在“切换后到底有没有生效”“为什么刚才崩了”这类场景下，日志面板失效会显著放大故障定位成本。
- 由于问题发生在恢复链路而不是主业务链路，用户常见感知是“明明有问题，但日志页什么都看不到”。

## 根因判断

- 日志恢复链路对运行时脏数据过于乐观，默认假设日志文件总是合法 UTF-8、日志行总能按固定字节宽度切片、缓冲区 mutex 不会 poison。
- 配置读取链路与运行时生效语义不一致，`from_file(...)` 没把 runtime overlay 当成真相源的一部分，导致日志恢复有机会看错目录。
- 缺少针对坏 UTF-8、poisoned mutex、多字节纯文本日志和 runtime overlay 的回归测试，使这类恢复缺口长期未被自动化拦住。

## 下一步建议

- 后续 desktop 排障相关接口继续沿用“部分数据坏了也不要拖垮整个接口”的恢复策略，尤其是日志、会话快照和 runtime 状态读取。
- 如果后面再出现 release `.app` runtime 与 CLI runtime 路径分叉问题，优先复用同一套 merged config / runtime overlay 读取逻辑，避免再次出现“接口读到的是旧路径”。
- 该缺陷源码层已修复；后续可在真实 bundled app 场景补一次手工验证，确认日志面板能稳定显示 runtime 重启后的最新日志。
