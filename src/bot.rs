use std::{path::Path, sync::{Arc}};
use serenity::{async_trait, client::{Context, EventHandler}, http::AttachmentType, model::{channel::Message, prelude::Ready}};
use crate::{config::Config, rova, rova::{Station}};
use tokio::sync::Mutex;

pub struct Handler {
    pub config: Config,
    pub stations: Vec<rova::Station>,
    pub on_air: Arc<Mutex<Vec<rova::OnAir>>>
}

impl Handler {
    async fn info(&self, _ctx: Context, message: &Message, _args: Option<Vec<&str>>) {
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

        if let Some(args) = args {
            let station_id = args.first().unwrap();
            let guild = message.guild(&_ctx.cache).await.unwrap();
            // select station
            match self.fetch_station_by_id(station_id) {
                Some(station) => match guild.voice_states.get(&message.author.id).and_then(|state| state.channel_id) {
                    Some(channel_id) => {
                        let songbird = self.songbird(&_ctx).await;

                        // join channel
                        let (handler, result) = songbird.join(guild.id, channel_id).await;
                        
                        if let Err(e) = result {
                            println!("error joining channel: {:?}", e);
                            let _ = message.reply(&_ctx, format!("Error joining voice channel: {}", e.to_string())).await;
                            return
                        }

                        let source = match songbird::ffmpeg(&station.high_quality_stream_url).await {
                            Ok(source) => source,
                            Err(e) => {
                                println!("error starting ffmpeg: {:?}", e);
                                return
                            }
                        };

                        handler.lock().await.play_source(source);
                    },
                    None => {
                        let resp = message.channel_id.send_message(&_ctx.http, |m| {
                            m.content("Join a voice channel and try again.");
                            m
                        }).await;
                    }
                },
                None => {
                    let _ = message.channel_id.send_message(&_ctx.http, |m| {
                        m.content("Could not find the given station.");
                        m
                    }).await;
                }
            }
            return
        }

        let chunks = self.stations.chunks(25);

        for chunk in chunks {
            let resp = message.channel_id.send_message(&_ctx.http, |m| {
                m.embed(|e| {
                    e.title("Rova Stations");
                    e.description("Stations available on Rova");
                    for station in chunk {
                        e.field(
                            format!("{} - {}", station.brand_name, station.sort_name), 
                           format!("{} station {}", self.config.prefix, station.id),
                          false
                        );
                    }
                    e
                });
                m
            }).await;

            if let Err(e) = resp {
                println!("error sending station embed: {:?}", e);
            }
        }
    }

    async fn playing(&self, _ctx: Context, message: &Message, args: Option<Vec<&str>>) {
        println!("playing command");
        let on_air = self.on_air.lock().await;

        if let Some(args) = args {
            let station_id = args.first().unwrap();
            for station in on_air.iter() {
                if station.id.eq(station_id) {
                    let meta = self.fetch_station_by_id(station_id);

                    let resp = message.channel_id.send_message(&_ctx.http, |m| {
                        m.embed(|e| {
                            e.title(match meta {
                                Some(station) => station.brand_name.clone(),
                                None => station_id.to_string()
                            });
                            
                            for song in station.now_playing.iter() {
                                e.field(
                                    song.title.clone(),
                                    match &song.artist {
                                        Some(artist) => artist.to_string(),
                                        None => "No Artist".to_string()
                                    },
                                    false
                                );
                            }
                            e
                        });
                        m
                    }).await;

                    if let Err(e) = resp {
                        println!("error sending playing response: {:?}", e);
                    }

                    return
                }
            }
            
            let _ = message.channel_id.send_message(&_ctx.http, |m| {
                m.content("Could not find the given channel.");
                m
            }).await;
            return
        }

        for chunk in on_air.chunks(25) {
            let resp = message.channel_id.send_message( &_ctx.http, |m| {
                m.embed(|e| {
                    e.title("Rova Playing");
                    e.description("Currently playing on Rova");

                    for station in chunk {
                        if let Some(playing) = station.now_playing.first() {
                            e.field(
                                format!("{}", station.id),
                                format!("{} - {}", playing.title, match &playing.artist {
                                    Some(artist) => artist.to_string(),
                                    None => "No Artist".to_string()
                                }),
                                false
                            );
                        }
                    }
                    e
                });
                m
            }).await;

            if let Err(e) = resp {
                println!("error sending playing chunk: {:?}", e);
            }
        }
    }

    fn fetch_station_by_id(&self, id: &str) -> Option<&Station> {
        for station in self.stations.iter() {
            if station.id.eq(id) { return Some(station) }
        }
        None
    }

    async fn songbird(&self, ctx: &Context) -> Arc<songbird::Songbird> {
        songbird::get(ctx).await.unwrap().clone()
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