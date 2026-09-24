//! The command tree for linkedin CLI.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "linkedin",
    version,
    about = "CLI for fetching LinkedIn profiles and posts via Apify - YAML-first output for LLM agents",
    after_help = "An LLM agent should start with: linkedin agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Fetch a LinkedIn profile and output as YAML
    Profile(ProfileArgs),
    /// Fetch LinkedIn posts and output as YAML
    Post(PostArgs),
    /// Get current identity and token context
    Me(MeArgs),
    /// Log in with an Apify API token
    Login(LoginArgs),
    /// Print the operating manual for an LLM agent driving this CLI
    AgentReadme,
    /// Manage Apify accounts and their API keys
    #[command(subcommand)]
    Accounts(Accounts),
}

#[derive(Args, Clone, Debug)]
pub struct ProfileArgs {
    /// LinkedIn profile URL (e.g. https://www.linkedin.com/in/username)
    #[arg(value_name = "URL")]
    pub url: String,

    /// Comma-separated sections to include (e.g. experience,skills,education)
    #[arg(long, value_name = "SECTIONS")]
    pub include: Option<String>,

    /// Account to run against (see 'linkedin accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct PostArgs {
    /// LinkedIn URL — post, profile, company, or search URL
    #[arg(value_name = "URL")]
    pub url: String,

    /// Max posts per source URL (default: unlimited)
    #[arg(long, value_name = "N")]
    pub limit: Option<u32>,

    /// Only include posts newer than this date (e.g. 2025-01-01)
    #[arg(long, value_name = "DATE")]
    pub since: Option<String>,

    /// Skip additional info (likes, comments, etc.)
    #[arg(long)]
    pub no_deep: bool,

    /// Return raw unprocessed data from the scraper
    #[arg(long)]
    pub raw: bool,

    /// Comma-separated fields to include in output
    #[arg(long, value_name = "SECTIONS")]
    pub include: Option<String>,

    /// Account to run against (see 'linkedin accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct MeArgs {
    /// Account to run against (see 'linkedin accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct LoginArgs {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// Apify API key / token (prompted for securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API key from stdin, e.g. `pbpaste | linkedin login --api-key-stdin`
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not open the browser to the API keys page automatically
    #[arg(long)]
    pub no_browser: bool,

    /// Replace the key on an account that already exists
    #[arg(long)]
    pub force: bool,

    /// Store the key without calling the API to check it first
    #[arg(long)]
    pub no_verify: bool,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Accounts {
    /// Add an account and store its API key in the OS keystore
    Add {
        /// Short name for this account, used as --account elsewhere
        name: String,
        /// Apify API key (prompted for, without echo, if omitted)
        #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
        api_key: Option<String>,
        /// Read the API key from stdin, e.g. `pbpaste | linkedin accounts add work --api-key-stdin`
        #[arg(long)]
        api_key_stdin: bool,
        /// Replace the key on an account that already exists
        #[arg(long)]
        force: bool,
        /// Store the key without calling the API to check it first
        #[arg(long)]
        no_verify: bool,
    },
    /// List configured accounts
    List {
        /// Call the API once per account instead of reporting stored state
        #[arg(long)]
        check: bool,
    },
    /// Check that an account's stored key still works
    Test {
        /// Account name
        name: String,
    },
    /// Remove an account and delete its stored key
    Remove {
        /// Account name
        name: String,
        /// Skip the confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_tree_is_valid() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }
}
