#!/usr/bin/env python3
"""Run all Hone legacy-data migration scripts with one command."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def run_step(script: str, extra_args: list[str]) -> int:
    cmd = [sys.executable, script, *extra_args]
    print(f"==> {' '.join(cmd)}")
    return subprocess.run(cmd, check=False).returncode


def main() -> int:
    parser = argparse.ArgumentParser(description="Run all Hone legacy-data migrations.")
    parser.add_argument("--sessions-dir", default="./data/sessions")
    parser.add_argument("--cron-jobs-dir", default="./data/cron_jobs")
    parser.add_argument(
        "--skills-dir",
        action="append",
        dest="skills_dirs",
        help="Skill root directory; may be passed multiple times. Defaults to ./skills and HONE_DATA_DIR/custom_skills when present.",
    )
    parser.add_argument("--write", action="store_true", help="Rewrite files in place.")
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Validate current files without applying migration logic.",
    )
    args = parser.parse_args()

    scripts_dir = Path(__file__).resolve().parent
    skills_dirs = args.skills_dirs or ["./skills"]
    hone_data_dir = os.environ.get("HONE_DATA_DIR")
    if hone_data_dir:
        custom_skills = str(Path(hone_data_dir) / "custom_skills")
        if custom_skills not in skills_dirs:
            skills_dirs.append(custom_skills)

    mode_flags: list[str] = []
    if args.write:
        mode_flags.append("--write")
    if args.validate_only:
        mode_flags.append("--validate-only")

    steps = [
        (
            "sessions",
            str(scripts_dir / "migrate_sessions.py"),
            ["--sessions-dir", args.sessions_dir, *mode_flags],
        ),
        (
            "cron_jobs",
            str(scripts_dir / "migrate_cron_jobs.py"),
            ["--cron-jobs-dir", args.cron_jobs_dir, *mode_flags],
        ),
        (
            "skills",
            str(scripts_dir / "migrate_skills.py"),
            [item for skill_dir in skills_dirs for item in ("--skills-dir", skill_dir)] + mode_flags,
        ),
    ]

    failed = []
    for name, script, extra_args in steps:
        code = run_step(script, extra_args)
        if code != 0:
            failed.append((name, code))

    if failed:
        for name, code in failed:
            print(f"[ERROR] {name} migration failed with exit code {code}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
