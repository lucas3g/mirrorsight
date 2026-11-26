//! Upgrade: o item do anúncio é MELHOR, PIOR ou EQUIVALENTE ao seu para a build?
//!
//! Compara o conjunto, não mod a mod: soma os stats efetivos de cada item (explícitos,
//! implícitos, crafted, enchants e propriedades da base, com aliases como "all resistances"),
//! normaliza cada diferença pela escala do stat, pondera pela relevância para a build e soma
//! sinergias e restrições (resistência abaixo do cap, requisito de atributo).
//!
//! EQUIVALENTE é estrito: o anúncio tem exatamente os mesmos mods com os mesmos rolls do seu
//! item (nem um a mais, nem um a menos), ou o Δ é nulo. Qualquer outra diferença vira MELHOR ou
//! PIOR pelo sinal do Δ.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::knowledge::{GameKnowledge, Rules, TierSource};
use super::mods::{evaluated_kind, mod_tags};
use super::profile::BuildProfile;
use super::quality::Confidence;
use super::stat::{stat_hash, template_key};
use crate::item::{Item, ModifierKind, PROPERTY_PREFIX};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Better,
    Worse,
    Equivalent,
}

/// Contribuição de um stat para o veredito.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatDelta {
    pub key: String,
    /// Texto de exemplo do stat (do anúncio ou do seu item).
    pub text: String,
    pub yours: f64,
    pub theirs: f64,
    /// Relevância para a build.
    pub weight: f64,
    /// Peso × diferença normalizada (positivo = ganho).
    pub contribution: f64,
}

/// Restrição da build que a troca quebra (ou conserta).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeedChange {
    /// "fire", "cold", "lightning", "chaos", "str", "dex", "int".
    pub stat: String,
    /// Falta (pontos abaixo do cap/requisito) antes e depois da troca.
    pub short_before: f64,
    pub short_after: f64,
    pub contribution: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpgradeVerdict {
    pub verdict: Verdict,
    /// Soma ponderada das diferenças (positivo = o anúncio é melhor).
    pub delta: f64,
    /// `delta` relativo ao valor do seu item.
    pub relative: f64,
    /// Valor do seu item na mesma conta (soma ponderada dos seus stats, mínimo 1): cada
    /// contribuição ÷ `reference` = quanto ela vale em % do seu item.
    pub reference: f64,
    pub gains: Vec<StatDelta>,
    pub losses: Vec<StatDelta>,
    /// Sinergias (positivo = o anúncio combina melhor).
    pub synergy: f64,
    pub needs: Vec<NeedChange>,
    /// Alguma perda isolada acima do limiar de "perda relevante".
    pub relevant_loss: bool,
    pub confidence: Confidence,
}

#[derive(Debug, Clone)]
struct StatEntry {
    value: f64,
    text: String,
    tags: Vec<String>,
    /// Escala para normalizar (máximo típico do stat); `None` = sem referência.
    scale: Option<f64>,
}

