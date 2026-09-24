# LinkedIn CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/Linkedin-Cli?color=blue&label=version)](https://github.com/SpaceCorps/Linkedin-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/Linkedin-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Linkedin-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/Linkedin-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A blazing fast, native command-line tool and agent interface for scraping LinkedIn profiles and posts via Apify. Built in Rust for developers and autonomous AI workflows.

---

## Highlights

- ⚡ **Sub-3ms Startup**: Compiled as a native static binary with zero runtime dependencies. Executes in ~1–3 ms (compared to ~75 ms for managed runtimes).
- 🔐 **OS Keystore Integration**: `linkedin login` prompts for your token securely and saves it to native OS vaults (macOS Keychain, Linux Secret Service / Keyutils, Windows DPAPI).
- 🌐 **Profile & Post Scraping**: Powered by battle-tested Apify actors (`harvestapi~linkedin-profile-scraper` and `supreme_coder~linkedin-post`).
- 🎯 **Selective Section Filtering**: Slashing LLM token consumption by extracting only the sections you need with `--include`.
- 🤖 **AI Agent Native**: Machine-readable `--json` output, standardized error envelopes with stable numeric exit codes, and explicit `llms.txt` agent guidance.
- 🛡️ **Safety Guardrails**: Multi-account management prevents credential collision across environments.

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/Linkedin-Cli --locked
```

### Pre-built Standalone Binaries

Download standalone binary archives directly from the [GitHub Releases](https://github.com/SpaceCorps/Linkedin-Cli/releases/latest) page:

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | [`linkedin-v1.0.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Linkedin-Cli/releases/download/v1.0.0/linkedin-v1.0.0-aarch64-apple-darwin.tar.gz) |
| **macOS** | Intel (`x86_64`) | [`linkedin-v1.0.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Linkedin-Cli/releases/download/v1.0.0/linkedin-v1.0.0-x86_64-apple-darwin.tar.gz) |
| **Linux** | x86_64 (musl static) | [`linkedin-v1.0.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Linkedin-Cli/releases/download/v1.0.0/linkedin-v1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| **Linux** | ARM64 (musl static) | [`linkedin-v1.0.0-aarch64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Linkedin-Cli/releases/download/v1.0.0/linkedin-v1.0.0-aarch64-unknown-linux-musl.tar.gz) |
| **Windows**| x64 (MSVC) | [`linkedin-v1.0.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/Linkedin-Cli/releases/download/v1.0.0/linkedin-v1.0.0-x86_64-pc-windows-msvc.zip) |

---

## Quickstart

### 1. Authenticate

Get your API token at [Apify](https://console.apify.com/account/integrations) and authenticate:

```bash
# Interactive login (stores in macOS Keychain, Linux secret-tool, or Windows DPAPI)
linkedin login

# Or set via environment variable
export APIFY_TOKEN=your-token-here

# Or pass per-command
linkedin profile https://www.linkedin.com/in/username --api-key apify_api_xxx
```

### 2. Fetch Profiles

```bash
# Fetch full profile in YAML
linkedin profile https://www.linkedin.com/in/satyanadella

# Fetch only specific sections
linkedin profile https://www.linkedin.com/in/satyanadella --include experiences,skills,educations

# Output as JSON for scripts or jq
linkedin profile https://www.linkedin.com/in/satyanadella --json | jq '.[0].skills'
```

#### Available Sections for `--include`

`experiences`, `skills`, `educations`, `languages`, `licenseAndCertificates`, `honorsAndAwards`, `volunteerAndAwards`, `projects`, `publications`, `patents`, `courses`, `testScores`, `organizations`, `interests`, `recommendations`, `updates`, `profilePicAllDimensions`, `verifications`, `promos`, `highlights`, `volunteerCauses`

### 3. Fetch Posts

```bash
# Fetch from update, profile, company, or search URL
linkedin post https://www.linkedin.com/feed/update/urn:li:activity:123456789 --limit 10

# Only include posts newer than date, without comments/likes deep scrape
linkedin post https://www.linkedin.com/company/apple --since 2025-01-01 --no-deep
```

### 4. Verify Context

```bash
linkedin me
```

---

## Command Reference

| Command | Description |
|:---|:---|
| `linkedin profile <URL>` | Fetch a LinkedIn profile and output as YAML or JSON |
| `linkedin post <URL>` | Fetch LinkedIn posts (activity, profile, company, or search) |
| `linkedin me` | Display current authenticated user and token status |
| `linkedin login [<NAME>]` | Log in and store API key in the native OS keystore |
| `linkedin accounts add <NAME>` | Add a named account to the keystore |
| `linkedin accounts list` | List configured accounts (use `--check` for live status) |
| `linkedin accounts test <NAME>` | Verify stored credentials for an account |
| `linkedin accounts remove <NAME>` | Remove account and delete credentials from keystore |
| `linkedin agent-readme` | Print embedded LLM agent operating manual (`--json` supported) |

---

## Agentic Integration

Autonomous LLM agents should start by invoking:

```bash
linkedin agent-readme --json
```

This returns structured guidance containing tool metadata, operational rules, and error-to-exit-code mappings.

---

## License

MIT License. Copyright (c) 2026 SpaceCorps.
Original prototype created by Niels Bosma.
