#!/usr/bin/env bash
# create_skill.sh - interactive custom skill creator
#
# Purpose: Collect skill details step by step in the terminal and write them to data/custom_skills/<name>/SKILL.md
# Usage: bash skills/skill_manager/create_skill.sh
# Expiry condition: update this script if the skill system changes storage format

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CUSTOM_SKILLS_DIR="$PROJECT_ROOT/data/custom_skills"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${BOLD}  Hone - Create a Custom Skill${NC}"
echo -e "  The skill will be saved to: ${CYAN}data/custom_skills/\${name}/${NC}"
echo ""

# Step 1: English ID
while true; do
    echo -e "${BOLD}[1/6] English identifier (name)${NC}"
    echo -e "  ${YELLOW}Use only letters, numbers, and underscores, and start with a letter. Example: MY_SKILL or news_digest${NC}"
    read -r -p "  > " SKILL_NAME

    SKILL_NAME="${SKILL_NAME// /_}"

    if [[ -z "$SKILL_NAME" ]]; then
        echo -e "  ${RED}It cannot be empty. Please try again.${NC}"
        continue
    fi

    if ! [[ "$SKILL_NAME" =~ ^[a-zA-Z][a-zA-Z0-9_]*$ ]]; then
        echo -e "  ${RED}Invalid format. Please enter only letters, numbers, and underscores, starting with a letter.${NC}"
        continue
    fi

    TARGET_DIR="$CUSTOM_SKILLS_DIR/$SKILL_NAME"
    if [[ -d "$TARGET_DIR" ]]; then
        echo -e "  ${RED}Skill '$SKILL_NAME' already exists ($TARGET_DIR). Please choose another name.${NC}"
        continue
    fi

    # Also check the system skill directory
    SYS_SKILL_DIR="$PROJECT_ROOT/skills/$SKILL_NAME"
    if [[ -d "$SYS_SKILL_DIR" ]]; then
        echo -e "  ${RED}A built-in system skill with the same name already exists: '$SKILL_NAME'. Please choose another name.${NC}"
        continue
    fi

    break
done
echo ""

# Step 2: Display name
echo -e "${BOLD}[2/6] Display name (display_name)${NC}"
echo -e "  ${YELLOW}This can be Chinese if you want, for example: Daily News Digest${NC}"
read -r -p "  > " DISPLAY_NAME
DISPLAY_NAME="${DISPLAY_NAME:-$SKILL_NAME}"
echo ""

# Step 3: Aliases
echo -e "${BOLD}[3/6] Aliases / trigger keywords (optional)${NC}"
echo -e "  ${YELLOW}Separate multiple items with commas, for example: news, daily updates. Leave empty to skip.${NC}"
read -r -p "  > " ALIASES_RAW
echo ""

# Parse aliases into a YAML list
ALIASES_YAML=""
if [[ -n "$ALIASES_RAW" ]]; then
    IFS=',' read -ra ALIAS_ARR <<< "$ALIASES_RAW"
    for alias in "${ALIAS_ARR[@]}"; do
        alias="$(echo "$alias" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
        if [[ -n "$alias" ]]; then
            ALIASES_YAML+="  - $alias"$'\n'
        fi
    done
fi

# Step 4: One-sentence description
echo -e "${BOLD}[4/6] One-sentence description (description)${NC}"
echo -e "  ${YELLOW}Example: Automatically fetches financial news every day and generates a summary push${NC}"
read -r -p "  > " DESCRIPTION
DESCRIPTION="${DESCRIPTION:-[not provided]}"
echo ""

# Step 5: Required tools
echo -e "${BOLD}[5/6] Required tools (optional)${NC}"
echo -e "  ${YELLOW}Available tools: web_search, data_fetch, portfolio_tool, cron_job, image_gen, skill_tool${NC}"
echo -e "  ${YELLOW}Separate them with commas; leave empty to skip.${NC}"
read -r -p "  > " TOOLS_RAW
echo ""

TOOLS_YAML=""
if [[ -n "$TOOLS_RAW" ]]; then
    IFS=',' read -ra TOOLS_ARR <<< "$TOOLS_RAW"
    for tool in "${TOOLS_ARR[@]}"; do
        tool="$(echo "$tool" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
        if [[ -n "$tool" ]]; then
            TOOLS_YAML+="  - $tool"$'\n'
        fi
    done
fi

# Step 6: Execution logic prompt
echo -e "${BOLD}[6/6] Execution logic / prompt${NC}"
echo -e "  ${YELLOW}Describe how the AI should act when triggered. Multi-line input is supported; type END on a new line to finish.${NC}"
echo -e "  ${YELLOW}For example: Step 1 call web_search to fetch the latest financial news; Step 2 format the result as a summary...${NC}"
PROMPT_LINES=()
while IFS= read -r -p "  > " line; do
    [[ "$line" == "END" ]] && break
    PROMPT_LINES+=("$line")
done
PROMPT="$(printf '%s\n' "${PROMPT_LINES[@]}")"
echo ""

# Preview
echo -e "${BOLD}  -- Skill Preview --------------------------------------------${NC}"
echo -e "${CYAN}---"
echo "name: $DISPLAY_NAME"
echo "description: $DESCRIPTION"
if [[ -n "$ALIASES_YAML" ]]; then
    echo "aliases:"
    echo -n "$ALIASES_YAML"
fi
if [[ -n "$TOOLS_YAML" ]]; then
    echo "tools:"
    echo -n "$TOOLS_YAML"
fi
echo "---"
echo ""
echo "$PROMPT"
echo -e "${NC}"
echo -e "  ------------------------------------------------------------"
echo ""

read -r -p "  Confirm creation of this skill? [y/N] " CONFIRM
if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
    echo -e "  ${YELLOW}Cancelled${NC}"
    exit 0
fi

# Write the file
mkdir -p "$TARGET_DIR"

{
    echo "---"
    echo "name: $DISPLAY_NAME"
    echo "description: $DESCRIPTION"
    if [[ -n "$ALIASES_YAML" ]]; then
        echo "aliases:"
        echo -n "$ALIASES_YAML"
    fi
    if [[ -n "$TOOLS_YAML" ]]; then
        echo "tools:"
        echo -n "$TOOLS_YAML"
    fi
    echo "---"
    echo ""
    echo "$PROMPT"
} > "$TARGET_DIR/SKILL.md"

echo ""
echo -e "  ${GREEN}✓ Skill '$DISPLAY_NAME' ($SKILL_NAME) created${NC}"
echo -e "  Path: ${CYAN}$TARGET_DIR/SKILL.md${NC}"
echo ""
