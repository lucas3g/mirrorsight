//! Testes de ponta a ponta do parser com textos de item reais (fixtures).

use mirrorsight_domain::item::{Influence, Item, ModifierKind, Rarity};
use mirrorsight_domain::ports::ItemParser;
use mirrorsight_domain::GameVersion;
use mirrorsight_parser::ClipboardItemParser;

fn parse(fixture: &str) -> Item {
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("fixture {fixture}: {e}"));
    ClipboardItemParser::default()
        .parse(GameVersion::Poe1, &text)
        .unwrap_or_else(|e| panic!("{fixture}: {e}"))
}

fn ids(item: &Item) -> Vec<&str> {
    item.modifiers.iter().map(|m| m.trade_id.as_str()).collect()
}

#[test]
fn rare_gloves_normal_copy() {
    let item = parse("poe1_rare_gloves.txt");

    assert_eq!(item.rarity, Rarity::Rare);
    assert_eq!(item.quality, Some(20));
    assert_eq!(item.item_level, Some(86));
    assert_eq!(item.sockets.as_deref(), Some("B-B-R G"));
    assert_eq!(item.max_links(), 3);
    assert_eq!(item.influences, vec![Influence::Hunter]);
    assert!(item.unknown_lines.is_empty(), "linhas não reconhecidas: {:?}", item.unknown_lines);

    assert_eq!(item.modifiers.len(), 6);
    let life = &item.modifiers[0];
    assert_eq!(life.kind, ModifierKind::Fractured);
    assert!(life.trade_id.starts_with("fractured."), "{}", life.trade_id);
    assert_eq!(life.roll.map(|r| r.value), Some(78.0));

    // "Adds # to #" em luvas é global (to Attacks); valor = média.
    let adds = &item.modifiers[3];
    assert_eq!(adds.trade_id, "explicit.stat_3032590688");
    assert_eq!(adds.roll.map(|r| r.value), Some(16.5));

    let crafted = &item.modifiers[5];
    assert_eq!(crafted.kind, ModifierKind::Crafted);
    assert_eq!(crafted.text, "+16% to Chaos Resistance");
}

#[test]
fn advanced_copy_matches_same_stats_with_ranges_and_tiers() {
    let normal = parse("poe1_rare_gloves.txt");
    let advanced = parse("poe1_rare_gloves_advanced.txt");

    assert_eq!(ids(&normal), ids(&advanced));
    assert!(advanced.unknown_lines.is_empty(), "{:?}", advanced.unknown_lines);

    let life = &advanced.modifiers[0];
    assert_eq!(life.tier, Some(2));
    assert_eq!(life.text, "+78 to maximum Life");
    let roll = life.roll.expect("roll");
    assert_eq!((roll.min, roll.max), (Some(70.0), Some(79.0)));

    // Crafted tem "Rank", não "Tier".
    assert_eq!(advanced.modifiers[5].tier, None);
    assert_eq!(advanced.modifiers[5].kind, ModifierKind::Crafted);
}

#[test]
fn weapon_prefers_local_stats_and_reads_implicit() {
    let item = parse("poe1_rare_sword.txt");

    assert!(item.corrupted);
    assert!(item.unknown_lines.is_empty(), "{:?}", item.unknown_lines);
    assert_eq!(
        ids(&item),
        vec![
            "implicit.stat_3556824919",  // crit multi (implicit)
            "explicit.stat_1940865751",  // Adds # to # Physical Damage (Local)
            "explicit.stat_1509134228",  // #% increased Physical Damage
            "explicit.stat_3261801346",  // +# to Dexterity
        ]
    );
    assert_eq!(item.modifiers[0].kind, ModifierKind::Implicit);
}

#[test]
fn unique_keeps_flavour_text_as_unknown() {
    let item = parse("poe1_unique_helmet.txt");

    assert_eq!(item.rarity, Rarity::Unique);
    assert_eq!(item.modifiers.len(), 2);
    assert_eq!(item.modifiers[0].trade_id, "explicit.stat_53045048"); // Evasion (Local) em capacete
    assert_eq!(item.modifiers[1].trade_id, "explicit.stat_2901986750");
    assert_eq!(item.unknown_lines, vec!["No metal slips as easily through the fingers as gold."]);
}

