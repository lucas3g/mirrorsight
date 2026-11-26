use serde::{Deserialize, Serialize};

/// Raridade conforme exibida no texto copiado do jogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
    Unique,
    Gem,
    Currency,
    DivinationCard,
    Other,
}

/// Origem de um modificador. Cada variante corresponde a um grupo de stats da trade API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModifierKind {
    Implicit,
    Explicit,
    Crafted,
    Fractured,
    Enchant,
    /// PoE2: runas/soul cores engastadas.
    Rune,
    /// PoE2: mods "desecrated".
    Desecrated,
    Pseudo,
    /// Propriedade da base (armadura, ES, DPS…): vira filtro de `armour_filters`/`weapon_filters`,
    /// não de `stats`. O `trade_id` é `property.<id do filtro>` (ex.: `property.es`).
    Property,
}

/// Prefixo do `trade_id` das propriedades.
pub const PROPERTY_PREFIX: &str = "property.";

/// Influências de itens (PoE1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Influence {
    Shaper,
    Elder,
    Crusader,
    Hunter,
    Redeemer,
    Warlord,
    Searing,
    Eater,
}

/// Valor rolado e, quando conhecido, a faixa do tier.
/// Para stats com vários números (ex.: "Adds # to #") usa-se a média, como a trade API.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StatRoll {
    pub value: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// Tipo de afixo de um mod explícito: cada raridade tem um limite de prefixos e sufixos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Affix {
    Prefix,
    Suffix,
}

/// Slots de afixo **ocupados** no item (um mod híbrido ocupa um slot só).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffixSlots {
    pub prefixes: u32,
    pub suffixes: u32,
}

impl AffixSlots {
    pub const fn used(&self, affix: Affix) -> u32 {
        match affix {
            Affix::Prefix => self.prefixes,
            Affix::Suffix => self.suffixes,
        }
    }

    pub fn add(&mut self, affix: Affix) {
        match affix {
            Affix::Prefix => self.prefixes += 1,
            Affix::Suffix => self.suffixes += 1,
        }
    }
}

/// Mod reconhecido no banco de stats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    /// ID usado pela trade API (ex.: `explicit.stat_3299347043`).
    pub trade_id: String,
    /// Texto original da linha (sem marcadores como "(crafted)").
    pub text: String,
    pub kind: ModifierKind,
    pub roll: Option<StatRoll>,
    /// Tier do mod, disponível quando o item foi copiado no modo avançado (Ctrl+Alt+C).
    #[serde(default)]
    pub tier: Option<u32>,
    /// Prefixo ou sufixo, quando a origem informa (Ctrl+Alt+C ou `extended.mods` da trade API).
    #[serde(default)]
    pub affix: Option<Affix>,
}

/// Item parseado a partir do texto do clipboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub item_class: String,
    pub rarity: Rarity,
    pub name: Option<String>,
    pub base_type: String,
    pub item_level: Option<u32>,
    pub quality: Option<u32>,
    pub corrupted: bool,
    pub identified: bool,
    /// Sockets no formato do jogo, ex.: "B-B-R G".
    #[serde(default)]
    pub sockets: Option<String>,
    #[serde(default)]
    pub influences: Vec<Influence>,
    pub modifiers: Vec<Modifier>,
    /// Defesas e dano da base, já com qualidade e mods locais (tipo [`ModifierKind::Property`]).
    #[serde(default)]
    pub properties: Vec<Modifier>,
    /// Linhas que o parser não reconheceu (úteis para debug/telemetria local).
    pub unknown_lines: Vec<String>,
    /// Único Foulborn (mutado). O nome fica sem o prefixo "Foulborn ", como na trade API.
    #[serde(default)]
    pub foulborn: bool,
    /// Discriminador da base na trade API (gemas transfiguradas: "alt_x", "alt_y"…).
    #[serde(default)]
    pub discriminator: Option<String>,
    /// `type` da trade API quando difere do `base_type` exibido
    /// (mapas blighted: "Blighted Map (Strand)" → "10021").
    #[serde(default)]
    pub trade_type: Option<String>,
    /// Tier do mapa ("Map (Tier 16)" → 16).
    #[serde(default)]
    pub map_tier: Option<u32>,
    /// Slots de prefixo/sufixo ocupados; só presente quando a origem conhece os afixos.
    #[serde(default)]
    pub affix_slots: Option<AffixSlots>,
}

impl Item {
    /// Maior grupo de sockets ligados ("B-B-R G" → 3).
    pub fn max_links(&self) -> u32 {
        self.sockets
            .as_deref()
            .map(|s| {
                s.split_whitespace()
                    .map(|group| u32::try_from(group.split('-').count()).unwrap_or(u32::MAX))
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_largest_link_group() {
        let item = Item {
            item_class: "Body Armours".into(),
            rarity: Rarity::Rare,
            name: None,
            base_type: "Vaal Regalia".into(),
            item_level: None,
            quality: None,
            corrupted: false,
            identified: true,
            sockets: Some("B-B-R G-G-G-G".into()),
            influences: vec![],
            modifiers: vec![],
            properties: Vec::new(),
            unknown_lines: vec![],
            foulborn: false,
            discriminator: None,
            trade_type: None,
            map_tier: None,
            affix_slots: None,
        };
        assert_eq!(item.max_links(), 4);
    }
}
