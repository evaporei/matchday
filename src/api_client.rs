use std::env;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

use crate::client::Client;
use crate::error::{EnvVarError, Error};
use crate::types::*;

const SEASON_23_24_ID: &str = "sr:season:105353";

#[cfg(not(test))]
const SEASON_COMPETITORS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors.json?api_key=$API_KEY";

#[cfg(not(test))]
const COMPETITOR_STATS_URL: &str = "https://api.sportradar.com/soccer/trial/v4/en/seasons/$SEASON/competitors/$COMPETITOR/statistics.json?api_key=$API_KEY";

pub struct SportsApiClient {
    client: reqwest::Client,
    api_key: String,

    #[cfg(test)]
    pub(crate) mock_url: Option<String>,
}

impl SportsApiClient {
    // requires SPORTRADAR_API_KEY env var
    // can use dotenv
    pub fn new() -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json"));

        Ok(Self {
            api_key: env::var("SPORTRADAR_API_KEY")
                .map_err(|e| EnvVarError::new("SPORTRADAR_API_KEY", e))?,
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            #[cfg(test)]
            mock_url: None,
        })
    }

    #[cfg(test)]
    fn set_mock_url(&mut self, url: String) {
        self.mock_url = Some(url);
    }

    async fn competitors(&self) -> Result<SeasonCompetitors, Error> {
        #[cfg(not(test))]
        let base_url = SEASON_COMPETITORS_URL;
        #[cfg(test)]
        let base_url = self.mock_url.as_ref().unwrap();
        let url = base_url
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

    async fn competitor_stats(&self, id: &str) -> Result<CompetitorStats, Error> {
        #[cfg(not(test))]
        let base_url = COMPETITOR_STATS_URL;
        #[cfg(test)]
        let base_url = self.mock_url.as_ref().unwrap();
        let url = base_url
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

fn retry_strategy() -> impl Iterator<Item = Duration> {
    ExponentialBackoff::from_millis(20).map(jitter).take(3)
}

#[async_trait]
impl Client for SportsApiClient {
    async fn fetch_competitors(&self) -> Result<SeasonCompetitors, Error> {
        Retry::spawn(retry_strategy(), || self.competitors()).await
    }
    async fn fetch_competitor_stats(&self, id: &str) -> Result<CompetitorStats, Error> {
        Retry::spawn(retry_strategy(), || self.competitor_stats(id)).await
    }
}

#[cfg(test)]
mod test {
    use crate::api_client::SportsApiClient;
    use crate::client::Client;
    use crate::types::{
        CompetitorPlayers, CompetitorStats, Player, PlayerStats, SeasonCompetitor,
        SeasonCompetitors,
    };

    #[tokio::test]
    async fn test_fetch_competitors() {
        dotenv::from_filename(".env.example").ok();
        let mut client = SportsApiClient::new().unwrap();

        let mut server = mockito::Server::new_async().await;

        let base_url = server.url();
        let route = "/soccer/trial/v4/en/seasons/$SEASON/competitors.json?api_key=$API_KEY";
        client.set_mock_url(format!("{base_url}{route}"));

        let json = r###"
            {
              "generated_at": "2024-06-28T16:18:14+00:00",
              "season_competitors": [
                {
                  "id": "sr:competitor:3",
                  "name": "Wolverhampton Wanderers",
                  "short_name": "Wolverhampton",
                  "abbreviation": "WOL"
                },
                {
                  "id": "sr:competitor:6",
                  "name": "Burnley FC",
                  "short_name": "Burnley",
                  "abbreviation": "BUR"
                }
              ]
            }
        "###;

        let mock = server
            .mock(
                "GET",
                "/soccer/trial/v4/en/seasons/sr:season:105353/competitors.json?api_key=asdf1234",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json)
            .create_async()
            .await;

        let competitors = client.fetch_competitors().await.unwrap();
        mock.assert();

        assert_eq!(
            competitors,
            SeasonCompetitors {
                season_competitors: vec![
                    SeasonCompetitor {
                        id: "sr:competitor:3".into(),
                    },
                    SeasonCompetitor {
                        id: "sr:competitor:6".into(),
                    },
                ],
            }
        );
    }

    #[tokio::test]
    async fn test_fetch_competitor_stats() {
        dotenv::from_filename(".env.example").ok();
        let mut client = SportsApiClient::new().unwrap();

        let mut server = mockito::Server::new_async().await;

        let base_url = server.url();
        let route = "/soccer/trial/v4/en/seasons/$SEASON/competitors/$COMPETITOR/statistics.json?api_key=$API_KEY";
        client.set_mock_url(format!("{base_url}{route}"));

        let json = r###"
           {
                "generated_at": "2024-06-30T20:46:17+00:00",
                "season": {
                  "id": "sr:season:105353",
                  "name": "Premier League 23/24",
                  "start_date": "2023-08-11",
                  "end_date": "2024-05-19",
                  "year": "23/24",
                  "competition_id": "sr:competition:17",
                  "sport": {
                    "id": "sr:sport:1",
                    "name": "Soccer"
                  }
                },
                "competitor": {
                  "id": "sr:competitor:17",
                  "name": "Manchester City",
                  "country": "England",
                  "country_code": "ENG",
                  "abbreviation": "MCI",
                  "gender": "male",
                  "statistics": {
                    "average_ball_possession": 65.53,
                    "cards_given": 55,
                    "corner_kicks": 286,
                    "free_kicks": 501,
                    "goals_by_foot": 83,
                    "goals_by_head": 11,
                    "goals_conceded": 34,
                    "goals_conceded_first_half": 16,
                    "goals_conceded_second_half": 18,
                    "goals_scored": 96,
                    "goals_scored_first_half": 40,
                    "goals_scored_second_half": 56,
                    "matches_played": 38,
                    "offsides": 42,
                    "penalties_missed": 1,
                    "red_cards": 1,
                    "shots_blocked": 177,
                    "shots_off_target": 193,
                    "shots_on_bar": 3,
                    "shots_on_post": 3,
                    "shots_on_target": 261,
                    "shots_total": 631,
                    "yellow_cards": 53,
                    "yellow_red_cards": 1
                  },
                  "players": [
                    {
                      "id": "sr:player:44614",
                      "name": "Walker, Kyle",
                      "statistics": {
                        "assists": 4,
                        "cards_given": 2,
                        "goals_by_head": 0,
                        "goals_by_penalty": 0,
                        "goals_conceded": 29,
                        "goals_scored": 0,
                        "matches_played": 32,
                        "offsides": 7,
                        "own_goals": 0,
                        "penalties_missed": 0,
                        "red_cards": 0,
                        "shots_blocked": 7,
                        "shots_off_target": 6,
                        "shots_on_target": 3,
                        "substituted_in": 2,
                        "substituted_out": 3,
                        "yellow_cards": 2,
                        "yellow_red_cards": 0
                      }
                    },
                    {
                      "id": "sr:player:70996",
                      "name": "De Bruyne, Kevin",
                      "statistics": {
                        "assists": 10,
                        "cards_given": 2,
                        "corner_kicks": 66,
                        "goals_by_head": 1,
                        "goals_by_penalty": 0,
                        "goals_conceded": 8,
                        "goals_scored": 4,
                        "matches_played": 18,
                        "offsides": 1,
                        "own_goals": 0,
                        "penalties_missed": 0,
                        "red_cards": 0,
                        "shots_blocked": 14,
                        "shots_off_target": 13,
                        "shots_on_target": 14,
                        "substituted_in": 3,
                        "substituted_out": 10,
                        "yellow_cards": 2,
                        "yellow_red_cards": 0
                      }
                    }
                  ]
                }
           }
        "###;

        let mock = server.mock("GET", "/soccer/trial/v4/en/seasons/sr:season:105353/competitors/sr:competitor:17/statistics.json?api_key=asdf1234")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json)
            .create_async().await;

        let competitors = client
            .fetch_competitor_stats("sr:competitor:17")
            .await
            .unwrap();
        mock.assert();

        assert_eq!(
            competitors,
            CompetitorStats {
                competitor: CompetitorPlayers {
                    players: vec![
                        Player {
                            id: "sr:player:44614".into(),
                            name: "Walker, Kyle".into(),
                            statistics: PlayerStats {
                                assists: 4,
                                goals_scored: 0
                            }
                        },
                        Player {
                            id: "sr:player:70996".into(),
                            name: "De Bruyne, Kevin".into(),
                            statistics: PlayerStats {
                                assists: 10,
                                goals_scored: 4
                            }
                        }
                    ],
                },
            }
        );
    }
}
