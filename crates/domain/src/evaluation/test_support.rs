//! Base de conhecimento falsa para os testes do domínio (regras reais de `rules-poe1.json`).

use std::collections::HashMap;

use super::knowledge::{BaseSpec, GameKnowledge, Rules, TierEntry, TierSource, UniqueModSpec, UniqueSpec};
use crate::item::{AffixSlots, Item, ItemFlags, Modifier, ModifierKind, Rarity, StatRoll};

pub struct FakeKnowledge {
    uniques: Vec<UniqueSpec>,
    tiers: HashMap<String, Vec<TierEntry>>,
    tags: HashMap<String, Vec<String>>,
    bases: HashMap<String, BaseSpec>,
    gems: HashMap<String, Vec<String>>,
    rules: Rules,
}

impl GameKnowledge for FakeKnowledge {
    fn source(&self) -> String {
        "teste".into()
    }
    fn uniques_named(&self, name: &str) -> Vec<&UniqueSpec> {
        self.uniques.iter().filter(|u| u.name == name).collect()
    }
    fn tier_entries(&self, stat_hash: &str) -> &[TierEntry] {
        self.tiers.get(stat_hash).map_or(&[], Vec::as_slice)
    }
    fn stat_tags(&self, stat_hash: &str) -> &[String] {
        self.tags.get(stat_hash).map_or(&[], Vec::as_slice)
    }
    fn base(&self, name: &str) -> Option<&BaseSpec> {
        self.bases.get(name)
    }
    fn gem_tags(&self, gem: &str) -> Option<&[String]> {
        self.gems.get(gem).map(Vec::as_slice)
    }
    fn rules(&self) -> &Rules {
        &self.rules
    }
}

pub const LIFE: &str = "3299347043";
pub const FIRE_RES: &str = "3372524247";
pub const DEX: &str = "3261801346";
pub const STR: &str = "4080418644";
pub const ALL_RES: &str = "2901986750";
pub const CRIT_MULTI: &str = "3556824919";
pub const CRIT_CHANCE: &str = "587431675";
pub const ES_PCT: &str = "4015621042";

/// Texto do mod → hash da trade (só o que os testes usam).
fn hash_of(text: &str) -> &'static str {
    let t = text.to_lowercase();
    if t.contains("maximum life") {
        LIFE
    } else if t.contains("all elemental resistances") {
        ALL_RES
    } else if t.contains("fire resistance") {
        FIRE_RES
    } else if t.contains("dexterity") {
        DEX
    } else if t.contains("strength") {
        STR
    } else if t.contains("critical strike multiplier") {
        CRIT_MULTI
    } else if t.contains("critical strike chance") {
        CRIT_CHANCE
    } else if t.contains("energy shield") {
        ES_PCT
    } else {
        "0"
    }
}

fn tier(group: &str, name: &str, level: u32, range: [f64; 2], weights: &[(&str, u32)]) -> TierEntry {
    TierEntry {
        group: group.into(),
        name: name.into(),
        kind: "Prefix".into(),
        level,
        range,
        source: TierSource::Explicit,
        weights: weights.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
    }
}

