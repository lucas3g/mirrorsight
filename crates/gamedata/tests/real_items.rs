//! Ponta a ponta: texto real do jogo → parser → avaliação com a base de conhecimento embutida.

use mirrorsight_domain::evaluation::mods::ModStatus;
use mirrorsight_domain::evaluation::{compare_items, evaluate_item, BuildProfile, Confidence, GameKnowledge, Verdict};
use mirrorsight_domain::item::Item;
use mirrorsight_domain::ports::ItemParser;
use mirrorsight_domain::GameVersion;
use mirrorsight_gamedata::EmbeddedKnowledge;
use mirrorsight_parser::ClipboardItemParser;

fn parse_text(text: &str) -> Item {
    ClipboardItemParser::default().parse(GameVersion::Poe1, text).unwrap_or_else(|e| panic!("{e}"))
}

fn fixture(name: &str) -> Item {
    let path = format!("{}/../parser/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    parse_text(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}")))
}

#[test]
fn unique_advanced_copy_uses_game_ranges() {
    let k = EmbeddedKnowledge::poe1();
    let item = fixture("poe1_unique_belt_advanced.txt");
    let q = evaluate_item(&item, k);
    let found = q.unique.as_ref().expect("único");
    assert!(found.found, "{found:?}");
    assert_eq!(found.variable_mods, 5, "{:#?}", q.mods);
    // "When you Kill a Rare monster…" pode nem casar com um stat da trade (fica em unknown_lines).
    assert!(found.fixed_mods <= 1);
    assert!(q.mods.iter().all(|m| !m.range_mismatch), "{:#?}", q.mods);
    let str_mod = q.mods.iter().find(|m| m.text.contains("Strength")).expect("força");
    assert_eq!(str_mod.status, ModStatus::Exact);
    assert_eq!((str_mod.min, str_mod.max), (Some(40.0), Some(55.0)));
    assert_eq!(q.confidence, Confidence::High);
    assert!(q.score.is_some_and(|s| s > 40.0 && s < 100.0), "{:?}", q.score);
}

#[test]
fn unique_plain_copy_gets_ranges_from_database() {
    let k = EmbeddedKnowledge::poe1();
    let item = fixture("poe1_unique_helmet.txt");
    let q = evaluate_item(&item, k);
    assert!(q.unique.as_ref().is_some_and(|u| u.found), "{:?}", q.unique);
    let res = q.mods.iter().find(|m| m.text.contains("Elemental Resistances")).expect("res");
    // Goldrim: +(30-40)% to all Elemental Resistances no snapshot atual.
    assert_eq!(res.status, ModStatus::FromDatabase, "{res:#?}");
    assert!(res.quality.is_some());
}

#[test]
fn rare_advanced_copy_has_tiers_and_crafting() {
    let k = EmbeddedKnowledge::poe1();
    let item = fixture("poe1_rare_gloves_advanced.txt");
    let q = evaluate_item(&item, k);
    let crafting = q.crafting.as_ref().expect("crafting");
    assert_eq!(crafting.crafted, 1);
    assert!(item.flags.fractured);
    assert!(q.mods.iter().filter(|m| m.tier.is_some()).count() >= 5);
    assert!(q.score.is_some());
}

#[test]
fn rare_upgrade_against_itself_is_equivalent() {
    let k = EmbeddedKnowledge::poe1();
    let item = fixture("poe1_rare_gloves_advanced.txt");
    let profile = BuildProfile::generic(k.rules());
    assert_eq!(compare_items(&item, &item, &profile, k).verdict, Verdict::Equivalent);
}
