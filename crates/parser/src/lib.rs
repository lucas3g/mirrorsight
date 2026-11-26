//! Parser do texto que o jogo coloca no clipboard (Ctrl+C ou Ctrl+Alt+C sobre um item).
//!
//! Pipeline:
//! 1. [`sections::split`] divide o texto pelas linhas `--------`.
//! 2. [`nameplate::parse`] lê Item Class / Rarity / nome / base.
//! 3. Cada [`SectionParser`] da lista ordenada tenta consumir as seções restantes.
//!    Os mods são casados com o catálogo de stats da trade API ([`catalog`]).
//!    Para adicionar suporte a algo novo, crie um parser e inclua-o em
//!    [`section_parsers::default_pipeline`] — sem editar os existentes (Open/Closed).

#![forbid(unsafe_code)]

pub mod bases;
pub mod catalog;
pub mod maps;
pub mod matcher;
pub mod mod_line;
pub mod nameplate;
pub mod properties;
pub mod section_parsers;
pub mod sections;
pub mod viscera;

use mirrorsight_domain::item::Item;
use mirrorsight_domain::ports::{ItemParseError, ItemParser};
use mirrorsight_domain::GameVersion;

use crate::catalog::StatCatalog;
use crate::section_parsers::{SectionOutcome, SectionParser};

/// Implementação de [`ItemParser`] baseada em seções, com uma pipeline por jogo
/// (cada jogo tem seu catálogo de stats).
pub struct ClipboardItemParser {
    poe1: Vec<Box<dyn SectionParser>>,
    poe2: Vec<Box<dyn SectionParser>>,
}

impl Default for ClipboardItemParser {
    fn default() -> Self {
        Self {
            poe1: section_parsers::default_pipeline(StatCatalog::embedded(GameVersion::Poe1)),
            poe2: section_parsers::default_pipeline(StatCatalog::embedded(GameVersion::Poe2)),
        }
    }
}

impl ItemParser for ClipboardItemParser {
    fn parse(&self, game: GameVersion, text: &str) -> Result<Item, ItemParseError> {
        let pipeline = match game {
            GameVersion::Poe1 => &self.poe1,
            GameVersion::Poe2 => &self.poe2,
        };
        let sections = sections::split(text);
        let (first, rest) = sections.split_first().ok_or(ItemParseError::NotAnItem)?;
        let mut item = nameplate::parse(first)?;
        if let Some((base, disc)) = bases::discriminated(game, &item.base_type) {
            // Mantém o nome completo para exibição; a busca usa base + discriminador.
            item.name.get_or_insert_with(|| item.base_type.clone());
            item.base_type = base.to_owned();
            item.discriminator = Some(disc.to_owned());
        }

        for section in rest {
            let consumed = pipeline
                .iter()
                .any(|p| p.parse(section, &mut item) == SectionOutcome::Parsed);
            if !consumed {
                item.unknown_lines.extend(section.iter().map(|l| (*l).to_owned()));
            }
        }
        maps::resolve(game, &mut item, rest);
        viscera::resolve(game, &mut item);
        Ok(item)
    }
}
