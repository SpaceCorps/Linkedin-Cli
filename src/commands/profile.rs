//! `linkedin profile <URL>`. Fetches a LinkedIn profile via Apify.

use std::collections::HashSet;
use std::io::IsTerminal;

use serde_json::Value;

use crate::account;
use crate::cli::ProfileArgs;
use crate::error::Result;
use crate::output;
use crate::url;

const OPTIONAL_FIELDS: &[&str] = &[
    "experience",
    "experiences",
    "education",
    "educations",
    "skills",
    "skill",
    "certifications",
    "licenseandcertificates",
    "projects",
    "project",
    "volunteering",
    "volunteerandawards",
    "volunteercauses",
    "publications",
    "publication",
    "courses",
    "course",
    "honorsandawards",
    "languages",
    "language",
    "causes",
    "featured",
    "receivedrecommendations",
    "recommendations",
    "moreprofiles",
    "patents",
    "testscores",
    "organizations",
    "interests",
    "updates",
    "profilepicalldimensions",
    "verifications",
    "promos",
    "highlights",
];

pub fn run(args: ProfileArgs) -> Result<()> {
    let normalized = url::normalize_profile(&args.url)?;

    if std::io::stderr().is_terminal() {
        eprintln!("Fetching profile (this may take 30–60s)...");
    }

    let account = account::resolve(args.account.as_deref(), args.api_key.as_deref())?;
    let mut data = account.client().fetch_profile(&normalized)?;

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

pub fn is_optional_field(name: &str) -> bool {
    let lower = name.to_lowercase();
    OPTIONAL_FIELDS.contains(&lower.as_str())
}

pub fn is_included(name: &str, include: &HashSet<String>) -> bool {
    let lower = name.to_lowercase();
    if include.contains(&lower) {
        return true;
    }
    if let Some(stripped) = lower.strip_suffix('s')
        && include.contains(stripped)
    {
        return true;
    }
    let with_s = format!("{lower}s");
    if include.contains(&with_s) {
        return true;
    }
    false
}

pub fn filter_fields(data: &mut Value, include: &HashSet<String>) {
    match data {
        Value::Array(arr) => {
            for item in arr {
                filter_fields(item, include);
            }
        }
        Value::Object(map) => {
            let to_remove: Vec<String> =
                map.keys().filter(|k| is_optional_field(k) && !is_included(k, include)).cloned().collect();
            for k in to_remove {
                map.remove(&k);
            }
            for v in map.values_mut() {
                filter_fields(v, include);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn filters_profile_sections() {
        let mut doc = json!({
            "fullName": "Jane Doe",
            "experience": [{"title": "Engineer"}],
            "skills": [{"name": "Rust"}],
            "education": [{"school": "MIT"}]
        });

        let mut include = HashSet::new();
        include.insert("skills".to_string());

        filter_fields(&mut doc, &include);

        assert_eq!(doc["fullName"], "Jane Doe");
        assert_eq!(doc["skills"], json!([{"name": "Rust"}]));
        assert!(doc.get("experience").is_none());
        assert!(doc.get("education").is_none());
    }
}
