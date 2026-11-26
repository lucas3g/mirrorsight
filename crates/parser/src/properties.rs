//! Propriedades da base no bloco "Chave: valor" (defesas e dano), como filtros de busca.
//!
//! Os valores do texto já incluem a qualidade atual e os mods locais. A trade API do PoE1 indexa
//! defesas e DPS a 20% de qualidade (`extended.ar`…): ver `mirrorsight_domain::property_filter`.
//! Armas viram DPS (físico, elemental e total), como os filtros `pdps`/`edps`/`dps` do site.

use mirrorsight_domain::item::{Modifier, ModifierKind, StatRoll, PROPERTY_PREFIX};

/// Defesas: (rótulos no texto do jogo, id do filtro, rótulo exibido).
const DEFENCES: &[(&[&str], &str, &str)] = &[
    (&["Armour"], "ar", "Armour"),
    (&["Evasion Rating"], "ev", "Evasion Rating"),
    (&["Energy Shield"], "es", "Energy Shield"),
    (&["Ward"], "ward", "Ward"),
    (&["Chance to Block", "Block chance"], "block", "Chance to Block"),
    (&["Spirit"], "spirit", "Spirit"),
];

const ELEMENTAL: &[&str] = &["Elemental Damage", "Fire Damage", "Cold Damage", "Lightning Damage"];

/// Extrai as propriedades das linhas do bloco (as que não forem "Chave: valor" são ignoradas).
pub fn extract(lines: &[&str]) -> Vec<Modifier> {
    let mut out = Vec::new();
    let value_of = |labels: &[&str]| {
        lines.iter().find_map(|l| {
            let (key, raw) = l.split_once(": ")?;
            labels.contains(&key.trim()).then_some(raw)
        })
    };

    for (labels, id, label) in DEFENCES {
        if let Some(v) = value_of(labels).and_then(leading_decimal).filter(|v| *v > 0.0) {
            let suffix = if *id == "block" { "%" } else { "" };
            out.push(property(id, &format!("{label}: {}{suffix}", fmt(v)), v));
        }
    }

    // Dano de ataque: média de cada faixa × ataques por segundo.
    let aps = value_of(&["Attacks per Second"]).and_then(leading_decimal);
    let physical = value_of(&["Physical Damage"]).map_or(0.0, average_damage);
    let elemental: f64 = lines
        .iter()
        .filter_map(|l| l.split_once(": ").filter(|(k, _)| ELEMENTAL.contains(&k.trim())))
        .map(|(_, raw)| average_damage(raw))
        .sum();
    let chaos = value_of(&["Chaos Damage"]).map_or(0.0, average_damage);
    if let Some(aps) = aps.filter(|a| *a > 0.0) {
        let round = |v: f64| (v * 10.0).round() / 10.0;
        let (pdps, edps) = (round(physical * aps), round(elemental * aps));
        let dps = round((physical + elemental + chaos) * aps);
        if dps > 0.0 {
            out.push(property("dps", &format!("Total DPS: {}", fmt(dps)), dps));
        }
        if pdps > 0.0 {
            out.push(property("pdps", &format!("Physical DPS: {}", fmt(pdps)), pdps));
        }
        if edps > 0.0 {
            out.push(property("edps", &format!("Elemental DPS: {}", fmt(edps)), edps));
        }
        out.push(property("aps", &format!("Attacks per Second: {}", fmt(aps)), aps));
        // Só armas (têm APS): gemas de feitiço também mostram "Critical Strike Chance",
        // e o filtro `crit` de `weapon_filters` zeraria a busca delas.
        if let Some(crit) = value_of(&["Critical Strike Chance", "Critical Hit Chance"]).and_then(leading_decimal) {
            out.push(property("crit", &format!("Critical Strike Chance: {}%", fmt(crit)), crit));
        }
    }
    out
}

fn property(id: &str, text: &str, value: f64) -> Modifier {
    Modifier {
        trade_id: format!("{PROPERTY_PREFIX}{id}"),
        text: text.to_owned(),
        kind: ModifierKind::Property,
        roll: Some(StatRoll { value, min: None, max: None }),
        tier: None,
        affix: None,
        ..Default::default()
    }
}

/// "6.50% (augmented)" → 6.5; "1373 (augmented)" → 1373.
fn leading_decimal(raw: &str) -> Option<f64> {
    let number: String = raw
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    number.parse().ok()
}

/// Soma das médias das faixas: "50-90 (augmented), 10-200 (augmented)" → 70 + 105.
fn average_damage(raw: &str) -> f64 {
    raw.split(',')
        .filter_map(|part| {
            let range = part.split_whitespace().next()?;
            let (lo, hi) = range.split_once('-')?;
            Some((lo.parse::<f64>().ok()? + hi.parse::<f64>().ok()?) / 2.0)
        })
        .sum()
}

/// Sem casas decimais desnecessárias: 1373 → "1373", 1.45 → "1.45", 285.4 → "285.4".
fn fmt(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(props: &[Modifier]) -> Vec<(&str, f64)> {
        props.iter().map(|p| (p.trade_id.as_str(), p.roll.unwrap().value)).collect()
    }

    #[test]
    fn shield_defences_and_block() {
        let p = extract(&["Chance to Block: 32% (augmented)", "Armour: 1373 (augmented)", "Energy Shield: 276 (augmented)"]);
        assert_eq!(ids(&p), [("property.ar", 1373.0), ("property.es", 276.0), ("property.block", 32.0)]);
        assert_eq!(p[0].text, "Armour: 1373");
        assert_eq!(p[2].text, "Chance to Block: 32%");
    }

    #[test]
    fn weapon_dps_from_damage_and_speed() {
        let p = extract(&[
            "Quality: +20% (augmented)",
            "Physical Damage: 137-253 (augmented)",
            "Elemental Damage: 10-20 (augmented), 5-15 (augmented)",
            "Critical Strike Chance: 6.50%",
            "Attacks per Second: 1.50 (augmented)",
        ]);
        // físico 195 × 1.5 = 292.5; elemental (15 + 10) × 1.5 = 37.5
        assert_eq!(
            ids(&p),
            [
                ("property.dps", 330.0),
                ("property.pdps", 292.5),
                ("property.edps", 37.5),
                ("property.aps", 1.5),
                ("property.crit", 6.5),
            ]
        );
        assert_eq!(p[1].text, "Physical DPS: 292.5");
        assert_eq!(p[3].text, "Attacks per Second: 1.5");
    }

    #[test]
    fn poe2_labels() {
        let p = extract(&["Fire Damage: 10-20", "Critical Hit Chance: 5.00%", "Attacks per Second: 1.20", "Spirit: 100"]);
        assert_eq!(
            ids(&p),
            [("property.spirit", 100.0), ("property.dps", 18.0), ("property.edps", 18.0), ("property.aps", 1.2), ("property.crit", 5.0)]
        );
    }

    #[test]
    fn non_equipment_block_has_no_properties() {
        assert!(extract(&["Stack Size: 3/10", "Quality: +20%"]).is_empty());
    }
}
