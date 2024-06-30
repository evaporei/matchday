use crate::api_client::Player;

pub struct TopPlayers(Vec<Player>);

impl TopPlayers {
    pub fn new(players: Vec<Player>) -> Self {
        Self(players)
    }
    pub fn by_assists(mut self) -> impl Iterator<Item = Player> {
        self.0.sort_by_key(|p| p.statistics.assists);
        self.0.into_iter().rev().take(10)
    }
    pub fn by_goals(mut self) -> impl Iterator<Item = Player> {
        self.0.sort_by_key(|p| p.statistics.goals_scored);
        self.0.into_iter().rev().take(10)
    }
    pub fn by_both(mut self) -> impl Iterator<Item = Player> {
        self.0
            .sort_unstable_by_key(|p| (p.statistics.goals_scored, p.statistics.assists));
        self.0.into_iter().rev().take(10)
    }
}
