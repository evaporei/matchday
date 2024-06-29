use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

const SEASON_23_24_ID: &str = "sr:season:105353";

const SEASON_COMPETITORS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors.json?api_key=$API_KEY";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SeasonCompetitors {
    season_competitors: Vec<SeasonCompetitor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

struct SportsApiClient {
    client: reqwest::Client,
    api_key: String,
}

impl SportsApiClient {
    // requires SPORTRADAD_API_KEY env var
    // can use dotenv
    fn new() -> Self {
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

    async fn fetch_competitors(&self) -> Result<SeasonCompetitors, Box<dyn std::error::Error>> {
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

    async fn fetch_competitor_stats(
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
}

use std::fs;
use std::path::PathBuf;

struct Cache {
    api_client: SportsApiClient,
    base_path: PathBuf,
    competitors: Option<SeasonCompetitors>,
    stats: HashMap<PathBuf, CompetitorStats>,
}

impl Cache {
    fn new(client: SportsApiClient) -> Self {
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

        let mut stats_dir = base_dir.clone();
        stats_dir.push("stats");

        let mut stats = HashMap::new();
        if stats_dir.exists() {
            for entry in fs::read_dir(stats_dir).expect("stats should be a valid dir") {
                let file = entry.unwrap();
                let mut file_name = file.path();
                file_name.set_extension("json");
                let raw_stat = fs::read_to_string(&file_name).unwrap();
                let stat = serde_json::from_str(&raw_stat)
                    .expect("competitors cache should be valid JSON");
                stats.insert(file_name, stat);
            }
        } else {
            // we ignore the error if it already exists
            let _ = fs::create_dir(&stats_dir);
        };

        Self {
            api_client: client,
            base_path: base_dir,
            competitors,
            stats,
        }
    }

    /// gets from loaded cache or fetches them and saves to cache
    async fn get_competitors(&mut self) -> Result<SeasonCompetitors, Box<dyn std::error::Error>> {
        match self.competitors {
            Some(ref competitors) => Ok(competitors.clone()),
            None => {
                let retry_strategy = ExponentialBackoff::from_millis(500).map(jitter).take(3);
                let competitors =
                    Retry::spawn(retry_strategy, || self.api_client.fetch_competitors()).await?;

                let mut competitors_file = self.base_path.clone();
                competitors_file.push("competitors.json");

                let _ = fs::File::create(&competitors_file);
                fs::write(
                    &competitors_file,
                    serde_json::to_string(&competitors)
                        .expect("competitors should be serializable to JSON"),
                )?;

                self.competitors = Some(competitors);

                Ok(self.competitors.as_ref().unwrap().clone())
            }
        }
    }

    async fn get_competitor_stats(
        &mut self,
        id: &str,
    ) -> Result<&CompetitorStats, Box<dyn std::error::Error>> {
        let mut stats_file = self.base_path.clone();
        stats_file.push("stats");
        stats_file.push(id);
        stats_file.set_extension("json");
        if self.stats.contains_key(&stats_file) {
            return Ok(self.stats.get(&stats_file).unwrap());
        }
        let retry_strategy = ExponentialBackoff::from_millis(500).map(jitter).take(3);

        let stats = Retry::spawn(retry_strategy, || {
            self.api_client.fetch_competitor_stats(&id)
        })
        .await?;

        let _ = fs::File::create(&stats_file);

        fs::write(
            &stats_file,
            serde_json::to_string(&stats).expect("stats should be serializable to JSON"),
        )?;

        self.stats.insert(stats_file.clone(), stats);

        Ok(self.stats.get(&stats_file).unwrap())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client = SportsApiClient::new();

    let mut cache = Cache::new(client);

    let competitors = cache.get_competitors().await?;
    // println!("{competitors:#?}");

    for competitor in competitors.season_competitors {
        println!("fetching for id: {}", competitor.id);

        let competitor_stats = cache.get_competitor_stats(&competitor.id).await?;

        println!("{competitor_stats:#?}");
    }
    Ok(())
}
