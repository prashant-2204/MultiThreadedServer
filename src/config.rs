use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub worker_threads: usize,
    pub max_connections: usize,
    pub db_url: String,
    pub cache_size: usize,
    pub request_timeout: u64,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("config.json")
            .unwrap_or_else(|_| Self::default_config());
        Ok(serde_json::from_str(&config_str)?)
    }

    fn default_config() -> String {
        let default = Config {
            host: "0.0.0.0".to_string(),
            port: 8080,
            worker_threads: num_cpus::get() * 2,
            max_connections: 10000,
            db_url: "postgres://user:pass@localhost/db".to_string(),
            cache_size: 10000,
            request_timeout: 30,
        };
        serde_json::to_string(&default).unwrap()
    }
}