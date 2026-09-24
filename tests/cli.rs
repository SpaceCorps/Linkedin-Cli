//! Drives the built binary against an in-process mock of the Apify API. Every test gets its
//! own config directory and the plaintext store, so nothing touches a real keystore or account.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    /// Routes are (method, path-prefix-or-exact, status, body). Unmatched requests get a 404.
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v2/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let raw_path = parts.next().unwrap_or("").to_string();
                    let path = raw_path.trim_start_matches("/v2/").to_string();
                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap_or(Value::Null));
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    let (status, resp) = routes
                        .iter()
                        .find(|(m, p, _, _)| {
                            *m == method && (*p == path || path.starts_with(&format!("{p}?")) || path.starts_with(*p))
                        })
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"error": {"message": "no route"}})));

                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "linkedin-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_linkedin"))
            .args(args)
            .env("LINKEDIN_CONFIG_DIR", &self.dir)
            .env("LINKEDIN_SECRET_STORE", "plaintext")
            .env("LINKEDIN_ALLOW_PLAINTEXT_STORE", "1")
            .env("APIFY_API_URL", &self.api)
            .env_remove("APIFY_TOKEN")
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap_or(-1), parse(&out.stdout), parse(&out.stderr))
    }

    fn with_account(self) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", "work", "--api-key", "test_key"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn user_me() -> Route {
    (
        "GET",
        "users/me",
        200,
        json!({
            "data": {
                "id": "u1",
                "username": "janedoe",
                "email": "jane@example.com"
            }
        }),
    )
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![user_me()]);
    let env = Env::new(&mock).with_account();

    assert_eq!(mock.last("GET").headers.iter().find(|(k, _)| k == "authorization").unwrap().1, "Bearer test_key");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["identity"], "jane@example.com");
    assert_eq!(out["accounts"][0]["username"], "janedoe");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");
    assert_eq!(out["secretStore"], "plaintext");

    let (_, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    // Lookup is case-insensitive, and duplicate needs --force
    let (code, _, err) = env.json(&["accounts", "add", "WORK", "--api-key", "x"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    let (code, out, _) = env.json(&["accounts", "test", "Work"]);
    assert_eq!(code, 0);
    assert_eq!(out["keyStatus"], "valid");

    // Non-interactive remove needs --yes
    let (code, _, err) = env.json(&["accounts", "remove", "work"]);
    assert_eq!(code, 6);
    assert_eq!(err["remediation"], "linkedin accounts remove work --yes");

    let (code, out, _) = env.json(&["accounts", "remove", "work", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (_, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(out["count"], 0);
}

#[test]
fn login_command_lifecycle() {
    let mock = Mock::start(vec![user_me(), user_me(), user_me()]);
    let env = Env::new(&mock);

    let (code, out, err) = env.json(&["login", "--api-key", "login_key_test"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "logged_in");
    assert_eq!(out["name"], "default");
    assert_eq!(out["identity"], "jane@example.com");
    assert_eq!(out["username"], "janedoe");
    assert_eq!(out["secretStore"], "plaintext");

    // Re-login without --force fails
    let (code, _, err) = env.json(&["login", "--api-key", "new_key"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--force"));

    // Re-login with --force succeeds
    let (code, out, _) = env.json(&["login", "--api-key", "new_key", "--force"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "logged_in");

    // Login with named account
    let (code, out, _) = env.json(&["login", "staging", "--api-key", "staging_key"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "staging");
}

#[test]
fn api_key_from_stdin() {
    let mock = Mock::start(vec![user_me()]);
    let env = Env::new(&mock);
    let mut child = Command::new(env!("CARGO_BIN_EXE_linkedin"))
        .args(["accounts", "add", "piped", "--api-key-stdin", "--json"])
        .env("LINKEDIN_CONFIG_DIR", &env.dir)
        .env("LINKEDIN_SECRET_STORE", "plaintext")
        .env("LINKEDIN_ALLOW_PLAINTEXT_STORE", "1")
        .env("APIFY_API_URL", &env.api)
        .env_remove("APIFY_TOKEN")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(b"piped_token_123\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let auth = mock.last("GET").headers.into_iter().find(|(k, _)| k == "authorization").unwrap().1;
    assert_eq!(auth, "Bearer piped_token_123");
}

#[test]
fn profile_fetch_and_filtering() {
    let mock = Mock::start(vec![
        user_me(),
        (
            "POST",
            "acts/harvestapi~linkedin-profile-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "fullName": "Alice Smith",
                    "headline": "Lead Rust Developer",
                    "skills": [{"name": "Rust"}, {"name": "Tokio"}],
                    "experience": [{"title": "Senior Engineer"}],
                    "education": [{"school": "Stanford"}]
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    // 1. Fetch full profile
    let (code, out, err) = env.json(&["profile", "https://www.linkedin.com/in/alicesmith", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["fullName"], "Alice Smith");
    assert_eq!(out[0]["headline"], "Lead Rust Developer");
    assert!(out[0]["skills"].is_array());
    assert!(out[0]["experience"].is_array());
    assert!(out[0]["education"].is_array());

    // 2. Fetch with --include
    let (code, out, err) =
        env.json(&["profile", "https://www.linkedin.com/in/alicesmith", "--include", "skills", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["fullName"], "Alice Smith");
    assert!(out[0]["skills"].is_array());
    assert!(out[0].get("experience").is_none());
    assert!(out[0].get("education").is_none());

    // 3. Inline --api-key
    let (code, _, err) = env.json(&["profile", "https://www.linkedin.com/in/alicesmith", "--api-key", "inline_secret"]);
    assert_eq!(code, 0, "{err}");
    let auth = mock.last("POST").headers.into_iter().find(|(k, _)| k == "authorization").unwrap().1;
    assert_eq!(auth, "Bearer inline_secret");
}

#[test]
fn post_fetch() {
    let mock = Mock::start(vec![
        user_me(),
        (
            "POST",
            "acts/supreme_coder~linkedin-post/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": "urn:li:activity:12345",
                    "content": "Rust 2024 is awesome!",
                    "likesCount": 42
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "post",
        "https://www.linkedin.com/feed/update/urn:li:activity:12345",
        "--limit",
        "5",
        "--no-deep",
        "-a",
        "work",
    ]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["id"], "urn:li:activity:12345");
    assert_eq!(out[0]["content"], "Rust 2024 is awesome!");

    let req = mock.last("POST");
    let body = req.body.unwrap();
    assert_eq!(body["urls"], json!(["https://www.linkedin.com/feed/update/urn:li:activity:12345"]));
    assert_eq!(body["limitPerSource"], 5);
    assert_eq!(body["deepScrape"], false);
    assert_eq!(body["rawData"], false);
}

#[test]
fn apify_token_env_var_works() {
    let mock = Mock::start(vec![(
        "POST",
        "acts/harvestapi~linkedin-profile-scraper/run-sync-get-dataset-items",
        200,
        json!([{"fullName": "Bob"}]),
    )]);
    let env = Env::new(&mock);

    let out = Command::new(env!("CARGO_BIN_EXE_linkedin"))
        .args(["profile", "https://www.linkedin.com/in/bob", "--json"])
        .env("LINKEDIN_CONFIG_DIR", &env.dir)
        .env("LINKEDIN_SECRET_STORE", "plaintext")
        .env("LINKEDIN_ALLOW_PLAINTEXT_STORE", "1")
        .env("APIFY_API_URL", &env.api)
        .env("APIFY_TOKEN", "env_token_456")
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(0));
    let auth = mock.last("POST").headers.into_iter().find(|(k, _)| k == "authorization").unwrap().1;
    assert_eq!(auth, "Bearer env_token_456");
}

#[test]
fn http_errors_map_to_exit_codes() {
    let mock = Mock::start(vec![
        user_me(),
        (
            "POST",
            "acts/harvestapi~linkedin-profile-scraper/run-sync-get-dataset-items",
            401,
            json!({"error": {"message": "Invalid token"}}),
        ),
        (
            "POST",
            "acts/supreme_coder~linkedin-post/run-sync-get-dataset-items",
            429,
            json!({"error": {"message": "Rate limited"}}),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, _, err) = env.json(&["profile", "https://www.linkedin.com/in/someone", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    let (code, _, err) = env.json(&["post", "https://www.linkedin.com/feed/update/urn:li:activity:999", "-a", "work"]);
    assert_eq!(code, 5);
    assert_eq!(err["code"], "rate_limited");
}

#[test]
fn parse_errors_are_envelopes() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    // Invalid URL format
    let (code, _, err) = env.json(&["profile", "https://example.com", "--api-key", "key"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
    assert!(err["error"].as_str().unwrap().contains("Invalid LinkedIn profile URL"));

    // Missing arguments
    let (code, _, err) = env.json(&["profile"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");

    // Help prints usage with exit code 0
    let out = env.run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn yaml_default_and_json_flag() {
    let mock = Mock::start(vec![
        user_me(),
        (
            "POST",
            "acts/harvestapi~linkedin-profile-scraper/run-sync-get-dataset-items",
            200,
            json!([{"fullName": "Sarah Connor"}]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    // Default: YAML
    let out = env.run(&["profile", "https://www.linkedin.com/in/sarah", "-a", "work"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("- fullName: Sarah Connor"));

    // With --json: JSON
    let (code, val, _) = env.json(&["profile", "https://www.linkedin.com/in/sarah", "-a", "work"]);
    assert_eq!(code, 0);
    assert_eq!(val[0]["fullName"], "Sarah Connor");
}

#[test]
fn agent_readme_as_data() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);
    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "linkedin");
    assert_eq!(out["exitCodes"]["7"], "no_account - run linkedin accounts list");

    let md = String::from_utf8(env.run(&["agent-readme"]).stdout).unwrap();
    assert!(md.starts_with("# linkedin - agent operating manual"));
}
