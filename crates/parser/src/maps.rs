//! Mapas do PoE1. O jogo mostra a base como "Map (Tier 16)" / "Blighted Map (Tier 13)"
//! (mágicos: "Savage Map (Tier 1) of ...") e a área numa propriedade "Map Area: Strand".
//! A trade API busca:
//! - mapa comum: type "Map" + discriminator "map";
//! - blighted / blight-ravaged: type numérico da área ("10021") + discriminator
//!   "blighted"/"uberblighted", vindo da tabela de bases ("Blighted Map (Strand)").

use mirrorsight_domain::item::Item;
use mirrorsight_domain::GameVersion;

use crate::bases;

/// Variantes de mapa: (texto da base, discriminator da trade).
const KINDS: &[(&str, &str)] = &[("Blight-ravaged Map", "uberblighted"), ("Blighted Map", "blighted"), ("Map", "map")];

/// Ajusta `base_type`/`trade_type`/`discriminator`/`map_tier` de mapas.
/// `sections` são as seções após a primeira (propriedades, mods…).
pub fn resolve(game: GameVersion, item: &mut Item, sections: &[Vec<&str>]) {
    if game != GameVersion::Poe1 || item.item_class != "Maps" {
        return;
    }
    let property = |key: &str| {
        sections
            .iter()
            .flatten()
            .find_map(|l| l.strip_prefix(key))
            .map(str::trim)
    };
    let Some((kind, disc, base_tier)) = split_base(&item.base_type) else {
        return;
    };
    item.map_tier = property("Map Tier: ").and_then(|t| t.parse().ok()).or(base_tier);

    let area = property("Map Area: ");
    let display = match area {
        Some(area) => format!("{kind} ({area})"),
        None => kind.to_owned(),
    };
    item.discriminator = Some(disc.to_owned());
    item.trade_type = match disc {
        "map" => Some("Map".to_owned()),
        // Sem a área a busca cai no filtro "Blighted Map" da trade, sem type.
        _ => area.and_then(|_| bases::discriminated(game, &display)).map(|(ty, _)| ty.to_owned()),
    };
    item.base_type = display;
}

/// "Savage Blighted Map (Tier 13) of Fog" → ("Blighted Map", "blighted", Some(13)).
fn split_base(base: &str) -> Option<(&'static str, &'static str, Option<u32>)> {
    // KINDS vai do mais específico ao mais genérico: "Blighted Map (Tier" também contém "Map (Tier".
    // Bases sem tier ("Valdo Map", "Nightmare Map"…) são types próprios na trade: ficam como estão.
    let (kind, disc) = KINDS.iter().find(|(k, _)| base.contains(&format!("{k} (Tier ")))?;
    let tier = base
        .split_once("(Tier ")
        .and_then(|(_, rest)| rest.split(')').next())
        .and_then(|t| t.trim().parse().ok());
    Some((kind, disc, tier))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_map_bases() {
        assert_eq!(split_base("Map (Tier 16)"), Some(("Map", "map", Some(16))));
        assert_eq!(split_base("Blighted Map (Tier 13)"), Some(("Blighted Map", "blighted", Some(13))));
        assert_eq!(
            split_base("Savage Blight-ravaged Map (Tier 16) of Fog"),
            Some(("Blight-ravaged Map", "uberblighted", Some(16)))
        );
        assert_eq!(split_base("Valdo Map"), None);
    }
}
