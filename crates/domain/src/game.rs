use serde::{Deserialize, Serialize};

/// Versão do jogo. Toda diferença PoE1 × PoE2 deve passar por aqui (strategy), nunca por `if` espalhado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameVersion {
    #[default]
    Poe1,
    Poe2,
}

impl GameVersion {
    /// Título padrão da janela do jogo, usado para anexar o overlay.
    pub const fn window_title(self) -> &'static str {
        match self {
            Self::Poe1 => "Path of Exile",
            Self::Poe2 => "Path of Exile 2",
        }
    }
}

/// Liga de trade (ex.: "Standard", "Hardcore", liga de desafio atual).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct League {
    pub id: String,
    pub game: GameVersion,
}

impl League {
    pub fn new(id: impl Into<String>, game: GameVersion) -> Self {
        Self { id: id.into(), game }
    }
}
