# Codex Project Notes

This directory stores repository-shared Codex project artifacts that are safe to version.

Automation runtime state is not repo-scoped by default. The live Codex app stores each
automation under `~/.codex/automations/<id>/`.

Files in this repo should be treated as versioned snapshots or templates that document
the intended automation configuration for collaborators.

Do not commit local runtime state such as per-automation `memory.md`.
