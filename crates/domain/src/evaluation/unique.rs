//! Únicos: identifica o único (nome + base), descobre a versão/variante pelos mods do item e
//! avalia cada roll na faixa do snapshot. Sem tiers.

use serde::{Deserialize, Serialize};

use super::knowledge::{GameKnowledge, UniqueModSpec, UniqueSpec};
use super::mods::{evaluate_rare_mod, item_range, mod_tags, InconclusiveReason, ModEvaluation, ModStatus};
use super::stat::{fit, percentile, template_key};
use crate::item::{Item, ModOrigin, Modifier, ModifierKind};

/// Qual único (e qual versão/variante) o item é.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniqueMatch {
    pub name: String,
    /// O único está na base de conhecimento (com mods).
    pub found: bool,
    pub version: Option<String>,
    pub variant: Option<String>,
    /// A versão escolhida não é a atual (item legado).
    pub legacy: bool,
    pub variable_mods: u32,
    pub fixed_mods: u32,
}

/// Avalia os mods de um único. Devolve o match e a avaliação de cada mod do item.
pub fn evaluate_unique(item: &Item, knowledge: &dyn GameKnowledge) -> (UniqueMatch, Vec<ModEvaluation>) {
    let name = item.name.clone().unwrap_or_else(|| item.base_type.clone());
    let mods: Vec<&Modifier> = item.modifiers.iter().filter(|m| super::mods::evaluated_kind(m.kind)).collect();
    let specs = knowledge.uniques_named(&name);
    let spec = specs.iter().find(|s| s.has_base(&item.base_type)).or_else(|| specs.first()).copied();

    let Some(spec) = spec.filter(|s| !s.generated && !s.mods.is_empty()) else {
        let reason = if spec.is_some() { InconclusiveReason::UniqueWithoutData } else { InconclusiveReason::UniqueNotInDatabase };
        let evals = mods.iter().map(|m| unmatched(m, item, knowledge, reason)).collect();
        let found = UniqueMatch {
            name,
            found: false,
            version: None,
            variant: None,
            legacy: false,
            variable_mods: 0,
            fixed_mods: 0,
        };
        return (found, evals);
    };

    let (version, variant) = choose_version(spec, &mods, &item.base_type);
    let keys: Vec<String> = mods.iter().map(|m| template_key(&m.text)).collect();
    let mut used = vec![false; spec.mods.len()];
    let mut evals = Vec::with_capacity(mods.len());
    for (m, key) in mods.iter().zip(&keys) {
        let pick = pick_spec(spec, m, key, version, variant, &used);
        match pick {
            Some(idx) => {
                used[idx] = true;
                evals.push(evaluate_with_spec(m, &spec.mods[idx], spec, variant, knowledge));
            }
            None => match foulborn_mutation(spec, m, key, item) {
                Some(mutation) => evals.push(evaluate_mutation(m, mutation, spec, knowledge)),
                None => {
                    let reason = if item.foulborn { InconclusiveReason::Foulborn } else { InconclusiveReason::ModNotInDatabase };
                    evals.push(unmatched(m, item, knowledge, reason));
                }
            },
        }
    }

    let variable = evals.iter().filter(|e| e.quality.is_some()).count();
    let fixed = evals.iter().filter(|e| matches!(e.status, ModStatus::Fixed | ModStatus::Variant | ModStatus::Mutated)).count();
    let name_of = |list: &[String], idx: Option<u32>| {
        idx.and_then(|i| list.get(usize::try_from(i).ok()?.checked_sub(1)?)).cloned()
    };
    let legacy = match version {
        Some(v) => usize::try_from(v).is_ok_and(|v| v < spec.versions.len()),
        None => false,
    };
    let found = UniqueMatch {
        name: spec.name.clone(),
        found: true,
        version: name_of(&spec.versions, version),
        variant: if spec.pool { None } else { name_of(&spec.variants, variant) },
        legacy,
        variable_mods: u32::try_from(variable).unwrap_or(u32::MAX),
        fixed_mods: u32::try_from(fixed).unwrap_or(u32::MAX),
    };
    (found, evals)
}

/// Mutação Foulborn do único que corresponde à linha do item (só em item Foulborn).
fn foulborn_mutation<'a>(spec: &'a UniqueSpec, m: &Modifier, key: &str, item: &Item) -> Option<&'a UniqueModSpec> {
    if !item.foulborn || m.kind == ModifierKind::Implicit {
        return None;
    }
    spec.foulborn.iter().find(|f| template_key(&f.text) == key)
}

