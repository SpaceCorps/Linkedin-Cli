//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "linkedin",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - run linkedin accounts list",
            },
        });
        return;
    }
    println!("{README}");
}

/// The Apify API version and actor integration this CLI targets.
pub const API_VERSION: &str = "v2";

const RULES: &[&str] = &[
    "Always specify a valid LinkedIn URL (profile, post, company, or search).",
    "Authenticate with 'linkedin login', pass --api-key <key>, or set the APIFY_TOKEN environment variable.",
    "Pass -a/--account when multiple accounts are configured in the keystore.",
    "Use --include to filter output to specific sections (e.g. experiences,skills,educations).",
    "On code auth_required, stop and surface the remediation string to the user. Do not retry.",
    "Use --json when you are going to parse the output with jq or within an agent tool loop.",
];

const README: &str = r#"# linkedin - agent operating manual

A native Rust CLI for fetching LinkedIn profiles and posts via Apify. Results are YAML on stdout,
errors are structured envelopes on stderr, and `--json` switches both to JSON. Progress indicators
go to stderr when attached to an interactive terminal, ensuring stdout remains clean for piping.

## Authentication & Accounts

The CLI supports three credential methods:

1. **OS Keystore Accounts (Recommended):**
   Run `linkedin login` to save your Apify token into the native OS Keystore (macOS Keychain,
   Linux Secret Service / libsecret, or Windows DPAPI).
2. **Environment Variable:**
   Set `export APIFY_TOKEN=your_token_here`.
3. **Per-Command Flag:**
   Pass `--api-key <token>`.

### Managing Accounts

    linkedin login [<name>] [--api-key <key>]  # interactive login (default name: "default")
    linkedin accounts add <name> --api-key <key> [--force]
    printf %s "$KEY" | linkedin accounts add <name> --api-key-stdin
    linkedin accounts list [--check]
    linkedin accounts test <name>
    linkedin accounts remove <name> --yes

## Fetching Profiles

    linkedin profile https://www.linkedin.com/in/username
    linkedin profile https://www.linkedin.com/in/username --include experiences,skills,educations
    linkedin profile https://www.linkedin.com/in/username -a work

### Available sections for `--include`

`experiences`, `skills`, `educations`, `languages`, `licenseAndCertificates`, `honorsAndAwards`,
`volunteerAndAwards`, `projects`, `publications`, `patents`, `courses`, `testScores`, `organizations`,
`interests`, `recommendations`, `updates`, `profilePicAllDimensions`, `verifications`, `promos`,
`highlights`, `volunteerCauses`

## Fetching Posts

    linkedin post https://www.linkedin.com/feed/update/urn:li:activity:123456789
    linkedin post https://www.linkedin.com/posts/username-post-id --limit 10
    linkedin post https://www.linkedin.com/company/apple --since 2025-01-01
    linkedin post https://www.linkedin.com/search/results/all/?keywords=rust --no-deep

Options for `post`:
- `--limit <N>`: Maximum posts per source URL.
- `--since <DATE>`: Only include posts newer than this date (e.g. 2025-01-01).
- `--no-deep`: Skip additional details like comments and likes.
- `--raw`: Return raw unprocessed data from the scraper.
- `--include <SECTIONS>`: Comma-separated fields to keep.

## Identity Context

    linkedin me [-a <account>]

Displays the currently authenticated Apify username, user email, and account status.

## Errors & Exit Codes

Failures print structured YAML (or JSON with `--json`) on stderr with a stable numeric exit code:

    0  ok
    1  error          unclassified - report it and stop
    2  network        retry once, then stop
    3  auth_required  stop; surface the remediation string verbatim
    4  not_found      requested actor or dataset not found; do not retry
    5  rate_limited   back off before trying again
    6  invalid_input  fix the call (invalid URL or options)
    7  no_account     multiple accounts configured; specify --account <name>"#;
