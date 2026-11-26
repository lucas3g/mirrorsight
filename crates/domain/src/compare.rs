//! Agrupa as listagens pelo veredito de upgrade em relação ao item do usuário (só PoE1).
//!
//! Cada anúncio recebe duas avaliações independentes (ver [`crate::evaluation`]):
//! - **upgrade** (MELHOR / PIOR / EQUIVALENTE), que decide o grupo: compara o conjunto de stats
//!   com o do seu item, ponderado pela build;
//! - **qualidade** do item (0–100), que não depende da build — um anúncio pode ter rolls
//!   excelentes e ainda assim ser PIOR para você.
//!
//! Cada grupo é ordenado pelo preço em chaos, do menor para o maior.

use serde::{Deserialize, Serialize};

use crate::evaluation::mods::evaluated_kind;
use crate::evaluation::{compare_items, evaluate_item, BuildProfile, BuildSummary, GameKnowledge, ItemQuality, UpgradeVerdict, Verdict};
use crate::item::{Affix, Item, Rarity};
use crate::price::{CurrencyRates, Listing};
use crate::GameVersion;

/// Tipo de afixo cujo slot livre valoriza o item, por classe (nomes do PoE1).
pub fn valued_affix(item_class: &str) -> Option<Affix> {
    const SUFFIX: &[&str] = &[
        "Helmets", "Gloves", "Boots", "Rings", "Amulets", "Belts", "Claws", "Daggers", "Rune Daggers", "Quivers",
        "One Hand Swords", "Thrusting One Hand Swords",
    ];
    const PREFIX: &[&str] = &[
        "Body Armours", "Shields", "Wands", "Sceptres", "Bows", "Two Hand Swords", "One Hand Axes", "Two Hand Axes",
        "One Hand Maces", "Two Hand Maces", "Staves", "Warstaves",
    ];
    if SUFFIX.contains(&item_class) {
        Some(Affix::Suffix)
    } else if PREFIX.contains(&item_class) {
        Some(Affix::Prefix)
    } else {
        None
    }
}

/// Quantos slots de cada tipo a raridade permite (PoE1). `None` fora de Rare/Magic.
pub const fn affix_limit(rarity: Rarity) -> Option<u32> {
    match rarity {
        Rarity::Rare => Some(3),
        Rarity::Magic => Some(1),
        _ => None,
    }
}

/// Avaliação do item do usuário (faixa "Seu item").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemStatus {
    pub quality: ItemQuality,
    /// Build usada na relevância (genérica quando nenhum build do PoB está carregado).
    pub build: BuildSummary,
    /// Origem da base de conhecimento ("PoB v2.69.0 · 2026-11-26").
    pub data_source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKind {
    Worse,
    Equal,
    Better,
    NotComparable,
    NoItem,
}

