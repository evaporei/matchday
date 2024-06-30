use std::env;

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

const SEASON_23_24_ID: &str = "sr:season:105353";

const SEASON_COMPETITORS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors.json?api_key=$API_KEY";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeasonCompetitors {
    pub season_competitors: Vec<SeasonCompetitor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeasonCompetitor {
    pub id: String,
}

const COMPETITOR_STATS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors/$COMPETITOR/statistics.json?api_key=$API_KEY";

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub assists: usize,
    pub goals_scored: usize,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Player {
    id: String,
    pub name: String,
    pub statistics: PlayerStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompetitorPlayers {
    pub players: Vec<Player>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompetitorStats {
    pub competitor: CompetitorPlayers,
}

pub struct SportsApiClient {
    client: reqwest::Client,
    api_key: String,
}

impl SportsApiClient {
    // requires SPORTRADAD_API_KEY env var
    // can use dotenv
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json"));

        Self {
            api_key: env::var("SPORTRADAR_API_KEY").expect("SPORTRADAR_API_KEY env var is not set"),
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_competitors(&self) -> Result<SeasonCompetitors, Box<dyn std::error::Error>> {
        let url = SEASON_COMPETITORS_URL
            .replace("$SEASON", SEASON_23_24_ID)
            .replace("$API_KEY", &self.api_key);

        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .json::<SeasonCompetitors>()
            .await?)
    }

    pub async fn fetch_competitors_with_retry(
        &self,
    ) -> Result<SeasonCompetitors, Box<dyn std::error::Error>> {
        let strategy = ExponentialBackoff::from_millis(100)
            .clone()
            .map(jitter)
            .take(3);

        Ok(Retry::spawn(strategy, || self.fetch_competitors()).await?)
    }

    pub async fn fetch_competitor_stats(
        &self,
        id: &str,
    ) -> Result<CompetitorStats, Box<dyn std::error::Error>> {
        let url = COMPETITOR_STATS_URL
            .replace("$SEASON", SEASON_23_24_ID)
            .replace("$COMPETITOR", id)
            .replace("$API_KEY", &self.api_key);

        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .json::<CompetitorStats>()
            .await?)
    }

    pub async fn fetch_competitor_stats_with_retry(
        &self,
        id: &str,
    ) -> Result<CompetitorStats, Box<dyn std::error::Error>> {
        let strategy = ExponentialBackoff::from_millis(100)
            .clone()
            .map(jitter)
            .take(3);

        Ok(Retry::spawn(strategy, || self.fetch_competitor_stats(id)).await?)
    }
}
