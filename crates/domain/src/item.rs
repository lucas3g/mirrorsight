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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModifierKind {
    Implicit,
    #[default]
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

/// Nível de um implícito eldritch (Searing Exarch / Eater of Worlds), do cabeçalho avançado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EldritchLevel {
    Lesser,
    Greater,
    Grand,
    Exceptional,
    Exquisite,
    Perfect,
}

impl EldritchLevel {
    pub const ALL: [Self; 6] =
        [Self::Lesser, Self::Greater, Self::Grand, Self::Exceptional, Self::Exquisite, Self::Perfect];

    /// 1 (Lesser) a 6 (Perfect).
    pub fn rank(self) -> u32 {
        Self::ALL.iter().position(|l| *l == self).map_or(1, |p| u32::try_from(p + 1).unwrap_or(1))
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| format!("{l:?}").eq_ignore_ascii_case(name.trim()))
    }
}

/// De onde o mod vem, quando o cabeçalho avançado (ou a trade) informa algo além do tipo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModOrigin {
    /// Implícito do Searing Exarch (`searing = true`) ou do Eater of Worlds.
    Eldritch { searing: bool, level: EldritchLevel },
    /// Implícito de corrupção (Vaal Orb).
    Corruption,
    Veiled,
    /// Mod de influência (Shaper, Elder, conquistadores).
    Influence { influence: Influence },
    /// Mod de único.
    Unique,
}

/// Mod reconhecido no banco de stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    /// Rank de bancada ("(Rank: 3)" / "R3" na trade), só em mods crafted.
    #[serde(default)]
    pub rank: Option<u32>,
    /// Nome do afixo ("Fecund", "of the Lightning"): identifica o mod exato na base de tiers.
    #[serde(default)]
    pub mod_name: Option<String>,
    #[serde(default)]
    pub origin: Option<ModOrigin>,
}

/// Marcas do item que afetam o potencial de crafting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemFlags {
    pub fractured: bool,
    pub synthesised: bool,
    pub mirrored: bool,
    pub split: bool,
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
    #[serde(default)]
    pub flags: ItemFlags,
}

impl Item {
    /// Número de sockets ("B-B-R G" → 4).
    pub fn socket_count(&self) -> u32 {
        self.sockets
            .as_deref()
            .map(|s| s.split(|c: char| c == '-' || c.is_whitespace()).filter(|g| !g.is_empty()).count())
            .map_or(0, |n| u32::try_from(n).unwrap_or(u32::MAX))
    }

    /// Sockets brancos (aceitam qualquer gema).
    pub fn white_sockets(&self) -> u32 {
        self.sockets
            .as_deref()
            .map(|s| s.chars().filter(|c| *c == 'W').count())
            .map_or(0, |n| u32::try_from(n).unwrap_or(u32::MAX))
    }

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
            flags: ItemFlags::default(),
        };
        assert_eq!(item.max_links(), 4);
        assert_eq!(item.socket_count(), 7);
        assert_eq!(item.white_sockets(), 0);
    }

    #[test]
    fn eldritch_level_rank() {
        assert_eq!(EldritchLevel::from_name("Exquisite").map(EldritchLevel::rank), Some(5));
        assert_eq!(EldritchLevel::Lesser.rank(), 1);
        assert_eq!(EldritchLevel::from_name("Nope"), None);
    }
}
