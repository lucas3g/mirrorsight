use serde::{Deserialize, Serialize};

/// Evento tipado extraído de uma linha do Client.txt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum GameLogEvent {
    WhisperReceived { from: String, message: String },
    WhisperSent { to: String, message: String },
    ZoneEntered { zone: String },
    TradeAccepted,
    PlayerDied { character: String },
    /// Linha que não casou com nenhum padrão conhecido.
    Raw { line: String },
}