/// Mod mutado: com faixa, o roll entra na nota como qualquer mod do único; fixo vira `Mutated`.
fn evaluate_mutation(m: &Modifier, mutation: &UniqueModSpec, spec: &UniqueSpec, knowledge: &dyn GameKnowledge) -> ModEvaluation {
    let mut ev = evaluate_with_spec(m, mutation, spec, None, knowledge);
    if matches!(ev.status, ModStatus::Fixed | ModStatus::Variant) {
        ev.status = ModStatus::Mutated;
    }
    ev.variant = Some("Foulborn".to_owned());
    ev
}

/// Mod do item sem correspondente no único: implícito de corrupção e crafted ainda têm faixa
/// própria na base; o resto fica inconclusivo.
fn unmatched(m: &Modifier, item: &Item, knowledge: &dyn GameKnowledge, reason: InconclusiveReason) -> ModEvaluation {
    let corrupted_implicit = m.kind == ModifierKind::Implicit && (item.corrupted || m.origin == Some(ModOrigin::Corruption));
    if m.kind == ModifierKind::Crafted || m.kind == ModifierKind::Enchant || corrupted_implicit {
        let mut probe = m.clone();
        if corrupted_implicit {
            probe.origin = Some(ModOrigin::Corruption);
        }
        let ev = evaluate_rare_mod(&probe, item, knowledge);
        if !ev.status.is_inconclusive() {
            return ev;
        }
    }
    let tags = mod_tags(m, knowledge);
    let importance = knowledge.rules().generic_weight(&tags);
    ModEvaluation::new(m, tags, importance, ModStatus::Exact).inconclusive(reason)
}

fn evaluate_with_spec(
    m: &Modifier,
    spec_mod: &UniqueModSpec,
    spec: &UniqueSpec,
    variant: Option<u32>,
    knowledge: &dyn GameKnowledge,
) -> ModEvaluation {
    let mut tags = mod_tags(m, knowledge);
    if tags.is_empty() {
        tags.clone_from(&spec_mod.tags);
    }
    let importance = knowledge.rules().generic_weight(&tags);
    let mut ev = ModEvaluation::new(m, tags, importance, ModStatus::Exact);
    // Variante a que o mod pertence (pool: a do próprio mod; senão, a escolhida).
    ev.variant = match (&spec_mod.variants, spec.pool) {
        (Some(list), true) => list.first().and_then(|i| spec.variants.get(usize::try_from(*i).ok()?.checked_sub(1)?)).cloned(),
        (Some(_), false) => variant.and_then(|i| spec.variants.get(usize::try_from(i).ok()?.checked_sub(1)?)).cloned(),
        _ => None,
    };
    let is_choice = spec_mod.variants.is_some() || spec_mod.group.is_some();

    if !spec_mod.is_variable() {
        // Valor fixo no jogo. Se o mod é uma escolha entre variantes, ele é a "variação sem faixa".
        ev.status = if is_choice { ModStatus::Variant } else { ModStatus::Fixed };
        if let Some(r) = spec_mod.range() {
            ev = ev.with_range(r);
        }
        return ev;
    }
    let Some(db_range) = spec_mod.range() else { return ev.inconclusive(InconclusiveReason::NoRange) };
    let Some(value) = ev.value else { return ev.inconclusive(InconclusiveReason::NoRange) };
    // A faixa do jogo (Ctrl+Alt+C) vence; a divergência é sinalizada.
    let own = item_range(m).filter(|r| (r.1 - r.0).abs() > f64::EPSILON);
    let range = own.unwrap_or(db_range);
    if let Some(own) = own {
        ev.range_mismatch = (own.0 - db_range.0).abs() > 0.5 || (own.1 - db_range.1).abs() > 0.5;
    }
    let Some(v) = fit(value, range) else { return ev.with_range(range).inconclusive(InconclusiveReason::OutOfRange) };
    ev = ev.with_range(range);
    ev.status = if own.is_some() { ModStatus::Exact } else { ModStatus::FromDatabase };
    ev.roll = percentile(v, range);
    ev.quality = ev.roll;
    ev
}

/// Melhor mod do único para a linha do item: mesmo template, aplicável à versão/variante,
/// implícito com implícito, valor dentro da faixa — nessa ordem de preferência.
fn pick_spec(
    spec: &UniqueSpec,
    m: &Modifier,
    key: &str,
    version: Option<u32>,
    variant: Option<u32>,
    used: &[bool],
) -> Option<usize> {
    let variant = if spec.pool { None } else { variant };
    let value = m.roll.map(|r| r.value);
    let implicit = m.kind == ModifierKind::Implicit;
    spec.mods
        .iter()
        .enumerate()
        .filter(|(i, s)| !used[*i] && template_key(&s.text) == key)
        .map(|(i, s)| {
            let mut score = 0;
            if s.applies(version, variant) {
                score += 4;
            }
            if s.implicit == implicit {
                score += 2;
            }
            if let (Some(v), Some(r)) = (value, s.range()) {
                if fit(v, r).is_some() {
                    score += 1;
                }
            }
            (i, score)
        })
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(i, _)| i)
}