#[test]
fn poe2_uses_its_own_catalog() {
    let text = "Item Class: Body Armours\nRarity: Rare\nGale Shell\nExpert Leather Vest\n--------\nItem Level: 79\n--------\n+80 to maximum Life\n+30% to Cold Resistance\n";
    let item = ClipboardItemParser::default().parse(GameVersion::Poe2, text).expect("poe2");
    assert_eq!(item.modifiers.len(), 2, "{:?}", item.unknown_lines);
    assert!(item.modifiers.iter().all(|m| m.trade_id.starts_with("explicit.")));
}

#[test]
fn currency_has_no_modifiers() {
    let item = parse("poe1_currency.txt");
    assert_eq!(item.rarity, Rarity::Currency);
    assert_eq!(item.base_type, "Divine Orb");
    assert!(item.modifiers.is_empty());
}

#[test]
fn foulborn_unique_searches_by_plain_name() {
    let item = parse("poe1_foulborn_unique.txt");
    assert_eq!(item.rarity, Rarity::Unique);
    assert_eq!(item.name.as_deref(), Some("Aegis Aurora"));
    assert_eq!(item.base_type, "Champion Kite Shield");
    assert!(item.foulborn);
    assert!(item.corrupted);
    assert!(ids(&item).contains(&"explicit.stat_2670993553"), "{:?}", ids(&item));
    // "+6% Chance to Block" casa com "+#% Chance to Block (Shields)".
    assert!(ids(&item).contains(&"explicit.stat_4253454700"), "{:?}", ids(&item));
    assert_eq!(item.modifiers.len(), 6);
}

#[test]
fn transfigured_gem_gets_discriminator() {
    let item = parse("poe1_transfigured_gem.txt");
    assert_eq!(item.rarity, Rarity::Gem);
    assert_eq!(item.base_type, "Kinetic Blast");
    assert_eq!(item.discriminator.as_deref(), Some("alt_x"));
    assert_eq!(item.name.as_deref(), Some("Kinetic Blast of Clustering"));
}

#[test]
fn blighted_map_resolves_area_code() {
    let item = parse("poe1_blighted_map.txt");
    assert_eq!(item.base_type, "Blighted Map (Strand)");
    assert_eq!(item.trade_type.as_deref(), Some("10021"));
    assert_eq!(item.discriminator.as_deref(), Some("blighted"));
    assert_eq!(item.map_tier, Some(13));
}

#[test]
fn magic_map_uses_generic_map_type() {
    let text = "Item Class: Maps\nRarity: Magic\nSavage Map (Tier 16) of Fog\n--------\nMap Area: Strand\nMap Tier: 16\n--------\nItem Level: 83\n";
    let item = ClipboardItemParser::default().parse(GameVersion::Poe1, text).unwrap();
    assert_eq!(item.trade_type.as_deref(), Some("Map"));
    assert_eq!(item.discriminator.as_deref(), Some("map"));
    assert_eq!(item.map_tier, Some(16));
}

#[test]
fn viscera_without_rarity_line() {
    let item = parse("poe1_viscera_advanced.txt");

    assert_eq!(item.item_class, "Viscera");
    assert_eq!(item.rarity, Rarity::Other);
    assert_eq!(item.name.as_deref(), Some("Greater Carved Haunches"));
    assert_eq!(item.base_type, "Carved Haunches");
    assert_eq!(item.trade_type.as_deref(), Some("Carved Haunches"));
    assert_eq!(item.item_level, Some(82));
    assert_eq!(ids(&item), vec!["implicit.stat_1870732546"]);
    assert_eq!(item.modifiers[0].roll.map(|r| r.value), Some(21.0));
}

#[test]
fn helmet_defences_become_property_filters() {
    let item = parse("poe1_rare_helmet_advanced.txt");
    let props: Vec<(&str, f64)> = item.properties.iter().map(|p| (p.trade_id.as_str(), p.roll.unwrap().value)).collect();
    assert_eq!(props, [("property.ar", 855.0), ("property.es", 171.0)]);
    assert!(item.properties.iter().all(|p| p.kind == ModifierKind::Property));
    assert_eq!(item.quality, Some(20));
    assert_eq!(item.modifiers.len(), 8);
}
