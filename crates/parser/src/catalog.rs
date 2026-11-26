//! Catálogo de stats da trade API, embutido no binário (ver `tools/stats/update-stats.mjs`).

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Serialize;

use mirrorsight_domain::item::ModifierKind;
use mirrorsight_domain::GameVersion;

const POE1_TSV: &str = include_str!("../data/stats-poe1.tsv");
const POE2_TSV: &str = include_str!("../data/stats-poe2.tsv");

/// Um stat do catálogo, para a busca "Adicionar mod" da UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CatalogStat {
    pub trade_id: String,
    /// Template com `#` no lugar dos números (ex.: "+# to maximum Life").
    pub text: String,
    pub kind: ModifierKind,
}

/// Mapa grupo → (template com `#` → id da trade API), mais a lista plana na ordem do TSV.
#[derive(Debug, Default)]
pub struct StatCatalog {
    groups: HashMap<String, HashMap<String, String>>,
    entries: Vec<CatalogStat>,
}

impl StatCatalog {
    /// Lê o TSV `grupo \t id \t texto` (texto com `\n` literal para quebras de linha).
    pub fn from_tsv(tsv: &str) -> Self {
        let mut groups: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut entries = Vec::new();
        for line in tsv.lines() {
            let mut cols = line.splitn(3, '\t');
            let (Some(group), Some(id), Some(text)) = (cols.next(), cols.next(), cols.next()) else {
                continue;
            };
            let text = text.replace("\\n", "\n");
            // Mesmo critério de dedupe dos grupos: o primeiro id de cada texto vence.
            if let Entry::Vacant(slot) = groups.entry(group.to_owned()).or_default().entry(text.clone()) {
                slot.insert(id.to_owned());
                if let Some(kind) = kind_of(group) {
                    entries.push(CatalogStat {
                        trade_id: id.to_owned(),
                        text,
                        kind,
                    });
                }
            }
        }
        Self { groups, entries }
    }

    /// Catálogo embutido do jogo, construído uma única vez por processo.
    pub fn embedded(game: GameVersion) -> &'static Self {
        static POE1: OnceLock<StatCatalog> = OnceLock::new();
        static POE2: OnceLock<StatCatalog> = OnceLock::new();
        match game {
            GameVersion::Poe1 => POE1.get_or_init(|| Self::from_tsv(POE1_TSV)),
            GameVersion::Poe2 => POE2.get_or_init(|| Self::from_tsv(POE2_TSV)),
        }
    }

    pub fn lookup(&self, kind: ModifierKind, template: &str) -> Option<&str> {
        self.groups
            .get(group_of(kind))
            .and_then(|g| g.get(template))
            .map(String::as_str)
    }

    /// Todos os stats conhecidos (grupos que a UI entende), sem duplicatas por texto.
    pub fn entries(&self) -> &[CatalogStat] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.groups.values().map(HashMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Nome do grupo de stats da trade API para cada tipo de mod.
pub const fn group_of(kind: ModifierKind) -> &'static str {
    match kind {
        ModifierKind::Implicit => "implicit",
        ModifierKind::Explicit => "explicit",
        ModifierKind::Crafted => "crafted",
        ModifierKind::Fractured => "fractured",
        ModifierKind::Enchant => "enchant",
        ModifierKind::Rune => "rune",
        ModifierKind::Desecrated => "desecrated",
        ModifierKind::Pseudo => "pseudo",
        // Não existe no catálogo de stats: propriedades viram filtros próprios da busca.
        ModifierKind::Property => "property",
    }
}

/// Inverso de [`group_of`]; `None` para grupos que o app não entende.
pub fn kind_of(group: &str) -> Option<ModifierKind> {
    Some(match group {
        "implicit" => ModifierKind::Implicit,
        "explicit" => ModifierKind::Explicit,
        "crafted" => ModifierKind::Crafted,
        "fractured" => ModifierKind::Fractured,
        "enchant" => ModifierKind::Enchant,
        "rune" => ModifierKind::Rune,
        "desecrated" => ModifierKind::Desecrated,
        "pseudo" => ModifierKind::Pseudo,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalogs_load() {
        let poe1 = StatCatalog::embedded(GameVersion::Poe1);
        assert!(poe1.len() > 10_000, "poe1 tem {} stats", poe1.len());
        assert_eq!(
            poe1.lookup(ModifierKind::Explicit, "+# to maximum Life"),
            Some("explicit.stat_3299347043")
        );
        assert!(!StatCatalog::embedded(GameVersion::Poe2).is_empty());
    }

    #[test]
    fn restores_multiline_text() {
        let c = StatCatalog::from_tsv("explicit\texplicit.x\tLine one\\nLine two\n");
        assert_eq!(c.lookup(ModifierKind::Explicit, "Line one\nLine two"), Some("explicit.x"));
    }

    #[test]
    fn entries_carry_kind_and_skip_duplicates() {
        let c = StatCatalog::from_tsv(
            "explicit\texplicit.a\t+# to Strength\n\
             explicit\texplicit.b\t+# to Strength\n\
             pseudo\tpseudo.pseudo_total_life\t+# total maximum Life\n\
             monster\tmonster.x\tIgnored\n",
        );
        let entries = c.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].trade_id, "explicit.a");
        assert_eq!(entries[0].kind, ModifierKind::Explicit);
        assert_eq!(entries[1].kind, ModifierKind::Pseudo);

        let poe1 = StatCatalog::embedded(GameVersion::Poe1);
        assert_eq!(poe1.entries().len(), poe1.len());
        assert!(poe1.entries().iter().any(|e| e.kind == ModifierKind::Implicit));
    }

    #[test]
    fn kind_of_is_inverse_of_group_of() {
        for kind in [ModifierKind::Explicit, ModifierKind::Pseudo, ModifierKind::Rune] {
            assert_eq!(kind_of(group_of(kind)), Some(kind));
        }
    }
}
