use thiserror::Error;

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
    InvalidPreset(String),
}
