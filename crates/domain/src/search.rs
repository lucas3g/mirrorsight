//! Filtros de busca editáveis pelo usuário no price check.

use serde::{Deserialize, Serialize};

use crate::item::{Item, Modifier, ModifierKind, StatRoll};
use crate::settings::Settings;

/// Filtro de um modificador. Carrega texto/tipo/roll para a UI não precisar
/// cruzar dados com o item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatFilter {
    pub trade_id: String,
    pub text: String,
    pub kind: ModifierKind,
    pub roll: Option<StatRoll>,
    #[serde(default)]
    pub tier: Option<u32>,
    pub enabled: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchFilters {
    pub online_only: bool,
    pub stats: Vec<StatFilter>,
    pub item_level_min: Option<u32>,
    pub quality_min: Option<u32>,
    #[serde(default)]
    pub links_min: Option<u32>,
}

impl SearchFilters {
    /// Filtros iniciais a partir do item e das preferências do usuário.
    pub fn defaults_for(item: &Item, settings: &Settings) -> Self {
        let range = f64::from(settings.mod_range_percent.min(100)) / 100.0;
        // Viscera (Covens): o único mod (implicit) é o que define o preço.
        let all_mods = item.item_class == "Viscera";
        Self {
            online_only: !settings.show_offline,
            // Propriedades (defesas/DPS) primeiro; ligadas só se o tipo estiver nos padrões do usuário.
            stats: item
                .properties
                .iter()
                .map(|p| stat_filter(p, settings.default_filter_kinds.contains(&p.kind), range))
                .chain(
                    item.modifiers
                        .iter()
                        .map(|m| stat_filter(m, all_mods || settings.default_filter_kinds.contains(&m.kind), range)),
                )
                .collect(),
            item_level_min: None,
            quality_min: None,
            // 5L/6L definem o preço de armaduras corporais; abaixo disso não vale filtrar.
            links_min: Some(item.max_links()).filter(|links| *links >= 5),
        }
    }
}

fn stat_filter(modifier: &Modifier, enabled: bool, range: f64) -> StatFilter {
    StatFilter {
        trade_id: modifier.trade_id.clone(),
        text: modifier.text.clone(),
        kind: modifier.kind,
        roll: modifier.roll,
        tier: modifier.tier,
        enabled,
        min: modifier.roll.map(|r| floor_like(r.value - r.value.abs() * range, r.value)),
        max: None,
    }
}

/// Arredonda para baixo com a mesma precisão do valor rolado: inteiros ficam inteiros
/// (78 → 70), decimais mantêm 2 casas (1.45 APS → 1.30, não 1).
fn floor_like(value: f64, rolled: f64) -> f64 {
    if rolled.fract() == 0.0 {
        value.floor()
    } else {
        (value * 100.0).floor() / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Rarity;

    fn item_with(mods: Vec<Modifier>) -> Item {
        Item {
            item_class: "Gloves".into(),
            rarity: Rarity::Rare,
            name: Some("Doom Grip".into()),
            base_type: "Sorcerer Gloves".into(),
            item_level: Some(86),
            quality: None,
            corrupted: false,
            identified: true,
            sockets: None,
            influences: vec![],
            modifiers: mods,
            properties: Vec::new(),
            unknown_lines: vec![],
            foulborn: false,
            discriminator: None,
            trade_type: None,
            map_tier: None,
        }
    }

    fn modifier(kind: ModifierKind, value: f64) -> Modifier {
        Modifier {
            trade_id: format!("{kind:?}.stat"),
            text: format!("+{value} to maximum Life"),
            kind,
            roll: Some(StatRoll {
                value,
                min: None,
                max: None,
            }),
            tier: None,
        }
    }

    #[test]
    fn enables_only_default_kinds_and_applies_range() {
        let settings = Settings {
            mod_range_percent: 10,
            default_filter_kinds: vec![ModifierKind::Explicit],
            ..Settings::default()
        };
        let item = item_with(vec![
            modifier(ModifierKind::Explicit, 78.0),
            modifier(ModifierKind::Crafted, 16.0),
        ]);
        let filters = SearchFilters::defaults_for(&item, &settings);

        assert!(filters.stats[0].enabled);
        assert!(!filters.stats[1].enabled);
        assert_eq!(filters.stats[0].min, Some(70.0)); // 78 - 7.8 → 70
        assert!(filters.online_only);
    }

    #[test]
    fn properties_come_first_and_keep_decimals() {
        let mut item = item_with(vec![modifier(ModifierKind::Explicit, 78.0)]);
        item.properties = vec![Modifier {
            trade_id: "property.aps".into(),
            text: "Attacks per Second: 1.45".into(),
            kind: ModifierKind::Property,
            roll: Some(StatRoll { value: 1.45, min: None, max: None }),
            tier: None,
        }];
        let settings = Settings { mod_range_percent: 10, ..Settings::default() };
        let filters = SearchFilters::defaults_for(&item, &settings);
        assert_eq!(filters.stats[0].trade_id, "property.aps");
        assert_eq!(filters.stats[0].min, Some(1.3)); // 1.305 → 1.30
        assert_eq!(filters.stats.len(), 2);
    }

    #[test]
    fn negative_rolls_widen_downwards() {
        let filters = SearchFilters::defaults_for(
            &item_with(vec![modifier(ModifierKind::Explicit, -20.0)]),
            &Settings::default(),
        );
        assert_eq!(filters.stats[0].min, Some(-22.0));
    }
}
