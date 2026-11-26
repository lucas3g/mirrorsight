//! Formatos de resposta da trade API e conversão para o domínio.
//! Só os campos que usamos; o resto é ignorado pelo serde.

use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Deserialize;

use mirrorsight_domain::item::{Affix, AffixSlots, Item, ModifierKind};
use mirrorsight_domain::ports::ItemParser;
use mirrorsight_domain::price::{Currency, Listing, Price};
use mirrorsight_domain::GameVersion;

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub id: String,
    #[serde(default)]
    pub result: Vec<String>,
    #[serde(default)]
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct FetchResponse {
    #[serde(default)]
    pub result: Vec<Option<FetchEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct FetchEntry {
    pub listing: ListingDto,
    pub item: ItemDto,
}

#[derive(Debug, Deserialize)]
pub struct ListingDto {
    pub indexed: String,
    pub price: Option<PriceDto>,
    pub account: AccountDto,
    #[serde(default)]
    pub whisper: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PriceDto {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountDto {
    pub name: String,
    /// Presente (com a liga) quando o vendedor está online.
    #[serde(default)]
    pub online: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ItemDto {
    #[serde(rename = "stackSize", default)]
    pub stack_size: Option<u32>,
    #[serde(default)]
    pub extended: Option<ExtendedDto>,
    /// Arte do item no poecdn (raros/mágicos: arte da base).
    #[serde(default)]
    pub icon: Option<String>,
    /// Linhas de mod explícitas (inclui fractured e crafted, pelo `domain`). No formato atual
    /// cada linha é um objeto com `hash` e os mods de origem (tier P/S e faixa); lidas uma a uma
    /// em [`ModLineDto`] para que uma linha inesperada não derrube o fetch.
    #[serde(rename = "explicitMods", default)]
    pub explicit_mods: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ExtendedDto {
    /// Texto do item em base64, no mesmo formato do Ctrl+C do jogo.
    #[serde(default)]
    pub text: Option<String>,
}

/// `{"domain":"explicit","hash":"stat.explicit.stat_3299347043","mods":[{"name":"Robust","tier":"P4",
/// "magnitudes":[{"min":"70","max":"84"}]}]}`
#[derive(Debug, Deserialize)]
pub struct ModLineDto {
    pub hash: String,
    #[serde(default)]
    pub domain: Option<String>,
    /// Texto exibido da linha ("+80 to maximum Life").
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mods: Vec<TradeModDto>,
}

#[derive(Debug, Deserialize)]
pub struct TradeModDto {
    #[serde(default)]
    pub name: Option<String>,
    /// "P2"/"S1" (prefixo/sufixo + tier); mods de bancada vêm com "R3" (rank).
    #[serde(default)]
    pub tier: Option<String>,
    /// Nível do mod; junto com nome e tier identifica as linhas de um mesmo mod híbrido.
    #[serde(default)]
    pub level: Option<u32>,
    #[serde(default)]
    pub magnitudes: Option<Vec<MagnitudeDto>>,
}

#[derive(Debug, Deserialize)]
pub struct MagnitudeDto {
    /// Stat da magnitude. No formato atual (nov/2026) não vem: cada linha já traz só as
    /// magnitudes do próprio stat. Quando vier, filtra as de outros stats do híbrido.
    #[serde(default)]
    pub hash: Option<String>,
    /// A API manda como string ("70"); aceita número também.
    #[serde(default, deserialize_with = "flexible_number")]
    pub min: Option<f64>,
    #[serde(default, deserialize_with = "flexible_number")]
    pub max: Option<f64>,
}

fn flexible_number<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Option<f64>, D::Error> {
    Ok(match serde_json::Value::deserialize(de)? {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LeaguesResponse {
    pub result: Vec<LeagueDto>,
}

#[derive(Debug, Deserialize)]
pub struct LeagueDto {
    pub id: String,
    #[serde(default)]
    pub realm: Option<String>,
}

impl FetchEntry {
    /// Listagem do domínio. `None` para itens sem preço (não servem ao price check).
    /// O item listado é reconstruído pelo mesmo parser do clipboard, a partir do texto em base64.
    pub fn into_listing(self, game: GameVersion, parser: &dyn ItemParser) -> Option<Listing> {
        let price = self.listing.price?;
        // Só imagens do CDN oficial, em https (vão direto para um <img>).
        let icon = self.item.icon.filter(|url| crate::ninja::is_https_poecdn(url));
        let lines: Vec<ModLineDto> = self
            .item
            .explicit_mods
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        let item = self
            .item
            .extended
            .and_then(|e| e.text)
            .and_then(|b64| STANDARD.decode(b64).ok())
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| parser.parse(game, &text).ok())
            .map(|mut item| {
                enrich(&mut item, &lines);
                item
            });
        Some(Listing {
            price: Price {
                amount: price.amount,
                currency: Currency(price.currency),
            },
            stack_size: self.item.stack_size,
            account: self.listing.account.name,
            online: self.listing.account.online.is_some_and(|v| !v.is_null()),
            indexed_at: self.listing.indexed,
            item,
            whisper: self.listing.whisper,
            icon,
        })
    }
}

impl TradeModDto {
    /// "P2" → (Prefix, 2); "S0" → (Suffix, 0). Rank de bancada ("R3") não é tier: o tipo
    /// sai do nome (sufixos do PoE começam com "of ").
    fn affix_and_tier(&self) -> (Option<Affix>, Option<u32>) {
        let tier = self.tier.as_deref().unwrap_or_default();
        let number = || tier.get(1..).and_then(|n| n.parse().ok());
        match tier.chars().next() {
            Some('P') => (Some(Affix::Prefix), number()),
            Some('S') => (Some(Affix::Suffix), number()),
            Some('R') => (self.affix_from_name(), None),
            _ => (None, None),
        }
    }

    fn affix_from_name(&self) -> Option<Affix> {
        let name = self.name.as_deref()?.trim();
        if name.is_empty() {
            None
        } else if name.starts_with("of ") {
            Some(Affix::Suffix)
        } else {
            Some(Affix::Prefix)
        }
    }

    /// Faixa do tier para o stat `stat` (ex.: "stat_3299347043"); stats com vários números
    /// ("Adds # to #") usam a média, como o roll. Se as magnitudes trazem `hash`, só entram as
    /// do próprio stat (num híbrido armadura + vida, a faixa da armadura não vaza para a vida).
    #[allow(clippy::cast_precision_loss)]
    fn range(&self, stat: &str) -> Option<(f64, f64)> {
        let all = self.magnitudes.as_deref()?;
        let hashed = all.iter().any(|m| m.hash.is_some());
        let mags: Vec<&MagnitudeDto> = all
            .iter()
            .filter(|m| !hashed || m.hash.as_deref().and_then(|h| h.rsplit('.').next()) == Some(stat))
            .collect();
        let mins: Vec<f64> = mags.iter().filter_map(|m| m.min).collect();
        let maxs: Vec<f64> = mags.iter().filter_map(|m| m.max).collect();
        if mins.is_empty() || mins.len() != maxs.len() {
            return None;
        }
        let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        Some((avg(&mins), avg(&maxs)))
    }
}

/// Completa o item parseado do texto (formato Ctrl+C, sem tiers) com o tier, o tipo de afixo
/// e a faixa de cada mod, e conta os slots ocupados. Linha com mais de um mod de origem
/// (dois mods somando no mesmo stat) fica sem faixa.
fn enrich(item: &mut Item, lines: &[ModLineDto]) {
    const DOMAINS: &[&str] = &["explicit", "fractured", "crafted"];
    let mut enriched = vec![false; item.modifiers.len()];
    // Chave de cada mod → afixo. Um híbrido aparece em várias linhas com o mesmo nome, tier e
    // nível, e ocupa um slot só. Mods de bancada não se juntam: nomes como "Upgraded"/"of Craft"
    // se repetem entre mods diferentes (multimod), então cada um conta um slot.
    let mut seen: HashMap<String, Affix> = HashMap::new();

    for (line_index, line) in lines.iter().enumerate() {
        if line.domain.as_deref().is_some_and(|d| !DOMAINS.contains(&d)) {
            continue;
        }
        let crafted = line.domain.as_deref() == Some("crafted");
        for (mod_index, m) in line.mods.iter().enumerate() {
            let (Some(affix), Some(name)) = (m.affix_and_tier().0, m.name.as_deref()) else { continue };
            let key = if crafted {
                format!("crafted#{line_index}#{mod_index}")
            } else {
                format!("{name}|{}|{:?}|{affix:?}", m.tier.as_deref().unwrap_or_default(), m.level)
            };
            seen.insert(key, affix);
        }
        let [source] = line.mods.as_slice() else { continue };
        // "stat.fractured.stat_1" casa com "fractured.stat_1" e também com "explicit.stat_1"
        // (o parser cai para explicit quando o stat não existe no grupo fractured/crafted).
        let stat = line.hash.rsplit('.').next().unwrap_or(&line.hash);
        let free = |i: usize| {
            !enriched[i]
                && matches!(
                    item.modifiers[i].kind,
                    ModifierKind::Explicit | ModifierKind::Fractured | ModifierKind::Crafted
                )
        };
        // Se o parser escolheu outra variante do stat (ex.: "(Local)"), casa pelo texto da linha.
        let Some(pos) = (0..item.modifiers.len())
            .find(|&i| free(i) && item.modifiers[i].trade_id.rsplit('.').next() == Some(stat))
            .or_else(|| {
                let text = line.description.as_deref()?;
                (0..item.modifiers.len()).find(|&i| free(i) && item.modifiers[i].text == text)
            })
        else {
            continue;
        };
        enriched[pos] = true;
        let modifier = &mut item.modifiers[pos];
        let (affix, tier) = source.affix_and_tier();
        modifier.affix = modifier.affix.or(affix);
        modifier.tier = modifier.tier.or(tier);
        if let (Some(roll), Some((min, max))) = (modifier.roll.as_mut(), source.range(stat)) {
            // "reduced" vira "increased" com valor negativo; a faixa da API vem positiva.
            let (min, max) = if roll.value < 0.0 && min >= 0.0 { (-max, -min) } else { (min, max) };
            roll.min = roll.min.or(Some(min));
            roll.max = roll.max.or(Some(max));
        }
    }

    if !seen.is_empty() {
        let mut slots = AffixSlots::default();
        for affix in seen.values() {
            slots.add(*affix);
        }
        item.affix_slots = Some(slots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirrorsight_domain::item::{Item, Rarity};
    use mirrorsight_domain::ports::ItemParseError;

    /// Parser falso: só confirma que recebeu o texto decodificado.
    struct EchoParser;
    impl ItemParser for EchoParser {
        fn parse(&self, _game: GameVersion, text: &str) -> Result<Item, ItemParseError> {
            Ok(Item {
                item_class: "Gloves".into(),
                rarity: Rarity::Rare,
                name: None,
                base_type: text.lines().next().unwrap_or_default().into(),
                item_level: None,
                quality: None,
                corrupted: false,
                identified: true,
                sockets: None,
                influences: vec![],
                modifiers: vec![],
                properties: Vec::new(),
                unknown_lines: vec![],
                foulborn: false,
                discriminator: None,
                trade_type: None,
                map_tier: None,
                affix_slots: None,
            })
        }
    }

    const FETCH: &str = r#"{"result":[
      {"id":"a","listing":{"indexed":"2026-11-25T02:47:18Z","price":{"type":"~b/o","amount":12,"currency":"chaos"},
        "account":{"name":"Vendedor#1","online":{"league":"Covens"}},"whisper":"@Fulano Hi"},
       "item":{"extended":{"text":"U29yY2VyZXIgR2xvdmVz"},"icon":"https://web.poecdn.com/gen/image/gloves.png"}},
      {"id":"b","listing":{"indexed":"2026-11-24T10:00:00Z","price":null,
        "account":{"name":"SemPreco","online":null}},"item":{}},
      null,
      {"id":"c","listing":{"indexed":"2026-11-24T10:00:00Z","price":{"amount":0.5,"currency":"divine"},
        "account":{"name":"Offline"}},"item":{"stackSize":3,"icon":"http://evil.example/x.png"}}
    ]}"#;

    #[test]
    fn maps_listings_skipping_unpriced_and_null_entries() {
        let resp: FetchResponse = serde_json::from_str(FETCH).unwrap();
        let listings: Vec<Listing> = resp
            .result
            .into_iter()
            .flatten()
            .filter_map(|e| e.into_listing(GameVersion::Poe1, &EchoParser))
            .collect();

        assert_eq!(listings.len(), 2);
        assert_eq!(listings[0].price.amount, 12.0);
        assert!(listings[0].online);
        assert_eq!(listings[0].item.as_ref().unwrap().base_type, "Sorcerer Gloves");
        assert_eq!(listings[0].whisper.as_deref(), Some("@Fulano Hi"));
        assert!(!listings[1].online);
        assert_eq!(listings[1].price.currency.0, "divine");
        assert_eq!(listings[1].stack_size, Some(3));
        assert!(listings[1].item.is_none());
        assert_eq!(listings[0].icon.as_deref(), Some("https://web.poecdn.com/gen/image/gloves.png"));
        // Fora do poecdn/https é descartada.
        assert!(listings[1].icon.is_none());
    }

    const GLOVES_TEXT: &str = "Item Class: Gloves\nRarity: Rare\nDoom Grip\nSorcerer Gloves\n--------\nItem Level: 86\n--------\n\
14% increased Attack Speed (fractured)\n+122 to maximum Life\nAdds 6 to 16 Cold Damage to Attacks\n\
+17% to Cold and Lightning Resistances (crafted)\n";

    /// Formato real do fetch (PoE1, nov/2026): `explicitMods` com objetos, incluindo fractured e crafted.
    const MOD_LINES: &str = r#"[
      {"description":"14% increased Attack Speed","domain":"fractured","hash":"stat.fractured.stat_681332047",
       "mods":[{"level":76,"magnitudes":[{"max":"16","min":"14"}],"name":"of Grandmastery","tier":"S1"}]},
      {"description":"+122 to maximum Life","domain":"explicit","hash":"stat.explicit.stat_3299347043",
       "mods":[{"level":54,"magnitudes":[{"max":"129","min":"115"}],"name":"Athlete's","tier":"P1"}]},
      {"description":"Adds 6 to 16 Cold Damage to Attacks","domain":"explicit","hash":"stat.explicit.stat_4067062424",
       "mods":[{"level":29,"magnitudes":[{"max":"9","min":"6"},{"max":16,"min":13}],"name":"Frigid","tier":"P3"}]},
      {"description":"+17% to Cold and Lightning Resistances","domain":"crafted","hash":"stat.crafted.stat_4277795662",
       "mods":[{"level":75,"magnitudes":[{"max":"20","min":"17"}],"name":"of Craft","tier":"R3"}]},
      "linha no formato antigo"
    ]"#;

    fn fetched(mods: &str) -> Listing {
        let json = format!(
            r#"{{"listing":{{"indexed":"x","price":{{"amount":1,"currency":"chaos"}},"account":{{"name":"a"}}}},
                "item":{{"extended":{{"text":"{}"}},"explicitMods":{mods}}}}}"#,
            STANDARD.encode(GLOVES_TEXT)
        );
        let entry: FetchEntry = serde_json::from_str(&json).unwrap();
        entry.into_listing(GameVersion::Poe1, &mirrorsight_parser::ClipboardItemParser::default()).unwrap()
    }

    #[test]
    fn enriches_mods_with_tier_affix_and_range() {
        let item = fetched(MOD_LINES).item.unwrap();
        let by_kind = |kind| item.modifiers.iter().find(|m| m.kind == kind).unwrap();
        let speed = by_kind(ModifierKind::Fractured);
        assert_eq!((speed.affix, speed.tier), (Some(Affix::Suffix), Some(1)));
        let roll = speed.roll.unwrap();
        assert_eq!((roll.min, roll.max), (Some(14.0), Some(16.0)));

        let life = item.modifiers.iter().find(|m| m.trade_id == "explicit.stat_3299347043").unwrap();
        assert_eq!((life.affix, life.tier), (Some(Affix::Prefix), Some(1)));

        // "Adds # to #": média das magnitudes (6–9 e 13–16 → 9.5–12.5), número ou string.
        let adds = item.modifiers.iter().find(|m| m.trade_id == "explicit.stat_4067062424").unwrap();
        let roll = adds.roll.unwrap();
        assert_eq!((roll.value, roll.min, roll.max), (11.0, Some(9.5), Some(12.5)));
        assert_eq!(adds.tier, Some(3));

        // Bancada: rank não é tier; o sufixo sai do nome "of Craft".
        let crafted = by_kind(ModifierKind::Crafted);
        assert_eq!((crafted.affix, crafted.tier), (Some(Affix::Suffix), None));

        assert_eq!(item.affix_slots, Some(AffixSlots { prefixes: 2, suffixes: 2 }));
    }

    #[test]
    fn line_with_two_source_mods_gets_no_range() {
        let lines = r#"[{"domain":"explicit","hash":"stat.explicit.stat_3299347043","mods":[
            {"name":"Athlete's","tier":"P1","magnitudes":[{"min":"115","max":"129"}]},
            {"name":"of the Whelpling","tier":"S9","magnitudes":[{"min":"3","max":"5"}]}]}]"#;
        let item = fetched(lines).item.unwrap();
        let life = item.modifiers.iter().find(|m| m.trade_id == "explicit.stat_3299347043").unwrap();
        assert_eq!(life.tier, None);
        assert_eq!(life.roll.unwrap().min, None);
        // Os dois mods ainda ocupam slot.
        assert_eq!(item.affix_slots, Some(AffixSlots { prefixes: 1, suffixes: 1 }));
    }

    #[test]
    fn hybrid_range_uses_only_magnitudes_of_its_own_stat() {
        // Híbrido armadura + vida com `hash` nas magnitudes: a linha de vida ignora a armadura.
        let lines = r#"[{"domain":"explicit","hash":"stat.explicit.stat_3299347043","mods":[
            {"name":"Rhino's","tier":"P3","level":40,"magnitudes":[
              {"hash":"explicit.stat_3484657501","min":"300","max":"400"},
              {"hash":"explicit.stat_3299347043","min":"110","max":"130"}]}]}]"#;
        let item = fetched(lines).item.unwrap();
        let life = item.modifiers.iter().find(|m| m.trade_id == "explicit.stat_3299347043").unwrap();
        let roll = life.roll.unwrap();
        assert_eq!((roll.min, roll.max), (Some(110.0), Some(130.0)));
    }

    #[test]
    fn hybrid_counts_one_slot_and_crafted_mods_with_same_name_count_apart() {
        // Formato real: o híbrido "Abbot's" aparece em duas linhas (cada uma com as próprias
        // magnitudes); os dois "Upgraded" de bancada são mods diferentes.
        let lines = r#"[
          {"domain":"explicit","hash":"stat.explicit.stat_4052037485",
           "mods":[{"level":62,"magnitudes":[{"max":"25","min":"16"}],"name":"Abbot's","tier":"P2"}]},
          {"domain":"explicit","hash":"stat.explicit.stat_3299347043",
           "mods":[{"level":62,"magnitudes":[{"max":"33","min":"29"}],"name":"Abbot's","tier":"P2"}]},
          {"domain":"crafted","hash":"stat.crafted.stat_1050105434",
           "mods":[{"level":60,"magnitudes":[{"max":"54","min":"45"}],"name":"Upgraded","tier":"R3"}]},
          {"domain":"crafted","hash":"stat.crafted.stat_3240769289",
           "mods":[{"level":60,"magnitudes":[{"max":"25","min":"20"}],"name":"Upgraded","tier":"R3"}]}
        ]"#;
        let item = fetched(lines).item.unwrap();
        assert_eq!(item.affix_slots, Some(AffixSlots { prefixes: 3, suffixes: 0 }));
    }

    #[test]
    fn without_mod_objects_nothing_is_known() {
        let item = fetched("null").item.unwrap();
        assert!(item.modifiers.iter().all(|m| m.tier.is_none() && m.affix.is_none()));
        assert_eq!(item.affix_slots, None);
    }
}
