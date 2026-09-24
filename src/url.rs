//! Normalization and validation for LinkedIn URLs.

use crate::error::{Error, Result};

pub fn normalize_profile(url: &str) -> Result<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();

    if let Some(pos) = lower.find("linkedin.com/in/") {
        let after = &trimmed[pos + "linkedin.com/in/".len()..];
        let id: String =
            after.chars().take_while(|c| *c != '/' && *c != '?' && *c != '#' && !c.is_whitespace()).collect();
        if !id.is_empty() {
            return Ok(format!("https://www.linkedin.com/in/{id}"));
        }
    }

    Err(Error::invalid(format!("Invalid LinkedIn profile URL: {url}")))
}

pub fn normalize_post(url: &str) -> Result<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();

    let is_post = lower.contains("linkedin.com/feed/update") || lower.contains("linkedin.com/posts/");
    let is_profile = lower.contains("linkedin.com/in/");
    let is_company = lower.contains("linkedin.com/company/");
    let is_search = lower.contains("linkedin.com/search/");

    if is_post || is_profile || is_company || is_search {
        return Ok(trimmed.to_string());
    }

    Err(Error::invalid(format!("Invalid LinkedIn URL: {url}. Expected a post, profile, company, or search URL.")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_profile_urls() {
        assert_eq!(
            normalize_profile("https://www.linkedin.com/in/johndoe").unwrap(),
            "https://www.linkedin.com/in/johndoe"
        );
        assert_eq!(
            normalize_profile("http://linkedin.com/in/john-doe-123/").unwrap(),
            "https://www.linkedin.com/in/john-doe-123"
        );
        assert_eq!(normalize_profile("linkedin.com/in/user?key=val").unwrap(), "https://www.linkedin.com/in/user");
        assert!(normalize_profile("https://google.com").is_err());
        assert!(normalize_profile("linkedin.com/in/").is_err());
    }

    #[test]
    fn normalizes_post_urls() {
        assert_eq!(
            normalize_post("https://www.linkedin.com/feed/update/urn:li:activity:12345").unwrap(),
            "https://www.linkedin.com/feed/update/urn:li:activity:12345"
        );
        assert_eq!(
            normalize_post("https://www.linkedin.com/posts/username-post-id").unwrap(),
            "https://www.linkedin.com/posts/username-post-id"
        );
        assert_eq!(
            normalize_post("https://www.linkedin.com/company/spacecorps").unwrap(),
            "https://www.linkedin.com/company/spacecorps"
        );
        assert_eq!(
            normalize_post("https://www.linkedin.com/search/results/all/?keywords=test").unwrap(),
            "https://www.linkedin.com/search/results/all/?keywords=test"
        );
        assert!(normalize_post("https://example.com/not-linkedin").is_err());
    }
}
