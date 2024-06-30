use dotenv::dotenv;

use matchday::api_client::{Player, SportsApiClient};
use matchday::cached_client::CachedClient;

use clap::Parser;

#[derive(Parser, Debug)]
enum Cmd {
    TopAssists,
    TopGoals,
    TopPlayers,
    ClearCache,
}

struct TopPlayers(Vec<Player>);

impl TopPlayers {
    fn new(players: Vec<Player>) -> Self {
        Self(players)
    }
    fn by_assists(mut self) -> impl Iterator<Item = Player> {
        self.0.sort_by_key(|p| p.statistics.assists);
        self.0.into_iter().rev().take(10)
    }
    fn by_goals(mut self) -> impl Iterator<Item = Player> {
        self.0.sort_by_key(|p| p.statistics.goals_scored);
        self.0.into_iter().rev().take(10)
    }
    fn by_both(mut self) -> impl Iterator<Item = Player> {
        self.0
            .sort_unstable_by_key(|p| (p.statistics.goals_scored, p.statistics.assists));
        self.0.into_iter().rev().take(10)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cmd = Cmd::parse();

    let client = SportsApiClient::new();

    let mut cache = CachedClient::new(client);

    println!("fetching season data...");
    let competitors = cache.get_competitors().await?;
    let mut players = Vec::with_capacity(20 * 28);

    for competitor in competitors.season_competitors.clone() {
        let stats = cache.get_competitor_stats(&competitor.id).await?;
        players.extend(stats.competitor.players.clone());
    }

    let top_players = TopPlayers::new(players);
    match cmd {
        Cmd::TopAssists => {
            println!("Assists | Player Name");
            for player in top_players.by_assists() {
                println!(" {} | {}", player.statistics.assists, player.name);
            }
        }
        Cmd::TopGoals => {
            println!("Goals | Player Name");
            for player in top_players.by_goals() {
                println!(" {} | {}", player.statistics.goals_scored, player.name);
            }
        }
        Cmd::TopPlayers => {
            println!("Goals | Assists | Player Name");
            for player in top_players.by_both() {
                println!(
                    " {} | {} | {}",
                    player.statistics.goals_scored, player.statistics.assists, player.name
                );
            }
        }
        Cmd::ClearCache => {
            println!("deleting season data");
            cache.clear()?;
        }
    }

    Ok(())
}
