//! Filtros de propriedade (Armour, Evasion, ES, DPS…) contra a regra da trade API do PoE1.
//!
//! A trade compara defesas e DPS **a 20% de qualidade** (\`extended.ar\` etc.): um escudo com
//! Armour 1110 a 0% vale 1332 para o filtro e passa num mínimo de 1304. Para um item que ainda
//! aceita qualidade isso é justo (dá para chegar lá). Para um **corrompido** não: ele nunca passa
//! do valor atual. Por isso:
//! - item que aceita qualidade: as propriedades passam a valer o número a 20% (o mesmo da trade),
//!   e o anúncio fica marcado (\`Listing::at_max_quality\`) para a interface avisar;
//! - item que não aceita mais qualidade (corrompido ou espelhado) abaixo do mínimo de um filtro
//!   de propriedade ligado: o anúncio sai da lista.
//!
//! A remoção ([`apply_property_minimums`]) e a troca dos valores ([`normalize_to_max_quality`])
//! são separadas: a comparação com o seu item (que está na qualidade atual) roda entre as duas,
//! com os valores reais do anúncio.

use crate::item::{Item, PROPERTY_PREFIX};
use crate::price::Listing;
use crate::search::SearchFilters;

/// Tolerância de arredondamento entre o texto do item e o valor indexado.
const TOLERANCE: f64 = 0.5;

/// Corrompidos e espelhados não aceitam mais qualidade: valem o que mostram.
const fn accepts_quality(item: &Item) -> bool {
    !item.corrupted && !item.flags.mirrored
}

/// Valor da propriedade `id` a 20% de qualidade, quando a trade informa um número maior que o
/// atual (qualidade acima de 20% nunca é rebaixada). Só vale para itens que aceitam qualidade.
fn raised_value(listing: &Listing, id: &str, current: f64) -> Option<f64> {
    listing.max_quality.get(id).copied().filter(|q20| q20 - current > TOLERANCE)
}

/// Tira da lista os anúncios que não aceitam mais qualidade e estão abaixo do mínimo de um
/// filtro de propriedade ligado. Não altera os anúncios que ficam. Devolve quantos saíram.
pub fn apply_property_minimums(listings: &mut Vec<Listing>, filters: &SearchFilters) -> u32 {
    let minimums: Vec<(&str, f64)> = filters
        .stats
        .iter()
        .filter(|s| s.enabled)
        .filter_map(|s| Some((s.trade_id.strip_prefix(PROPERTY_PREFIX)?, s.min?)))
        .collect();
    let before = listings.len();
    listings.retain(|listing| {
        let Some(item) = listing.item.as_ref() else { return true };
        // Quem ainda aceita qualidade já passou no filtro da trade (que compara a 20%): fica,
        // mesmo que a API não informe o valor a 20% (`max_quality` vazio).
        if accepts_quality(item) {
            return true;
        }
        // Corrompido/espelhado: vale o valor atual.
        minimums.iter().all(|(id, min)| {
            item.properties
                .iter()
                .find(|p| p.trade_id.strip_prefix(PROPERTY_PREFIX) == Some(id))
                .and_then(|p| p.roll)
                .filter(|r| r.value + TOLERANCE < *min)
                .is_none()
        })
    });
    u32::try_from(before - listings.len()).unwrap_or(u32::MAX)
}

/// Troca as propriedades dos anúncios que aceitam qualidade pelos valores a 20% (os que a trade
/// compara) e marca o anúncio. Rodar depois da comparação com o seu item.
pub fn normalize_to_max_quality(listings: &mut [Listing]) {
    for listing in listings.iter_mut() {
        let Some(mut item) = listing.item.take() else { continue };
        if accepts_quality(&item) {
            for prop in &mut item.properties {
                let Some(id) = prop.trade_id.strip_prefix(PROPERTY_PREFIX) else { continue };
                let Some(roll) = prop.roll.as_mut() else { continue };
                if let Some(raised) = raised_value(listing, id, roll.value) {
                    roll.value = raised;
                    prop.text = relabel(&prop.text, raised);
                    listing.at_max_quality = true;
                }
            }
        }
        listing.item = Some(item);
    }
}

