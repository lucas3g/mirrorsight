//! Avaliação individual de cada modificador (independe da build).

use serde::{Deserialize, Serialize};

use super::knowledge::{ladder_for, GameKnowledge, Range, TierLadder, TierSource};
use super::stat::{fit, percentile, stat_hash, template_key};
use crate::item::{Item, ModOrigin, Modifier, ModifierKind, PROPERTY_PREFIX};

/// Por que não dá para medir a qualidade de um mod.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InconclusiveReason {
    /// O único não está na base de conhecimento (liga nova, snapshot antigo).
    UniqueNotInDatabase,
    /// O único existe, mas o PoB o monta por código: sem mods no snapshot.
    UniqueWithoutData,
    /// O mod não está entre os mods conhecidos do item.
    ModNotInDatabase,
    /// O valor não cabe em nenhuma faixa conhecida (versão legada? dado desatualizado?).
    OutOfRange,
    /// Nem o item nem a base trazem a faixa do mod.
    NoRange,
    /// Mod mutado de único Foulborn (o snapshot não tem esses mods).
    Foulborn,
}

/// Como a qualidade do mod foi (ou não) determinada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModStatus {
    /// Faixa vinda do próprio item (Ctrl+Alt+C ou trade).
    Exact,
    /// O item veio sem faixa; a faixa é a da base de conhecimento.
    FromDatabase,
    /// Valor fixo: igual em toda cópia, não entra na qualidade.
    Fixed,
    /// Escolha entre alternativas sem faixa numérica (variante de único, pool do Watcher's Eye).
    Variant,
    /// Mutação Foulborn de valor fixo: troca um mod do único; não entra na nota.
    Mutated,
    Inconclusive { reason: InconclusiveReason },
}

