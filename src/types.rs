use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonCompetitors {
    pub season_competitors: Vec<SeasonCompetitor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonCompetitor {
    pub id: String,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub assists: usize,
    pub goals_scored: usize,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub statistics: PlayerStats,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CompetitorPlayers {
    pub players: Vec<Player>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CompetitorStats {
    pub competitor: CompetitorPlayers,
}
