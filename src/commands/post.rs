//! `linkedin post <URL>`. Fetches LinkedIn posts via Apify.

use std::collections::HashSet;
use std::io::IsTerminal;

use crate::account;
use crate::cli::PostArgs;
use crate::commands::profile::filter_fields;
use crate::error::Result;
use crate::output;
use crate::url;

pub fn run(args: PostArgs) -> Result<()> {
    let normalized = url::normalize_post(&args.url)?;

    if std::io::stderr().is_terminal() {
        eprintln!("Fetching posts (this may take 30–60s)...");
    }

    let account = account::resolve(args.account.as_deref(), args.api_key.as_deref())?;
    let mut data =
        account.client().fetch_posts(&normalized, args.limit, args.since.as_deref(), !args.no_deep, args.raw)?;

    if let Some(include_str) = args.include {
        let include_set: HashSet<String> =
            include_str.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
        if !include_set.is_empty() {
            filter_fields(&mut data, &include_set);
        }
    }

    output::write(&data);
    Ok(())
}
