use std::env;
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Serialize, Deserialize};

const COMPETITOR_STATS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/sr%3Aseason%3A93741/competitors/sr%3Acompetitor%3A17/statistics.json";

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("SPORTRADAR_API_KEY")
        .expect("SPORTRADAR_API_KEY env var is not set");

    let url = format!("{COMPETITOR_STATS_URL}?api_key={api_key}");

    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/json"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let resp = client.get(url)
        .send()
        .await?
        .json::<CompetitorStats>()
        .await?;
    println!("{resp:#?}");
    Ok(())
}
