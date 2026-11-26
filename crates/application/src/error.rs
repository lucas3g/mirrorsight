use serde::Serialize;
use thiserror::Error;

use mirrorsight_domain::ports::{ItemParseError, PortError};
use mirrorsight_domain::error::Reason;
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
    /// `reason` presente → a UI traduz pelo motivo; senão mostra `message` (técnica, do transporte).
    #[error("falha de rede: {message}")]
    Network {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<Reason>,
    },
    #[error("configuração inválida: {message}")]
    InvalidSettings {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<Reason>,
    },
    #[error("erro interno: {message}")]
    Internal {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<Reason>,
    },
}

impl AppError {
    /// Validação recusada, com motivo traduzível.
    #[must_use]
    pub fn invalid(reason: Reason) -> Self {
        Self::InvalidSettings { message: reason.to_string(), reason: Some(reason) }
    }

    /// Falha conhecida fora da validação (clipboard, jogo como admin…).
    #[must_use]
    pub fn known(reason: Reason) -> Self {
        Self::Internal { message: reason.to_string(), reason: Some(reason) }
    }

    /// Falha inesperada: só a mensagem técnica.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into(), reason: None }
    }
}

impl From<PortError> for AppError {
    fn from(err: PortError) -> Self {
        match err {
            PortError::RateLimited { retry_after } => Self::RateLimited {
                retry_after_ms: u64::try_from(retry_after.as_millis()).unwrap_or(u64::MAX),
            },
            PortError::Network(message) => Self::Network { message, reason: None },
            PortError::Timeout => Self::Network {
                message: "timeout".into(),
                reason: Some(Reason::Timeout),
            },
            PortError::Known(reason) => Self::known(reason),
            other => Self::internal(other.to_string()),
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
        Self::invalid(err.reason())
    }
}
