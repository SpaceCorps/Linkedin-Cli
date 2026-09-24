---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the LinkedIn CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for LinkedIn CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the LinkedIn CLI.

## Overview
LinkedIn CLI interfaces with LinkedIn data via Apify actors. Authentication requires an Apify API token. Tokens can be stored in the host operating system's native keychain or supplied directly via environment variables and standard input.

## Prerequisites
- An Apify account ([apify.com](https://www.apify.com))
- A valid API token from Apify Integrations Settings (`https://console.apify.com/account/integrations`)
- LinkedIn CLI installed on your machine (`cargo install --git https://github.com/SpaceCorps/Linkedin-Cli --locked`)

## Authentication Flow

### Interactive Browser Login (`linkedin login`)
The recommended flow for local developer machines:
```bash
linkedin login [account_name]
```
1. The CLI launches your system browser to `https://console.apify.com/account/integrations`.
2. You copy or generate your personal Apify token.
3. Paste the token into the CLI prompt (input characters are masked).
4. The CLI validates the key with a live request to `GET /v2/users/me`.
5. Upon confirmation, the key is securely saved to the native OS keyring under the account name (defaults to `default`).

### Non-Interactive / Headless Login
For headless CI/CD environments, Docker containers, or autonomous agent runners:
```bash
printf %s "$APIFY_TOKEN" | linkedin login [account_name] --api-key-stdin
```
Or pass the token directly as a CLI flag:
```bash
linkedin login [account_name] --api-key "$APIFY_TOKEN"
```

## Environment Variables
The CLI checks the environment for credentials when no keychain account is specified:
- `APIFY_TOKEN`: Fallback API token used if no keystore account is explicitly selected.
- `LINKEDIN_SECRET_STORE`: Force keystore backend (`keychain`, `libsecret`, `dpapi`, `plaintext`).
- `LINKEDIN_ALLOW_PLAINTEXT_STORE=1`: Allow unencrypted file storage in headless environments without a secret service daemon.

## Multi-Account Management
Switch or verify accounts using:
```bash
linkedin accounts list --check
linkedin accounts test [account_name]
```

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized JSON error payloads:
- `auth_required` (exit code 3): No token provided or token expired.
- `no_account` (exit code 7): Multiple accounts exist without `--account`, or named account does not exist.
- `invalid_input` (exit code 6): Invalid URL format or arguments.
- `rate_limited` (exit code 5): Apify API rate limits reached.

## Security Best Practices
1. **Never Commit Tokens**: Keep `.env` or plaintext token files out of version control.
2. **Use OS Keystore**: The CLI automatically utilizes macOS Keychain, Windows DPAPI, or Linux Secret Service.
3. **Machine Verification**: When writing agent automation scripts, always pass `--json` to reliably capture error codes.
