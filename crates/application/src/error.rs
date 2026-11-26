use serde::Serialize;
use thiserror::Error;

use mirrorsight_domain::ports::{ItemParseError, PortError};
use mirrorsight_domain::DomainError;

/// Erro que atravessa a fronteira para a UI. Cada variante vira um `code` estável.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "code", content = "data", rename_all = "kebab-case")]
pub enum AppError {
    #[error("nenhum item foi copiado (o jogo está em foco?)")]
    NothingCopied,
    #[error("item não reconhecido")]
    ItemNotRecognized { detail: String },
    #[error("limite de requisições: tente novamente em {retry_after_ms} ms")]
    RateLimited { retry_after_ms: u64 },
    #[error("falha de rede: {message}")]
    Network { message: String },
    #[error("configuração inválida: {message}")]
    InvalidSettings { message: String },
    #[error("erro interno: {message}")]
    Internal { message: String },
}

impl From<PortError> for AppError {
    fn from(err: PortError) -> Self {
        match err {
            PortError::RateLimited { retry_after } => Self::RateLimited {
                retry_after_ms: u64::try_from(retry_after.as_millis()).unwrap_or(u64::MAX),
            },
            PortError::Network(message) => Self::Network { message },
            PortError::Timeout => Self::Network {
                message: "timeout".into(),
            },
            other => Self::Internal {
                message: other.to_string(),
            },
        }
    }
}

impl From<ItemParseError> for AppError {
    fn from(err: ItemParseError) -> Self {
        Self::ItemNotRecognized {
            detail: err.to_string(),
        }
    }
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        Self::InvalidSettings {
            message: err.to_string(),
        }
    }
}
