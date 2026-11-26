//! Histórico local de price checks (os mais recentes primeiro).

use serde::{Deserialize, Serialize};

use crate::item::Rarity;
use crate::search::SearchFilters;
use crate::GameVersion;

/// Quantos price checks o histórico guarda; os mais antigos saem primeiro.
pub const HISTORY_LIMIT: usize = 50;

/// Identificação mínima do item, para listar sem parsear o texto de novo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryItem {
    pub name: Option<String>,
    pub base_type: String,
    pub rarity: Rarity,
    /// Imagem do item (poecdn, via poe.ninja). Ausente em entradas antigas.
    #[serde(default)]
    pub icon: Option<String>,
}

/// De onde veio o preço registrado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistorySource {
    /// Listagens do trade oficial (Compra/Justo/Venda = percentis).
    Trade,
    /// Currency: só o preço de referência do poe.ninja (em `fair_chaos`).
    Ninja,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub game: GameVersion,
    pub league: String,
    /// Texto original do item: "Reabrir" refaz o price check com ele.
    pub item_text: String,
    pub item: HistoryItem,
    /// Filtros usados na busca; `None` = filtros padrão.
    #[serde(default)]
    pub filters: Option<SearchFilters>,
    pub buy_chaos: Option<f64>,
    pub fair_chaos: Option<f64>,
    pub sell_chaos: Option<f64>,
    /// Listagens consideradas no cálculo.
    pub sample: u32,
    pub source: HistorySource,
    pub divine_chaos: Option<f64>,
    /// Epoch em milissegundos.
    pub at_ms: i64,
}

/// Insere no início e descarta o excedente além de [`HISTORY_LIMIT`].
pub fn push(entries: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
    entries.retain(|e| e.id != entry.id);
    entries.insert(0, entry);
    entries.truncate(HISTORY_LIMIT);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.into(),
            game: GameVersion::Poe1,
            league: "Standard".into(),
            item_text: String::new(),
            item: HistoryItem {
                name: None,
                base_type: "Divine Orb".into(),
                rarity: Rarity::Currency,
                icon: None,
            },
            filters: None,
            buy_chaos: None,
            fair_chaos: Some(200.0),
            sell_chaos: None,
            sample: 0,
            source: HistorySource::Ninja,
            divine_chaos: Some(200.0),
            at_ms: 0,
        }
    }

    #[test]
    fn newest_first_and_capped_at_limit() {
        let mut entries = Vec::new();
        for i in 0..(HISTORY_LIMIT + 5) {
            push(&mut entries, entry(&i.to_string()));
        }
        assert_eq!(entries.len(), HISTORY_LIMIT);
        assert_eq!(entries[0].id, (HISTORY_LIMIT + 4).to_string());
        // Os 5 mais antigos saíram.
        assert!(entries.iter().all(|e| e.id.parse::<usize>().unwrap() >= 5));
    }

    #[test]
    fn same_id_is_replaced_not_duplicated() {
        let mut entries = vec![entry("a"), entry("b")];
        push(&mut entries, entry("b"));
        assert_eq!(entries.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), ["b", "a"]);
    }
}
