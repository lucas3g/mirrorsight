//! Base de conhecimento do PoE1 embutida no binário.
//!
//! Os JSON em `data/` são gerados por `node tools/gamedata/update-gamedata.mjs` a partir do
//! Path of Building Community (rodar a cada liga/patch); `rules-poe1.json` é editado à mão.
//! A carga é preguiçosa: o primeiro uso faz o parse (alguns MB) e o resultado fica em memória.

use std::collections::HashMap;
use std::sync::OnceLock;

use mirrorsight_domain::evaluation::knowledge::{BaseSpec, GameKnowledge, Rules, TierEntry, UniqueSpec};
use serde::Deserialize;

const UNIQUES: &str = include_str!("../data/uniques-poe1.json");
const MODS: &str = include_str!("../data/mods-poe1.json");
const BASES: &str = include_str!("../data/bases-poe1.json");
const GEMS: &str = include_str!("../data/gems-poe1.json");
const RULES: &str = include_str!("../data/rules-poe1.json");

#[derive(Debug, Default, Deserialize)]
struct Meta {
    #[serde(default)]
    source: String,
    #[serde(default)]
    generated: String,
}

#[derive(Deserialize)]
struct UniquesFile {
    #[serde(default)]
    meta: Meta,
    uniques: Vec<UniqueSpec>,
}

#[derive(Deserialize)]
struct ModsFile {
    stats: HashMap<String, Vec<TierEntry>>,
    #[serde(default)]
    tags: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct BasesFile {
    bases: HashMap<String, BaseSpec>,
}

#[derive(Deserialize)]
struct GemsFile {
    gems: HashMap<String, Vec<String>>,
}

/// Implementação de [`GameKnowledge`] com os dados embutidos.
pub struct EmbeddedKnowledge {
    source: String,
    uniques: Vec<UniqueSpec>,
    /// Nome em minúsculas → índices em `uniques`.
    by_name: HashMap<String, Vec<usize>>,
    stats: HashMap<String, Vec<TierEntry>>,
    tags: HashMap<String, Vec<String>>,
    bases: HashMap<String, BaseSpec>,
    gems: HashMap<String, Vec<String>>,
    rules: Rules,
}

impl EmbeddedKnowledge {
    /// Base do PoE1 (carregada uma vez).
    ///
    /// # Panics
    /// Se os JSON embutidos estiverem corrompidos — eles são validados pelos testes do crate.
    #[allow(clippy::expect_used)]
    pub fn poe1() -> &'static Self {
        static CELL: OnceLock<EmbeddedKnowledge> = OnceLock::new();
        CELL.get_or_init(|| Self::from_json(UNIQUES, MODS, BASES, GEMS, RULES).expect("dados embutidos do PoE1 válidos"))
    }

    /// Monta a base a partir dos textos JSON (útil para testes e para dados externos).
    pub fn from_json(uniques: &str, mods: &str, bases: &str, gems: &str, rules: &str) -> Result<Self, String> {
        let uniques: UniquesFile = serde_json::from_str(uniques).map_err(|e| format!("uniques: {e}"))?;
        let mods: ModsFile = serde_json::from_str(mods).map_err(|e| format!("mods: {e}"))?;
        let bases: BasesFile = serde_json::from_str(bases).map_err(|e| format!("bases: {e}"))?;
        let gems: GemsFile = serde_json::from_str(gems).map_err(|e| format!("gems: {e}"))?;
        let rules: Rules = serde_json::from_str(rules).map_err(|e| format!("rules: {e}"))?;
        let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, u) in uniques.uniques.iter().enumerate() {
            by_name.entry(u.name.to_lowercase()).or_default().push(i);
        }
        let source = match (uniques.meta.source.is_empty(), uniques.meta.generated.is_empty()) {
            (false, false) => format!("{} · {}", uniques.meta.source, uniques.meta.generated),
            (false, true) => uniques.meta.source,
            _ => "embutido".to_owned(),
        };
        Ok(Self {
            source,
            uniques: uniques.uniques,
            by_name,
            stats: mods.stats,
            tags: mods.tags,
            bases: bases.bases,
            gems: gems.gems,
            rules,
        })
    }
}

impl GameKnowledge for EmbeddedKnowledge {
    fn source(&self) -> String {
        self.source.clone()
    }

    fn uniques_named(&self, name: &str) -> Vec<&UniqueSpec> {
        self.by_name
            .get(&name.trim().to_lowercase())
            .map(|idx| idx.iter().filter_map(|i| self.uniques.get(*i)).collect())
            .unwrap_or_default()
    }

