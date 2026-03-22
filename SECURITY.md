# Security Policy

## Reporting a Vulnerability

Please do not report sensitive vulnerabilities through public issues.

Preferred disclosure channels:

- GitHub Security Advisories, if enabled for the repository
- A private issue or internal maintainer channel
- A dedicated security email address, if one is configured for the public copy

When reporting, please include:

- A short summary of the issue
- The affected component or file path
- Reproduction steps
- Whether any credentials, tokens, or user data may have been exposed

## Sensitive Data Handling

- Do not commit API keys, OAuth tokens, cookies, exported runtime configs, or database files
- If a secret is ever committed, rotate it immediately and remove it from history before publishing
- Treat `config.yaml`, `data/`, logs, and local caches as private runtime state

## Current Repository Notes

This repository is the private staging copy used to prepare a separate public repository later.
Before publishing the public copy, confirm that any seed configuration files and scan allowlists are still appropriate for that export.
