//! HTTP client to the Apify API, and translation from HTTP status to [`ErrorCode`].
//!
//! One blocking agent per process: a CLI makes a handful of requests, so an async runtime
//! would cost more in startup than it could save.

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://api.apify.com/v2/";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    token: String,
}

enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(token: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(360)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("linkedin-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        // For pointing the CLI at a mock server in tests.
        let mut base = std::env::var("APIFY_API_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, token: token.trim().to_string() }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    pub fn fetch_profile(&self, profile_url: &str) -> Result<Value> {
        let endpoint =
            format!("acts/harvestapi~linkedin-profile-scraper/run-sync-get-dataset-items?token={}", seg(&self.token));
        let body = serde_json::json!({ "urls": [profile_url] });
        self.post(&endpoint, &body)
    }

    pub fn fetch_posts(
        &self,
        url: &str,
        limit: Option<u32>,
        since: Option<&str>,
        deep_scrape: bool,
        raw: bool,
    ) -> Result<Value> {
        let endpoint =
            format!("acts/supreme_coder~linkedin-post/run-sync-get-dataset-items?token={}", seg(&self.token));
        let mut map = serde_json::Map::new();
        map.insert("urls".into(), serde_json::json!([url]));
        if let Some(l) = limit {
            map.insert("limitPerSource".into(), serde_json::json!(l));
        }
        if let Some(s) = since {
            map.insert("scrapeUntil".into(), serde_json::json!(s));
        }
        map.insert("deepScrape".into(), serde_json::json!(deep_scrape));
        map.insert("rawData".into(), serde_json::json!(raw));

        let body = Value::Object(map);
        self.post(&endpoint, &body)
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        let auth = format!("Bearer {}", self.token);

        macro_rules! headers {
            ($req:expr) => {
                $req.header("Authorization", &auth).header("Accept", "application/json")
            };
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => headers!(self.agent.post(&url)).send_empty(),
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Object(Default::default()));
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the Apify API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    let e = match status {
        401 | 403 => Error::new(ErrorCode::AuthRequired, "Apify API token was rejected or unauthorized.")
            .fix("Pass a valid token via --api-key, export APIFY_TOKEN, or run: linkedin login"),
        404 => Error::new(ErrorCode::NotFound, "The requested Apify resource was not found."),
        429 => Error::new(ErrorCode::RateLimited, "Rate limited by the Apify API.").fix("Back off before retrying."),
        400 | 422 => Error::new(ErrorCode::InvalidInput, "The Apify API refused the request."),
        s if s >= 500 => Error::new(ErrorCode::Network, "The Apify API returned a server error.")
            .fix("Retry; if it persists, check Apify platform status."),
        _ => Error::new(ErrorCode::Error, "The request failed."),
    };
    e.detail(detail)
}

/// Percent-encodes one path segment or parameter.
pub fn seg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seg_escapes_reserved() {
        assert_eq!(seg("token_abc"), "token_abc");
        assert_eq!(seg("A B/C"), "A%20B%2FC");
    }

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(403, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(400, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(500, "").code, ErrorCode::Network);
    }
}
