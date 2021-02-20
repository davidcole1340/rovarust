mod rova;

use rova::{Region, Rova};
use serenity::{async_trait, client::{Client, EventHandler, Context}, futures::TryFutureExt, http::AttachmentType, model::prelude::{Message, Ready}};
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

impl Handler {
    async fn info(&self, _ctx: Context, message: &Message, args: Option<Vec<&str>>) {
        let repsonse = message.channel_id.send_message(&_ctx.http, |m| {
            m.embed(|e| {
                e.title("Rova");
                e.description("Rova bot help");
                e.image("attachment://rova.png");
                e.fields(vec![
                    (format!("{} station", self.config.prefix), "shows a list of stations", false),
                    (format!("{} station [station]", self.config.prefix), "selects a station", false),
                    (format!("{} playing", self.config.prefix), "outputs the current song on the playing station", false),
                    (format!("{} playing [station]", self.config.prefix), "outputs the current song on a given station", false),
                    (format!("{} invite", self.config.prefix), "outputs an invite link for the bot", false)
                ]);
                e
            });
            m.add_file(AttachmentType::Path(Path::new("./rova.png")));
            m
        }).await;

        if let Err(e) = repsonse {
            println!("error sending help embed: {:?}", e);
        }
    }

    async fn station(&self, _ctx: Context, message: &Message, args: Option<Vec<&str>>) {
        println!("station command");

        if let Some(station) = args {
            // select station
            return
        }
    }

    async fn playing(&self, _ctx: Context, message: &Message, args: Option<Vec<&str>>) {
        println!("playing command");

        if let Some(station) = args {

        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, bot: Ready) {
        println!("bot is ready: {}", bot.user.name);
    }

    async fn message(&self, _ctx: Context, message: Message) {
        let msg: Vec<&str> = message.content.split_whitespace().collect();

        match msg.get(0) {
            Some(prefix) if self.config.prefix.eq(prefix) => (),
            _ => return
        };

        println!("found prefix");

        let args = if msg.len() >= 3 {
            Some(msg[2..].to_vec())
        } else {
            None
        };

        let cmd: &str = match msg.get(1) {
            Some(cmd) => cmd,
            _ => {
                self.info(_ctx, &message, args).await;
                return
            }
        };

        match cmd {
            "station" => self.station(_ctx, &message, args).await,
            "playing" => self.playing(_ctx, &message, args).await,
            _ => {
                self.info(_ctx, &message, args).await;
                return
            }
        };
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