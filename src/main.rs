use serenity::{
    client::{Client, EventHandler, Context},
    model::prelude::{Message, Ready},
    async_trait 
};
use std::{
    path::Path,
    fs
};
use serde::{Deserialize};

#[derive(Deserialize)]
struct Config {
    prefix: String,
    token: String
}

struct Handler {
    config: Config
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("bot is ready: {}", _data_about_bot.user.name);
    }

    async fn message(&self, _ctx: Context, message: Message) {
        let msg: Vec<&str> = message.content.split(' ').collect();

        if msg.len() < 1 { return }
        if **msg.first().unwrap() != self.config.prefix { return }
    }
}

#[tokio::main]
async fn main() {
    let config_path = Path::new("config.toml");

    if !config_path.exists() {
        println!("no config exists - please create one at config.toml");
        return
    }

    let config_file = fs::read_to_string("config.toml").expect("unable to read config file");
    let config: Config = toml::from_str(&config_file).expect("unable to parse config");
    
    // let config: Config = toml::from_str
    let mut client = Client::builder(&config.token)
        .event_handler(Handler { config })
        .await
        .expect("error creating client");

    if let Err(e) = client.start().await {
        println!("client error: {}", e);
    }
}