#!/usr/bin/env python3
"""Migrate legacy Hone YAML skills into `SKILL.md` directories."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


def unquote(value: str) -> str:
    text = value.strip()
    if len(text) >= 2 and text[0] == text[-1] and text[0] in {"'", '"'}:
        return text[1:-1]
    return text


def parse_legacy_skill_yaml(path: Path) -> tuple[dict[str, object], str]:
    frontmatter: dict[str, object] = {
        "name": "",
        "description": "",
        "aliases": [],
        "tools": [],
    }
    prompt_lines: list[str] = []
    current_list: str | None = None
    in_prompt = False
    prompt_indent = 0

    for raw_line in path.read_text().splitlines():
        if in_prompt:
            if raw_line.strip() == "":
                prompt_lines.append("")
                continue
            indent = len(raw_line) - len(raw_line.lstrip(" "))
            if indent <= prompt_indent:
                in_prompt = False
            else:
                prompt_lines.append(raw_line[prompt_indent + 2 :])
                continue

        stripped = raw_line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if raw_line.startswith("  - ") and current_list in {"aliases", "tools"}:
            frontmatter[current_list].append(unquote(raw_line[4:]))
            continue

        current_list = None
        if ":" not in raw_line:
            raise ValueError(f"unsupported YAML syntax: {raw_line}")
        key, value = raw_line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if key in {"aliases", "tools"}:
            current_list = key
            frontmatter[key] = []
            continue
        if key == "prompt":
            if value != "|":
                raise ValueError("prompt must use block scalar syntax `|`")
            in_prompt = True
            prompt_indent = len(raw_line) - len(raw_line.lstrip(" "))
            continue
        if key in {"name", "description"}:
            frontmatter[key] = unquote(value)
            continue
        raise ValueError(f"unsupported YAML key: {key}")

    prompt = "\n".join(prompt_lines).strip()
    return frontmatter, prompt


def render_skill_md(frontmatter: dict[str, object], prompt: str) -> str:
    def escape(value: object) -> str:
        return str(value).replace('"', '\\"')

    lines = [
        "---",
        f'name: "{escape(frontmatter["name"])}"',
        f'description: "{escape(frontmatter["description"])}"',
    ]
    aliases = frontmatter.get("aliases") or []
    if aliases:
        lines.append("aliases:")
        for alias in aliases:
            lines.append(f"  - {alias}")
    else:
        lines.append("aliases: []")
    tools = frontmatter.get("tools") or []
    if tools:
        lines.append("tools:")
        for tool in tools:
            lines.append(f"  - {tool}")
    else:
        lines.append("tools: []")
    lines.extend(["---", "", prompt.strip(), ""])
    return "\n".join(lines)


def migrate_dir(skills_dir: Path, write: bool) -> tuple[int, int, list[str]]:
    migrated = 0
    untouched = 0
    errors: list[str] = []

    for path in sorted(list(skills_dir.glob("*.yaml")) + list(skills_dir.glob("*.yml"))):
        target_dir = skills_dir / path.stem
        target_md = target_dir / "SKILL.md"
        if target_md.exists():
            untouched += 1
            continue
        try:
            frontmatter, prompt = parse_legacy_skill_yaml(path)
        except Exception as exc:
            errors.append(f"{path}: {exc}")
            continue
        if not str(frontmatter["name"]).strip():
            errors.append(f"{path}: name missing")
            continue
        if not str(frontmatter["description"]).strip():
            errors.append(f"{path}: description missing")
            continue
        migrated += 1
        print(f"[MIGRATE] {path}")
        print(f"  - create {target_md}")
        print(f"  - remove {path.name}")
        if write:
            target_dir.mkdir(parents=True, exist_ok=True)
            target_md.write_text(render_skill_md(frontmatter, prompt))
            path.unlink()

    return migrated, untouched, errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Migrate legacy YAML skills to SKILL.md.")
    parser.add_argument(
        "--skills-dir",
        action="append",
        dest="skills_dirs",
        required=True,
        help="Skill root directory to scan; may be passed multiple times.",
    )
    parser.add_argument("--write", action="store_true", help="Rewrite files in place.")
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Validate current files without applying migration logic.",
    )
    args = parser.parse_args()

    migrated = 0
    untouched = 0
    errors: list[str] = []

    for raw_dir in args.skills_dirs:
        skills_dir = Path(raw_dir)
        if not skills_dir.exists():
            errors.append(f"{skills_dir}: skills dir not found")
            continue
        if args.validate_only:
            legacy_files = list(skills_dir.glob("*.yaml")) + list(skills_dir.glob("*.yml"))
            if legacy_files:
                errors.extend(f"{path}: legacy YAML skill must be migrated" for path in legacy_files)
            untouched += len(list(skills_dir.iterdir()))
            continue
        m, u, errs = migrate_dir(skills_dir, args.write)
        migrated += m
        untouched += u
        errors.extend(errs)

    mode = "write" if args.write else "dry-run"
    if args.validate_only:
        mode = "validate-only"
    print(
        f"[SUMMARY] mode={mode} dirs={len(args.skills_dirs)} migrated={migrated} untouched={untouched} errors={len(errors)}"
    )
    for err in errors:
        print(f"[ERROR] {err}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
