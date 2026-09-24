//! Account resolution and identity probing.
//!
//! Keys can come from an explicit account (--account <name>), an inline key (--api-key <key>),
//! the APIFY_TOKEN environment variable, or a single configured account.

use serde_json::Value;

use crate::client::Client;
use crate::config::{self, AccountConfig, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    pub name: String,
    pub config: AccountConfig,
    pub api_key: String,
}

impl Resolved {
    pub fn client(&self) -> Client {
        Client::new(&self.api_key)
    }
}

pub fn resolve(requested: Option<&str>, api_key_override: Option<&str>) -> Result<Resolved> {
    if let Some(key) = api_key_override.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(Resolved { name: "api-key".into(), config: AccountConfig::default(), api_key: key.to_string() });
    }

    let config = config::load()?;

    if let Some(requested) = requested.map(str::trim).filter(|s| !s.is_empty()) {
        let Some((name, account)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("linkedin accounts list"));
        };

        let key = secrets::store()?.get(&secrets::account_key(name))?;
        let Some(api_key) = key.filter(|k| !k.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored API key."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("linkedin accounts add {name} --api-key <key>")));
        };

        return Ok(Resolved { name: name.clone(), config: account.clone(), api_key });
    }

    if let Some(token) = std::env::var("APIFY_TOKEN").ok().filter(|s| !s.trim().is_empty()) {
        return Ok(Resolved {
            name: "env".into(),
            config: AccountConfig::default(),
            api_key: token.trim().to_string(),
        });
    }

    if config.accounts.len() == 1 {
        let (name, account) = config.accounts.iter().next().unwrap();
        let key = secrets::store()?.get(&secrets::account_key(name))?;
        if let Some(api_key) = key.filter(|k| !k.trim().is_empty()) {
            return Ok(Resolved { name: name.clone(), config: account.clone(), api_key });
        }
    }

    if config.accounts.is_empty() {
        return Err(Error::new(ErrorCode::AuthRequired, "No Apify API token specified.")
            .detail("Set APIFY_TOKEN, pass --api-key <key>, or run 'linkedin login' to configure an account.")
            .fix("linkedin login"));
    }

    Err(Error::new(ErrorCode::NoAccount, "No account specified. Pass --account <name>.")
        .detail(describe(&config))
        .fix("linkedin accounts list"))
}

fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'linkedin login' or 'linkedin accounts add <name>'.".into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}

pub mod identity {
    use super::Value;

    const CANDIDATES: &[&str] = &["email", "userEmail", "username", "userName", "name", "id"];

    pub fn describe(me: &Value) -> String {
        if !me.is_object() {
            return String::new();
        }
        first_string(me, CANDIDATES).or_else(|| nested(me, "data", CANDIDATES)).unwrap_or_default()
    }

    pub fn username(me: &Value) -> String {
        if !me.is_object() {
            return String::new();
        }
        first_string(me, &["username", "userName", "name"])
            .or_else(|| nested(me, "data", &["username", "userName", "name"]))
            .unwrap_or_default()
    }

    fn nested(root: &Value, property: &str, names: &[&str]) -> Option<String> {
        root.get(property).filter(|c| c.is_object()).and_then(|c| first_string(c, names))
    }

    fn first_string(v: &Value, names: &[&str]) -> Option<String> {
        names
            .iter()
            .filter_map(|n| v.get(*n).and_then(Value::as_str))
            .find(|s| !s.trim().is_empty())
            .map(str::to_string)
    }

    #[cfg(test)]
    mod tests {
        use serde_json::json;

        #[test]
        fn reads_apify_user_data() {
            let me = json!({
                "data": {
                    "id": "u123",
                    "username": "janedoe",
                    "email": "jane@example.com"
                }
            });
            assert_eq!(super::describe(&me), "jane@example.com");
            assert_eq!(super::username(&me), "janedoe");
        }
    }
}
