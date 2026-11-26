use std::sync::Arc;

use mirrorsight_domain::ports::LeaguesPort;
use mirrorsight_domain::GameVersion;

use crate::AppError;

/// Ligas de trade disponíveis para o jogo (a primeira é a liga de desafio atual).
pub struct ListLeagues {
    source: Arc<dyn LeaguesPort>,
}

impl ListLeagues {
    pub fn new(source: Arc<dyn LeaguesPort>) -> Self {
        Self { source }
    }

    pub async fn execute(&self, game: GameVersion) -> Result<Vec<String>, AppError> {
        Ok(self.source.leagues(game).await?)
    }
}
