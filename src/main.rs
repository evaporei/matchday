use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;

const SEASON_23_24_ID: &str = "sr:season:105353";

const COMPETITOR_STATS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors/$COMPETITOR/statistics.json?api_key=$API_KEY";

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Competitor {
    players: Vec<Player>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompetitorStats {
    competitor: Competitor,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client = build_client();

    let competitor_stats = get_competitor_stats(&client, "sr:competitor:17").await?;

    println!("{competitor_stats:#?}");
    Ok(())
}
