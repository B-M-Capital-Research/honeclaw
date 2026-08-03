#!/bin/bash
#=============================================================================
# 脚本名称: detect_dev_tool.sh
# 功能描述: 智能识别 AI 编程工具，返回工具名称
# 调用位置: 集成流程和签约流程的环境检查
# 调用前置: 无
# 返回值: coze | meoo | claudeCode | codex | traeSolo | cursor | qoder | unknown
#=============================================================================

detect_dev_tool() {
    if [ -n "$COZE_PROJECT_ID" ]; then
        echo "coze"
        return
    fi
    if [ -n "$MEOO_PROJECT_ID" ]; then
        echo "meoo"
        return
    fi
    if [ -n "$CLAUDECODE" ] || [ -n "$CLAUDE_CODE_ENTRYPOINT" ]; then
        echo "claudeCode"
        return
    fi
    if [ -n "$CODEX_CI" ] || [ -n "$CODEX_THREAD_ID" ]; then
        echo "codex"
        return
    fi
    if [ -n "$TRAE_BRAND_NAME" ]; then
        echo "traeSolo"
        return
    fi
    if [ -n "$CURSOR_AGENT" ] || [ -n "$CURSOR_LAYOUT" ]; then
        echo "cursor"
        return
    fi
    if [ -n "$QODER_CLI" ]; then
        echo "qoder"
        return
    fi
    echo "unknown"
}

# 仅在直接执行时输出，被 source 时不执行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    DEV_TOOL_NAME=$(detect_dev_tool)
    echo "$DEV_TOOL_NAME"
fi
