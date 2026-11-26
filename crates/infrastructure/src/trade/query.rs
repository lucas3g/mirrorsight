//! Montagem do corpo de busca da trade API a partir do item e dos filtros.

use serde_json::{json, Map, Value};

use mirrorsight_domain::item::{Item, Rarity, PROPERTY_PREFIX};
use mirrorsight_domain::search::SearchFilters;
use mirrorsight_domain::GameVersion;

pub fn build(game: GameVersion, item: &Item, filters: &SearchFilters) -> Value {
    let mut query = Map::new();
    query.insert(
        "status".into(),
        json!({ "option": if filters.online_only { "online" } else { "any" } }),
    );

    // Itens mágicos têm afixos no nome da base ("Hale Sorcerer Gloves of the Whale"):
    // sem a base limpa, a busca fica só pelos stats. Mapas já chegam com o type resolvido.
    let ty = item.trade_type.as_deref().unwrap_or(&item.base_type);
    let blight_without_area = item.trade_type.is_none() && blight_filter(item).is_some();
    if (item.rarity != Rarity::Magic || item.trade_type.is_some()) && !blight_without_area {
        let ty = match &item.discriminator {
            Some(disc) => json!({ "option": ty, "discriminator": disc }),
            None => json!(ty),
        };
        query.insert("type".into(), ty);
    }
    if item.rarity == Rarity::Unique {
        if let Some(name) = &item.name {
            query.insert("name".into(), json!(name));
        }
    }

    let (properties, mods): (Vec<_>, Vec<_>) = filters
        .stats
        .iter()
        .filter(|s| s.enabled)
        .partition(|s| s.trade_id.starts_with(PROPERTY_PREFIX));
    let stats: Vec<Value> = mods
        .iter()
        .map(|s| json!({ "id": s.trade_id, "value": range(s.min, s.max), "disabled": false }))
        .collect();
    query.insert("stats".into(), json!([{ "type": "and", "filters": stats }]));

    let mut groups = Map::new();
    // Defesas/DPS: grupos de filtro próprios (PoE2 junta tudo em `equipment_filters`).
    for prop in &properties {
        let id = &prop.trade_id[PROPERTY_PREFIX.len()..];
        let Some(group) = property_group(game, id) else { continue };
        let entry = groups.entry(group).or_insert_with(|| json!({ "filters": {} }));
        if let Some(map) = entry["filters"].as_object_mut() {
            map.insert(id.to_owned(), range(prop.min, prop.max));
        }
    }
    if let Some(rarity) = rarity_option(item.rarity) {
        groups.insert("type_filters".into(), json!({ "filters": { "rarity": { "option": rarity } } }));
    }

    let mut map = Map::new();
    if let Some(tier) = item.map_tier {
        map.insert("map_tier".into(), json!({ "min": tier, "max": tier }));
    }
    // Sem a área não há type: o filtro "Blighted Map" / "Blight-ravaged Map" separa a variante.
    if let Some(filter) = blight_filter(item).filter(|_| blight_without_area) {
        map.insert(filter.into(), json!({ "option": "true" }));
    }
    if !map.is_empty() {
        groups.insert("map_filters".into(), json!({ "filters": map }));
    }

    let mut misc = Map::new();
    if let Some(min) = filters.item_level_min {
        misc.insert("ilvl".into(), json!({ "min": min }));
    }
    if let Some(min) = filters.quality_min {
        misc.insert("quality".into(), json!({ "min": min }));
    }
    // Foulborn (mutado) tem preço próprio: não misturar com o único comum.
    if item.foulborn && game == GameVersion::Poe1 {
        misc.insert("mutated".into(), json!({ "option": "true" }));
    }
    if !misc.is_empty() {
        groups.insert("misc_filters".into(), json!({ "filters": misc }));
    }

    // Links só existem no PoE1.
    if let (GameVersion::Poe1, Some(min)) = (game, filters.links_min) {
        groups.insert("socket_filters".into(), json!({ "filters": { "links": { "min": min } } }));
    }
    if !groups.is_empty() {
        query.insert("filters".into(), Value::Object(groups));
    }

    json!({ "query": query, "sort": { "price": "asc" } })
}

