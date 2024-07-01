use clap::Parser;

use crate::cached_client::CachedClient;
use crate::top_players::TopPlayers;
use crate::types::Player;

#[derive(Parser, Debug)]
pub enum Cmd {
    TopAssists,
    TopGoals,
    TopPlayers,
    ClearCache,
}

async fn load_players(mut cache: CachedClient) -> anyhow::Result<Vec<Player>> {
    println!("fetching season data...");
    let competitors = cache.get_competitors().await?;
    let mut players = Vec::with_capacity(20 * 28);

    for competitor in competitors.season_competitors.clone() {
        let stats = cache.get_competitor_stats(&competitor.id).await?;
        players.extend(stats.competitor.players.clone());
    }

    Ok(players)
}

fn top_assists(top_players: TopPlayers) {
    println!("Assists | Player Name");
    for player in top_players.by_assists() {
        println!(" {} | {}", player.statistics.assists, player.name);
    }
}

fn top_goals(top_players: TopPlayers) {
    println!("Goals | Player Name");
    for player in top_players.by_goals() {
        println!(" {} | {}", player.statistics.goals_scored, player.name);
    }
}

fn top_players(top_players: TopPlayers) {
    println!("Goals | Assists | Player Name");
    for player in top_players.by_both() {
        println!(
            " {} | {} | {}",
            player.statistics.goals_scored, player.statistics.assists, player.name
        );
    }
}

impl Cmd {
    pub async fn run(self) -> anyhow::Result<()> {
        let mut cache = CachedClient::new()?;

        match self {
            Cmd::TopAssists => {
                let players = load_players(cache).await?;
                top_assists(TopPlayers::new(players));
            }
            Cmd::TopGoals => {
                let players = load_players(cache).await?;
                top_goals(TopPlayers::new(players));
            }
            Cmd::TopPlayers => {
                let players = load_players(cache).await?;
                top_players(TopPlayers::new(players));
            }
            Cmd::ClearCache => {
                println!("deleting season data");
                cache.clear()?;
            }
        }

        Ok(())
    }
}