/// Escolhe a versão (patch) e a variante que melhor explicam os mods do item.
/// Empate: versão mais nova; variante indeterminada vira `None`.
fn choose_version(spec: &UniqueSpec, mods: &[&Modifier], base: &str) -> (Option<u32>, Option<u32>) {
    let count = |n: usize| u32::try_from(n).unwrap_or(0);
    let versions: Vec<Option<u32>> =
        if spec.versions.is_empty() { vec![None] } else { (1..=count(spec.versions.len())).map(Some).collect() };
    let variants: Vec<Option<u32>> = if spec.variants.is_empty() || spec.pool {
        vec![None]
    } else {
        (1..=count(spec.variants.len())).map(Some).collect()
    };
    let keyed: Vec<(String, Option<f64>, bool)> =
        mods.iter().map(|m| (template_key(&m.text), m.roll.map(|r| r.value), m.kind == ModifierKind::Implicit)).collect();

    let mut scored = Vec::with_capacity(versions.len() * variants.len());
    for &version in &versions {
        for &variant in &variants {
            scored.push((combo_score(spec, &keyed, version, variant, base), version, variant));
        }
    }
    let top = scored.iter().map(|s| s.0).fold(f64::NEG_INFINITY, f64::max);
    let tied: Vec<&(f64, Option<u32>, Option<u32>)> = scored.iter().filter(|s| (s.0 - top).abs() <= 1e-9).collect();
    // Empate entre versões: a mais nova (as versões vêm em ordem de patch).
    let version = tied.iter().map(|s| s.1).max().flatten();
    let mut tied_variants = tied.iter().filter(|s| s.1 == version).map(|s| s.2);
    let variant = tied_variants.next().flatten();
    // Mais de uma variante explica o item igualmente: indeterminada.
    let variant = if tied_variants.next().is_some() { None } else { variant };
    (version, variant)
}

