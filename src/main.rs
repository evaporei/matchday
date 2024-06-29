use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct PlayerStats {
    assists: usize,
    goals_scored: usize,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct Player {
    id: String,
    name: String,
    statistics: PlayerStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CompetitorPlayers {
    players: Vec<Player>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    async fn fetch_competitors_with_retry(
        &self,
    ) -> Result<SeasonCompetitors, Box<dyn std::error::Error>> {
        let strategy = ExponentialBackoff::from_millis(100)
            .clone()
            .map(jitter)
            .take(3);

        Ok(Retry::spawn(strategy, || self.fetch_competitors()).await?)
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

    async fn fetch_competitor_stats_with_retry(
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

use std::fs;
use std::path::PathBuf;

struct CachedClient {
    api_client: SportsApiClient,
    base_path: PathBuf,
    competitors: Option<SeasonCompetitors>,
    stats: HashMap<PathBuf, CompetitorStats>,
}

impl CachedClient {
    fn new(client: SportsApiClient) -> Self {
        let base_path = Self::base_path();
        // we ignore the error if it already exists
        let _ = fs::create_dir(&base_path);

        let competitors = Self::read_competitors_file(&base_path);
        let stats = Self::read_stats_dir(&base_path);

        Self {
            api_client: client,
            base_path,
            competitors,
            stats,
        }
    }

    /// gets from loaded cache or fetches them and saves to cache
    async fn get_competitors(&mut self) -> Result<&SeasonCompetitors, Box<dyn std::error::Error>> {
        match self.competitors {
            Some(ref competitors) => Ok(competitors),
            None => {
                let competitors = self.api_client.fetch_competitors_with_retry().await?;

                self.write_competitors_to_file(&competitors)?;

                self.competitors = Some(competitors);

                Ok(self.competitors.as_ref().unwrap())
            }
        }
    }

    async fn get_competitor_stats(
        &mut self,
        id: &str,
    ) -> Result<&CompetitorStats, Box<dyn std::error::Error>> {
        let stats_file = Self::stats_file(&self.base_path, id);
        if self.stats.contains_key(&stats_file) {
            return Ok(self.stats.get(&stats_file).unwrap());
        }

        let stats = self
            .api_client
            .fetch_competitor_stats_with_retry(&id)
            .await?;

        self.write_stats_to_file(&stats_file, &stats)?;

        self.stats.insert(stats_file.clone(), stats);

        Ok(self.stats.get(&stats_file).unwrap())
    }

    // path methods
    fn base_path() -> PathBuf {
        // ideally could use lib to run consistently
        // on windows
        #[allow(deprecated)]
        let mut base_path = std::env::home_dir().expect("should have home dir");
        base_path.push(".matchday");
        base_path
    }
    fn competitors_file(base_path: &PathBuf) -> PathBuf {
        let mut competitors_file = base_path.clone();
        competitors_file.push("competitors.json");
        competitors_file
    }
    fn stats_dir(base_path: &PathBuf) -> PathBuf {
        let mut stats_dir = base_path.clone();
        stats_dir.push("stats");
        stats_dir
    }
    fn stats_file(base_path: &PathBuf, id: &str) -> PathBuf {
        let mut stats_file = Self::stats_dir(base_path);
        stats_file.push(id);
        stats_file.set_extension("json");
        stats_file
    }

    // fs methods
    fn read_competitors_file(base_path: &PathBuf) -> Option<SeasonCompetitors> {
        let competitors_file = Self::competitors_file(&base_path);
        if competitors_file.exists() {
            let raw_competitors = fs::read_to_string(competitors_file).unwrap();
            Some(
                serde_json::from_str(&raw_competitors)
                    .expect("competitors cache should be valid JSON"),
            )
        } else {
            None
        }
    }
    fn read_stats_dir(base_path: &PathBuf) -> HashMap<PathBuf, CompetitorStats> {
        let mut stats = HashMap::new();
        let stats_dir = Self::stats_dir(&base_path);
        if stats_dir.exists() {
            for entry in fs::read_dir(stats_dir).expect("stats should be a valid dir") {
                let file = entry.unwrap();
                let raw_stat = fs::read_to_string(&file.path()).unwrap();
                let stat = serde_json::from_str(&raw_stat)
                    .expect("competitors cache should be valid JSON");
                stats.insert(file.path(), stat);
            }
        } else {
            // we ignore the error if it already exists
            let _ = fs::create_dir(&stats_dir);
        };
        stats
    }
    fn write_competitors_to_file(&self, competitors: &SeasonCompetitors) -> Result<(), std::io::Error> {
        let competitors_file = Self::competitors_file(&self.base_path);
        let _ = fs::File::create(&competitors_file);
        fs::write(
            &competitors_file,
            serde_json::to_string(&competitors)
                .expect("competitors should be serializable to JSON"),
        )?;
        Ok(())
    }
    fn write_stats_to_file(&self, stats_file: &PathBuf, stats: &CompetitorStats) -> Result<(), std::io::Error> {
        let _ = fs::File::create(&stats_file);
        fs::write(
            &stats_file,
            serde_json::to_string(&stats).expect("stats should be serializable to JSON"),
        )?;
        Ok(())
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
enum Cmd {
    TopAssists,
    TopGoals,
    TopPlayers,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cmd = Cmd::parse();

    let client = SportsApiClient::new();

    let mut cache = CachedClient::new(client);

    println!("fetching season data...");
    let competitors = cache.get_competitors().await?;
    let mut players_stats = Vec::with_capacity(20 * 28);

    for competitor in competitors.season_competitors.clone() {
        let stats = cache.get_competitor_stats(&competitor.id).await?;
        players_stats.extend(stats.competitor.players.clone());
    }

    match cmd {
        Cmd::TopPlayers => {
            let mut set = HashSet::new();

            let mut ord_goals = players_stats.clone();
            let mut ord_assists = players_stats.clone();

            ord_goals.sort_by_key(|p| p.statistics.goals_scored);
            for player in ord_goals.iter().rev().take(10) {
                set.insert(player);
            }

            ord_assists.sort_by_key(|p| p.statistics.assists);
            for player in ord_assists.iter().rev().take(10) {
                set.insert(player);
            }

            println!("Goals | Assists | Player Name");
            for player in set {
                println!(
                    " {} | {} | {}",
                    player.statistics.goals_scored, player.statistics.assists, player.name
                );
            }
        }
        Cmd::TopAssists => {
            players_stats.sort_by_key(|p| p.statistics.assists);
            println!("Assists | Player Name");
            for player in players_stats.iter().rev().take(10) {
                println!(" {} | {}", player.statistics.assists, player.name);
            }
        }
        Cmd::TopGoals => {
            players_stats.sort_by_key(|p| p.statistics.goals_scored);
            println!("Goals | Player Name");
            for player in players_stats.iter().rev().take(10) {
                println!(" {} | {}", player.statistics.goals_scored, player.name);
            }
        }
    }

    Ok(())
}
