use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub prefix: String,
    pub token: String,
    pub client_id: String
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
        match fs::read_to_string(path) {
            Ok(contents) => match toml::from_str::<Self>(&contents) {
                Ok(cfg) => Ok(cfg),
                Err(e) => Err(e.to_string())
            },
            Err(e) => Err(e.to_string())
        }
    }
}