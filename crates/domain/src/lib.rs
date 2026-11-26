//! Núcleo de domínio do Mirrorsight.
//!
//! Regra: este crate não depende de nenhum outro crate do projeto e não faz I/O.
//! Tudo que toca o mundo externo é expresso como um *port* (trait) em [`ports`].

#![forbid(unsafe_code)]

pub mod compare;
pub mod error;
pub mod game;
pub mod history;
pub mod hotkey;
pub mod item;
pub mod log_event;
pub mod ports;
pub mod presets;
pub mod price;
pub mod search;
pub mod settings;

pub use error::DomainError;
pub use game::{GameVersion, League};