/// Avaliação de um anúncio comparável (bloco "Comparação" do olho).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompareDetail {
    pub upgrade: UpgradeVerdict,
    pub quality: ItemQuality,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupEntry {
    /// Posição em `PriceCheckResult::listings`.
    pub index: usize,
    pub chaos: Option<f64>,
    pub detail: Option<CompareDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListingGroup {
    pub kind: GroupKind,
    pub entries: Vec<GroupEntry>,
}

/// Por que a lista não foi agrupada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SingleReason {
    Poe2,
    Rarity,
    Class { class: String },
    NoStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
// Montado uma vez por price check e serializado: o tamanho da variante não importa.
#[allow(clippy::large_enum_variant)]
pub enum ListingComparison {
    Single { reason: SingleReason },
    /// Sempre os 5 grupos, na ordem de [`GroupKind`].
    Grouped { you: ItemStatus, groups: Vec<ListingGroup> },
}

/// Compara cada listagem com o item do usuário e monta os grupos.
pub fn compare_listings(
    game: GameVersion,
    mine: &Item,
    listings: &[Listing],
    rates: &CurrencyRates,
    knowledge: &dyn GameKnowledge,
    profile: &BuildProfile,
) -> ListingComparison {
    let single = |reason| ListingComparison::Single { reason };
    if game == GameVersion::Poe2 {
        return single(SingleReason::Poe2);
    }
    if !matches!(mine.rarity, Rarity::Rare | Rarity::Magic | Rarity::Unique) {
        return single(SingleReason::Rarity);
    }
    if knowledge.rules().excluded_classes.contains(&mine.item_class) {
        return single(SingleReason::Class { class: mine.item_class.clone() });
    }
    if !has_stats(mine) {
        return single(SingleReason::NoStatus);
    }
    let you = ItemStatus { quality: evaluate_item(mine, knowledge), build: profile.summary(), data_source: knowledge.source() };

    let order = [GroupKind::Worse, GroupKind::Equal, GroupKind::Better, GroupKind::NotComparable, GroupKind::NoItem];
    let mut groups: Vec<ListingGroup> = order.iter().map(|&kind| ListingGroup { kind, entries: Vec::new() }).collect();
    for (index, listing) in listings.iter().enumerate() {
        let (kind, detail) = match &listing.item {
            None => (GroupKind::NoItem, None),
            Some(item) if item.item_class != mine.item_class || !has_stats(item) => (GroupKind::NotComparable, None),
            Some(item) => {
                let upgrade = compare_items(mine, item, profile, knowledge);
                let kind = match upgrade.verdict {
                    Verdict::Better => GroupKind::Better,
                    Verdict::Worse => GroupKind::Worse,
                    Verdict::Equivalent => GroupKind::Equal,
                };
                (kind, Some(CompareDetail { upgrade, quality: evaluate_item(item, knowledge) }))
            }
        };
        let entry = GroupEntry { index, chaos: rates.to_chaos(&listing.price), detail };
        if let Some(group) = groups.iter_mut().find(|g| g.kind == kind) {
            group.entries.push(entry);
        }
    }
    for group in &mut groups {
        // Estável: preços iguais mantêm a ordem da trade API; sem cotação vai para o fim.
        group.entries.sort_by(|a, b| match (a.chaos, b.chaos) {
            (Some(x), Some(y)) => x.total_cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }
    ListingComparison::Grouped { you, groups }
}

/// O item tem algo para comparar (mods avaliáveis ou propriedades da base).
fn has_stats(item: &Item) -> bool {
    !item.properties.is_empty() || item.modifiers.iter().any(|m| evaluated_kind(m.kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::test_support::{fake_knowledge, plain, rare_gloves, unique_item, unique_mod};
    use crate::evaluation::Grade;
    use crate::item::ModifierKind;
    use crate::price::{Currency, Price};

    fn listing(amount: f64, currency: &str, item: Option<Item>) -> Listing {
        Listing {
            price: Price { amount, currency: Currency(currency.into()) },
            stack_size: None,
            account: "acc".into(),
            online: true,
            indexed_at: String::new(),
            item,
            whisper: None,
            icon: None,
            max_quality: std::collections::HashMap::new(),
            at_max_quality: false,
        }
    }

    fn rates() -> CurrencyRates {
        CurrencyRates::from_pairs([("divine".into(), 200.0)])
    }

    fn mine() -> Item {
        rare_gloves(vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0)], (1, 1))
    }

    fn grouped(mine: &Item, listings: &[Listing]) -> (ItemStatus, Vec<ListingGroup>) {
        let k = fake_knowledge();
        let profile = BuildProfile::generic(k.rules());
        match compare_listings(GameVersion::Poe1, mine, listings, &rates(), &k, &profile) {
            ListingComparison::Grouped { you, groups } => (you, groups),
            ListingComparison::Single { reason } => panic!("esperava agrupado: {reason:?}"),
        }
    }

    fn group_of(groups: &[ListingGroup], index: usize) -> GroupKind {
        groups.iter().find(|g| g.entries.iter().any(|e| e.index == index)).map(|g| g.kind).expect("no grupo")
    }

    fn detail_of(groups: &[ListingGroup], index: usize) -> CompareDetail {
        groups
            .iter()
            .flat_map(|g| &g.entries)
            .find(|e| e.index == index)
            .and_then(|e| e.detail.clone())
            .expect("detalhe")
    }

    #[test]
    fn always_five_groups_and_your_status() {
        let (you, groups) = grouped(&mine(), &[]);
        let kinds: Vec<GroupKind> = groups.iter().map(|g| g.kind).collect();
        assert_eq!(
            kinds,
            [GroupKind::Worse, GroupKind::Equal, GroupKind::Better, GroupKind::NotComparable, GroupKind::NoItem]
        );
        assert!(you.build.generic);
        assert_eq!(you.quality.mods.len(), 2);
        assert_eq!(you.data_source, "teste");
    }

    #[test]
    fn groups_by_upgrade_verdict() {
        let better = rare_gloves(
            vec![plain("+130 to maximum Life", 130.0), plain("+45% to Fire Resistance", 45.0)],
            (1, 1),
        );
        let worse = rare_gloves(vec![plain("+40% to Fire Resistance", 40.0)], (0, 1));
        let (_, groups) = grouped(&mine(), &[
            listing(1.0, "chaos", Some(better)),
            listing(1.0, "chaos", Some(worse)),
            listing(1.0, "chaos", Some(mine())),
        ]);
        assert_eq!(group_of(&groups, 0), GroupKind::Better);
        assert_eq!(group_of(&groups, 1), GroupKind::Worse);
        assert_eq!(group_of(&groups, 2), GroupKind::Equal);
        assert_eq!(detail_of(&groups, 0).upgrade.verdict, Verdict::Better);
    }

    #[test]
    fn unique_listings_use_database_ranges() {
        let hh = |life: f64, str: f64| {
            unique_item(
                "Headhunter",
                "Leather Belt",
                vec![
                    unique_mod(ModifierKind::Explicit, &format!("+{life} to maximum Life"), life),
                    unique_mod(ModifierKind::Explicit, &format!("+{str} to Strength"), str),
                ],
            )
        };
        let (you, groups) = grouped(&hh(55.0, 47.0), &[listing(1.0, "chaos", Some(hh(60.0, 55.0)))]);
        assert!(you.quality.unique.as_ref().is_some_and(|u| u.found));
        let d = detail_of(&groups, 0);
        assert_eq!(d.upgrade.verdict, Verdict::Better);
        assert_eq!(d.quality.score, Some(100.0));
    }

    #[test]
    fn excellent_listing_can_still_be_worse() {
        let mine = rare_gloves(vec![plain("+80 to maximum Life", 80.0), plain("+40% to Fire Resistance", 40.0)], (1, 1));
        // Um mod só (T1 com roll máximo) e slots livres: qualidade excelente, mas sem a resistência.
        let top = rare_gloves(
            vec![crate::evaluation::test_support::tiered("+144 to maximum Life", 144.0, 1, (130.0, 144.0))],
            (1, 0),
        );
        let (_, groups) = grouped(&mine, &[listing(1.0, "chaos", Some(top))]);
        let d = detail_of(&groups, 0);
        assert_eq!(d.quality.grade, Some(Grade::Excellent));
        assert_eq!(d.upgrade.verdict, Verdict::Worse);
    }

    #[test]
    fn not_comparable_and_no_item_go_to_their_groups() {
        let mut ring = mine();
        ring.item_class = "Rings".into();
        let empty = rare_gloves(vec![], (0, 0));
        let (_, groups) = grouped(&mine(), &[
            listing(1.0, "chaos", Some(ring)),
            listing(1.0, "chaos", Some(empty)),
            listing(1.0, "chaos", None),
        ]);
        assert_eq!(group_of(&groups, 0), GroupKind::NotComparable);
        assert_eq!(group_of(&groups, 1), GroupKind::NotComparable);
        assert_eq!(group_of(&groups, 2), GroupKind::NoItem);
    }

    #[test]
    fn groups_are_sorted_by_chaos_value() {
        let (_, groups) = grouped(&mine(), &[
            listing(0.5, "divine", Some(mine())), // 100c
            listing(150.0, "chaos", Some(mine())),
            listing(3.0, "mystery", Some(mine())), // sem cotação: fim
            listing(20.0, "chaos", Some(mine())),
        ]);
        let equal = groups.iter().find(|g| g.kind == GroupKind::Equal).unwrap();
        let order: Vec<usize> = equal.entries.iter().map(|e| e.index).collect();
        assert_eq!(order, vec![3, 0, 1, 2]);
        assert_eq!(equal.entries[1].chaos, Some(100.0));
        assert_eq!(equal.entries[3].chaos, None);
    }

    #[test]
    fn single_list_cases() {
        let k = fake_knowledge();
        let profile = BuildProfile::generic(k.rules());
        let reason = |game, item: &Item| match compare_listings(game, item, &[], &rates(), &k, &profile) {
            ListingComparison::Single { reason } => reason,
            ListingComparison::Grouped { .. } => panic!("esperava lista única"),
        };
        assert_eq!(reason(GameVersion::Poe2, &mine()), SingleReason::Poe2);

        let mut normal = mine();
        normal.rarity = Rarity::Normal;
        assert_eq!(reason(GameVersion::Poe1, &normal), SingleReason::Rarity);

        let mut map = mine();
        map.item_class = "Maps".into();
        assert_eq!(reason(GameVersion::Poe1, &map), SingleReason::Class { class: "Maps".into() });

        let empty = rare_gloves(vec![], (0, 0));
        assert_eq!(reason(GameVersion::Poe1, &empty), SingleReason::NoStatus);
    }

    #[test]
    fn valued_affix_table() {
        assert_eq!(valued_affix("Bows"), Some(Affix::Prefix));
        assert_eq!(valued_affix("Helmets"), Some(Affix::Suffix));
        assert_eq!(valued_affix("Thrusting One Hand Swords"), Some(Affix::Suffix));
        assert_eq!(valued_affix("Two Hand Swords"), Some(Affix::Prefix));
        assert_eq!(valued_affix("Jewels"), None);
        assert_eq!(valued_affix("Life Flasks"), None);
    }
}
