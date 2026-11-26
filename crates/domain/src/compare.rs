//! Agrupa as listagens pelo "status" do item comparado ao item do usuário (só PoE1).
//!
//! Critérios, em ordem (lexicográfica):
//! 1. tier médio dos mods explícitos em comum (menor é melhor);
//! 2. percentil médio dos rolls desses mods (maior é melhor);
//! 3. slots livres do tipo de afixo valorizado para a classe (mais é melhor).
//!
//! Os dois primeiros são comparados com 2 casas decimais. Cada grupo é ordenado pelo preço
//! em chaos, do menor para o maior.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::item::{Affix, Item, Modifier, ModifierKind, Rarity};
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

/// Tier e percentil de um mod que entra na qualidade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModStat {
    pub trade_id: String,
    pub tier: u32,
    /// Posição do roll dentro da faixa do tier, de 0 a 1.
    pub percentile: f64,
}

/// Status do item do usuário (faixa "Seu item").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemStatus {
    pub avg_tier: f64,
    pub avg_percentile: f64,
    pub free_valued: u32,
    pub valued: Affix,
    pub mods: Vec<ModStat>,
}

/// Valores comparados de um lado (anúncio ou seu item), só nos mods em comum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Score {
    pub avg_tier: f64,
    pub avg_percentile: f64,
    pub free_valued: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criterion {
    Tier,
    Percentile,
    Slots,
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

/// Detalhe da comparação de um anúncio comparável (bloco "Comparação" do olho).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompareDetail {
    pub common: u32,
    /// `trade_id`s do anúncio que estão em comum com o seu item.
    pub common_ids: Vec<String>,
    pub listing: Score,
    pub yours: Score,
    /// Critério que decidiu o grupo; `None` quando é igual.
    pub decided: Option<Criterion>,
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
) -> ListingComparison {
    let single = |reason| ListingComparison::Single { reason };
    if game == GameVersion::Poe2 {
        return single(SingleReason::Poe2);
    }
    if affix_limit(mine.rarity).is_none() {
        return single(SingleReason::Rarity);
    }
    let Some(valued) = valued_affix(&mine.item_class) else {
        return single(SingleReason::Class { class: mine.item_class.clone() });
    };
    let Some(you) = item_status(mine, valued) else {
        return single(SingleReason::NoStatus);
    };

    let order = [GroupKind::Worse, GroupKind::Equal, GroupKind::Better, GroupKind::NotComparable, GroupKind::NoItem];
    let mut groups: Vec<ListingGroup> = order.iter().map(|&kind| ListingGroup { kind, entries: Vec::new() }).collect();
    for (index, listing) in listings.iter().enumerate() {
        let (kind, detail) = match &listing.item {
            None => (GroupKind::NoItem, None),
            Some(item) => match compare_one(&you, item) {
                Some((kind, detail)) => (kind, Some(detail)),
                None => (GroupKind::NotComparable, None),
            },
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

/// Fractured e explicit são o mesmo stat para a comparação ("fractured.stat_1" ≡ "explicit.stat_1").
fn stat_key(trade_id: &str) -> &str {
    trade_id
        .strip_prefix("explicit.")
        .or_else(|| trade_id.strip_prefix("fractured."))
        .unwrap_or(trade_id)
}

/// Só explícitos (e fractured): implícitos ficam de fora; crafted tem rank, não tier.
const fn counts_for_quality(m: &Modifier) -> bool {
    matches!(m.kind, ModifierKind::Explicit | ModifierKind::Fractured)
}

/// Tier e percentil; `None` se o mod não tem tier ou faixa.
fn mod_stat(m: &Modifier) -> Option<ModStat> {
    let tier = m.tier?;
    let roll = m.roll?;
    let (min, max) = (roll.min?, roll.max?);
    let span = max - min;
    let percentile = if span.abs() <= f64::EPSILON { 1.0 } else { ((roll.value - min) / span).clamp(0.0, 1.0) };
    Some(ModStat { trade_id: m.trade_id.clone(), tier, percentile })
}

fn free_slots(item: &Item, valued: Affix) -> u32 {
    match (affix_limit(item.rarity), item.affix_slots) {
        (Some(limit), Some(slots)) => limit.saturating_sub(slots.used(valued)),
        // Slot desconhecido conta como 0.
        _ => 0,
    }
}

fn item_status(mine: &Item, valued: Affix) -> Option<ItemStatus> {
    // Sem afixos conhecidos (cópia sem tiers) não há como contar slots livres.
    mine.affix_slots?;
    let mods: Vec<ModStat> = mine.modifiers.iter().filter(|m| counts_for_quality(m)).filter_map(mod_stat).collect();
    if mods.is_empty() {
        return None;
    }
    let (avg_tier, avg_percentile) = averages(mods.iter());
    Some(ItemStatus { avg_tier, avg_percentile, free_valued: free_slots(mine, valued), valued, mods })
}

#[allow(clippy::cast_precision_loss)]
fn averages<'a>(stats: impl Iterator<Item = &'a ModStat>) -> (f64, f64) {
    let (mut n, mut tiers, mut pct) = (0_usize, 0.0, 0.0);
    for s in stats {
        n += 1;
        tiers += f64::from(s.tier);
        pct += s.percentile;
    }
    if n == 0 {
        (0.0, 0.0)
    } else {
        (tiers / n as f64, pct / n as f64)
    }
}

/// Valor com 2 casas, como inteiro (evita comparar floats).
#[allow(clippy::cast_possible_truncation)]
fn round2(v: f64) -> i64 {
    (v * 100.0).round() as i64
}

/// `None` = não comparável (nenhum mod em comum, ou mod em comum sem tier/faixa no anúncio).
fn compare_one(you: &ItemStatus, item: &Item) -> Option<(GroupKind, CompareDetail)> {
    let mut by_key: HashMap<&str, &Modifier> = HashMap::new();
    for m in item.modifiers.iter().filter(|m| counts_for_quality(m)) {
        by_key.entry(stat_key(&m.trade_id)).or_insert(m);
    }

    let mut theirs = Vec::new();
    let mut yours = Vec::new();
    for mine in &you.mods {
        if let Some(m) = by_key.get(stat_key(&mine.trade_id)) {
            theirs.push(mod_stat(m)?);
            yours.push(mine.clone());
        }
    }
    if theirs.is_empty() {
        return None;
    }

    let (l_tier, l_pct) = averages(theirs.iter());
    let (y_tier, y_pct) = averages(yours.iter());
    let listing = Score { avg_tier: l_tier, avg_percentile: l_pct, free_valued: free_slots(item, you.valued) };
    let yours_score = Score { avg_tier: y_tier, avg_percentile: y_pct, free_valued: you.free_valued };

    use std::cmp::Ordering;
    // Ordering "do ponto de vista do anúncio": Greater = anúncio melhor.
    let steps = [
        (Criterion::Tier, round2(y_tier).cmp(&round2(l_tier))),
        (Criterion::Percentile, round2(l_pct).cmp(&round2(y_pct))),
        (Criterion::Slots, listing.free_valued.cmp(&you.free_valued)),
    ];
    let decided = steps.iter().find(|(_, ord)| *ord != Ordering::Equal);
    let (kind, decided) = match decided {
        Some((c, Ordering::Greater)) => (GroupKind::Better, Some(*c)),
        Some((c, _)) => (GroupKind::Worse, Some(*c)),
        None => (GroupKind::Equal, None),
    };
    let detail = CompareDetail {
        common: u32::try_from(theirs.len()).unwrap_or(u32::MAX),
        common_ids: theirs.into_iter().map(|s| s.trade_id).collect(),
        listing,
        yours: yours_score,
        decided,
    };
    Some((kind, detail))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{AffixSlots, StatRoll};
    use crate::price::{Currency, Price};

    fn modifier(id: &str, tier: u32, value: f64, min: f64, max: f64) -> Modifier {
        Modifier {
            trade_id: format!("explicit.{id}"),
            text: String::new(),
            kind: ModifierKind::Explicit,
            roll: Some(StatRoll { value, min: Some(min), max: Some(max) }),
            tier: Some(tier),
            affix: None,
        }
    }

    fn item(class: &str, rarity: Rarity, mods: Vec<Modifier>, slots: Option<(u32, u32)>) -> Item {
        Item {
            item_class: class.into(),
            rarity,
            name: None,
            base_type: "Base".into(),
            item_level: None,
            quality: None,
            corrupted: false,
            identified: true,
            sockets: None,
            influences: vec![],
            modifiers: mods,
            properties: vec![],
            unknown_lines: vec![],
            foulborn: false,
            discriminator: None,
            trade_type: None,
            map_tier: None,
            affix_slots: slots.map(|(prefixes, suffixes)| AffixSlots { prefixes, suffixes }),
        }
    }

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
        }
    }

    fn rates() -> CurrencyRates {
        CurrencyRates::from_pairs([("divine".into(), 200.0)])
    }

    /// Luvas (suffix valorizado) com life T2 50% e fire res T1 50%, 2 suffixes usados → 1 livre.
    fn mine() -> Item {
        item(
            "Gloves",
            Rarity::Rare,
            vec![modifier("life", 2, 75.0, 70.0, 80.0), modifier("fire", 1, 43.0, 42.0, 44.0)],
            Some((1, 2)),
        )
    }

    fn grouped(listings: &[Listing]) -> (ItemStatus, Vec<ListingGroup>) {
        match compare_listings(GameVersion::Poe1, &mine(), listings, &rates()) {
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

    fn gloves(mods: Vec<Modifier>, slots: Option<(u32, u32)>) -> Option<Item> {
        Some(item("Gloves", Rarity::Rare, mods, slots))
    }

    #[test]
    fn computes_your_status() {
        let (you, groups) = grouped(&[]);
        assert_eq!(you.avg_tier, 1.5);
        assert_eq!(you.avg_percentile, 0.5);
        assert_eq!(you.free_valued, 1);
        assert_eq!(you.valued, Affix::Suffix);
        let kinds: Vec<GroupKind> = groups.iter().map(|g| g.kind).collect();
        assert_eq!(
            kinds,
            [GroupKind::Worse, GroupKind::Equal, GroupKind::Better, GroupKind::NotComparable, GroupKind::NoItem]
        );
        assert!(groups.iter().all(|g| g.entries.is_empty()));
    }

    #[test]
    fn lower_average_tier_is_better() {
        let better = gloves(vec![modifier("life", 1, 71.0, 70.0, 80.0), modifier("fire", 1, 42.0, 42.0, 44.0)], Some((1, 3)));
        let worse = gloves(vec![modifier("life", 3, 80.0, 70.0, 80.0), modifier("fire", 1, 44.0, 42.0, 44.0)], Some((1, 1)));
        let (_, groups) = grouped(&[listing(1.0, "chaos", better), listing(1.0, "chaos", worse)]);
        assert_eq!(group_of(&groups, 0), GroupKind::Better);
        assert_eq!(detail_of(&groups, 0).decided, Some(Criterion::Tier));
        assert_eq!(group_of(&groups, 1), GroupKind::Worse);
    }

    #[test]
    fn uses_average_of_common_mods_only() {
        // Só life em comum (T2 = seu T2): o fire res ausente não favorece o anúncio.
        let one_mod = gloves(vec![modifier("life", 2, 75.0, 70.0, 80.0), modifier("other", 5, 1.0, 0.0, 2.0)], Some((1, 2)));
        let (_, groups) = grouped(&[listing(1.0, "chaos", one_mod)]);
        let d = detail_of(&groups, 0);
        assert_eq!(d.common, 1);
        assert_eq!(d.common_ids, vec!["explicit.life".to_owned()]);
        assert_eq!(d.yours.avg_tier, 2.0);
        assert_eq!(group_of(&groups, 0), GroupKind::Equal);
    }

    #[test]
    fn percentile_breaks_tier_ties() {
        let high = gloves(vec![modifier("life", 2, 80.0, 70.0, 80.0), modifier("fire", 1, 44.0, 42.0, 44.0)], Some((1, 3)));
        let low = gloves(vec![modifier("life", 2, 70.0, 70.0, 80.0), modifier("fire", 1, 42.0, 42.0, 44.0)], Some((1, 0)));
        let (_, groups) = grouped(&[listing(1.0, "chaos", high), listing(1.0, "chaos", low)]);
        assert_eq!(group_of(&groups, 0), GroupKind::Better);
        assert_eq!(detail_of(&groups, 0).decided, Some(Criterion::Percentile));
        assert_eq!(group_of(&groups, 1), GroupKind::Worse);
    }

    #[test]
    fn free_valued_slot_decides_when_quality_ties() {
        let same = || vec![modifier("life", 2, 75.0, 70.0, 80.0), modifier("fire", 1, 43.0, 42.0, 44.0)];
        // Seu item: 1 suffix livre. Anúncios: 2 livres, 0 livres, 1 livre (prefixes não pesam).
        let (_, groups) = grouped(&[
            listing(1.0, "chaos", gloves(same(), Some((3, 1)))),
            listing(1.0, "chaos", gloves(same(), Some((0, 3)))),
            listing(1.0, "chaos", gloves(same(), Some((0, 2)))),
        ]);
        assert_eq!(group_of(&groups, 0), GroupKind::Better);
        assert_eq!(detail_of(&groups, 0).decided, Some(Criterion::Slots));
        assert_eq!(group_of(&groups, 1), GroupKind::Worse);
        assert_eq!(group_of(&groups, 2), GroupKind::Equal);
        assert_eq!(detail_of(&groups, 2).decided, None);
    }

    #[test]
    fn unknown_slots_count_as_zero_free() {
        let same = vec![modifier("life", 2, 75.0, 70.0, 80.0), modifier("fire", 1, 43.0, 42.0, 44.0)];
        let (_, groups) = grouped(&[listing(1.0, "chaos", gloves(same, None))]);
        assert_eq!(group_of(&groups, 0), GroupKind::Worse);
        assert_eq!(detail_of(&groups, 0).listing.free_valued, 0);
    }

    #[test]
    fn equality_uses_two_decimals() {
        // 0.501 vs 0.5 de percentil → 0.50 = 0.50.
        let close = gloves(
            vec![modifier("life", 2, 75.01, 70.0, 80.0), modifier("fire", 1, 43.0, 42.0, 44.0)],
            Some((1, 2)),
        );
        let (_, groups) = grouped(&[listing(1.0, "chaos", close)]);
        assert_eq!(group_of(&groups, 0), GroupKind::Equal);
    }

    #[test]
    fn fractured_and_explicit_are_the_same_stat() {
        let mut life = modifier("life", 2, 75.0, 70.0, 80.0);
        life.kind = ModifierKind::Fractured;
        life.trade_id = "fractured.life".into();
        let fire = modifier("fire", 1, 43.0, 42.0, 44.0);
        let (_, groups) = grouped(&[listing(1.0, "chaos", gloves(vec![life, fire], Some((1, 2))))]);
        assert_eq!(group_of(&groups, 0), GroupKind::Equal);
        assert_eq!(detail_of(&groups, 0).common, 2);
    }

    #[test]
    fn not_comparable_and_no_item_go_to_their_groups() {
        let unrelated = gloves(vec![modifier("other", 1, 1.0, 0.0, 2.0)], Some((1, 0)));
        let mut no_range = modifier("life", 1, 75.0, 70.0, 80.0);
        no_range.roll = Some(StatRoll { value: 75.0, min: None, max: None });
        let without_range = gloves(vec![no_range], Some((1, 0)));
        let mut no_tier = modifier("life", 1, 75.0, 70.0, 80.0);
        no_tier.tier = None;
        let without_tier = gloves(vec![no_tier], None);
        let implicit_only = {
            let mut m = modifier("life", 1, 75.0, 70.0, 80.0);
            m.kind = ModifierKind::Implicit;
            gloves(vec![m], Some((0, 0)))
        };
        let (_, groups) = grouped(&[
            listing(1.0, "chaos", unrelated),
            listing(1.0, "chaos", without_range),
            listing(1.0, "chaos", without_tier),
            listing(1.0, "chaos", None),
            listing(1.0, "chaos", implicit_only),
        ]);
        for i in [0, 1, 2, 4] {
            assert_eq!(group_of(&groups, i), GroupKind::NotComparable, "anúncio {i}");
        }
        assert_eq!(group_of(&groups, 3), GroupKind::NoItem);
    }

    #[test]
    fn groups_are_sorted_by_chaos_value() {
        let same = || gloves(vec![modifier("life", 2, 75.0, 70.0, 80.0), modifier("fire", 1, 43.0, 42.0, 44.0)], Some((1, 2)));
        let (_, groups) = grouped(&[
            listing(0.5, "divine", same()), // 100c
            listing(150.0, "chaos", same()),
            listing(3.0, "mystery", same()), // sem cotação: fim
            listing(20.0, "chaos", same()),
        ]);
        let equal = groups.iter().find(|g| g.kind == GroupKind::Equal).unwrap();
        let order: Vec<usize> = equal.entries.iter().map(|e| e.index).collect();
        assert_eq!(order, vec![3, 0, 1, 2]);
        assert_eq!(equal.entries[1].chaos, Some(100.0));
        assert_eq!(equal.entries[3].chaos, None);
    }

    #[test]
    fn single_list_cases() {
        let reason = |game, item: &Item| match compare_listings(game, item, &[], &rates()) {
            ListingComparison::Single { reason } => reason,
            ListingComparison::Grouped { .. } => panic!("esperava lista única"),
        };
        assert_eq!(reason(GameVersion::Poe2, &mine()), SingleReason::Poe2);

        let mut unique = mine();
        unique.rarity = Rarity::Unique;
        assert_eq!(reason(GameVersion::Poe1, &unique), SingleReason::Rarity);

        let mut jewel = mine();
        jewel.item_class = "Jewels".into();
        assert_eq!(reason(GameVersion::Poe1, &jewel), SingleReason::Class { class: "Jewels".into() });

        // Ctrl+C sem tiers: sem afixos nem tiers.
        let mut plain = mine();
        plain.affix_slots = None;
        assert_eq!(reason(GameVersion::Poe1, &plain), SingleReason::NoStatus);
        let mut no_tiers = mine();
        for m in &mut no_tiers.modifiers {
            m.tier = None;
        }
        assert_eq!(reason(GameVersion::Poe1, &no_tiers), SingleReason::NoStatus);
    }

    #[test]
    fn magic_items_have_one_slot_per_type() {
        let mut magic = mine();
        magic.rarity = Rarity::Magic;
        magic.affix_slots = Some(AffixSlots { prefixes: 1, suffixes: 0 });
        match compare_listings(GameVersion::Poe1, &magic, &[], &rates()) {
            ListingComparison::Grouped { you, .. } => assert_eq!(you.free_valued, 1),
            ListingComparison::Single { reason } => panic!("{reason:?}"),
        }
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
