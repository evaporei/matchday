use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

const SEASON_23_24_ID: &str = "sr:season:105353";

const SEASON_COMPETITORS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors.json?api_key=$API_KEY";

#[derive(Debug, Serialize, Deserialize)]
struct SeasonCompetitors {
    season_competitors: Vec<SeasonCompetitor>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SeasonCompetitor {
    id: String,
}

const COMPETITOR_STATS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors/$COMPETITOR/statistics.json?api_key=$API_KEY";

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompetitorPlayers {
    players: Vec<Player>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompetitorStats {
    competitor: CompetitorPlayers,
}

fn build_client() -> reqwest::Client {
    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/json"));

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

async fn get_competitor_stats(
    client: &reqwest::Client,
    id: &str,
) -> Result<CompetitorStats, Box<dyn std::error::Error>> {
    let api_key = env::var("SPORTRADAR_API_KEY").expect("SPORTRADAR_API_KEY env var is not set");

    let url = COMPETITOR_STATS_URL
        .replace("$SEASON", SEASON_23_24_ID)
        .replace("$COMPETITOR", id)
        .replace("$API_KEY", &api_key);

    Ok(client
        .get(url)
        .send()
        .await?
        .json::<CompetitorStats>()
        .await?)
}

async fn get_competitors(
    client: &reqwest::Client,
) -> Result<SeasonCompetitors, Box<dyn std::error::Error>> {
    let api_key = env::var("SPORTRADAR_API_KEY").expect("SPORTRADAR_API_KEY env var is not set");

    let url = SEASON_COMPETITORS_URL
        .replace("$SEASON", SEASON_23_24_ID)
        .replace("$API_KEY", &api_key);

    Ok(client
        .get(url)
        .send()
        .await?
        .json::<SeasonCompetitors>()
        .await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client = build_client();

    let retry_strategy = ExponentialBackoff::from_millis(500).map(jitter).take(3);

    let competitors = Retry::spawn(retry_strategy.clone(), || get_competitors(&client)).await?;
    // println!("{competitors:#?}");

    for competitor in competitors.season_competitors {
        println!("fetching for id: {}", competitor.id);
        let competitor_stats = Retry::spawn(retry_strategy.clone(), || {
            get_competitor_stats(&client, &competitor.id)
        })
        .await?;

        println!("{competitor_stats:#?}");
    }
    Ok(())
}
