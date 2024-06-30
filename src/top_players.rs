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

#[cfg(test)]
mod test {
    use crate::api_client::{Player, PlayerStats};
    use crate::top_players::TopPlayers;

    type Goals = usize;
    type Assists = usize;
    type Stat = (Goals, Assists);

    fn mock_players(stats: &Vec<Stat>) -> Vec<Player> {
        stats
            .iter()
            .map(|(goals, assists)| Player {
                id: Default::default(),
                name: Default::default(),
                statistics: PlayerStats {
                    assists: *assists,
                    goals_scored: *goals,
                },
            })
            .collect()
    }

    #[test]
    fn test_by_goals() {
        // 11 players
        let stats = vec![
            (8, 0),
            (3, 0),
            (5, 0),
            (2, 0),
            (10, 0),
            (11, 0),
            (3, 0),
            (4, 0),
            (12, 0),
            (22, 0),
            (1, 0),
        ];
        let players = mock_players(&stats);
        let top = TopPlayers::new(players);
        let expected = vec![
            (22, 0),
            (12, 0),
            (11, 0),
            (10, 0),
            (8, 0),
            (5, 0),
            (4, 0),
            (3, 0),
            (3, 0),
            (2, 0),
        ];
        assert_eq!(
            top.by_goals()
                .map(|p| (p.statistics.goals_scored, p.statistics.assists))
                .collect::<Vec<Stat>>(),
            expected
        );
    }

    #[test]
    fn test_by_assists() {
        // 11 players
        let stats = vec![
            (0, 8),
            (0, 3),
            (0, 5),
            (0, 2),
            (0, 10),
            (0, 11),
            (0, 3),
            (0, 4),
            (0, 12),
            (0, 22),
            (0, 1),
        ];
        let players = mock_players(&stats);
        let top = TopPlayers::new(players);
        let expected = vec![
            (0, 22),
            (0, 12),
            (0, 11),
            (0, 10),
            (0, 8),
            (0, 5),
            (0, 4),
            (0, 3),
            (0, 3),
            (0, 2),
        ];
        assert_eq!(
            top.by_assists()
                .map(|p| (p.statistics.goals_scored, p.statistics.assists))
                .collect::<Vec<Stat>>(),
            expected
        );
    }

    #[test]
    fn test_by_both() {
        // 11 players
        let stats = vec![
            (5, 8),
            (4, 3),
            (33, 5),
            (9, 2),
            (2, 10),
            (0, 11),
            (1, 3),
            (11, 4),
            (3, 12),
            (9, 22),
            (2, 1),
        ];
        let players = mock_players(&stats);
        let top = TopPlayers::new(players);
        let expected = vec![
            (33, 5),
            (11, 4),
            (9, 22),
            (9, 2),
            (5, 8),
            (4, 3),
            (3, 12),
            (2, 10),
            (2, 1),
            (1, 3),
        ];
        assert_eq!(
            top.by_both()
                .map(|p| (p.statistics.goals_scored, p.statistics.assists))
                .collect::<Vec<Stat>>(),
            expected
        );
    }
}