impl ModStatus {
    pub const fn is_inconclusive(&self) -> bool {
        matches!(self, Self::Inconclusive { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModEvaluation {
    pub trade_id: String,
    pub text: String,
    pub kind: ModifierKind,
    #[serde(default)]
    pub origin: Option<ModOrigin>,
    pub value: Option<f64>,
    /// Faixa usada na avaliação (do tier, do único ou da bancada).
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub tier: Option<u32>,
    /// Quantos tiers o stat tem nessa base.
    pub tier_count: Option<u32>,
    /// Posição do roll na faixa (0–1).
    pub roll: Option<f64>,
    /// Posição na faixa de todos os tiers (0–1), em raros.
    pub global_roll: Option<f64>,
    /// Nota do mod (0–1). `None` = não entra na qualidade (fixo, variante ou inconclusivo).
    pub quality: Option<f64>,
    /// Peso do mod na qualidade do item (genérico, sem build).
    pub importance: f64,
    pub status: ModStatus,
    /// Variante/versão do único a que o mod pertence.
    #[serde(default)]
    pub variant: Option<String>,
    /// A faixa do jogo difere da base de conhecimento (a do jogo vence).
    #[serde(default)]
    pub range_mismatch: bool,
    pub tags: Vec<String>,
}

impl ModEvaluation {
    pub(crate) fn new(m: &Modifier, tags: Vec<String>, importance: f64, status: ModStatus) -> Self {
        Self {
            trade_id: m.trade_id.clone(),
            text: m.text.clone(),
            kind: m.kind,
            origin: m.origin,
            value: m.roll.map(|r| r.value),
            min: None,
            max: None,
            tier: m.tier,
            tier_count: None,
            roll: None,
            global_roll: None,
            quality: None,
            importance,
            status,
            variant: None,
            range_mismatch: false,
            tags,
        }
    }

    pub(crate) fn with_range(mut self, range: Range) -> Self {
        self.min = Some(range.0);
        self.max = Some(range.1);
        self
    }

    pub(crate) fn inconclusive(mut self, reason: InconclusiveReason) -> Self {
        self.status = ModStatus::Inconclusive { reason };
        self.quality = None;
        self
    }
}

/// Mods que entram na avaliação (pseudo e propriedades ficam de fora).
pub fn evaluated_kind(kind: ModifierKind) -> bool {
    matches!(
        kind,
        ModifierKind::Explicit
            | ModifierKind::Fractured
            | ModifierKind::Crafted
            | ModifierKind::Implicit
            | ModifierKind::Enchant
            | ModifierKind::Desecrated
            | ModifierKind::Rune
    )
}

/// Tags de um mod: as do stat no jogo; sem elas, as das regras por palavra-chave.
pub fn mod_tags(m: &Modifier, knowledge: &dyn GameKnowledge) -> Vec<String> {
    if let Some(id) = m.trade_id.strip_prefix(PROPERTY_PREFIX) {
        return knowledge.rules().property_tags.get(id).cloned().unwrap_or_default();
    }
    if let Some(hash) = stat_hash(&m.trade_id) {
        let tags = knowledge.stat_tags(hash);
        if !tags.is_empty() {
            return tags.to_vec();
        }
    }
    keyword_tags(&m.text, knowledge)
}

pub fn keyword_tags(text: &str, knowledge: &dyn GameKnowledge) -> Vec<String> {
    let key = template_key(text);
    let mut tags: Vec<String> = Vec::new();
    for rule in &knowledge.rules().keyword_tags {
        if key.contains(&rule.contains) {
            for t in &rule.tags {
                if !tags.contains(t) {
                    tags.push(t.clone());
                }
            }
        }
    }
    tags
}

/// Escada de tiers do mod no item (base + influências + nome do afixo).
pub fn ladder_of(m: &Modifier, item: &Item, knowledge: &dyn GameKnowledge) -> Option<TierLadder> {
    let hash = stat_hash(&m.trade_id)?;
    let entries = knowledge.tier_entries(hash);
    if entries.is_empty() {
        return None;
    }
    let tags = knowledge.base(&item.base_type).map(|b| b.tags_with(&item.influences));
    ladder_for(entries, tags.as_deref(), m.mod_name.as_deref())
}

/// Maior faixa de uma fonte (crafted/corrompido) para o stat: a do mod de nível mais alto.
fn best_range(m: &Modifier, knowledge: &dyn GameKnowledge, source: TierSource) -> Option<Range> {
    let hash = stat_hash(&m.trade_id)?;
    knowledge
        .tier_entries(hash)
        .iter()
        .filter(|e| e.source == source)
        .max_by(|a, b| a.level.cmp(&b.level).then(a.range[1].total_cmp(&b.range[1])))
        .map(|e| e.range())
}

/// Faixa de uma fonte (crafted/corrompido) em que o valor cabe; senão a melhor.
fn source_range(m: &Modifier, value: f64, knowledge: &dyn GameKnowledge, source: TierSource) -> Option<Range> {
    let hash = stat_hash(&m.trade_id)?;
    knowledge
        .tier_entries(hash)
        .iter()
        .filter(|e| e.source == source)
        .map(|e| e.range())
        .find(|r| fit(value, *r).is_some())
        .or_else(|| best_range(m, knowledge, source))
}

/// Faixa do próprio item (Ctrl+Alt+C / trade), se tiver amplitude ou for fixa conhecida.
pub fn item_range(m: &Modifier) -> Option<Range> {
    let roll = m.roll?;
    Some((roll.min?, roll.max?))
}

/// Nota de tier: T1 = 1, o pior tier da base = 0.
#[allow(clippy::cast_precision_loss)]
fn tier_score(tier: u32, count: Option<u32>) -> f64 {
    match count {
        Some(c) if c > 1 => (1.0 - f64::from(tier.saturating_sub(1)) / f64::from(c - 1)).clamp(0.0, 1.0),
        Some(_) => 1.0,
        // Sem a escada: cada tier abaixo do T1 tira um pouco.
        None => (1.0 - 0.12 * f64::from(tier.saturating_sub(1))).max(0.0),
    }
}

/// Avalia um mod de item raro/mágico/normal.
pub fn evaluate_rare_mod(m: &Modifier, item: &Item, knowledge: &dyn GameKnowledge) -> ModEvaluation {
    let rules = knowledge.rules();
    let tags = mod_tags(m, knowledge);
    let generic = rules.generic_weight(&tags);
    let value = m.roll.map(|r| r.value);

    match m.kind {
        ModifierKind::Crafted => {
            let importance = generic * rules.crafted_weight;
            let ev = ModEvaluation::new(m, tags, importance, ModStatus::Exact);
            let Some(v) = value else { return ev.inconclusive(InconclusiveReason::NoRange) };
            // Nota do crafted: valor em relação ao melhor craft possível do stat.
            let own = item_range(m);
            let best = best_range(m, knowledge, TierSource::Crafted);
            let range = own.or_else(|| source_range(m, v, knowledge, TierSource::Crafted));
            let Some(range) = range else { return ev.inconclusive(InconclusiveReason::NoRange) };
            let mut ev = ev.with_range(range);
            ev.status = if own.is_some() { ModStatus::Exact } else { ModStatus::FromDatabase };
            let v = fit(v, range).unwrap_or(v);
            ev.roll = percentile(v, range);
            ev.quality = match best {
                Some((_, top)) if top.abs() > f64::EPSILON => Some((v / top).clamp(0.0, 1.0)),
                _ => ev.roll.or(Some(1.0)),
            };
            ev
        }
        ModifierKind::Implicit => evaluate_implicit(m, tags, generic, value, knowledge),
        ModifierKind::Enchant => {
            let ev = ModEvaluation::new(m, tags, generic * rules.implicit_weight, ModStatus::Fixed);
            match item_range(m) {
                Some(r) if percentile(value.unwrap_or(r.1), r).is_some() => {
                    let mut ev = ev.with_range(r);
                    ev.status = ModStatus::Exact;
                    ev.roll = value.and_then(|v| percentile(v, r));
                    ev.quality = ev.roll;
                    ev
                }
                _ => ev,
            }
        }
        _ => evaluate_explicit(m, item, tags, generic, value, knowledge),
    }
}

fn evaluate_implicit(
    m: &Modifier,
    tags: Vec<String>,
    generic: f64,
    value: Option<f64>,
    knowledge: &dyn GameKnowledge,
) -> ModEvaluation {
    let rules = knowledge.rules();
    match m.origin {
        Some(ModOrigin::Eldritch { level, .. }) => {
            // Lesser → Perfect: a nota é o nível (o roll dentro do nível é fixo).
            let mut ev = ModEvaluation::new(m, tags, generic, ModStatus::Exact);
            ev.tier = Some(7 - level.rank());
            ev.tier_count = Some(6);
            ev.quality = Some(f64::from(level.rank()) / 6.0);
            ev.roll = item_range(m).and_then(|r| value.and_then(|v| percentile(v, r)));
            ev
        }
        Some(ModOrigin::Corruption) => {
            let ev = ModEvaluation::new(m, tags, generic, ModStatus::Exact);
            let Some(v) = value else { return ev.inconclusive(InconclusiveReason::NoRange) };
            let own = item_range(m);
            match own.or_else(|| source_range(m, v, knowledge, TierSource::Corrupted)) {
                Some(range) => {
                    let mut ev = ev.with_range(range);
                    ev.status = if own.is_some() { ModStatus::Exact } else { ModStatus::FromDatabase };
                    ev.roll = percentile(fit(v, range).unwrap_or(v), range);
                    ev.quality = ev.roll.or(Some(1.0));
                    ev
                }
                None => ev.inconclusive(InconclusiveReason::NoRange),
            }
        }
        _ => {
            // Implícito da base: só a faixa do item diz algo; pesa menos.
            let ev = ModEvaluation::new(m, tags, generic * rules.implicit_weight, ModStatus::Exact);
            match (item_range(m), value) {
                (Some(r), Some(v)) => {
                    let mut ev = ev.with_range(r);
                    match percentile(v, r) {
                        Some(p) => {
                            ev.roll = Some(p);
                            ev.quality = Some(p);
                        }
                        None => ev.status = ModStatus::Fixed,
                    }
                    ev
                }
                (None, None) => {
                    let mut ev = ev;
                    ev.status = ModStatus::Fixed;
                    ev
                }
                _ => ev.inconclusive(InconclusiveReason::NoRange),
            }
        }
    }
}

fn evaluate_explicit(
    m: &Modifier,
    item: &Item,
    tags: Vec<String>,
    generic: f64,
    value: Option<f64>,
    knowledge: &dyn GameKnowledge,
) -> ModEvaluation {
    let mut ev = ModEvaluation::new(m, tags, generic, ModStatus::Exact);
    let Some(v) = value else {
        // Mod sem número ("Cannot be Frozen"): nada a medir.
        ev.status = ModStatus::Fixed;
        return ev;
    };
    let ladder = ladder_of(m, item, knowledge);
    let own = item_range(m);
    let tier = m.tier.or_else(|| ladder.as_ref().and_then(|l| l.infer_tier(v)));
    ev.tier = tier;
    ev.tier_count = ladder.as_ref().map(TierLadder::count);

    let range = own.or_else(|| tier.and_then(|t| ladder.as_ref().and_then(|l| l.tier_range(t))));
    let Some(range) = range else {
        return ev.inconclusive(if ladder.is_some() { InconclusiveReason::OutOfRange } else { InconclusiveReason::NoRange });
    };
    if own.is_none() && fit(v, range).is_none() {
        return ev.inconclusive(InconclusiveReason::OutOfRange);
    }
    ev.status = if own.is_some() { ModStatus::Exact } else { ModStatus::FromDatabase };
    let v = fit(v, range).unwrap_or(v);
    ev = ev.with_range(range);
    ev.roll = percentile(v, range);
    ev.global_roll = ladder.as_ref().and_then(TierLadder::global_range).and_then(|g| percentile(v, g));
    ev.quality = match (tier, ev.roll) {
        // Tier pesa mais que o roll dentro do tier.
        (Some(t), roll) => Some(0.7 * tier_score(t, ev.tier_count) + 0.3 * roll.unwrap_or(1.0)),
        (None, Some(r)) => Some(ev.global_roll.unwrap_or(r)),
        (None, None) => {
            ev.status = ModStatus::Fixed;
            None
        }
    };
    ev
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_score_scales_with_ladder() {
        assert!((tier_score(1, Some(10)) - 1.0).abs() < 1e-9);
        assert!((tier_score(10, Some(10))).abs() < 1e-9);
        assert!((tier_score(2, None) - 0.88).abs() < 1e-9);
    }
}