/// Compara o anúncio (`theirs`) com o seu item (`mine`).
pub fn compare_items(mine: &Item, theirs: &Item, profile: &BuildProfile, knowledge: &dyn GameKnowledge) -> UpgradeVerdict {
    let rules = knowledge.rules();
    let a = stat_vector(mine, knowledge);
    let b = stat_vector(theirs, knowledge);

    let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
    keys.sort();
    keys.dedup();

    let mut deltas = Vec::new();
    let mut magnitude = 0.0;
    let mut unscaled = 0.0;
    let mut total_abs = 0.0;
    for key in keys {
        let (ea, eb) = (a.get(key), b.get(key));
        let Some(entry) = eb.or(ea) else { continue };
        let (va, vb) = (ea.map_or(0.0, |e| e.value), eb.map_or(0.0, |e| e.value));
        let tags = &entry.tags;
        let weight = profile.relevance(tags, rules);
        let scale = ea.and_then(|e| e.scale).or_else(|| eb.and_then(|e| e.scale));
        let reference = scale.unwrap_or_else(|| va.abs().max(vb.abs())).max(f64::EPSILON);
        let norm = |v: f64| (v / reference).clamp(-2.0, 2.0);
        let contribution = weight * (norm(vb) - norm(va));
        magnitude += weight * norm(va).abs();
        total_abs += contribution.abs();
        if scale.is_none() {
            unscaled += contribution.abs();
        }
        if contribution.abs() > 1e-9 {
            deltas.push(StatDelta {
                key: key.clone(),
                text: entry.text.clone(),
                yours: round2(va),
                theirs: round2(vb),
                weight: round2(weight),
                contribution,
            });
        }
    }

    let synergy = synergy_score(&b, profile, rules) - synergy_score(&a, profile, rules);
    // A troca tira do build o item equipado naquele slot (o copiado pode ser outro, do baú).
    let name = mine.name.as_deref().unwrap_or(&mine.base_type);
    let replaced = profile.equipped_for(&mine.item_class, name).map(|e| &e.needs);
    let needs = need_changes(&a, replaced, &b, profile, rules);
    let needs_total: f64 = needs.iter().map(|n| n.contribution).sum();
    let base: f64 = deltas.iter().map(|d| d.contribution).sum();
    let delta = base + synergy + needs_total;
    let reference = magnitude.max(1.0);
    let relative = delta / reference;

    let t = &rules.thresholds;
    // Δ nulo não tem sinal: mesmos stats escritos de outro jeito ("all resistances" × as três
    // separadas) ou diferenças sem peso não fazem o anúncio pior.
    let verdict = if same_rolls(mine, theirs) || delta.abs() <= 1e-9 {
        Verdict::Equivalent
    } else if delta > 0.0 {
        Verdict::Better
    } else {
        Verdict::Worse
    };
    let relevant_loss = deltas.iter().any(|d| d.contribution <= -t.relevant_loss) || needs.iter().any(|n| n.contribution < 0.0);

    let mut confidence = if profile.generic { Confidence::Medium } else { Confidence::High };
    if profile.needs_estimated {
        confidence = confidence.lower();
    }
    if total_abs > 0.0 && unscaled / total_abs > 0.4 {
        confidence = confidence.lower();
    }

    deltas.sort_by(|x, y| y.contribution.total_cmp(&x.contribution));
    let gains: Vec<StatDelta> = deltas.iter().filter(|d| d.contribution > 0.0).take(3).cloned().map(rounded).collect();
    let losses: Vec<StatDelta> =
        deltas.iter().rev().filter(|d| d.contribution < 0.0).take(3).cloned().map(rounded).collect();

    UpgradeVerdict {
        verdict,
        delta: round2(delta),
        relative: round2(relative),
        reference: round2(reference),
        gains,
        losses,
        synergy: round2(synergy),
        needs,
        relevant_loss,
        confidence,
    }
}

