//! Qualidade do item — comercial, independente da build.

use serde::{Deserialize, Serialize};

use super::knowledge::GameKnowledge;
use super::mods::{evaluate_rare_mod, evaluated_kind, ModEvaluation, ModStatus};
use super::unique::{evaluate_unique, UniqueMatch};
use crate::compare::{affix_limit, valued_affix};
use crate::item::{Affix, Item, ModifierKind, Rarity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grade {
    Excellent,
    Good,
    Average,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    /// Um nível abaixo.
    pub const fn lower(self) -> Self {
        match self {
            Self::High => Self::Medium,
            _ => Self::Low,
        }
    }
}

/// O que impede mexer no item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CraftBlock {
    Corrupted,
    Mirrored,
    Split,
}

/// Quanto o item ainda pode melhorar por crafting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CraftingPotential {
    pub open_prefixes: Option<u32>,
    pub open_suffixes: Option<u32>,
    /// Tipo de afixo mais valioso para a classe (sufixo em luvas, prefixo em armaduras…).
    pub valued: Option<Affix>,
    /// Mods de bancada (substituíveis).
    pub crafted: u32,
    /// Fractured de tier alto (T1/T2): base garantida.
    pub fractured_high_tier: u32,
    /// O ilvl permite o T1 de todos os mods com escada conhecida.
    pub ilvl_allows_top_tier: Option<bool>,
    pub blocked: Vec<CraftBlock>,
    /// 0–1.
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocketInfo {
    pub sockets: u32,
    pub links: u32,
    pub max: u32,
    /// Peso dos links na nota do item (classe).
    pub weight: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemQuality {
    /// 0–100. `None` = não deu para avaliar (sem mods mensuráveis).
    pub score: Option<f64>,
    pub grade: Option<Grade>,
    pub confidence: Confidence,
    /// Média ponderada dos rolls/tiers (0–1).
    pub mods_score: Option<f64>,
    pub mods: Vec<ModEvaluation>,
    pub crafting: Option<CraftingPotential>,
    pub unique: Option<UniqueMatch>,
    pub sockets: Option<SocketInfo>,
}

/// Avalia a qualidade do item.
pub fn evaluate_item(item: &Item, knowledge: &dyn GameKnowledge) -> ItemQuality {
    if item.rarity == Rarity::Unique {
        evaluate_unique_item(item, knowledge)
    } else {
        evaluate_rare_item(item, knowledge)
    }
}

fn evaluate_rare_item(item: &Item, knowledge: &dyn GameKnowledge) -> ItemQuality {
    let mods: Vec<ModEvaluation> = item
        .modifiers
        .iter()
        .filter(|m| evaluated_kind(m.kind))
        .map(|m| evaluate_rare_mod(m, item, knowledge))
        .collect();
    let crafting = crafting_potential(item, &mods, knowledge);
    let mods_score = weighted_mean(&mods);
    // Slots desconhecidos (origem sem prefix/suffix): o potencial não foi medido, e não pode
    // contar como zero — senão a mesma peça tira nota menor só por ter sido copiada sem afixos.
    let crafting_known = item.affix_slots.is_some() || !crafting.blocked.is_empty();
    let score = mods_score.map(|s| 100.0 * if crafting_known { 0.75 * s + 0.25 * crafting.score } else { s });
    finish(score, mods_score, mods, Some(crafting), None, None, knowledge)
}

fn evaluate_unique_item(item: &Item, knowledge: &dyn GameKnowledge) -> ItemQuality {
    let (found, mods) = evaluate_unique(item, knowledge);
    let sockets = socket_info(item, knowledge);
    let mods_score = weighted_mean(&mods);
    let links = sockets.as_ref().filter(|s| s.weight > 0.0 && s.max > 0);
    let score = match (mods_score, links) {
        (Some(m), Some(s)) => Some(100.0 * ((1.0 - s.weight) * m + s.weight * f64::from(s.links) / f64::from(s.max))),
        (Some(m), None) => Some(100.0 * m),
        // Só mods fixos: a nota fica nos links, quando importam.
        (None, Some(s)) if found.found => Some(100.0 * f64::from(s.links) / f64::from(s.max)),
        _ => None,
    };
    finish(score, mods_score, mods, None, Some(found), sockets, knowledge)
}

fn finish(
    score: Option<f64>,
    mods_score: Option<f64>,
    mods: Vec<ModEvaluation>,
    crafting: Option<CraftingPotential>,
    unique: Option<UniqueMatch>,
    sockets: Option<SocketInfo>,
    knowledge: &dyn GameKnowledge,
) -> ItemQuality {
    let t = &knowledge.rules().thresholds;
    let grade = score.map(|s| {
        if s >= t.grade_excellent {
            Grade::Excellent
        } else if s >= t.grade_good {
            Grade::Good
        } else if s >= t.grade_average {
            Grade::Average
        } else {
            Grade::Weak
        }
    });
    let confidence = confidence_of(&mods, score.is_some(), knowledge);
    ItemQuality { score: score.map(round1), grade, confidence, mods_score, mods, crafting, unique, sockets }
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Confiança pela fração (em peso) de mods inconclusivos.
fn confidence_of(mods: &[ModEvaluation], has_score: bool, knowledge: &dyn GameKnowledge) -> Confidence {
    if !has_score {
        return Confidence::Low;
    }
    let t = &knowledge.rules().thresholds;
    let relevant = mods.iter().filter(|m| !matches!(m.status, ModStatus::Fixed | ModStatus::Variant | ModStatus::Mutated));
    let (mut total, mut unknown) = (0.0, 0.0);
    for m in relevant {
        total += m.importance;
        if m.status.is_inconclusive() {
            unknown += m.importance;
        }
    }
    let share = if total > 0.0 { unknown / total } else { 0.0 };
    if share <= t.confidence_high {
        Confidence::High
    } else if share <= t.confidence_medium {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

/// Média das notas ponderada pela importância (só mods mensuráveis).
pub fn weighted_mean(mods: &[ModEvaluation]) -> Option<f64> {
    let (mut sum, mut weight) = (0.0, 0.0);
    for m in mods {
        if let Some(q) = m.quality {
            sum += q * m.importance;
            weight += m.importance;
        }
    }
    (weight > 0.0).then(|| sum / weight)
}

fn crafting_potential(item: &Item, mods: &[ModEvaluation], knowledge: &dyn GameKnowledge) -> CraftingPotential {
    let mut blocked = Vec::new();
    if item.corrupted {
        blocked.push(CraftBlock::Corrupted);
    }
    if item.flags.mirrored {
        blocked.push(CraftBlock::Mirrored);
    }
    if item.flags.split {
        blocked.push(CraftBlock::Split);
    }
    let limit = affix_limit(item.rarity);
    let open = |affix: Affix| match (limit, item.affix_slots) {
        (Some(l), Some(s)) => Some(l.saturating_sub(s.used(affix))),
        _ => None,
    };
    let (open_prefixes, open_suffixes) = (open(Affix::Prefix), open(Affix::Suffix));
    let valued = valued_affix(&item.item_class);
    let crafted = count(mods.iter().filter(|m| m.kind == ModifierKind::Crafted));
    let fractured_high_tier =
        count(mods.iter().filter(|m| m.kind == ModifierKind::Fractured && m.tier.is_some_and(|t| t <= 2)));
    let ilvl_allows_top_tier = item.item_level.and_then(|ilvl| {
        let tops: Vec<u32> = item
            .modifiers
            .iter()
            .filter(|m| matches!(m.kind, ModifierKind::Explicit | ModifierKind::Fractured))
            .filter_map(|m| super::mods::ladder_of(m, item, knowledge).and_then(|l| l.top_level()))
            .collect();
        (!tops.is_empty()).then(|| tops.iter().all(|lvl| ilvl >= *lvl))
    });

    let score = if blocked.is_empty() {
        let open_valued = match valued {
            Some(Affix::Prefix) => open_prefixes,
            Some(Affix::Suffix) => open_suffixes,
            None => None,
        }
        .unwrap_or(0);
        let open_total = open_prefixes.unwrap_or(0) + open_suffixes.unwrap_or(0);
        let open_other = open_total.saturating_sub(open_valued);
        let raw = 0.35 * f64::from(open_valued.min(2))
            + 0.12 * f64::from(open_other.min(3))
            + 0.2 * f64::from(crafted.min(1))
            + 0.2 * f64::from(fractured_high_tier.min(1))
            + if ilvl_allows_top_tier == Some(true) { 0.1 } else { 0.0 };
        raw.clamp(0.0, 1.0)
    } else {
        0.0
    };
    CraftingPotential {
        open_prefixes,
        open_suffixes,
        valued,
        crafted,
        fractured_high_tier,
        ilvl_allows_top_tier,
        blocked,
        score,
    }
}

fn count<T>(it: impl Iterator<Item = T>) -> u32 {
    u32::try_from(it.count()).unwrap_or(u32::MAX)
}

fn socket_info(item: &Item, knowledge: &dyn GameKnowledge) -> Option<SocketInfo> {
    let sockets = item.socket_count();
    let weight = knowledge.rules().links.get(&item.item_class).copied().unwrap_or(0.0);
    if sockets == 0 && weight <= 0.0 {
        return None;
    }
    let max = knowledge.base(&item.base_type).and_then(|b| b.sockets).unwrap_or(6).max(1);
    Some(SocketInfo { sockets, links: item.max_links(), max, weight })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::test_support::{fake_knowledge, rare_gloves, tiered};

    #[test]
    fn rare_quality_uses_tiers_rolls_and_crafting() {
        let k = fake_knowledge();
        let item = rare_gloves(vec![tiered("+140 to maximum Life", 140.0, 1, (130.0, 144.0))], (1, 0));
        let q = evaluate_item(&item, &k);
        let life = &q.mods[0];
        assert_eq!(life.tier_count, Some(2));
        assert!(life.quality.unwrap() > 0.9);
        let crafting = q.crafting.unwrap();
        assert_eq!((crafting.open_prefixes, crafting.open_suffixes), (Some(2), Some(3)));
        assert!(crafting.score > 0.5);
        assert_eq!(q.confidence, Confidence::High);
        assert!(q.score.unwrap() > 80.0);
    }

    #[test]
    fn corrupted_rare_has_no_crafting_potential() {
        let k = fake_knowledge();
        let mut item = rare_gloves(vec![tiered("+140 to maximum Life", 140.0, 1, (130.0, 144.0))], (1, 0));
        item.corrupted = true;
        let q = evaluate_item(&item, &k);
        let crafting = q.crafting.unwrap();
        assert_eq!(crafting.blocked, vec![CraftBlock::Corrupted]);
        assert!(crafting.score.abs() < f64::EPSILON);
    }

    #[test]
    fn mod_without_range_is_inferred_from_ladder_or_inconclusive() {
        let k = fake_knowledge();
        let mut known = tiered("+20 to maximum Life", 20.0, 1, (0.0, 0.0));
        known.tier = None;
        known.roll = Some(crate::item::StatRoll { value: 20.0, min: None, max: None });
        let mut unknown = known.clone();
        unknown.trade_id = "explicit.stat_999".into();
        unknown.text = "+20 to Something".into();
        let q = evaluate_item(&rare_gloves(vec![known, unknown], (1, 0)), &k);
        assert_eq!(q.mods[0].status, ModStatus::FromDatabase);
        assert_eq!(q.mods[0].tier, Some(2));
        assert!(q.mods[1].status.is_inconclusive());
        assert_eq!(q.mods[1].quality, None);
        // Metade do peso é inconclusiva: confiança cai.
        assert!(q.confidence < Confidence::High);
    }
}
