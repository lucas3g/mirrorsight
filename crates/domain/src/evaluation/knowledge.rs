//! Base de conhecimento do jogo (porta) e os tipos dos dados que ela entrega.
//!
//! Os dados vêm de um snapshot do Path of Building (ver `tools/gamedata`); as regras de peso,
//! sinergia e limiares vêm de `rules-poe1.json`. Nada aqui é escrito para um item específico.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::item::Influence;

/// Faixa `(min, max)`.
pub type Range = (f64, f64);

// ------------------------------------------------------------------ únicos

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UniqueBase {
    pub base: String,
    /// Variantes em que essa base vale (índices a partir de 1); `None` = todas.
    #[serde(default)]
    pub variants: Option<Vec<u32>>,
}

/// Um mod de um único, como o PoB descreve: texto com faixas e em quais versões/variantes vale.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UniqueModSpec {
    /// Texto com as faixas: "+(40-55) to Strength".
    pub text: String,
    /// Faixa de cada número do texto, na ordem (números fixos viram `[n, n]`).
    #[serde(default)]
    pub ranges: Vec<[f64; 2]>,
    #[serde(default)]
    pub implicit: bool,
    #[serde(default)]
    pub variants: Option<Vec<u32>>,
    #[serde(default)]
    pub versions: Option<Vec<u32>>,
    /// Mods do mesmo grupo são alternativas exclusivas (ex.: uma carga entre três).
    #[serde(default)]
    pub group: Option<u32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl UniqueModSpec {
    /// Faixa agregada (média de cada número, como o `StatRoll`). `None` = mod sem número.
    pub fn range(&self) -> Option<Range> {
        if self.ranges.is_empty() {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        let n = self.ranges.len() as f64;
        Some((
            self.ranges.iter().map(|r| r[0]).sum::<f64>() / n,
            self.ranges.iter().map(|r| r[1]).sum::<f64>() / n,
        ))
    }

    /// Algum número do mod rola (faixa com amplitude).
    pub fn is_variable(&self) -> bool {
        self.ranges.iter().any(|r| (r[1] - r[0]).abs() > f64::EPSILON)
    }

    /// Vale na versão/variante dadas (`None` = não filtrar por aquela dimensão).
    pub fn applies(&self, version: Option<u32>, variant: Option<u32>) -> bool {
        let ok = |list: &Option<Vec<u32>>, pick: Option<u32>| match (list, pick) {
            (Some(list), Some(p)) => list.contains(&p),
            _ => true,
        };
        ok(&self.versions, version) && ok(&self.variants, variant)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UniqueSpec {
    pub name: String,
    #[serde(default)]
    pub bases: Vec<UniqueBase>,
    /// Versões por patch ("Pre 3.19.0", …, "Current"): a última é a atual.
    #[serde(default)]
    pub versions: Vec<String>,
    /// Variantes (escolhas do item ou, em dados antigos, versões).
    #[serde(default)]
    pub variants: Vec<String>,
    /// Único de pool (Watcher's Eye, Bound by Destiny): vários mods de variante ao mesmo tempo.
    #[serde(default)]
    pub pool: bool,
    /// Gerado por código no PoB: conhecido, mas sem os mods no snapshot.
    #[serde(default)]
    pub generated: bool,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub mods: Vec<UniqueModSpec>,
    /// Mutações Foulborn possíveis: no item Foulborn, uma delas substitui um mod original.
    #[serde(default)]
    pub foulborn: Vec<UniqueModSpec>,
}

impl UniqueSpec {
    pub fn has_base(&self, base: &str) -> bool {
        self.bases.iter().any(|b| b.base.eq_ignore_ascii_case(base))
    }
}

// ------------------------------------------------------------------ tiers

/// De onde vem uma entrada de tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TierSource {
    Explicit,
    Crafted,
    Corrupted,
}

/// Um mod que pode rolar um stat (uma linha da escada de tiers).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TierEntry {
    /// Grupo do mod ("IncreasedLife"): tiers do mesmo grupo se excluem.
    #[serde(rename = "g", default)]
    pub group: String,
    /// Nome do afixo ("Fecund").
    #[serde(rename = "n", default)]
    pub name: String,
    /// "Prefix", "Suffix", "Corrupted"…
    #[serde(rename = "t", default)]
    pub kind: String,
    #[serde(rename = "l", default)]
    pub level: u32,
    /// Faixa do stat nesse mod (média, como o `StatRoll`).
    #[serde(rename = "r")]
    pub range: [f64; 2],
    #[serde(rename = "s")]
    pub source: TierSource,
    /// Pesos de spawn em ordem: vale o primeiro cuja tag a base tem.
    #[serde(rename = "w", default)]
    pub weights: Vec<(String, u32)>,
}

impl TierEntry {
    pub const fn range(&self) -> Range {
        (self.range[0], self.range[1])
    }

    /// Peso de spawn numa base com essas tags (regra do jogo: a primeira tag da lista que a base tem).
    pub fn weight_for(&self, tags: &[String]) -> u32 {
        self.weights.iter().find(|(k, _)| tags.iter().any(|t| t == k)).map_or(0, |(_, w)| *w)
    }

    /// Spawna em alguma base (peso positivo em alguma tag).
    pub fn spawns_anywhere(&self) -> bool {
        self.weights.iter().any(|(_, w)| *w > 0)
    }
}

/// Escada de tiers de um stat numa base: `tiers[0]` é o T1.
#[derive(Debug, Clone, PartialEq)]
pub struct TierLadder {
    pub group: String,
    pub tiers: Vec<TierEntry>,
}

impl TierLadder {
    pub fn count(&self) -> u32 {
        u32::try_from(self.tiers.len()).unwrap_or(u32::MAX)
    }

    /// Faixa do tier `n` (1 = melhor).
    pub fn tier_range(&self, n: u32) -> Option<Range> {
        let idx = usize::try_from(n.checked_sub(1)?).ok()?;
        self.tiers.get(idx).map(TierEntry::range)
    }

    /// Faixa de todos os tiers: do mínimo do pior ao máximo do T1.
    pub fn global_range(&self) -> Option<Range> {
        let min = self.tiers.iter().map(|t| t.range[0]).fold(f64::INFINITY, f64::min);
        let max = self.tiers.iter().map(|t| t.range[1]).fold(f64::NEG_INFINITY, f64::max);
        (min.is_finite() && max.is_finite()).then_some((min, max))
    }

    /// Nível de item mínimo para o T1.
    pub fn top_level(&self) -> Option<u32> {
        self.tiers.first().map(|t| t.level)
    }

    /// Tier em que o valor cabe, se só um couber.
    pub fn infer_tier(&self, value: f64) -> Option<u32> {
        let fits: Vec<usize> = self
            .tiers
            .iter()
            .enumerate()
            .filter(|(_, t)| crate::evaluation::stat::fit(value, t.range()).is_some())
            .map(|(i, _)| i)
            .collect();
        match fits.as_slice() {
            [one] => u32::try_from(one + 1).ok(),
            _ => None,
        }
    }
}

/// Monta a escada de tiers explícitos de um stat para uma base.
///
/// - `mod_name` (do Ctrl+Alt+C ou da trade) identifica o grupo exato do mod (puro ou híbrido);
///   sem ele, escolhe o grupo que spawna na base com mais tiers.
/// - `base_tags` são as tags da base (mais as de influência); `None` = base desconhecida
///   (aceita qualquer mod que spawne em alguma base).
pub fn ladder_for(entries: &[TierEntry], base_tags: Option<&[String]>, mod_name: Option<&str>) -> Option<TierLadder> {
    let spawns = |e: &TierEntry| match base_tags {
        Some(tags) => e.weight_for(tags) > 0,
        None => e.spawns_anywhere(),
    };
    let explicit: Vec<&TierEntry> = entries.iter().filter(|e| e.source == TierSource::Explicit).collect();
    let group = match mod_name.and_then(|n| explicit.iter().find(|e| e.name.eq_ignore_ascii_case(n))) {
        Some(e) => e.group.clone(),
        None => {
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for e in explicit.iter().filter(|e| spawns(e)) {
                *counts.entry(e.group.as_str()).or_default() += 1;
            }
            // Mais tiers primeiro; empate pelo nome (determinístico).
            let mut groups: Vec<(&str, usize)> = counts.into_iter().collect();
            groups.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
            groups.first()?.0.to_owned()
        }
    };
    let in_group: Vec<&TierEntry> = explicit.iter().copied().filter(|e| e.group == group).collect();
    let mut tiers: Vec<TierEntry> = in_group.iter().copied().filter(|e| spawns(e)).cloned().collect();
    if tiers.is_empty() {
        // Base sem esse grupo (ex.: tags de influência desconhecidas): usa o que spawna em alguma base.
        tiers = in_group.iter().copied().filter(|e| e.spawns_anywhere()).cloned().collect();
    }
    if tiers.is_empty() {
        return None;
    }
    tiers.sort_by(|a, b| b.level.cmp(&a.level).then(b.range[1].total_cmp(&a.range[1])));
    Some(TierLadder { group, tiers })
}

// ------------------------------------------------------------------ bases

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BaseSpec {
    /// Tipo no PoB ("Gloves", "Body Armour", "One Handed Sword").
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Influência → tag extra de spawn ("shaper" → "gloves_shaper").
    #[serde(rename = "infl", default)]
    pub influence_tags: HashMap<String, String>,
    /// Máximo de sockets da base.
    #[serde(default)]
    pub sockets: Option<u32>,
}

impl BaseSpec {
    /// Tags de spawn com as das influências do item.
    pub fn tags_with(&self, influences: &[Influence]) -> Vec<String> {
        let mut tags = self.tags.clone();
        for influence in influences {
            if let Some(tag) = self.influence_tags.get(influence_key(*influence)) {
                tags.push(tag.clone());
            }
        }
        tags
    }
}

/// Nome interno da influência nas tags do jogo.
pub const fn influence_key(influence: Influence) -> &'static str {
    match influence {
        Influence::Shaper => "shaper",
        Influence::Elder => "elder",
        Influence::Crusader => "crusader",
        Influence::Hunter => "basilisk",
        Influence::Redeemer => "eyrie",
        Influence::Warlord => "adjudicator",
        Influence::Searing => "cleansing",
        Influence::Eater => "tangle",
    }
}

