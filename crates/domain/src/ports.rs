//! Ports (Dependency Inversion): contratos que a camada de aplicação usa
//! e que a infraestrutura implementa. Mantidos pequenos (Interface Segregation).

use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;

use crate::history::HistoryEntry;
use crate::item::Item;
use crate::log_event::GameLogEvent;
use crate::presets::PresetBook;
use crate::price::{CurrencyRates, IndexedPrice, PriceCheckResult};
use crate::search::SearchFilters;
use crate::settings::Settings;
use crate::{GameVersion, League};

/// Erro genérico vindo de um adapter. A aplicação traduz para `AppError`.
#[derive(Debug, Error)]
pub enum PortError {
    #[error("limite de requisições atingido; tente novamente em {retry_after:?}")]
    RateLimited { retry_after: Duration },
    #[error("falha de rede: {0}")]
    Network(String),
    #[error("recurso não encontrado: {0}")]
    NotFound(String),
    #[error("timeout")]
    Timeout,
    #[error("{0}")]
    Other(String),
}

pub type PortResult<T> = Result<T, PortError>;

/// Leitura/escrita do clipboard do sistema.
pub trait ClipboardPort: Send + Sync {
    fn read_text(&self) -> PortResult<Option<String>>;
    fn write_text(&self, text: &str) -> PortResult<()>;
}

/// Envio de input sintético para o jogo.
pub trait InputPort: Send + Sync {
    /// Solta modificadores que o usuário ainda segura (evita Ctrl+Shift+C no jogo).
    fn release_modifiers(&self) -> PortResult<()>;
    /// Envia Ctrl+C para a janela em foco.
    fn send_copy(&self) -> PortResult<()>;
}

/// Retângulo em pixels físicos de tela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Estado da janela do jogo.
pub trait GameWindowPort: Send + Sync {
    fn is_game_focused(&self) -> bool;
    fn focus_game(&self) -> PortResult<()>;
    /// `true` se o jogo roda elevado e nós não (UIPI bloqueia input).
    fn has_elevation_mismatch(&self) -> bool;
    /// Área cliente da janela do jogo na tela (`None` se fechado ou minimizado).
    fn game_rect(&self) -> Option<WindowRect>;
}

/// Consulta à trade API oficial.
#[async_trait]
pub trait TradeApiPort: Send + Sync {
    async fn price_check(
        &self,
        league: &League,
        item: &Item,
        filters: &SearchFilters,
    ) -> PortResult<PriceCheckResult>;
}

/// Ligas disponíveis para trade (a primeira costuma ser a liga de desafio atual).
#[async_trait]
pub trait LeaguesPort: Send + Sync {
    async fn leagues(&self, game: GameVersion) -> PortResult<Vec<String>>;
}

/// Índice de preços de referência (poe.ninja).
#[async_trait]
pub trait PriceIndexPort: Send + Sync {
    async fn lookup(&self, league: &League, item: &Item) -> PortResult<Option<IndexedPrice>>;
    /// Cotação das moedas em chaos, para normalizar listagens.
    async fn currency_rates(&self, league: &League) -> PortResult<CurrencyRates>;
    /// Imagem genérica de uma base ("Divine Crown"), para itens sem preço indexado (raros etc.).
    async fn base_icon(&self, _league: &League, _base_type: &str) -> PortResult<Option<String>> {
        Ok(None)
    }
}

/// Persistência das configurações.
pub trait SettingsRepository: Send + Sync {
    fn load(&self) -> PortResult<Settings>;
    fn save(&self, settings: &Settings) -> PortResult<()>;
}

/// Persistência do histórico de price checks (mais recentes primeiro).
pub trait HistoryRepository: Send + Sync {
    fn load(&self) -> PortResult<Vec<HistoryEntry>>;
    fn save(&self, entries: &[HistoryEntry]) -> PortResult<()>;
}

/// Persistência dos presets de mods.
pub trait PresetRepository: Send + Sync {
    fn load(&self) -> PortResult<PresetBook>;
    fn save(&self, book: &PresetBook) -> PortResult<()>;
}

/// Fonte de linhas novas do Client.txt.
#[async_trait]
pub trait GameLogSource: Send + Sync {
    /// Retorna as linhas novas desde a última chamada.
    async fn poll_new_lines(&self) -> PortResult<Vec<String>>;
}

/// Converte linhas do log em eventos tipados.
pub trait GameLogInterpreter: Send + Sync {
    fn interpret(&self, line: &str) -> GameLogEvent;
}

/// Tradução do texto do clipboard em [`Item`]. Implementado pelo crate `parser`.
pub trait ItemParser: Send + Sync {
    fn parse(&self, game: GameVersion, text: &str) -> Result<Item, ItemParseError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ItemParseError {
    #[error("o texto não parece ser um item de Path of Exile")]
    NotAnItem,
    #[error("item não reconhecido: {0}")]
    Unknown(String),
}
