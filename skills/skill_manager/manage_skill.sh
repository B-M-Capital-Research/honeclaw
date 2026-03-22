#!/usr/bin/env bash
# manage_skill.sh - custom skill management utility
#
# Purpose: List, view, edit, and delete skills (only custom skills under data/custom_skills/)
# Usage:
#   bash skills/skill_manager/manage_skill.sh                # interactive menu
#   bash skills/skill_manager/manage_skill.sh list           # list all skills
#   bash skills/skill_manager/manage_skill.sh delete <name>  # delete a custom skill
#   bash skills/skill_manager/manage_skill.sh edit <name>    # edit a custom skill
#   bash skills/skill_manager/manage_skill.sh view <name>    # show skill details
# Expiry condition: update this script if the skill system changes storage format

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SYSTEM_SKILLS_DIR="$PROJECT_ROOT/skills"
CUSTOM_SKILLS_DIR="$PROJECT_ROOT/data/custom_skills"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Helpers

# Extract a frontmatter field value from SKILL.md
get_field() {
    local file="$1"
    local field="$2"
    if [[ ! -f "$file" ]]; then echo ""; return; fi
    # Read only the frontmatter block (from the first --- to the second ---)
    awk '/^---$/{if(++n==2)exit} n==1 && /^'"$field"':/{
        sub(/^'"$field"':[[:space:]]*/,""); print; exit
    }' "$file"
}

# List all skills
cmd_list() {
    echo ""
    echo -e "${BOLD}  -- Built-in skills (not editable) ----------------------------${NC}"
    local found_system=0
    if [[ -d "$SYSTEM_SKILLS_DIR" ]]; then
        while IFS= read -r -d '' skill_md; do
            skill_dir="$(dirname "$skill_md")"
            name="$(basename "$skill_dir")"
            display_name="$(get_field "$skill_md" "name")"
            description="$(get_field "$skill_md" "description")"
            display_name="${display_name:-$name}"
            echo -e "  ${CYAN}[system]${NC} ${BOLD}$display_name${NC} ${DIM}($name)${NC}"
            if [[ -n "$description" ]]; then
                echo -e "           ${DIM}$description${NC}"
            fi
            found_system=1
        done < <(find "$SYSTEM_SKILLS_DIR" -maxdepth 2 -name "SKILL.md" -print0 2>/dev/null | sort -z)
    fi
    [[ $found_system -eq 0 ]] && echo -e "  ${DIM}(none)${NC}"

    echo ""
    echo -e "${BOLD}  -- Custom skills (editable / deletable) ---------------------${NC}"
    local found_custom=0
    if [[ -d "$CUSTOM_SKILLS_DIR" ]]; then
        while IFS= read -r -d '' skill_md; do
            skill_dir="$(dirname "$skill_md")"
            name="$(basename "$skill_dir")"
            display_name="$(get_field "$skill_md" "name")"
            description="$(get_field "$skill_md" "description")"
            display_name="${display_name:-$name}"
            echo -e "  ${GREEN}[custom]${NC} ${BOLD}$display_name${NC} ${DIM}($name)${NC}"
            if [[ -n "$description" ]]; then
                echo -e "           ${DIM}$description${NC}"
            fi
            found_custom=1
        done < <(find "$CUSTOM_SKILLS_DIR" -maxdepth 2 -name "SKILL.md" -print0 2>/dev/null | sort -z)
    fi
    [[ $found_custom -eq 0 ]] && echo -e "  ${DIM}(no custom skills yet)${NC}"
    echo ""
}

# View skill details
cmd_view() {
    local name="$1"
    local skill_md=""

    if [[ -f "$CUSTOM_SKILLS_DIR/$name/SKILL.md" ]]; then
        skill_md="$CUSTOM_SKILLS_DIR/$name/SKILL.md"
        echo -e "  ${GREEN}[custom]${NC} $name"
    elif [[ -f "$SYSTEM_SKILLS_DIR/$name/SKILL.md" ]]; then
        skill_md="$SYSTEM_SKILLS_DIR/$name/SKILL.md"
        echo -e "  ${CYAN}[system]${NC} $name"
    else
        echo -e "  ${RED}Skill '$name' does not exist${NC}"
        return 1
    fi

    echo ""
    cat "$skill_md"
    echo ""
}

