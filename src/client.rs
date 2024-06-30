use async_trait::async_trait;

use crate::types::{CompetitorStats, SeasonCompetitors};

#[async_trait]
pub trait Client {
    async fn fetch_competitors(&self) -> Result<SeasonCompetitors, Box<dyn std::error::Error>>;
    async fn fetch_competitor_stats(
        &self,
        id: &str,
    ) -> Result<CompetitorStats, Box<dyn std::error::Error>>;
}
