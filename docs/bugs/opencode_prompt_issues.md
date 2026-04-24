# opencode ACP 相关的 Prompt 泄露与缓存失效问题分析

- **状态**: 2026-03-25 已完成两项止血修复：拦截 `### System Instructions ###` 前缀回显；将动态 Session 时间信息移出 system prompt 前缀。ACP 仍保持每轮 `session/new` 的隔离策略，因此“完整多轮 message 级缓存复用”仍未实现。

## 问题一：System Instructions 泄露 (Prompt Echo)

**现象**
在特定情况下（如用户发送特定的问题触发前缀回显），飞书 Bot 会回复完整的系统设定文本（包含 `### System Instructions ###`，以及诸如领域边界约束、强制中文等内部提示词规则），而不是对用户问题的正常回复。现象严重时，会让最终用户直接看到后台架构和约束明细。

**根因**
1. **Prompt 扁平化拼接**：在 `crates/hone-channels/src/runners/acp_common.rs` 的 `build_acp_prompt_text` 函数中，业务的 `system_prompt` 和真实的用户输入 `runtime_input` 被强行拼接成了单一段落纯文本（由 `### System Instructions ###` 和 `### User Input ###` 分割）。然后这段超长文本被当做用户的首轮输入，通过 `session/prompt` 发给了 opencode ACP。
2. **绝对缺乏外层拦截/过滤**：在 `crates/hone-channels/src/agent_session.rs` 及对应接入层（如 `bins/hone-feishu/src/main.rs`）中，没有任何针对模型非预期回答（Prompt Echo 退化现象）的拦截过滤。当 LLM 开始原样复制用户的请求前缀时，这段内部机密的 System 文案被当成了正常生成的 Response 直接暴露并推送给了即时通讯工具。

**如何解决**
1. **短期止血**：在 `crates/hone-channels/src/agent_session.rs` 中，等 runner 返回了 `response.content` 后、在发给外部监听渠道之前，检查其是否以 `### System Instructions ###` 或 `\n### System Instructions ###` 作为前缀起手。若命中，则直接标记该条消息为出错或将内容强行置空，阻断其流向用户。
2. **长期治本**：不应通过发送一整段假装是 user 说话的超长 text 来传达 system 规则。需要重构 `opencode_acp.rs` 对运行时的拼装逻辑，通过底层的 API 标准，将这部分业务规则放到 System Role 或者环境变量对应的真正配置里。

---

## 问题二：Prompt Cache 彻底失效 (100% Cache Miss)

**现象**
系统底层使用 OpenRouter 接入各大厂模型（如 DeepSeekV3 等支持 Prefix Cache 的模型），理论上面对静态的 System Prompt 和冗长的历史记录，能够有效通过缓存大幅度削减费用与响应时间。但现实是系统每一轮对话都无法享受到缓存，纯冷启动。

**根因**
1. **动态时间戳充当了 Cache Buster（缓存杀手）**：在 `crates/hone-channels/src/prompt.rs` 的 `build_prompt_bundle` 函数逻辑里，代码为了传递时间感知能力，把精确到“秒”的时间加入到了 System Instructions 的内部（前缀靠前的部分）：
 ```rust
   let session_context = format!(
       "【Session 上下文】\n当前时间：{} (北京时间)\n当前日期：{}...",
       frozen.format("%Y-%m-%d %H:%M:%S")
   );
 ```
   由于当今大模型的 Prompt Cache（如 Anthropic/DeepSeek）大多严格依赖绝对前缀 Token 的字符串完全静态一致匹配。一旦时间秒级变动，这个处于句子靠前位置的 Token 便发生更改，导致模型直接抛弃该位置之后所有的 Cache。数十 K 的背景设定与历史总结均被重新计费计算。
2. **全新会话的纯文本压缩导致无法复用 Messages**：在 `crates/hone-channels/src/runners/opencode_acp.rs` 里，为了解决其他脏历史重复显示等 Bug，现在每轮生成均被强制重置并开启独立全新的 `session/new`，不再利用历史 messages 数组的 `cache_control` 设计，而是将所有的“过去”压缩为一个无结构的纯文本总结附加在了长篇开场白中。这从结构上也杀死了标准多轮缓存能力。

**如何解决**
1. **剥离动态前缀**：如果只是为了给 LLM 输入当前时间以供感知，绝不能放在影响全局缓存的静态 System Block 开头！必须将其移动到 User Message 的尾部区域（随着用户请求动态附加的末尾部分）。只有保证靠前位置长达上千 token 的业务基础 Prompt 是**绝对固定不变**的，底层模型厂商才可能对其复用。
2. **结构化上下文**：从长远看，若想要利用模型的轮次复用降本增效，需要探索以标准 Message 数组结构发送记录，而不是“每轮归档拼接+纯文本发送首轮”。
