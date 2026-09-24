# LinkedIn CLI

A blazing fast native command-line tool and agent interface for scraping LinkedIn profiles and posts via Apify, written in Rust.

- **Speed:** Sub-3ms cold start, single standalone static binary.
- **Output:** YAML by default for terminal readability, `--json` for jq pipelines and agent tool loops.
- **Security:** Native OS Keystore integration (macOS Keychain, Linux Secret Service, Windows DPAPI).
- **Control:** Granular `--include` section filtering to minimize token usage.
- **Agent Ready:** Self-documenting `agent-readme` command with structured exit codes.

## Quickstart

```bash
# 1. Install via Cargo
cargo install --git https://github.com/SpaceCorps/Linkedin-Cli.git

# 2. Authenticate with your Apify API Token
linkedin login

# 3. Fetch a full LinkedIn profile
linkedin profile https://www.linkedin.com/in/satyanadella

# 4. Fetch only specific sections
linkedin profile https://www.linkedin.com/in/satyanadella --include experiences,skills,educations

# 5. Fetch posts from an activity URL or search
linkedin post https://www.linkedin.com/feed/update/urn:li:activity:123456789 --limit 10
```

## Available Sections for `--include`

`experiences`, `skills`, `educations`, `languages`, `licenseAndCertificates`, `honorsAndAwards`, `volunteerAndAwards`, `projects`, `publications`, `patents`, `courses`, `testScores`, `organizations`, `interests`, `recommendations`, `updates`, `profilePicAllDimensions`, `verifications`, `promos`, `highlights`, `volunteerCauses`

## Commands Overview

| Command | Description | Key Options |
| :--- | :--- | :--- |
| `linkedin profile <URL>` | Scrape full profile details for a LinkedIn user | `--include <SECTIONS>`, `-a <NAME>`, `--api-key <KEY>` |
| `linkedin post <URL>` | Scrape posts from update, profile, company, or search URL | `--limit <N>`, `--since <DATE>`, `--no-deep`, `--raw`, `--include` |
| `linkedin me` | Inspect authenticated user context and token validity | `-a <NAME>`, `--api-key <KEY>` |
| `linkedin login [<NAME>]` | Authenticate with Apify token and store into OS keystore | `--api-key <KEY>`, `--api-key-stdin`, `--no-browser`, `--force` |
| `linkedin accounts list` | List all configured accounts and keystore backend | `--check` |
| `linkedin accounts test <NAME>` | Verify that a stored account key still works | None |
| `linkedin accounts remove <NAME>` | Remove account and delete key from local keystore | `--yes` |
| `linkedin agent-readme` | Print the built-in operating manual for LLM agents | `--json` |

## Discovery & Standards

- [Agent Manual (llms.txt)](https://spacecorps.github.io/Linkedin-Cli/llms.txt)
- [Full LLM Specification (llms-full.txt)](https://spacecorps.github.io/Linkedin-Cli/llms-full.txt)
- [Authentication Guide](https://spacecorps.github.io/Linkedin-Cli/auth.html)
- [A2A Agent Card](https://spacecorps.github.io/Linkedin-Cli/.well-known/agent-card.json)
- [AgentSkills Discovery](https://spacecorps.github.io/Linkedin-Cli/.well-known/agent-skills/index.json)
