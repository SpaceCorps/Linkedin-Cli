# AGENTS.md

Notes for developers and agents extending or maintaining `linkedin`.

`linkedin` is a native Rust CLI for scraping LinkedIn profiles and posts via Apify actors, built to be driven by humans and autonomous LLM agents. It replaced a .NET global tool prototype (`Linkedin.Console`) by Niels Bosma and expands its interface: preserving the core `profile` and `post` syntax and section filtering, while adding native OS keystores (macOS Keychain, Linux secret-tool, Windows DPAPI), multi-account safety, structured JSON error envelopes, and self-documenting agent discovery.

For the manual the *agent* reads, run `linkedin agent-readme` - that text lives in `src/readme.rs` and is the tool's runtime interface for LLM callers. This file is for developers editing the source code.

## Development & Test Commands

```bash
cargo build --release              # target/release/linkedin
cargo test                         # unit tests + offline TCP mock integration tests
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install locally to ~/.cargo/bin
```

Use a throwaway config directory when testing so you never touch real credentials:

```bash
export LINKEDIN_CONFIG_DIR=$(mktemp -d) LINKEDIN_SECRET_STORE=plaintext LINKEDIN_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Effect |
| --- | --- |
| `LINKEDIN_CONFIG_DIR` | Overrides the config and secrets storage path |
| `LINKEDIN_SECRET_STORE` | Forces a specific backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `LINKEDIN_ALLOW_PLAINTEXT_STORE=1` | Permits unencrypted secrets fallback where no OS keyring is present |
| `APIFY_API_URL` | Overrides the API base URL — how `tests/cli.rs` points at its mock server |
| `APIFY_TOKEN` | Fallback token used when `--account` or `--api-key` is omitted |

## Layout

```
src/
  main.rs          arg parsing, --json pre-scan, clap errors -> invalid_input envelopes
  cli.rs           command tree (clap derive) and help strings
  commands/
    mod.rs         command dispatcher
    profile.rs     profile fetching and selective section filtering
    post.rs        post fetching with deep scrape and date filtering
    me.rs          identity probing and token validation
    login.rs       interactive/stdin login and keystore storage
    accounts.rs    accounts add|list|test|remove
  client.rs        blocking HTTP (ureq + rustls), status -> ErrorCode mapping
  error.rs         ErrorCode enum and Error {message, detail, remediation} envelope
  output.rs        YAML by default, JSON with --json, write_error, obj! macro
  account.rs       multi-account resolution and identity parsing
  config.rs        config.yaml, atomic writes, 0600 permissions, cross-process lock
  secrets.rs       macOS Keychain, Linux libsecret, Windows DPAPI, plaintext fallback
  url.rs           LinkedIn URL normalization and validation
  readme.rs        agent-readme text and rules
tests/
  cli.rs           in-process TCP mock HTTP server test suite
```

## Architectural Invariants

1. **Sub-3ms Startup:** Zero heavy async runtimes (Tokio is not used). Requests are synchronous via `ureq` with connection pooling.
2. **Stdout is Pure Data:** Logs, progress bars, and warnings are restricted to `stderr` when a TTY is detected. Piping stdout into `jq` or an agent tool loop is always deterministic.
3. **Deterministic Multi-Account Resolution:** If multiple accounts exist, the CLI demands `--account <name>`. If only one account exists, it is automatically resolved without manual flag burden.
4. **Offline Integration Testing:** `tests/cli.rs` tests the CLI against an in-process TCP server, verifying status code mappings, JSON flag formatting, and authentication headers with zero network calls.
