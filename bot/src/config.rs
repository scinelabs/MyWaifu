use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub discord: Discord,
    pub postgres: Postgres,
    pub mongo: Mongo,
}
impl Config {
    pub fn read() -> Self {
        let contents = fs::read_to_string("config.toml").expect("Cannot read configuration file");
        toml::from_str(&contents).expect("Cannot parse configuration file")
    }
}

#[derive(Deserialize)]
pub struct Discord {
    pub token: String,
    pub test_guild_id: u64,
}

#[derive(Deserialize)]
pub struct Postgres {
    pub connection_uri: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Deserialize)]
pub struct Mongo {
    pub connection_uri: String,
}
