# Skill 启动自更新检查

本文件定义 `alipay-aipay` Skill 启动时的自更新检查。该检查用于降低用户继续使用旧版 Skill 的概率，但不得阻断支付集成或签约入驻主流程。

## 执行原则

- 每次触发本 Skill 时，先执行本检查。
- 如果本轮对话中刚成功执行过 `npx -y @alipay/alipay-aipay@latest install`，或等价的 `@latest install --target ...` 命令，则视为自更新检查已完成，直接继续当前业务流程，不再查询 npm latest，也不再重复执行安装。
- 本检查不作为业务阻塞确认点，不单独询问用户是否更新。
- 若当前 Agent 环境要求联网或写入用户 Skill 目录授权，则按环境权限机制处理；不得在 Skill 内另设业务确认点。
- 任何读取版本、查询 npm、执行更新失败的情况，都只输出简短提示并继续当前业务流程。
- 更新命令必须使用 `--skip-tools`，只更新 Skill 文件，不重新检查或安装 GPG、alipay-cli、jq。

## 版本读取

1. 读取当前 `SKILL.md` 所在目录下的 `VERSION` 文件作为本地版本。
2. 如果 `VERSION` 不存在，或内容不是合法 SemVer 版本号（例如 `latest` 或 `v1`），将当前安装视为“未知版本/旧版安装”，直接进入「未知版本更新规则」，不要再猜测其他安装记录文件。
3. 如果成功读取本地版本，执行以下命令查询 npm latest：
   ```bash
   npm view @alipay/alipay-aipay version --json
   ```
4. 如果无法获取 npm latest，跳过更新并继续当前业务流程。

## 更新规则

如果本地版本低于 npm latest，自动执行 Skill 更新。

更新命令必须覆盖当前正在使用的 Skill 目录。取当前 `SKILL.md` 所在目录的父目录作为 `<当前Skill父目录>`，执行：

```bash
ALIPAY_AIPAY_EXTRA_SKILL_DIRS="<当前Skill父目录>" npx -y @alipay/alipay-aipay@latest install --target all --skip-tools
```

说明：
- `ALIPAY_AIPAY_EXTRA_SKILL_DIRS` 确保当前正在使用的 Skill 父目录会被纳入更新目标，避免只更新当前项目目录下的 `.agents/.codex/.claude`。
- `--target all` 继续覆盖安装器解析到的全部目标目录。
- `--skip-tools` 避免在启动检查中触发 GPG、alipay-cli、jq 的检查或安装。

## 未知版本更新规则

如果当前 Skill 目录没有 `VERSION`，或 `VERSION` 内容不是合法 SemVer 版本号，不做不可靠的版本推断，直接执行同一条更新命令：

```bash
ALIPAY_AIPAY_EXTRA_SKILL_DIRS="<当前Skill父目录>" npx -y @alipay/alipay-aipay@latest install --target all --skip-tools
```

说明：
- 这是对旧版安装的确定性兼容路径；更新成功后，新安装器会写入 `VERSION`，后续启动即可走正常版本比较。
- 如果该命令因网络、npm、权限或执行环境限制失败，输出简短提示并继续当前业务流程。

## 输出要求

- 已是最新：输出一句简短提示即可。
- 发现新版本并开始更新：输出本地版本、npm latest 和更新目标摘要。
- 未知版本并开始更新：输出“未检测到 VERSION，尝试安装 latest”及更新目标摘要。
- 更新成功：提示更新已完成，并继续当前业务流程。
- 更新失败或权限受限：提示本次自更新未完成，继续使用当前已加载版本执行流程。

不得输出长篇安装日志；如需日志，以命令输出或工具环境记录为准。