fn rounded(mut d: StatDelta) -> StatDelta {
    d.contribution = round2(d.contribution);
    d
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Os dois itens têm exatamente os mesmos mods com os mesmos rolls (comparando o texto com os
/// números, então "Adds 10 to 20" ≠ "Adds 12 to 18"). Fractured e explícito do mesmo texto contam
/// como iguais; a ordem não importa. Propriedades da base (defesas, DPS) e pseudo ficam de fora.
pub fn same_rolls(mine: &Item, theirs: &Item) -> bool {
    let texts = |item: &Item| {
        let mut v: Vec<String> = item
            .modifiers
            .iter()
            .filter(|m| evaluated_kind(m.kind))
            .map(|m| m.text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase())
            .collect();
        v.sort();
        v
    };
    let a = texts(mine);
    !a.is_empty() && a == texts(theirs)
}

/// Stats efetivos do item, somados por template (e propriedades por id).
fn stat_vector(item: &Item, knowledge: &dyn GameKnowledge) -> BTreeMap<String, StatEntry> {
    let rules = knowledge.rules();
    let mut out: BTreeMap<String, StatEntry> = BTreeMap::new();
    // O DPS total já é físico + elemental (+ caos): com pdps/edps presentes, contá-lo de novo
    // dobraria o peso do dano da arma.
    let total_dps = format!("{PROPERTY_PREFIX}dps");
    let split_dps = item
        .properties
        .iter()
        .any(|p| p.trade_id == format!("{PROPERTY_PREFIX}pdps") || p.trade_id == format!("{PROPERTY_PREFIX}edps"));
    let mods = item.modifiers.iter().filter(|m| evaluated_kind(m.kind));
    for m in mods.chain(item.properties.iter()) {
        if m.kind == ModifierKind::Pseudo || (split_dps && m.trade_id == total_dps) {
            continue;
        }
        let key = if m.trade_id.starts_with(PROPERTY_PREFIX) { m.trade_id.clone() } else { template_key(&m.text) };
        // Mod sem número ("Cannot be Frozen") conta como presente/ausente.
        let value = m.roll.map_or(1.0, |r| r.value);
        let scale = stat_hash(&m.trade_id).and_then(|h| stat_scale(h, knowledge)).or_else(|| rules.stat_scales.get(&key).copied());
        let tags = mod_tags(m, knowledge);
        let targets: Vec<String> = rules.stat_aliases.iter().find(|a| a.from == key).map_or_else(|| vec![key.clone()], |a| a.to.clone());
        for target in targets {
            let target_scale = if target == key { scale } else { rules.stat_scales.get(&target).copied().or(scale) };
            let e = out.entry(target.clone()).or_insert_with(|| StatEntry {
                value: 0.0,
                text: m.text.clone(),
                tags: tags.clone(),
                scale: target_scale,
            });
            e.value += value;
            if e.scale.is_none() {
                e.scale = target_scale;
            }
        }
    }
    out
}

/// Escala do stat: o maior valor do T1 entre os mods explícitos que spawnam; sem eles,
/// o maior craft/corrompido.
fn stat_scale(hash: &str, knowledge: &dyn GameKnowledge) -> Option<f64> {
    let entries = knowledge.tier_entries(hash);
    let top = |source: TierSource, spawning: bool| {
        entries
            .iter()
            .filter(|e| e.source == source && (!spawning || e.spawns_anywhere()))
            .map(|e| e.range[1].abs())
            .fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
    };
    top(TierSource::Explicit, true)
        .or_else(|| top(TierSource::Crafted, false))
        .or_else(|| top(TierSource::Corrupted, false))
        .filter(|v| *v > f64::EPSILON)
}

/// Sinergia do item: para cada par de stats, o menor dos dois totais normalizados × bônus ×
/// relevância (a maior entre os stats do par).
fn synergy_score(vector: &BTreeMap<String, StatEntry>, profile: &BuildProfile, rules: &Rules) -> f64 {
    let side = |term: &str| {
        let mut total = 0.0;
        let mut weight: f64 = 0.0;
        for (key, e) in vector {
            if key.contains(term) {
                let reference = e.scale.unwrap_or_else(|| e.value.abs()).max(f64::EPSILON);
                total += (e.value / reference).clamp(0.0, 1.5);
                weight = weight.max(profile.relevance(&e.tags, rules));
            }
        }
        (total, weight)
    };
    rules
        .synergies
        .iter()
        .map(|s| {
            let ((ta, wa), (tb, wb)) = (side(&s.a), side(&s.b));
            s.bonus * ta.min(tb) * wa.max(wb)
        })
        .sum()
}

/// Efeito da troca nas resistências e atributos do personagem.
/// `replaced`: o que o item equipado no slot dá (do build). Sem ele, usa o seu item.
fn need_changes(
    mine: &BTreeMap<String, StatEntry>,
    replaced: Option<&std::collections::HashMap<String, f64>>,
    theirs: &BTreeMap<String, StatEntry>,
    profile: &BuildProfile,
    rules: &Rules,
) -> Vec<NeedChange> {
    let Some(needs) = &profile.needs else { return Vec::new() };
    let sum = |v: &BTreeMap<String, StatEntry>, keys: &[String]| keys.iter().filter_map(|k| v.get(k)).map(|e| e.value).sum::<f64>();
    let removed = |need: &str, keys: &[String]| match replaced {
        Some(map) => map.get(need).copied().unwrap_or(0.0),
        None => sum(mine, keys),
    };
    let mut out = Vec::new();
    let mut elements: Vec<&String> = needs.resists.keys().collect();
    elements.sort();
    for element in elements {
        let state = &needs.resists[element];
        let Some(keys) = rules.needs.res_stats.get(element) else { continue };
        let total = state.value + state.overcap;
        let after = total - removed(element, keys) + sum(theirs, keys);
        let short_before = (state.cap - total).max(0.0);
        let short_after = (state.cap - after).max(0.0);
        let point = rules.needs.res_point_weight.get(element).copied().unwrap_or(0.0);
        let contribution = -(short_after - short_before) * point;
        if contribution.abs() > 1e-9 {
            out.push(NeedChange { stat: element.clone(), short_before, short_after, contribution: round2(contribution) });
        }
    }
    let mut attrs: Vec<&String> = needs.attributes.keys().collect();
    attrs.sort();
    for attr in attrs {
        let state = &needs.attributes[attr];
        let Some(keys) = rules.needs.attr_stats.get(attr) else { continue };
        let after = state.value - removed(attr, keys) + sum(theirs, keys);
        let short_before = (state.required - state.value).max(0.0);
        let short_after = (state.required - after).max(0.0);
        let contribution = match (short_before > 0.0, short_after > 0.0) {
            (false, true) => -rules.needs.attr_penalty,
            (true, false) => rules.needs.attr_penalty,
            _ => 0.0,
        };
        if contribution.abs() > 1e-9 {
            out.push(NeedChange { stat: attr.clone(), short_before, short_after, contribution });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::profile::{AttributeState, BuildNeeds, ResistState};
    use crate::evaluation::test_support::{fake_knowledge, plain, rare_gloves};

    fn profile(k: &dyn GameKnowledge) -> BuildProfile {
        BuildProfile::generic(k.rules())
    }

    #[test]
    fn same_item_is_equivalent() {
        let k = fake_knowledge();
        let item = rare_gloves(vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0)], (1, 1));
        let v = compare_items(&item, &item, &profile(&k), &k);
        assert_eq!(v.verdict, Verdict::Equivalent);
        assert!(v.gains.is_empty() && v.losses.is_empty());
    }

    #[test]
    fn whole_set_counts_not_only_common_mods() {
        let k = fake_knowledge();
        let mine = rare_gloves(vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0)], (1, 1));
        // Perde a vida (relevante) e ganha um mod de baixa relevância.
        let theirs = rare_gloves(vec![plain("+40% to Fire Resistance", 40.0), plain("+20 to Dexterity", 20.0)], (0, 2));
        let v = compare_items(&mine, &theirs, &profile(&k), &k);
        assert_eq!(v.verdict, Verdict::Worse);
        assert_eq!(v.losses[0].key, "# to maximum life");
        assert!(v.relevant_loss);
        // E o contrário é melhor.
        assert_eq!(compare_items(&theirs, &mine, &profile(&k), &k).verdict, Verdict::Better);
    }

    #[test]
    fn equivalent_only_with_the_same_rolls_on_every_mod() {
        let k = fake_knowledge();
        let p = profile(&k);
        let mine = rare_gloves(vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0)], (1, 1));
        let verdict = |mods: Vec<crate::item::Modifier>| compare_items(&mine, &rare_gloves(mods, (1, 1)), &p, &k).verdict;
        // Mesmos rolls, em outra ordem e com o mod fractured: igual.
        let mut fractured = plain("+80 to maximum Life", 80.0);
        fractured.kind = ModifierKind::Fractured;
        assert_eq!(verdict(vec![plain("+40% to Fire Resistance", 40.0), fractured]), Verdict::Equivalent);
        // Um ponto a mais num mod já não é igual.
        assert_eq!(verdict(vec![plain("+81 to maximum Life", 81.0), plain("+40% to Fire Resistance", 40.0)]), Verdict::Better);
        assert_eq!(verdict(vec![plain("+79 to maximum Life", 79.0), plain("+40% to Fire Resistance", 40.0)]), Verdict::Worse);
        // Todos os seus mods iguais mais um extra: melhor, não igual.
        assert_eq!(
            verdict(vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0), plain("+20 to Dexterity", 20.0)]),
            Verdict::Better
        );
        // Falta um dos seus mods: pior.
        assert_eq!(verdict(vec![plain("+80 to maximum Life", 80.0)]), Verdict::Worse);
    }

    #[test]
    fn same_average_with_different_numbers_is_not_equal() {
        let a = rare_gloves(vec![plain("Adds 10 to 20 Fire Damage", 15.0)], (1, 0));
        let b = rare_gloves(vec![plain("Adds 12 to 18 Fire Damage", 15.0)], (1, 0));
        assert!(!same_rolls(&a, &b));
        assert!(same_rolls(&a, &a));
    }

    #[test]
    fn all_resistances_alias_counts_for_each_element() {
        let k = fake_knowledge();
        let mine = rare_gloves(vec![plain("+40% to Fire Resistance", 40.0)], (0, 1));
        let theirs = rare_gloves(vec![plain("+15% to all Elemental Resistances", 15.0)], (0, 1));
        let v = compare_items(&mine, &theirs, &profile(&k), &k);
        // 15% em três elementos (45 no total) vale mais que 40% num só.
        assert_eq!(v.verdict, Verdict::Better);
    }

    #[test]
    fn same_stats_written_differently_are_equivalent_not_worse() {
        let k = fake_knowledge();
        let mine = rare_gloves(vec![plain("+15% to all Elemental Resistances", 15.0)], (0, 1));
        let theirs = rare_gloves(
            vec![
                plain("+15% to Fire Resistance", 15.0),
                plain("+15% to Cold Resistance", 15.0),
                plain("+15% to Lightning Resistance", 15.0),
            ],
            (0, 3),
        );
        let v = compare_items(&mine, &theirs, &profile(&k), &k);
        assert!(v.delta.abs() < 1e-9, "{}", v.delta);
        assert_eq!(v.verdict, Verdict::Equivalent);
    }

    #[test]
    fn dropping_below_resistance_cap_is_a_relevant_loss() {
        let k = fake_knowledge();
        let mut p = profile(&k);
        p.generic = false;
        p.needs = Some(BuildNeeds {
            resists: [("fire".to_owned(), ResistState { value: 75.0, overcap: 5.0, cap: 75.0 })].into_iter().collect(),
            attributes: [("dex".to_owned(), AttributeState { value: 120.0, required: 110.0 })].into_iter().collect(),
        });
        let mine = rare_gloves(
            vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0), plain("+20 to Dexterity", 20.0)],
            (1, 2),
        );
        // Mais vida, mas perde 40% de fogo (fica 35 abaixo do cap) e quebra o requisito de destreza.
        let theirs = rare_gloves(vec![plain("+100 to maximum Life", 100.0)], (1, 0));
        let v = compare_items(&mine, &theirs, &p, &k);
        assert_eq!(v.verdict, Verdict::Worse);
        assert!(v.relevant_loss);
        let fire = v.needs.iter().find(|n| n.stat == "fire").unwrap();
        assert!((fire.short_after - 35.0).abs() < 1e-9);
        assert!(v.needs.iter().any(|n| n.stat == "dex" && n.contribution < 0.0));
    }

    #[test]
    fn total_dps_is_not_counted_on_top_of_physical_and_elemental() {
        use crate::item::{Modifier, StatRoll};
        let k = fake_knowledge();
        let prop = |id: &str, value: f64| Modifier {
            trade_id: format!("{PROPERTY_PREFIX}{id}"),
            text: format!("{id}: {value}"),
            kind: ModifierKind::Property,
            roll: Some(StatRoll { value, min: None, max: None }),
            ..Default::default()
        };
        let mut weapon = rare_gloves(vec![plain("+80 to maximum Life", 80.0)], (1, 0));
        weapon.properties = vec![prop("dps", 450.0), prop("pdps", 300.0), prop("edps", 150.0)];
        let keys: Vec<String> = stat_vector(&weapon, &k).into_keys().collect();
        assert!(keys.iter().any(|key| key.ends_with("pdps")));
        assert!(!keys.iter().any(|key| key == &format!("{PROPERTY_PREFIX}dps")));
    }

    #[test]
    fn swap_removes_the_equipped_item_not_the_copied_one() {
        use crate::evaluation::profile::EquippedItem;
        let k = fake_knowledge();
        let mut p = profile(&k);
        p.generic = false;
        p.needs = Some(BuildNeeds {
            resists: [("fire".to_owned(), ResistState { value: 75.0, overcap: 5.0, cap: 75.0 })].into_iter().collect(),
            attributes: Default::default(),
        });
        // O build usa outras luvas, sem resistência a fogo.
        p.equipped = vec![EquippedItem { slot: "Gloves".into(), name: "Outra".into(), needs: Default::default() }];
        // Você checa luvas do baú com 40% de fogo; o anúncio não tem fogo.
        let mine = rare_gloves(vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0)], (1, 1));
        let theirs = rare_gloves(vec![plain("+80 to maximum Life", 80.0)], (1, 0));
        let v = compare_items(&mine, &theirs, &p, &k);
        // Trocar as luvas equipadas pelo anúncio não muda o fogo do build.
        assert!(v.needs.iter().all(|n| n.stat != "fire"), "{:?}", v.needs);
        // Sem item equipado no slot, a conta usa o seu item (cai 35% abaixo do cap).
        p.equipped.clear();
        let v = compare_items(&mine, &theirs, &p, &k);
        assert!(v.needs.iter().any(|n| n.stat == "fire" && n.contribution < 0.0));
    }

    #[test]
    fn synergy_rewards_pairs() {
        let k = fake_knowledge();
        let p = profile(&k);
        let mine = rare_gloves(vec![plain("+30% to Global Critical Strike Multiplier", 30.0), plain("+80 to maximum Life", 80.0)], (1, 1));
        let theirs = rare_gloves(
            vec![plain("+30% to Global Critical Strike Multiplier", 30.0), plain("30% increased Global Critical Strike Chance", 30.0)],
            (0, 2),
        );
        let v = compare_items(&mine, &theirs, &p, &k);
        assert!(v.synergy > 0.0);
    }
}