// ------------------------------------------------------------------ regras (rules-poe1.json)

/// Tags atribuídas por palavra-chave do texto quando o stat não está na base de tiers.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KeywordTags {
    /// Trecho em minúsculas procurado no template do mod.
    pub contains: String,
    pub tags: Vec<String>,
}

/// Um stat que equivale a outros ("+#% to all elemental resistances" → fogo, gelo, raio).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StatAlias {
    pub from: String,
    pub to: Vec<String>,
}

/// Dois stats que valem mais juntos (crit chance + crit multi). `a` e `b` são trechos do
/// template do stat ("critical strike chance"): casam com qualquer stat que os contenha.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Synergy {
    pub a: String,
    pub b: String,
    pub bonus: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Thresholds {
    /// Perda de um único stat acima disso conta como "perda relevante" (destacada).
    pub relevant_loss: f64,
    pub grade_excellent: f64,
    pub grade_good: f64,
    pub grade_average: f64,
    /// Fração (peso) de mods inconclusivos até a qual a confiança é alta/média.
    pub confidence_high: f64,
    pub confidence_medium: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NeedsRules {
    pub res_cap: f64,
    /// Elemento → templates do stat ("fire" → ["#% to fire resistance"]).
    pub res_stats: HashMap<String, Vec<String>>,
    /// Peso de cada ponto de resistência abaixo do cap, por elemento.
    pub res_point_weight: HashMap<String, f64>,
    /// Atributo → templates ("str" → ["# to strength"]).
    pub attr_stats: HashMap<String, Vec<String>>,
    /// Penalidade ao deixar de cumprir requisito de atributo.
    pub attr_penalty: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BuildRules {
    /// Multiplicador das tags que a skill principal usa.
    pub skill_boost: f64,
    /// Tag de gema → tags de mod ("spell" → ["caster"]).
    pub gem_tag_map: HashMap<String, Vec<String>>,
    /// Tags que, presentes na skill, são as únicas que contam (minion: o dano é dos lacaios,
    /// não do feitiço que os invoca).
    #[serde(default)]
    pub dominant_tags: Vec<String>,
    /// Crit chance (%) a partir da qual o build é de crit.
    pub crit_threshold: f64,
    pub crit_boost: f64,
    /// Pool dominante (life/ES): multiplica as tags dele e amortece as do outro.
    pub pool_boost: f64,
    pub pool_damp: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rules {
    /// Peso genérico (sem build) de cada tag de mod.
    pub tag_weights: HashMap<String, f64>,
    pub default_weight: f64,
    #[serde(default)]
    pub keyword_tags: Vec<KeywordTags>,
    /// `property.<id>` → tags.
    #[serde(default)]
    pub property_tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub stat_aliases: Vec<StatAlias>,
    /// Escala de referência por template quando a base não tem o stat.
    #[serde(default)]
    pub stat_scales: HashMap<String, f64>,
    #[serde(default)]
    pub synergies: Vec<Synergy>,
    /// Famílias de tags exclusivas (tipos de dano; ataque × feitiço × minion).
    #[serde(default)]
    pub exclusive_families: Vec<Vec<String>>,
    /// Tags que marcam um mod como "de dano" para o corte por família.
    #[serde(default)]
    pub damage_tags: Vec<String>,
    /// Multiplicador de mods de dano fora das famílias do build.
    pub offtype_factor: f64,
    /// Peso dos crafted na qualidade (são substituíveis).
    pub crafted_weight: f64,
    /// Peso dos implícitos comuns na qualidade.
    pub implicit_weight: f64,
    /// Classe → peso dos links na qualidade de únicos.
    #[serde(default)]
    pub links: HashMap<String, f64>,
    /// Classes sem comparação de equipamento (mapas, contratos…).
    #[serde(default)]
    pub excluded_classes: Vec<String>,
    pub thresholds: Thresholds,
    pub needs: NeedsRules,
    pub build: BuildRules,
}

impl Rules {
    /// Peso genérico de um conjunto de tags: o maior entre elas.
    pub fn generic_weight(&self, tags: &[String]) -> f64 {
        tags.iter().filter_map(|t| self.tag_weights.get(t)).copied().fold(None, |acc: Option<f64>, w| {
            Some(acc.map_or(w, |a| a.max(w)))
        })
        .unwrap_or(self.default_weight)
    }
}

// ------------------------------------------------------------------ porta

/// Acesso somente-leitura à base de conhecimento.
pub trait GameKnowledge: Send + Sync {
    /// Origem dos dados ("PoB v2.69.0 · 2026-11-26").
    fn source(&self) -> String;
    /// Únicos com esse nome (pode haver mais de um, com bases diferentes).
    fn uniques_named(&self, name: &str) -> Vec<&UniqueSpec>;
    /// Todas as entradas de tier (explícito, crafted, corrompido) de um stat da trade ("3299347043").
    fn tier_entries(&self, stat_hash: &str) -> &[TierEntry];
    /// Tags do stat no jogo ("life", "resource").
    fn stat_tags(&self, stat_hash: &str) -> &[String];
    fn base(&self, name: &str) -> Option<&BaseSpec>;
    /// Tags de uma gema ("Fireball" → spell, fire, projectile…).
    fn gem_tags(&self, gem: &str) -> Option<&[String]>;
    fn rules(&self) -> &Rules;
}

/// Referência estática (`&'static EmbeddedKnowledge`) também serve como base de conhecimento.
impl<T: GameKnowledge + ?Sized> GameKnowledge for &T {
    fn source(&self) -> String {
        (**self).source()
    }
    fn uniques_named(&self, name: &str) -> Vec<&UniqueSpec> {
        (**self).uniques_named(name)
    }
    fn tier_entries(&self, stat_hash: &str) -> &[TierEntry] {
        (**self).tier_entries(stat_hash)
    }
    fn stat_tags(&self, stat_hash: &str) -> &[String] {
        (**self).stat_tags(stat_hash)
    }
    fn base(&self, name: &str) -> Option<&BaseSpec> {
        (**self).base(name)
    }
    fn gem_tags(&self, gem: &str) -> Option<&[String]> {
        (**self).gem_tags(gem)
    }
    fn rules(&self) -> &Rules {
        (**self).rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(group: &str, name: &str, level: u32, range: [f64; 2], weights: &[(&str, u32)]) -> TierEntry {
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

    fn life() -> Vec<TierEntry> {
        vec![
            entry("IncreasedLife", "Healthy", 5, [10.0, 24.0], &[("default", 1000)]),
            entry("IncreasedLife", "Fecund", 64, [130.0, 144.0], &[("default", 1000)]),
            entry("IncreasedLife", "Prime", 86, [175.0, 189.0], &[("gloves", 0), ("default", 1000)]),
            entry("IncreasedLife", "Essences", 82, [91.0, 105.0], &[("default", 0)]),
            entry("DefencesAndLife", "Rhino's", 30, [20.0, 30.0], &[("default", 1000)]),
        ]
    }

    #[test]
    fn ladder_orders_by_level_and_respects_base_tags() {
        let tags = vec!["gloves".to_owned(), "default".to_owned()];
        let ladder = ladder_for(&life(), Some(&tags), None).unwrap();
        assert_eq!(ladder.group, "IncreasedLife");
        let names: Vec<&str> = ladder.tiers.iter().map(|t| t.name.as_str()).collect();
        // Prime não spawna em luvas; essência nunca spawna.
        assert_eq!(names, ["Fecund", "Healthy"]);
        assert_eq!(ladder.tier_range(1), Some((130.0, 144.0)));
        assert_eq!(ladder.global_range(), Some((10.0, 144.0)));
        assert_eq!(ladder.infer_tier(20.0), Some(2));
        assert_eq!(ladder.infer_tier(60.0), None);
    }

    #[test]
    fn mod_name_selects_hybrid_group() {
        let ladder = ladder_for(&life(), None, Some("Rhino's")).unwrap();
        assert_eq!(ladder.group, "DefencesAndLife");
        assert_eq!(ladder.count(), 1);
    }

    #[test]
    fn unique_mod_applies_by_version_and_variant() {
        let m = UniqueModSpec {
            text: "+(40-55) to Strength".into(),
            ranges: vec![[40.0, 55.0]],
            implicit: false,
            variants: Some(vec![2]),
            versions: None,
            group: None,
            tags: vec![],
        };
        assert!(m.applies(Some(1), Some(2)));
        assert!(!m.applies(None, Some(1)));
        assert!(m.applies(None, None));
        assert_eq!(m.range(), Some((40.0, 55.0)));
        assert!(m.is_variable());
    }
}