fn range(min: Option<f64>, max: Option<f64>) -> Value {
    let mut value = Map::new();
    if let Some(min) = min {
        value.insert("min".into(), json!(min));
    }
    if let Some(max) = max {
        value.insert("max".into(), json!(max));
    }
    Value::Object(value)
}

/// Grupo de filtros da trade API de cada propriedade; `None` = não existe nesse jogo.
fn property_group(game: GameVersion, id: &str) -> Option<String> {
    let group = match (game, id) {
        (GameVersion::Poe2, "ar" | "ev" | "es" | "ward" | "block" | "spirit" | "dps" | "pdps" | "edps" | "aps" | "crit") => {
            "equipment_filters"
        }
        (GameVersion::Poe1, "ar" | "ev" | "es" | "ward" | "block") => "armour_filters",
        (GameVersion::Poe1, "dps" | "pdps" | "edps" | "aps" | "crit") => "weapon_filters",
        _ => return None,
    };
    Some(group.to_owned())
}

fn blight_filter(item: &Item) -> Option<&'static str> {
    match item.discriminator.as_deref() {
        Some("blighted") => Some("map_blighted"),
        Some("uberblighted") => Some("map_uberblighted"),
        _ => None,
    }
}

const fn rarity_option(rarity: Rarity) -> Option<&'static str> {
    match rarity {
        Rarity::Unique => Some("unique"),
        Rarity::Normal | Rarity::Magic | Rarity::Rare => Some("nonunique"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirrorsight_domain::item::ModifierKind;
    use mirrorsight_domain::search::StatFilter;

    fn item(rarity: Rarity) -> Item {
        Item {
            item_class: "Gloves".into(),
            rarity,
            name: Some("Doom Grip".into()),
            base_type: "Sorcerer Gloves".into(),
            item_level: Some(86),
            quality: None,
            corrupted: false,
            identified: true,
            sockets: None,
            influences: vec![],
            modifiers: vec![],
            properties: Vec::new(),
            unknown_lines: vec![],
            foulborn: false,
            discriminator: None,
            trade_type: None,
            map_tier: None,
        }
    }

    fn filters() -> SearchFilters {
        let stat = |id: &str, enabled, min| StatFilter {
            trade_id: id.into(),
            text: String::new(),
            kind: ModifierKind::Explicit,
            roll: None,
            tier: None,
            enabled,
            min,
            max: None,
        };
        SearchFilters {
            online_only: true,
            stats: vec![stat("explicit.a", true, Some(70.0)), stat("explicit.b", false, Some(1.0))],
            item_level_min: Some(84),
            quality_min: None,
            links_min: Some(6),
        }
    }

    #[test]
    fn rare_query_has_type_rarity_and_only_enabled_stats() {
        let q = build(GameVersion::Poe1, &item(Rarity::Rare), &filters());
        assert_eq!(q["query"]["type"], "Sorcerer Gloves");
        assert!(q["query"].get("name").is_none());
        assert_eq!(q["query"]["status"]["option"], "online");
        assert_eq!(q["query"]["filters"]["type_filters"]["filters"]["rarity"]["option"], "nonunique");
        let stats = q["query"]["stats"][0]["filters"].as_array().unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0]["id"], "explicit.a");
        assert_eq!(stats[0]["value"]["min"], 70.0);
        assert!(stats[0]["value"].get("max").is_none());
        assert_eq!(q["query"]["filters"]["misc_filters"]["filters"]["ilvl"]["min"], 84);
        assert_eq!(q["query"]["filters"]["socket_filters"]["filters"]["links"]["min"], 6);
        assert_eq!(q["sort"]["price"], "asc");
    }

    #[test]
    fn unique_searches_by_name_and_poe2_has_no_links() {
        let q = build(GameVersion::Poe2, &item(Rarity::Unique), &filters());
        assert_eq!(q["query"]["name"], "Doom Grip");
        assert_eq!(q["query"]["filters"]["type_filters"]["filters"]["rarity"]["option"], "unique");
        assert!(q["query"]["filters"].get("socket_filters").is_none());
    }

    #[test]
    fn foulborn_unique_uses_mutated_filter() {
        let mut it = item(Rarity::Unique);
        it.foulborn = true;
        let q = build(GameVersion::Poe1, &it, &filters());
        assert_eq!(q["query"]["name"], "Doom Grip");
        assert_eq!(q["query"]["filters"]["misc_filters"]["filters"]["mutated"]["option"], "true");
        let q = build(GameVersion::Poe1, &item(Rarity::Unique), &filters());
        assert!(q["query"]["filters"]["misc_filters"]["filters"].get("mutated").is_none());
    }

    #[test]
    fn transfigured_gem_uses_discriminator() {
        let mut it = item(Rarity::Gem);
        it.base_type = "Kinetic Blast".into();
        it.discriminator = Some("alt_x".into());
        let q = build(GameVersion::Poe1, &it, &filters());
        assert_eq!(q["query"]["type"]["option"], "Kinetic Blast");
        assert_eq!(q["query"]["type"]["discriminator"], "alt_x");
    }

    #[test]
    fn blighted_map_uses_area_code_and_tier() {
        let mut it = item(Rarity::Magic);
        it.base_type = "Blighted Map (Strand)".into();
        it.trade_type = Some("10021".into());
        it.discriminator = Some("blighted".into());
        it.map_tier = Some(13);
        let q = build(GameVersion::Poe1, &it, &filters());
        assert_eq!(q["query"]["type"]["option"], "10021");
        assert_eq!(q["query"]["type"]["discriminator"], "blighted");
        let map = &q["query"]["filters"]["map_filters"]["filters"];
        assert_eq!(map["map_tier"]["min"], 13);
        assert!(map.get("map_blighted").is_none());
    }

    #[test]
    fn blighted_map_without_area_uses_blight_filter() {
        let mut it = item(Rarity::Normal);
        it.base_type = "Blight-ravaged Map".into();
        it.discriminator = Some("uberblighted".into());
        let q = build(GameVersion::Poe1, &it, &filters());
        assert!(q["query"].get("type").is_none());
        assert_eq!(q["query"]["filters"]["map_filters"]["filters"]["map_uberblighted"]["option"], "true");
    }

    #[test]
    fn stat_range_sends_min_and_max() {
        let mut f = filters();
        f.stats[0].min = Some(130.0);
        f.stats[0].max = Some(144.0);
        let q = build(GameVersion::Poe1, &item(Rarity::Rare), &f);
        let value = &q["query"]["stats"][0]["filters"][0]["value"];
        assert_eq!(value["min"], 130.0);
        assert_eq!(value["max"], 144.0);
    }

    #[test]
    fn properties_go_to_their_filter_groups() {
        let prop = |id: &str, enabled, min| StatFilter {
            trade_id: format!("property.{id}"),
            text: String::new(),
            kind: ModifierKind::Property,
            roll: None,
            tier: None,
            enabled,
            min,
            max: None,
        };
        let mut f = filters();
        f.stats.extend([prop("ar", true, Some(1200.0)), prop("pdps", true, Some(250.5)), prop("es", false, Some(1.0))]);

        let q = build(GameVersion::Poe1, &item(Rarity::Rare), &f);
        let groups = &q["query"]["filters"];
        assert_eq!(groups["armour_filters"]["filters"]["ar"]["min"], 1200.0);
        assert!(groups["armour_filters"]["filters"].get("es").is_none());
        assert_eq!(groups["weapon_filters"]["filters"]["pdps"]["min"], 250.5);
        // Propriedades não entram nos stats.
        assert_eq!(q["query"]["stats"][0]["filters"].as_array().unwrap().len(), 1);

        let q = build(GameVersion::Poe2, &item(Rarity::Rare), &f);
        let eq = &q["query"]["filters"]["equipment_filters"]["filters"];
        assert_eq!(eq["ar"]["min"], 1200.0);
        assert_eq!(eq["pdps"]["min"], 250.5);
    }

    #[test]
    fn magic_items_skip_type() {
        let q = build(GameVersion::Poe1, &item(Rarity::Magic), &filters());
        assert!(q["query"].get("type").is_none());
    }
}