fn combo_score(
    spec: &UniqueSpec,
    item_mods: &[(String, Option<f64>, bool)],
    version: Option<u32>,
    variant: Option<u32>,
    base: &str,
) -> f64 {
    let applicable: Vec<&UniqueModSpec> = spec.mods.iter().filter(|m| m.applies(version, variant)).collect();
    let mut score = 0.0;
    for (key, value, _) in item_mods {
        let candidates: Vec<&&UniqueModSpec> = applicable.iter().filter(|s| template_key(&s.text) == *key).collect();
        if candidates.is_empty() {
            continue;
        }
        let fits = candidates.iter().any(|s| match (value, s.range()) {
            (Some(v), Some(r)) => fit(*v, r).is_some(),
            _ => true,
        });
        score += if fits { 2.0 } else { 0.5 };
    }
    // Mods da combinação que o item não tem pesam contra (só fora de pool/grupos).
    let keys: Vec<&str> = item_mods.iter().map(|(k, _, _)| k.as_str()).collect();
    let missing = applicable
        .iter()
        .filter(|s| s.group.is_none() && !keys.contains(&template_key(&s.text).as_str()))
        .count();
    #[allow(clippy::cast_precision_loss)]
    let missing = missing as f64;
    score -= 0.5 * missing;
    // Base que muda entre variantes.
    if let Some(var) = variant {
        if spec.bases.iter().any(|b| b.base.eq_ignore_ascii_case(base) && b.variants.as_ref().is_some_and(|v| v.contains(&var))) {
            score += 1.0;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::test_support::{fake_knowledge, unique_item, unique_mod};

    #[test]
    fn evaluates_rolls_with_database_ranges() {
        let k = fake_knowledge();
        // Headhunter vindo da trade sem faixas: as faixas saem do snapshot.
        let item = unique_item(
            "Headhunter",
            "Leather Belt",
            vec![
                unique_mod(ModifierKind::Implicit, "+30 to maximum Life", 30.0),
                unique_mod(ModifierKind::Explicit, "+55 to Strength", 55.0),
                unique_mod(ModifierKind::Explicit, "+40 to Dexterity", 40.0),
                unique_mod(ModifierKind::Explicit, "+55 to maximum Life", 55.0),
                unique_mod(ModifierKind::Explicit, "When you Kill a Rare monster, you gain its Modifiers for 60 seconds", 60.0),
            ],
        );
        let (found, evals) = evaluate_unique(&item, &k);
        assert!(found.found);
        assert_eq!(found.variable_mods, 4);
        assert_eq!(found.fixed_mods, 1);
        // Implícito de vida 25–40 e explícito 50–60: cada linha na sua faixa.
        assert_eq!((evals[0].min, evals[0].max), (Some(25.0), Some(40.0)));
        assert_eq!((evals[3].min, evals[3].max), (Some(50.0), Some(60.0)));
        assert_eq!(evals[1].quality, Some(1.0));
        assert_eq!(evals[2].quality, Some(0.0));
        assert_eq!(evals[1].status, ModStatus::FromDatabase);
        assert_eq!(evals[4].status, ModStatus::Fixed);
        assert_eq!(evals[4].quality, None);
    }

    #[test]
    fn foulborn_mutation_is_recognized() {
        let k = fake_knowledge();
        let mut item = unique_item(
            "Headhunter",
            "Leather Belt",
            vec![
                unique_mod(ModifierKind::Explicit, "+50 to Strength", 50.0),
                unique_mod(ModifierKind::Explicit, "Damage cannot be Reflected", 1.0),
                unique_mod(ModifierKind::Explicit, "110% increased Chaos Damage", 110.0),
            ],
        );
        // Sem a marca Foulborn, as mutações não valem.
        let (_, plain) = evaluate_unique(&item, &k);
        assert!(plain[1].status.is_inconclusive());
        item.foulborn = true;
        let (found, evals) = evaluate_unique(&item, &k);
        assert_eq!(evals[1].status, ModStatus::Mutated);
        assert_eq!(evals[1].variant.as_deref(), Some("Foulborn"));
        // Mutação com faixa: o roll entra na nota (110 em 100–120 = 50%).
        assert_eq!(evals[2].quality, Some(0.5));
        assert_eq!(evals[2].status, ModStatus::FromDatabase);
        assert_eq!(found.fixed_mods, 1);
    }

    #[test]
    fn unknown_unique_is_inconclusive_not_perfect() {
        let k = fake_knowledge();
        let item = unique_item("Liga Nova", "Leather Belt", vec![unique_mod(ModifierKind::Explicit, "+50 to Strength", 50.0)]);
        let (found, evals) = evaluate_unique(&item, &k);
        assert!(!found.found);
        assert_eq!(evals[0].quality, None);
        assert_eq!(evals[0].status, ModStatus::Inconclusive { reason: InconclusiveReason::UniqueNotInDatabase });
    }

    #[test]
    fn out_of_range_value_is_inconclusive() {
        let k = fake_knowledge();
        let item = unique_item("Headhunter", "Leather Belt", vec![unique_mod(ModifierKind::Explicit, "+90 to Strength", 90.0)]);
        let (_, evals) = evaluate_unique(&item, &k);
        assert_eq!(evals[0].status, ModStatus::Inconclusive { reason: InconclusiveReason::OutOfRange });
    }

    #[test]
    fn picks_legacy_version_from_values() {
        let k = fake_knowledge();
        // Versão 1 tem ES 120–150; versão 2 (atual) tem 150–180.
        let old = unique_item("Versioned Helm", "Hubris Circlet", vec![unique_mod(ModifierKind::Explicit, "125% increased Energy Shield", 125.0)]);
        let (found, evals) = evaluate_unique(&old, &k);
        assert_eq!(found.version.as_deref(), Some("Pre 3.0.0"));
        assert!(found.legacy);
        assert_eq!((evals[0].min, evals[0].max), (Some(120.0), Some(150.0)));
        let new = unique_item("Versioned Helm", "Hubris Circlet", vec![unique_mod(ModifierKind::Explicit, "170% increased Energy Shield", 170.0)]);
        let (found, _) = evaluate_unique(&new, &k);
        assert_eq!(found.version.as_deref(), Some("Current"));
        assert!(!found.legacy);
    }

    #[test]
    fn variant_choice_without_range_is_marked_as_variant() {
        let k = fake_knowledge();
        let item = unique_item(
            "Charge Boots",
            "Riveted Boots",
            vec![unique_mod(ModifierKind::Explicit, "-1 to Maximum Frenzy Charges", -1.0)],
        );
        let (found, evals) = evaluate_unique(&item, &k);
        assert_eq!(found.variant.as_deref(), Some("Frenzy"));
        assert_eq!(evals[0].status, ModStatus::Variant);
        assert_eq!(evals[0].variant.as_deref(), Some("Frenzy"));
    }

    #[test]
    fn game_range_wins_and_mismatch_is_flagged() {
        let k = fake_knowledge();
        let mut m = unique_mod(ModifierKind::Explicit, "+52 to Strength", 52.0);
        m.roll = Some(crate::item::StatRoll { value: 52.0, min: Some(45.0), max: Some(55.0) });
        let item = unique_item("Headhunter", "Leather Belt", vec![m]);
        let (_, evals) = evaluate_unique(&item, &k);
        assert!(evals[0].range_mismatch);
        assert_eq!(evals[0].status, ModStatus::Exact);
        assert_eq!((evals[0].min, evals[0].max), (Some(45.0), Some(55.0)));
    }
}