    fn tier_entries(&self, stat_hash: &str) -> &[TierEntry] {
        self.stats.get(stat_hash).map_or(&[], Vec::as_slice)
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

/// Base do PoE1 carregada só no primeiro uso (o parse dos JSON fica fora da abertura do app).
/// Chame [`LazyPoe1::warm_up`] numa thread de fundo para não pesar no primeiro price check.
#[derive(Debug, Clone, Copy, Default)]
pub struct LazyPoe1;

impl LazyPoe1 {
    /// Faz o parse agora, numa thread própria.
    pub fn warm_up() {
        std::thread::spawn(|| {
            EmbeddedKnowledge::poe1();
        });
    }
}

impl GameKnowledge for LazyPoe1 {
    fn source(&self) -> String {
        EmbeddedKnowledge::poe1().source()
    }
    fn uniques_named(&self, name: &str) -> Vec<&UniqueSpec> {
        EmbeddedKnowledge::poe1().uniques_named(name)
    }
    fn tier_entries(&self, stat_hash: &str) -> &[TierEntry] {
        EmbeddedKnowledge::poe1().tier_entries(stat_hash)
    }
    fn stat_tags(&self, stat_hash: &str) -> &[String] {
        EmbeddedKnowledge::poe1().stat_tags(stat_hash)
    }
    fn base(&self, name: &str) -> Option<&BaseSpec> {
        EmbeddedKnowledge::poe1().base(name)
    }
    fn gem_tags(&self, gem: &str) -> Option<&[String]> {
        EmbeddedKnowledge::poe1().gem_tags(gem)
    }
    fn rules(&self) -> &Rules {
        EmbeddedKnowledge::poe1().rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirrorsight_domain::evaluation::knowledge::ladder_for;

    fn k() -> &'static EmbeddedKnowledge {
        EmbeddedKnowledge::poe1()
    }

    #[test]
    fn loads_snapshot_with_source() {
        assert!(k().source().contains("PoB"), "{}", k().source());
        assert!(k().uniques.len() > 1000);
    }

    #[test]
    fn headhunter_ranges() {
        let hh = k().uniques_named("Headhunter");
        assert_eq!(hh.len(), 1);
        let hh = hh[0];
        assert!(hh.has_base("Leather Belt"));
        let find = |text: &str, implicit: bool| {
            hh.mods.iter().find(|m| m.text.contains(text) && m.implicit == implicit).and_then(|m| m.range())
        };
        assert_eq!(find("to Strength", false), Some((40.0, 55.0)));
        assert_eq!(find("to Dexterity", false), Some((40.0, 55.0)));
        assert_eq!(find("maximum Life", false), Some((50.0, 60.0)));
        assert_eq!(find("maximum Life", true), Some((25.0, 40.0)));
        // Mod sem faixa (efeito) é fixo.
        assert!(hh.mods.iter().any(|m| m.text.starts_with("When you Kill a Rare monster") && !m.is_variable()));
    }

    #[test]
    fn variants_and_pools() {
        let hyper = k().uniques_named("Hyperboreus");
        assert!(!hyper.is_empty());
        assert!(hyper[0].variants.len() > 3, "{:?}", hyper[0].variants);
        let eye = k().uniques_named("Watcher's Eye");
        assert_eq!(eye.len(), 1);
        assert!(eye[0].pool);
        assert!(eye[0].mods.iter().filter(|m| m.variants.is_some()).count() > 50);
        // Únicos montados por código no PoB: conhecidos, sem mods.
        let emblem = k().uniques_named("Precursor's Emblem");
        assert!(emblem.first().is_some_and(|u| u.generated));
    }

    #[test]
    fn life_ladder_for_gloves() {
        let base = k().base("Iron Gauntlets").expect("base");
        let tags = base.tags_with(&[]);
        let ladder = ladder_for(k().tier_entries("3299347043"), Some(&tags), None).expect("escada de vida");
        assert_eq!(ladder.group, "IncreasedLife");
        // Luvas não rolam os tiers mais altos de vida de armaduras de corpo.
        assert!(ladder.count() >= 7, "{}", ladder.count());
        let (lo, hi) = ladder.tier_range(1).expect("T1");
        assert!(hi > lo && hi >= 80.0, "{lo}-{hi}");
        assert_eq!(k().stat_tags("3299347043"), ["resource", "life"]);
    }

    #[test]
    fn lazy_handle_reads_the_same_data() {
        let lazy = LazyPoe1;
        assert_eq!(lazy.source(), k().source());
        assert_eq!(lazy.uniques_named("Headhunter").len(), 1);
    }

    #[test]
    fn foulborn_mutations_are_in_the_snapshot() {
        let aegis = k().uniques_named("Aegis Aurora");
        assert!(aegis[0].foulborn.iter().any(|m| m.text == "Damage cannot be Reflected"), "{:?}", aegis[0].foulborn);
    }

    #[test]
    fn gem_tags() {
        let fireball = k().gem_tags("Fireball").expect("Fireball");
        assert!(fireball.iter().any(|t| t == "spell") && fireball.iter().any(|t| t == "fire"));
    }
}
