use std::fs;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub discord: Discord,
    pub postgres: Postgres,
    pub mongo: Mongo,
    pub stripe: Stripe,
}
impl Config {
    pub fn read() -> Self {
        let contents = fs::read_to_string("config.toml").expect("Cannot read configuration file");
        toml::from_str(&contents).expect("Cannot parse configuration file")
    }
}

#[derive(Clone, Deserialize)]
pub struct Discord {
    pub token: String,
    pub test_guild_id: u64,
}

#[derive(Clone, Deserialize)]
pub struct Postgres {
    pub connection_uri: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Clone, Deserialize)]
pub struct Mongo {
    pub connection_uri: String,
}

#[derive(Clone, Deserialize)]
pub struct Stripe {
    pub cloudflare_hook_base: String,
    pub cloudflare_auth: String,
}
impl Stripe {
    pub fn format_stripe_hook_url(&self, path: &str) -> String {
        format!("{}{path}", self.cloudflare_hook_base)
    }
}
