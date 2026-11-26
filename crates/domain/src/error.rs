use serde::Serialize;
use thiserror::Error;

/// Motivo de um erro mostrado ao usuário, com os parâmetros da mensagem.
///
/// Vai para a UI como `{ "key": "reserved-hotkey", "args": { "hotkey": "F1" } }` e o frontend
/// escolhe o texto no idioma da interface. O `Display` (português) fica para logs e fallback.
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "key", content = "args", rename_all = "kebab-case")]
pub enum Reason {
    #[error("combinação de teclas inválida: {hotkey}")]
    InvalidHotkey { hotkey: String },
    #[error("a tecla {hotkey} é reservada pelo jogo")]
    ReservedHotkey { hotkey: String },
    #[error("a tecla {hotkey} já está atribuída a outra ação")]
    DuplicateHotkey { hotkey: String },
    #[error("dê um nome ao preset")]
    PresetNameEmpty,
    #[error("o nome do preset pode ter até {max} caracteres")]
    PresetNameTooLong { max: usize },
    #[error("ligue ao menos um mod para salvar o preset")]
    PresetNoMods,
    #[error("a escala da UI deve estar entre 50% e 200%")]
    UiScaleRange,
    #[error("o range dos mods deve ficar entre 0% e 50%")]
    ModRangeRange,
    #[error("idioma da interface desconhecido: {lang}")]
    UnknownUiLanguage { lang: String },
    #[error("o jogo está rodando como administrador; abra o Mirrorsight como administrador também")]
    GameElevated,
    #[error("o Windows bloqueou o envio de teclas ao jogo")]
    InputBlocked,
    #[error("clipboard indisponível")]
    ClipboardUnavailable,
    #[error("falha ao ler o clipboard")]
    ClipboardRead,
    #[error("falha ao escrever no clipboard")]
    ClipboardWrite,
    #[error("janela do jogo não encontrada")]
    GameWindowNotFound,
    #[error("tempo de resposta esgotado")]
    Timeout,
    #[error("resposta inesperada do servidor")]
    UnexpectedResponse,
    #[error("trade API ainda não implementada")]
    TradeUnavailable,
}

/// Erros de regras de negócio puras (validação de valores do domínio).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("combinação de teclas inválida: {0}")]
    InvalidHotkey(String),

    #[error("a tecla {0} é reservada pelo jogo")]
    ReservedHotkey(String),

    #[error("a tecla {0} já está atribuída a outra ação")]
    DuplicateHotkey(String),

    #[error("{0}")]
    InvalidPreset(Reason),
}

impl DomainError {
    /// Motivo estruturado para a UI traduzir.
    #[must_use]
    pub fn reason(&self) -> Reason {
        match self {
            Self::InvalidHotkey(hotkey) => Reason::InvalidHotkey { hotkey: hotkey.clone() },
            Self::ReservedHotkey(hotkey) => Reason::ReservedHotkey { hotkey: hotkey.clone() },
            Self::DuplicateHotkey(hotkey) => Reason::DuplicateHotkey { hotkey: hotkey.clone() },
            Self::InvalidPreset(reason) => reason.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_serializes_as_key_and_args() {
        let json = serde_json::to_value(Reason::ReservedHotkey { hotkey: "F1".into() }).unwrap();
        assert_eq!(json, serde_json::json!({ "key": "reserved-hotkey", "args": { "hotkey": "F1" } }));
        let unit = serde_json::to_value(Reason::PresetNoMods).unwrap();
        assert_eq!(unit, serde_json::json!({ "key": "preset-no-mods" }));
    }
}
