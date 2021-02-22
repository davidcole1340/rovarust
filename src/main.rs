pub mod rova;
pub mod bot;
pub mod config;

use bot::Handler;
use config::Config;
use rova::Rova;
use serenity::client::Client;
use tokio::{time::sleep, sync::Mutex};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    let config = Config::load("config.toml")
        .expect("unable to load config file");
    let token = config.token.clone();

    let streams = Rova::get_stream_info()
        .await
        .expect("error retrieving station info from rova");

    let on_air: Arc<Mutex<Vec<crate::rova::OnAir>>> = Arc::new(Mutex::new(vec![]));
    let on_air_loop = on_air.clone();
    let on_air_handler = on_air.clone();

    let handler = Handler {
        config: config,
        stations: streams.stations,
        on_air: on_air_handler
    };

    tokio::spawn(async move {
        loop {
            let response = Rova::get_on_air_info().await;

            match response {
                Ok(resp) => {
                    let mut write_guard = on_air_loop.lock().await;
                    write_guard.clear();

                    for station in resp.stations {
                        write_guard.push(station);
                    }
                },
                Err(e) => println!("error retrieving on-air information: {:?}", e)
            };

            sleep(Duration::from_secs(60)).await;
        }
    });

    let mut client = Client::builder(&token)
        .event_handler(handler)
        .await
        .expect("error creating client");

    if let Err(e) = client.start().await {
        println!("client error: {}", e);
    }
}