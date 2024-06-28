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
struct PlayerStats {
    assists: usize,
    goals_scored: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    id: String,
    statistics: PlayerStats,
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

async fn fetch_competitor_stats(
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

async fn fetch_competitors(
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

use std::fs;
use std::path::PathBuf;

struct Cache {
    base_path: PathBuf,
    competitors: Option<SeasonCompetitors>,
}

impl Cache {
    fn new() -> Self {
        // ideally could use lib to run consistently
        // on windows
        #[allow(deprecated)]
        let mut base_dir = std::env::home_dir().expect("should have home dir");
        base_dir.push(".matchday");
        // we ignore the error if it already exists
        let _ = fs::create_dir(&base_dir);

        let mut competitors_file = base_dir.clone();
        competitors_file.push("competitors.json");

        let competitors = if competitors_file.exists() {
            let raw_competitors = fs::read_to_string(competitors_file).unwrap();
            Some(
                serde_json::from_str(&raw_competitors)
                    .expect("competitors cache should be valid JSON"),
            )
        } else {
            None
        };

        Self {
            base_path: base_dir,
            competitors,
        }
    }

    /// gets from loaded cache or fetches them and saves to cache
    async fn get_competitors(
        &mut self,
        client: &reqwest::Client,
    ) -> Result<&SeasonCompetitors, Box<dyn std::error::Error>> {
        match self.competitors {
            Some(ref competitors) => Ok(competitors),
            None => {
                let retry_strategy = ExponentialBackoff::from_millis(500).map(jitter).take(3);
                let competitors =
                    Retry::spawn(retry_strategy, || fetch_competitors(client)).await?;

                let mut competitors_file = self.base_path.clone();
                competitors_file.push("competitors.json");

                let _ = fs::File::create(&competitors_file);
                fs::write(
                    &competitors_file,
                    serde_json::to_string(&competitors)
                        .expect("competitors should be serializable to JSON"),
                )?;

                self.competitors = Some(competitors);

                Ok(self.competitors.as_ref().unwrap())
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut cache = Cache::new();

    let client = build_client();
    let retry_strategy = ExponentialBackoff::from_millis(500).map(jitter).take(3);

    let competitors = cache.get_competitors(&client).await?;
    // println!("{competitors:#?}");

    for competitor in &competitors.season_competitors {
        println!("fetching for id: {}", competitor.id);
        let competitor_stats = Retry::spawn(retry_strategy.clone(), || {
            fetch_competitor_stats(&client, &competitor.id)
        })
        .await?;

        println!("{competitor_stats:#?}");
    }
    Ok(())
}
