use mirrorsight_domain::item::{Item, Rarity};
use mirrorsight_domain::ports::ItemParseError;

/// Prefixos localizados de "Item Class:". Também usados pelo adapter de clipboard
/// para reconhecer que o texto copiado é um item.
/// TODO(fase 3): validar cada string localizada contra o cliente real do jogo.
pub const ITEM_CLASS_PREFIXES: &[&str] = &[
    "Item Class: ",      // en
    "Classe do Item: ",  // pt
    "Класс предмета: ",  // ru
    "Classe d'objet: ",  // fr
    "Gegenstandsklasse: ", // de
    "Clase de objeto: ", // es
    "ชนิดไอเทม: ",       // th
    "아이템 종류: ",      // ko
    "物品種類: ",        // zh-Hant
    "物品类别: ",        // zh-Hans
];

const RARITY_PREFIXES: &[&str] = &["Rarity: ", "Raridade: "];

/// `true` se o texto começa com algum prefixo conhecido de item.
pub fn looks_like_item(text: &str) -> bool {
    ITEM_CLASS_PREFIXES.iter().any(|p| text.starts_with(p))
}

fn strip_any<'a>(line: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes.iter().find_map(|p| line.strip_prefix(p))
}

fn parse_rarity(raw: &str) -> Rarity {
    match raw {
        "Normal" => Rarity::Normal,
        "Magic" | "Mágico" => Rarity::Magic,
        "Rare" | "Raro" => Rarity::Rare,
        "Unique" | "Único" => Rarity::Unique,
        "Gem" | "Gema" => Rarity::Gem,
        "Currency" | "Moeda" => Rarity::Currency,
        "Divination Card" => Rarity::DivinationCard,
        _ => Rarity::Other,
    }
}

/// Lê a primeira seção: `Item Class`, `Rarity` (opcional), nome (opcional) e base.
/// Alguns itens não têm raridade no texto (ex.: Viscera da Covens: só classe e nome).
pub fn parse(section: &[&str]) -> Result<Item, ItemParseError> {
    let (first, rest) = section.split_first().ok_or(ItemParseError::NotAnItem)?;
    let item_class = strip_any(first, ITEM_CLASS_PREFIXES).ok_or(ItemParseError::NotAnItem)?;
    let (rarity, rest) = match rest.split_first() {
        Some((line, after)) => match strip_any(line, RARITY_PREFIXES) {
            Some(raw) => (parse_rarity(raw), after),
            None => (Rarity::Other, rest),
        },
        None => return Err(ItemParseError::NotAnItem),
    };
    let (name, base_type) = match rest {
        [name, base] => (Some((*name).to_owned()), clean_base(base)),
        [base] => (None, clean_base(base)),
        _ => return Err(ItemParseError::Unknown(section.join(" / "))),
    };

    // A trade API conhece só "Aegis Aurora"; o Foulborn vira filtro à parte.
    let mut foulborn = false;
    let name = name.map(|n| match n.strip_prefix(FOULBORN_PREFIX) {
        Some(stripped) if rarity == Rarity::Unique => {
            foulborn = true;
            stripped.to_owned()
        }
        _ => n,
    });

    Ok(Item {
        item_class: item_class.to_owned(),
        rarity,
        name,
        base_type,
        item_level: None,
        quality: None,
        corrupted: false,
        identified: true,
        sockets: None,
        influences: Vec::new(),
        modifiers: Vec::new(),
        properties: Vec::new(),
        unknown_lines: Vec::new(),
        foulborn,
        discriminator: None,
        trade_type: None,
        map_tier: None,
        affix_slots: None,
    })
}

const FOULBORN_PREFIX: &str = "Foulborn ";

/// Prefixos que o jogo põe na linha da base, mas que não fazem parte do `type` da trade API.
const BASE_PREFIXES: &[&str] = &["Superior ", "Synthesised "];

/// "Superior Synthesised Champion Kite Shield" → "Champion Kite Shield".
fn clean_base(base: &str) -> String {
    let mut base = base;
    while let Some(stripped) = BASE_PREFIXES.iter().find_map(|p| base.strip_prefix(p)) {
        base = stripped;
    }
    base.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foulborn_unique_loses_prefix_and_gets_flag() {
        let item = parse(&["Item Class: Shields", "Rarity: Unique", "Foulborn Aegis Aurora", "Champion Kite Shield"])
            .unwrap();
        assert_eq!(item.name.as_deref(), Some("Aegis Aurora"));
        assert!(item.foulborn);
    }

    #[test]
    fn non_unique_keeps_name() {
        let item = parse(&["Item Class: Shields", "Rarity: Rare", "Foulborn Ward", "Champion Kite Shield"]).unwrap();
        assert_eq!(item.name.as_deref(), Some("Foulborn Ward"));
        assert!(!item.foulborn);
    }

    #[test]
    fn rarity_line_is_optional() {
        let item = parse(&["Item Class: Viscera", "Greater Carved Haunches"]).unwrap();
        assert_eq!(item.item_class, "Viscera");
        assert_eq!(item.rarity, Rarity::Other);
        assert_eq!(item.name, None);
        assert_eq!(item.base_type, "Greater Carved Haunches");
    }

    #[test]
    fn strips_superior_and_synthesised_from_base() {
        let item = parse(&["Item Class: Shields", "Rarity: Normal", "Superior Champion Kite Shield"]).unwrap();
        assert_eq!(item.base_type, "Champion Kite Shield");
        let item = parse(&["Item Class: Rings", "Rarity: Rare", "Doom Loop", "Synthesised Amethyst Ring"]).unwrap();
        assert_eq!(item.base_type, "Amethyst Ring");
    }
}
