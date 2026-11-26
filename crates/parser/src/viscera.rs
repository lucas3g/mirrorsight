//! Viscera (Covens): partes do Wickerman. O jogo mostra "Greater Carved Haunches",
//! mas a trade API só conhece o type "Carved Haunches" (categoria `viscera`).
//! Lista gerada por `node tools/stats/update-stats.mjs` a partir de `/api/trade/data/items`.

use std::sync::OnceLock;

use mirrorsight_domain::item::Item;
use mirrorsight_domain::GameVersion;

const POE1_TXT: &str = include_str!("../data/viscera-poe1.txt");
const POE2_TXT: &str = include_str!("../data/viscera-poe2.txt");

/// Classe do item no texto copiado.
const CLASS: &str = "Viscera";

fn types(game: GameVersion) -> &'static [&'static str] {
    static POE1: OnceLock<Vec<&'static str>> = OnceLock::new();
    static POE2: OnceLock<Vec<&'static str>> = OnceLock::new();
    let load = |txt: &'static str| txt.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    match game {
        GameVersion::Poe1 => POE1.get_or_init(|| load(POE1_TXT)),
        GameVersion::Poe2 => POE2.get_or_init(|| load(POE2_TXT)),
    }
}

/// Maior base conhecida que termina o nome exibido (`"Greater Carved Haunches"` → `"Carved Haunches"`).
pub fn base_for(game: GameVersion, shown: &str) -> Option<&'static str> {
    types(game)
        .iter()
        .copied()
        .filter(|ty| {
            shown == *ty || shown.strip_suffix(ty).is_some_and(|prefix| prefix.ends_with(' '))
        })
        .max_by_key(|ty| ty.len())
}

/// Viscera: busca pelo type da trade e mantém o nome completo para exibição.
pub fn resolve(game: GameVersion, item: &mut Item) {
    if item.item_class != CLASS {
        return;
    }
    let Some(base) = base_for(game, &item.base_type) else { return };
    if base != item.base_type {
        item.name.get_or_insert_with(|| item.base_type.clone());
        item.base_type = base.to_owned();
    }
    item.trade_type = Some(base.to_owned());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_size_prefix() {
        assert_eq!(base_for(GameVersion::Poe1, "Greater Carved Haunches"), Some("Carved Haunches"));
        assert_eq!(base_for(GameVersion::Poe1, "Carved Haunches"), Some("Carved Haunches"));
        assert_eq!(base_for(GameVersion::Poe1, "Greater Uncarved Haunches"), None);
    }
}