fn umod(text: &str, ranges: &[[f64; 2]]) -> UniqueModSpec {
    UniqueModSpec {
        text: text.into(),
        ranges: ranges.to_vec(),
        implicit: false,
        variants: None,
        versions: None,
        group: None,
        tags: vec![],
    }
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

pub fn fake_knowledge() -> FakeKnowledge {
    let rules: Rules =
        serde_json::from_str(include_str!("../../../gamedata/data/rules-poe1.json")).expect("rules-poe1.json válido");

    let mut tiers = HashMap::new();
    tiers.insert(
        LIFE.to_owned(),
        vec![
            tier("IncreasedLife", "Healthy", 5, [10.0, 24.0], &[("default", 1000)]),
            tier("IncreasedLife", "Fecund", 64, [130.0, 144.0], &[("default", 1000)]),
            tier("IncreasedLife", "Prime", 86, [175.0, 189.0], &[("gloves", 0), ("default", 1000)]),
        ],
    );
    tiers.insert(FIRE_RES.to_owned(), vec![tier("FireResistance", "of the Volcano", 60, [42.0, 48.0], &[("default", 1000)])]);
    tiers.insert(CRIT_MULTI.to_owned(), vec![tier("CritMulti", "of Destruction", 74, [35.0, 38.0], &[("default", 1000)])]);
    tiers.insert(CRIT_CHANCE.to_owned(), vec![tier("CritChance", "of Incision", 70, [35.0, 38.0], &[("default", 1000)])]);

    let tags: HashMap<String, Vec<String>> = [
        (LIFE, strings(&["resource", "life"])),
        (FIRE_RES, strings(&["elemental", "fire", "resistance"])),
        (DEX, strings(&["attribute"])),
        (STR, strings(&["attribute"])),
        (ALL_RES, strings(&["elemental", "resistance"])),
        (CRIT_MULTI, strings(&["damage", "critical"])),
        (CRIT_CHANCE, strings(&["critical"])),
        (ES_PCT, strings(&["defences", "energy_shield"])),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();

    let mut bases = HashMap::new();
    bases.insert(
        "Iron Gauntlets".to_owned(),
        BaseSpec { kind: "Gloves".into(), tags: strings(&["gloves", "armour", "default"]), influence_tags: HashMap::new(), sockets: Some(4) },
    );

    let gems: HashMap<String, Vec<String>> = [
        ("Fireball", strings(&["intelligence", "grants_active_skill", "projectile", "spell", "area", "fire"])),
        ("Spell Echo", strings(&["support", "spell"])),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();

    let mut life_implicit = umod("+(25-40) to maximum Life", &[[25.0, 40.0]]);
    life_implicit.implicit = true;
    let headhunter = UniqueSpec {
        name: "Headhunter".into(),
        bases: vec![super::knowledge::UniqueBase { base: "Leather Belt".into(), variants: None }],
        versions: vec![],
        variants: vec![],
        pool: false,
        generated: false,
        flags: vec![],
        foulborn: vec![umod("Damage cannot be Reflected", &[]), umod("(100-120)% increased Chaos Damage", &[[100.0, 120.0]])],
        mods: vec![
            life_implicit,
            umod("+(40-55) to Strength", &[[40.0, 55.0]]),
            umod("+(40-55) to Dexterity", &[[40.0, 55.0]]),
            umod("+(50-60) to maximum Life", &[[50.0, 60.0]]),
            umod("When you Kill a Rare monster, you gain its Modifiers for 60 seconds", &[[60.0, 60.0]]),
        ],
    };
    let mut old_es = umod("(120-150)% increased Energy Shield", &[[120.0, 150.0]]);
    old_es.versions = Some(vec![1]);
    let mut new_es = umod("(150-180)% increased Energy Shield", &[[150.0, 180.0]]);
    new_es.versions = Some(vec![2]);
    let versioned = UniqueSpec {
        name: "Versioned Helm".into(),
        bases: vec![super::knowledge::UniqueBase { base: "Hubris Circlet".into(), variants: None }],
        versions: strings(&["Pre 3.0.0", "Current"]),
        variants: vec![],
        pool: false,
        generated: false,
        flags: vec![],
        foulborn: vec![],
        mods: vec![old_es, new_es],
    };
    let charge = |text: &str, variant: u32| {
        let mut m = umod(text, &[[-1.0, -1.0]]);
        m.variants = Some(vec![variant]);
        m.group = Some(1);
        m
    };
    let boots = UniqueSpec {
        name: "Charge Boots".into(),
        bases: vec![super::knowledge::UniqueBase { base: "Riveted Boots".into(), variants: None }],
        versions: vec![],
        variants: strings(&["Endurance", "Frenzy", "Power"]),
        pool: false,
        generated: false,
        flags: vec![],
        foulborn: vec![],
        mods: vec![
            umod("+(15-25)% to Cold Resistance", &[[15.0, 25.0]]),
            charge("-1 to Maximum Endurance Charges", 1),
            charge("-1 to Maximum Frenzy Charges", 2),
            charge("-1 to Maximum Power Charges", 3),
        ],
    };

    FakeKnowledge { uniques: vec![headhunter, versioned, boots], tiers, tags, bases, gems, rules }
}

pub fn base_item(class: &str, rarity: Rarity, base: &str, mods: Vec<Modifier>) -> Item {
    Item {
        item_class: class.into(),
        rarity,
        name: None,
        base_type: base.into(),
        item_level: Some(86),
        quality: None,
        corrupted: false,
        identified: true,
        sockets: None,
        influences: vec![],
        modifiers: mods,
        properties: vec![],
        unknown_lines: vec![],
        foulborn: false,
        discriminator: None,
        trade_type: None,
        map_tier: None,
        affix_slots: None,
        flags: ItemFlags::default(),
    }
}

/// Luvas raras "Iron Gauntlets" com `(prefixos, sufixos)` ocupados.
pub fn rare_gloves(mods: Vec<Modifier>, slots: (u32, u32)) -> Item {
    let mut item = base_item("Gloves", Rarity::Rare, "Iron Gauntlets", mods);
    item.affix_slots = Some(AffixSlots { prefixes: slots.0, suffixes: slots.1 });
    item
}

/// Mod explícito sem tier nem faixa (Ctrl+C simples).
pub fn plain(text: &str, value: f64) -> Modifier {
    Modifier {
        trade_id: format!("explicit.stat_{}", hash_of(text)),
        text: text.into(),
        kind: ModifierKind::Explicit,
        roll: Some(StatRoll { value, min: None, max: None }),
        ..Modifier::default()
    }
}

/// Mod explícito com tier e faixa (Ctrl+Alt+C).
pub fn tiered(text: &str, value: f64, tier: u32, range: (f64, f64)) -> Modifier {
    Modifier {
        roll: Some(StatRoll { value, min: Some(range.0), max: Some(range.1) }),
        tier: Some(tier),
        ..plain(text, value)
    }
}

pub fn unique_mod(kind: ModifierKind, text: &str, value: f64) -> Modifier {
    let prefix = if kind == ModifierKind::Implicit { "implicit" } else { "explicit" };
    Modifier { trade_id: format!("{prefix}.stat_{}", hash_of(text)), kind, ..plain(text, value) }
}

pub fn unique_item(name: &str, base: &str, mods: Vec<Modifier>) -> Item {
    let mut item = base_item("Belts", Rarity::Unique, base, mods);
    item.name = Some(name.into());
    item
}