/// "Armour: 1110" → "Armour: 1332" (inteiros sem casas; DPS com uma).
fn relabel(text: &str, value: f64) -> String {
    let label = text.split_once(": ").map_or(text, |(label, _)| label);
    let number = if (value - value.round()).abs() < 1e-9 { format!("{value:.0}") } else { format!("{value:.1}") };
    format!("{label}: {number}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::item::{Item, ItemFlags, Modifier, ModifierKind, Rarity, StatRoll};
    use crate::price::{Currency, Price};
    use crate::search::StatFilter;

    fn shield(armour: f64, corrupted: bool) -> Item {
        Item {
            item_class: "Shields".into(),
            rarity: Rarity::Unique,
            name: Some("Aegis Aurora".into()),
            base_type: "Champion Kite Shield".into(),
            item_level: Some(83),
            quality: None,
            corrupted,
            identified: true,
            sockets: None,
            influences: vec![],
            modifiers: vec![],
            properties: vec![Modifier {
                trade_id: "property.ar".into(),
                text: format!("Armour: {armour}"),
                kind: ModifierKind::Property,
                roll: Some(StatRoll { value: armour, min: None, max: None }),
                ..Modifier::default()
            }],
            unknown_lines: vec![],
            foulborn: false,
            discriminator: None,
            trade_type: None,
            map_tier: None,
            affix_slots: None,
            flags: ItemFlags::default(),
        }
    }

    fn listing(item: Item, ar_at_20: f64) -> Listing {
        Listing {
            price: Price { amount: 1.0, currency: Currency::chaos() },
            stack_size: None,
            account: "acc".into(),
            online: true,
            indexed_at: String::new(),
            item: Some(item),
            whisper: None,
            icon: None,
            max_quality: HashMap::from([("ar".to_owned(), ar_at_20)]),
            at_max_quality: false,
        }
    }

    fn filters(min: Option<f64>, enabled: bool) -> SearchFilters {
        SearchFilters {
            online_only: false,
            stats: vec![StatFilter {
                trade_id: "property.ar".into(),
                text: "Armour: 1373".into(),
                kind: ModifierKind::Property,
                roll: None,
                tier: None,
                enabled,
                min,
                max: None,
            }],
            item_level_min: None,
            quality_min: None,
            links_min: None,
        }
    }

    #[test]
    fn corrupted_below_minimum_is_removed_and_others_use_20_percent_quality() {
        let mut listings = vec![
            listing(shield(1110.0, true), 1332.0),  // corrompido: 1110 < 1304 → sai
            listing(shield(1110.0, false), 1332.0), // aceita qualidade: vale 1332 → fica
            listing(shield(1367.0, true), 1367.0),  // corrompido acima do mínimo → fica
        ];
        let hidden = apply_property_minimums(&mut listings, &filters(Some(1304.0), true));
        assert_eq!(hidden, 1);
        assert_eq!(listings.len(), 2);
        // A remoção não mexe nos valores: a comparação com o seu item usa os reais.
        assert_eq!(listings[0].item.as_ref().unwrap().properties[0].roll.unwrap().value, 1110.0);
        assert!(!listings[0].at_max_quality);
        normalize_to_max_quality(&mut listings);
        let normalized = listings[0].item.as_ref().unwrap();
        assert!(listings[0].at_max_quality);
        assert_eq!(normalized.properties[0].roll.unwrap().value, 1332.0);
        assert_eq!(normalized.properties[0].text, "Armour: 1332");
        // Sem diferença (qualidade já ≥ 20%): nada muda nem é marcado.
        assert!(!listings[1].at_max_quality);
    }

    #[test]
    fn improvable_listing_without_20_percent_value_is_kept() {
        // A API não informou `extended.ar`: o item aceita qualidade, então não é escondido.
        let mut no_data = listing(shield(1110.0, false), 0.0);
        no_data.max_quality.clear();
        let mut listings = vec![no_data];
        assert_eq!(apply_property_minimums(&mut listings, &filters(Some(1304.0), true)), 0);
        assert_eq!(listings.len(), 1);
    }

    #[test]
    fn disabled_or_empty_minimum_keeps_corrupted_listings() {
        for f in [filters(Some(1304.0), false), filters(None, true)] {
            let mut listings = vec![listing(shield(1110.0, true), 1332.0)];
            assert_eq!(apply_property_minimums(&mut listings, &f), 0);
            assert_eq!(listings.len(), 1);
            // Corrompido nunca é normalizado.
            normalize_to_max_quality(&mut listings);
            assert_eq!(listings[0].item.as_ref().unwrap().properties[0].roll.unwrap().value, 1110.0);
        }
    }

    #[test]
    fn mirrored_counts_as_final_and_quality_above_20_is_not_lowered() {
        let mut mirrored = shield(1110.0, false);
        mirrored.flags.mirrored = true;
        // Qualidade 28%: vale mais que o número a 20% da trade.
        let mut listings = vec![listing(mirrored, 1332.0), listing(shield(1450.0, false), 1332.0)];
        assert_eq!(apply_property_minimums(&mut listings, &filters(Some(1304.0), true)), 1);
        normalize_to_max_quality(&mut listings);
        assert_eq!(listings.len(), 1);
        assert_eq!(listings[0].item.as_ref().unwrap().properties[0].roll.unwrap().value, 1450.0);
        assert!(!listings[0].at_max_quality);
    }
}