# Edit a custom skill
cmd_edit() {
    local name="$1"

    if [[ -f "$SYSTEM_SKILLS_DIR/$name/SKILL.md" ]]; then
        echo -e "  ${RED}Error: '$name' is a built-in system skill and cannot be edited through this tool${NC}"
        echo -e "  ${YELLOW}If you need to change a system skill, edit skills/$name/SKILL.md directly in the source tree${NC}"
        return 1
    fi

    local skill_md="$CUSTOM_SKILLS_DIR/$name/SKILL.md"
    if [[ ! -f "$skill_md" ]]; then
        echo -e "  ${RED}Custom skill '$name' does not exist${NC}"
        return 1
    fi

    local editor="${EDITOR:-vim}"
    echo -e "  ${YELLOW}Opening $skill_md in $editor ...${NC}"
    "$editor" "$skill_md"
    echo -e "  ${GREEN}✓ Saved${NC}"
}

# Delete a custom skill
cmd_delete() {
    local name="$1"

    if [[ -f "$SYSTEM_SKILLS_DIR/$name/SKILL.md" ]]; then
        echo -e "  ${RED}Error: '$name' is a built-in system skill and cannot be deleted${NC}"
        return 1
    fi

    local skill_dir="$CUSTOM_SKILLS_DIR/$name"
    if [[ ! -d "$skill_dir" ]]; then
        echo -e "  ${RED}Custom skill '$name' does not exist${NC}"
        return 1
    fi

    local display_name
    display_name="$(get_field "$skill_dir/SKILL.md" "name")"
    display_name="${display_name:-$name}"

    echo -e "  ${YELLOW}About to delete custom skill: ${BOLD}$display_name ($name)${NC}"
    echo -e "  ${RED}This action cannot be undone${NC}"
    read -r -p "  Confirm deletion? [y/N] " CONFIRM
    if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
        echo -e "  ${YELLOW}Cancelled${NC}"
        return 0
    fi

    rm -rf "$skill_dir"
    echo -e "  ${GREEN}✓ Deleted skill '$name'${NC}"
}

# Interactive menu
interactive_menu() {
    while true; do
        echo ""
        echo -e "${BOLD}  Hone - Skill Management${NC}"
        echo -e "  -------------------------------------"
        echo -e "  ${CYAN}1${NC}  List all skills"
        echo -e "  ${CYAN}2${NC}  View skill details"
        echo -e "  ${CYAN}3${NC}  Edit a custom skill"
        echo -e "  ${CYAN}4${NC}  Delete a custom skill"
        echo -e "  ${CYAN}5${NC}  Create a custom skill (runs create_skill.sh)"
        echo -e "  ${CYAN}q${NC}  Quit"
        echo ""
        read -r -p "  Choose an action > " CHOICE

        case "$CHOICE" in
            1)
                cmd_list
                ;;
            2)
                read -r -p "  Skill name (English ID) > " NAME
                cmd_view "$NAME"
                ;;
            3)
                cmd_list
                read -r -p "  Skill name to edit (English ID) > " NAME
                cmd_edit "$NAME"
                ;;
            4)
                cmd_list
                read -r -p "  Skill name to delete (English ID) > " NAME
                cmd_delete "$NAME"
                ;;
            5)
                bash "$SCRIPT_DIR/create_skill.sh"
                ;;
            q|Q)
                echo ""
                exit 0
                ;;
            *)
                echo -e "  ${RED}Invalid option. Please try again.${NC}"
                ;;
        esac
    done
}

# Entry point
mkdir -p "$CUSTOM_SKILLS_DIR"

COMMAND="${1:-menu}"

case "$COMMAND" in
    list)
        cmd_list
        ;;
    view)
        [[ -z "${2:-}" ]] && { echo -e "${RED}Usage: $0 view <name>${NC}"; exit 1; }
        cmd_view "$2"
        ;;
    edit)
        [[ -z "${2:-}" ]] && { echo -e "${RED}Usage: $0 edit <name>${NC}"; exit 1; }
        cmd_edit "$2"
        ;;
    delete)
        [[ -z "${2:-}" ]] && { echo -e "${RED}Usage: $0 delete <name>${NC}"; exit 1; }
        cmd_delete "$2"
        ;;
    menu|*)
        interactive_menu
        ;;
esac
