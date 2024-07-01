use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::api_client::SportsApiClient;
use crate::client::Client;
use crate::types::{CompetitorStats, SeasonCompetitors};

#[cfg(not(test))]
const CACHE_FOLDER: &str = ".matchday";
#[cfg(test)]
pub(crate) const CACHE_FOLDER: &str = ".tmp-cache-matchday";

pub struct CachedClient {
    api_client: Box<dyn Client>,
    base_path: PathBuf,
    competitors: Option<SeasonCompetitors>,
    stats: HashMap<PathBuf, CompetitorStats>,
}

impl CachedClient {
    pub fn new() -> Self {
        let base_path = Self::base_path();
        // we ignore the error if it already exists
        let _ = fs::create_dir(&base_path);

        let competitors = Self::read_competitors_file(&base_path);
        let stats = Self::read_stats_dir(&base_path);

        Self {
            api_client: Box::new(SportsApiClient::new()),
            base_path,
            competitors,
            stats,
        }
    }

    #[cfg(test)]
    fn set_client(&mut self, client: Box<dyn Client>) {
        self.api_client = client;
    }

    /// gets from loaded cache or fetches them and saves to cache
    pub async fn get_competitors(
        &mut self,
    ) -> Result<&SeasonCompetitors, Box<dyn std::error::Error>> {
        match self.competitors {
            Some(ref competitors) => Ok(competitors),
            None => {
                let competitors = self.api_client.fetch_competitors().await?;

                self.write_competitors_to_file(&competitors)?;

                self.competitors = Some(competitors);

                Ok(self.competitors.as_ref().unwrap())
            }
        }
    }

    pub async fn get_competitor_stats(
        &mut self,
        id: &str,
    ) -> Result<&CompetitorStats, Box<dyn std::error::Error>> {
        let stats_file = Self::stats_file(&self.base_path, id);
        if self.stats.contains_key(&stats_file) {
            return Ok(self.stats.get(&stats_file).unwrap());
        }

        let stats = self.api_client.fetch_competitor_stats(&id).await?;

        self.write_stats_to_file(&stats_file, &stats)?;

        self.stats.insert(stats_file.clone(), stats);

        Ok(self.stats.get(&stats_file).unwrap())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.competitors = None;
        self.stats.clear();
        fs::remove_dir_all(&self.base_path)?;
        Ok(())
    }

    // path methods
    fn base_path() -> PathBuf {
        // ideally could use lib to run consistently
        // on windows
        #[allow(deprecated)]
        let mut base_path = std::env::home_dir().expect("should have home dir");
        base_path.push(CACHE_FOLDER);
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
    fn write_competitors_to_file(&self, competitors: &SeasonCompetitors) -> io::Result<()> {
        let competitors_file = Self::competitors_file(&self.base_path);
        let _ = fs::File::create(&competitors_file);
        fs::write(
            &competitors_file,
            serde_json::to_string(&competitors)
                .expect("competitors should be serializable to JSON"),
        )?;
        Ok(())
    }
    fn write_stats_to_file(&self, stats_file: &PathBuf, stats: &CompetitorStats) -> io::Result<()> {
        let _ = fs::File::create(&stats_file);
        fs::write(
            &stats_file,
            serde_json::to_string(&stats).expect("stats should be serializable to JSON"),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use std::fs;
    use std::path::PathBuf;

    use crate::cached_client::{CachedClient, CACHE_FOLDER};
    use crate::client::Client;
    use crate::types::{
        CompetitorPlayers, CompetitorStats, Player, PlayerStats, SeasonCompetitor,
        SeasonCompetitors,
    };

    #[derive(Clone)]
    struct FakeClient {
        competitors: SeasonCompetitors,
        stats: CompetitorStats,
    }

    impl FakeClient {
        fn new() -> Self {
            Self {
                competitors: SeasonCompetitors {
                    season_competitors: vec![
                        SeasonCompetitor {
                            id: "sr:competitor:13".to_string(),
                        },
                        SeasonCompetitor {
                            id: "sr:competitor:17".to_string(),
                        },
                    ],
                },
                stats: CompetitorStats {
                    competitor: CompetitorPlayers {
                        players: vec![
                            Player {
                                id: "sr:player:1234".to_string(),
                                name: "Pelé".to_string(),
                                statistics: PlayerStats {
                                    assists: 100,
                                    goals_scored: 100,
                                },
                            },
                            Player {
                                id: "sr:player:256".to_string(),
                                name: "David Beckham".to_string(),
                                statistics: PlayerStats {
                                    assists: 50,
                                    goals_scored: 50,
                                },
                            },
                        ],
                    },
                },
            }
        }
    }

    #[async_trait]
    impl Client for FakeClient {
        async fn fetch_competitors(&self) -> Result<SeasonCompetitors, Box<dyn std::error::Error>> {
            Ok(self.competitors.clone())
        }
        async fn fetch_competitor_stats(
            &self,
            _id: &str,
        ) -> Result<CompetitorStats, Box<dyn std::error::Error>> {
            Ok(self.stats.clone())
        }
    }

    #[tokio::test]
    async fn test_fetching() {
        #[allow(deprecated)]
        let mut cache_dir = PathBuf::from(std::env::home_dir().unwrap());
        cache_dir.push(CACHE_FOLDER);
        let _ = fs::remove_dir_all(&cache_dir);

        let mut cached = CachedClient::new();
        let fake_client = FakeClient::new();
        cached.set_client(Box::new(fake_client.clone()));

        let competitors = cached.get_competitors().await.unwrap();
        assert_eq!(competitors, &fake_client.competitors);

        let stats = cached.get_competitor_stats("not used").await.unwrap();
        assert_eq!(stats, &fake_client.stats);

        let _ = fs::remove_dir_all(&cache_dir);
    }
}
