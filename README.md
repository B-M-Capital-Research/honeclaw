<p align="center">
  <img src="./resources/logo.svg" alt="Hone honepage - Your Financial Assistant" width="30%">
</p>

<p align="center">
  <strong> Hone 磨刀石 </strong><br>
  <strong>“并非迎合你的聊天玩具，而是你投资纪律的无情捍卫者。”</strong><br>
  HoneClaw专注于成为懂你的，专业投资助手。<br><br>
  <strong>为什么取名Hone：</strong><br>
  Hone 的意思，是磨刀、打磨锋刃。而真正严肃的投资，本质上就是这样一个过程：不是追逐每一条新闻，不是对每一次涨跌做情绪化反应，而是在研究、比较、复盘和长期纪律中，不断磨砺自己的判断力。
</p>

<p align="center">
  <strong>简体中文</strong> | <a href="./README_EN.md">English</a>  | <strong>💬 社群:</strong> <a href="https://discord.gg/TyDNfYXDGF" target="_blank">Discord</a> 
</p>

---

# 1. 🦅 Honeclaw (Hone Financial)

Honeclaw（或称 Hone）是一个使用**Rust**编写的开源个人投研辅助助手。与市面上习惯于附和用户的“闲聊机器人”不同，Honeclaw 被设计为一个**具备冷静思考能力、客观且克制的投研助手**。

它通过多端渠道（飞书、discord、telegram、imessages）接入你的日常工作流，帮助你跟踪持仓公司动态、执行严格的投资纪律、执行定时监控任务，并在你情绪化交易时提供理性的数据与逻辑对抗。

<p align="center">
  <img src="./resources/hone_introduction_zh" alt="Hone Introduction - Your Financial Assistant" width="80%">
</p>


**系统架构**：[交互式架构图（HTML）](./resources/architecture.html) — 克隆仓库后，在本地用浏览器打开该文件即可查看。

# 2. ✨ 核心特性 (Key Features)

- 🧠 **绝对理性的投研内核**：不附和、不盲从。在你做出投资决策时，它会基于数据和预设纪律进行交叉验证，指出你的逻辑漏洞。
- 📱 **全平台无缝接入**：支持 iMessage, 飞书 (Lark), Telegram, Discord，随时随地与你的投资大脑进行对话。
- 📊 **持仓监控与纪律管理**：设定你的止盈止损线、加仓逻辑与核心关注指标，Hone 会像冷酷的守望者一样帮你盯盘。
- ⏰ **强大的定时任务 (Cron-jobs)**：支持复杂的定时监控任务，例如盘前摘要、盘后总结、特定财报发布后的自动解析等。
- ⚡ **极致性能**：底层完全使用 Rust 构建，内存占用极低，并发处理能力极强，确保多端消息的毫秒级响应。

<p align="center">
  <a href="./resources/hone_channels_zh.jpg" target="_blank">
    <img src="./resources/hone_channels_zh.jpg" alt="Hone Channels" width="400">
  </a>
  &nbsp;&nbsp;
  <a href="./resources/hone_solution_zh.jpg" target="_blank">
    <img src="./resources/hone_solution_zh.jpg" alt="Hone Solution" width="400">
  </a>
</p>


# 3. 🏗️ 快速开始 (Getting Started)


## 前置依赖

- **Basic Env**: A basic Unix/Linux environment (macOS / Ubuntu recommended) 
- **Rust**: Edition 2021+

### 支持渠道

- Mac端APP
- 飞书（Feishu / Lark）
- Discord
- Telegram
- iMessage

## 安装与启动

1. 克隆仓库

```shell 
git clone https://github.com/B-M-Capital-Research/honeclaw.git
cd honeclaw
```

2. 一键启动

系统内置了启动脚本，将自动编译并拉起服务：

```shell
chmod +x launch.sh
./launch.sh --desktop
```


### 首次启动时，脚本在做什么？

运行 `./launch.sh --desktop` 时，内置脚本会**按顺序**完成环境准备、编译与进程拉起，把本地全栈跑起来。**第一次**完整执行通常需要约 **10 分钟**（随网络与机器性能浮动）。

1. **环境与依赖**：检查 `bun`、`rustup` 等运行时，并同步/安装项目依赖。
2. **编译构建**
   - **Rust 后端**：桌面壳 `hone-desktop`、核心 API `hone-web-api`、各通讯渠道的 **Sidecar**。
   - **前端**：SolidJS + Vite 构建桌面 UI，由壳程序加载。
