//! Formatos de resposta da trade API e conversão para o domínio.
//! Só os campos que usamos; o resto é ignorado pelo serde.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Deserialize;

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
}

#[derive(Debug, Deserialize)]
pub struct ExtendedDto {
    /// Texto do item em base64, no mesmo formato do Ctrl+C do jogo.
    #[serde(default)]
    pub text: Option<String>,
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
        let item = self
            .item
            .extended
            .and_then(|e| e.text)
            .and_then(|b64| STANDARD.decode(b64).ok())
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| parser.parse(game, &text).ok());
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
}
