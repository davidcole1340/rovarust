use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub name: Option<String>,
    pub id: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    pub id: String,
    pub sort_name: String,
    pub brand_name: String,
    pub high_quality_stream_url: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    pub title: String,
    pub status: String,
    pub image_url: String,
    pub duration: String,
    pub artist: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnAirNow {
    pub title: String,
    pub status: String,
    pub image_url: String,
    pub duration: String,
    pub artist: String,
    pub start_time: String,
    pub end_time: String,
    pub show_id: String,
    pub display_time: String,
    pub thumbnail_url: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnAir {
    pub id: String,
    pub now_playing: Vec<NowPlaying>,
    pub on_air: Vec<OnAirNow>,
    pub source: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationResponse {
    pub region: Region,
    pub stations: Vec<Station>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnAirResponse {
    pub stations: Vec<OnAir>
}

const GET_STATIONS: &str = "https://fred.aimapi.io/services/station/rova?region=Auckland";
const GET_ON_AIR: &str = "https://bruce.radioapi.io/services/onair/rova?region=Auckland";

pub struct Rova {}

impl Rova {
    pub async fn get_stream_info() -> Result<StationResponse, reqwest::Error> {
        reqwest::get(GET_STATIONS)
            .await?
            .json::<StationResponse>()
            .await
    }

    pub async fn get_on_air_info() -> Result<OnAirResponse, reqwest::Error> {
        reqwest::get(GET_ON_AIR)
            .await?
            .json::<OnAirResponse>()
            .await
    }
}