3. **服务拉起**：在守护模式下启动本地 Web 与数据访问层，并**自动打开**桌面窗口。

### 窗口打开后：为 Agent 配置推理后端

此时需要告诉系统：Agent 的「大脑」走**本地 CLI** 还是**兼容 OpenAI 的云 API**。

1. 点击主界面左下角的 **⚙️ 设置（Settings）**。
2. 在 **Agent / 推理** 相关区域选择一种方式：
   - **本地引擎（零配置）**：若本机已安装并运行 `gemini cli` 或 `codex`，应用可自动发现；在下拉框中选中即可，一般无需额外填写。
   - **云端 API（推荐）**：若无本地引擎，可配置任意 **OpenAI 兼容** 的 HTTP 接口（地址、密钥等，视服务商而定）。
     - **推荐组合**：`OpenRouter` + `Gemini 3.1 Pro` 或 `Gemini 3.1 Flash`。
     - **说明**：在团队实测中，该组合在推理深度、响应延迟与上下文吞吐之间较为均衡。

各渠道接入与完整设置界面，见下一节配图。


## 启动后，在端侧APP的设置，进行模型和渠道配置

<p align="center">
  <img src="./resources/hone_page.jpg" alt="端侧 App 首页：对话入口" width="100%">
  <br>
  <em>端侧 App 首页：主对话界面，可直接开始与 Hone 对话。</em>
</p>
<p align="center">
  <img src="./resources/hone_setting.jpg" alt="设置页：模型与渠道" width="100%">
  <br>
  <em>设置页：配置推理模型（云端 / 本地）以及飞书、Discord、Telegram、iMessage 等各接入渠道。</em>
</p>
---

# 4. 🌰 一些案例

<table>
<tr>
<th align="center">1. 正常问答</th>
<th align="center">2. Discord 群聊</th>
<th align="center">3. 定时播报</th>
</tr>
<tr>
<td valign="top" align="center"><img src="./resources/example1.jpg" alt="Honeclaw 示例截图 1" width="260"/></td>
<td valign="top" align="center"><img src="./resources/example2.jpg" alt="Honeclaw 示例截图 2" width="260"/></td>
<td valign="top" align="center"><img src="./resources/example3.jpg" alt="Honeclaw 示例截图 3" width="260"/></td>
</tr>
</table>

以上截图仅为示意；Honeclaw 还支持**更多用法与配置**，可在使用过程中逐步解锁。

[`CASES_ZH.md`](CASES_ZH.md) 汇总了 Hone 的**贴近真实场景的问答示例**（个股逻辑、追问基本面、结合持仓的每日建议、深度研究、定时任务、主题挖掘与宏观等），在 GitHub 上以两列表格呈现，便于浏览。英文版见 [`CASES_EN.md`](CASES_EN.md)。

# 5. 💡 维护者寄语

> “市场充满杂音，贪婪与恐惧是投资者的宿敌。希望 Honeclaw 能够成为你在交易市场中最冷静的锚。”

为遵守开源许可要求，一些专业估值工具、投研工作流以及专有知识库未包含在此公开仓库中。

这些内容包括但不限于：
-  高级 DCF 与相对估值模型 
-  行业专项深度研究工作流 
-  精选整理的投研知识库（如财报电话会纪要、分析师报告资料库） 

如果你有兴趣获取这些能力，欢迎联系我们：

1. [YouTube: 巴芒投研美股频道](https://www.youtube.com/@%E5%B7%B4%E8%8A%92%E6%8A%95%E7%A0%94%E7%BE%8E%E8%82%A1%E9%A2%91%E9%81%93) — 欢迎关注，获取投研内容

![BM YTB](./resources/bm_youtube.jpg)


2. [Discord](https://discord.gg/TyDNfYXDGF): 通过邀请码链接 (https://discord.gg/TyDNfYXDGF) 加入我们的社区频道


# 6. 🤝 Contributing

Honeclaw 致力于成为开源社区中最专业的个人投研基础设施。如果你对 Rust 后端开发、大模型 Prompt 工程或金融数据分析感兴趣，欢迎提交 PR。

贡献者：

- [carlisle0615](https://github.com/carlisle0615)
- [Finn-Fengming](https://github.com/Finn-Fengming)

📄 License

本项目采用 Apache-2.0 协议.