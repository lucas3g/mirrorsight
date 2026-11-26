//! Cliente HTTP compartilhado pelos adapters externos.

use std::time::Duration;

use mirrorsight_domain::ports::PortError;

/// A GGG pede um User-Agent identificável para clientes automatizados.
/// Sem dados pessoais do usuário: só o nome e a natureza do projeto.
pub const USER_AGENT: &str = concat!(
    "Mirrorsight/",
    env!("CARGO_PKG_VERSION"),
    " (open-source PoE price checker)"
);

const TIMEOUT: Duration = Duration::from_secs(15);

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(TIMEOUT)
        .build()
        // Só falha se o backend TLS não inicializar; sem cliente HTTP o app não tem utilidade.
        .unwrap_or_else(|err| panic!("falha ao criar cliente HTTP: {err}"))
}

/// Traduz erros de transporte para o vocabulário dos ports.
pub fn transport_error(err: &reqwest::Error) -> PortError {
    if err.is_timeout() {
        PortError::Timeout
    } else if err.is_decode() {
        tracing::warn!(%err, "resposta inesperada");
        PortError::Known(mirrorsight_domain::error::Reason::UnexpectedResponse)
    } else {
        PortError::Network(err.to_string())
    }
}
