use std::sync::Arc;

use serde::Serialize;

use mirrorsight_domain::compare::{compare_listings, ListingComparison};
use mirrorsight_domain::item::{Item, Rarity};
use mirrorsight_domain::ports::{ItemParser, PortError, PriceIndexPort, TradeApiPort};
use mirrorsight_domain::price::{summarize, CurrencyRates, IndexedPrice, PriceCheckResult, PriceSummary};
use mirrorsight_domain::search::SearchFilters;
use mirrorsight_domain::settings::Settings;
use mirrorsight_domain::League;

use crate::AppError;

#[derive(Debug, Serialize)]
pub struct PriceCheckOutput {
    pub item: Item,
    /// Filtros efetivamente usados (a UI os edita e reenvia).
    pub filters: SearchFilters,
    /// Preço de referência (poe.ninja), quando existir.
    pub indexed: Option<IndexedPrice>,
    /// Listagens do trade oficial.
    pub trade: Option<PriceCheckResult>,
    /// Estatística das listagens em chaos.
    pub summary: Option<PriceSummary>,
    /// Motivo da ausência de listagens quando a trade falhou e o índice respondeu.
    pub trade_error: Option<AppError>,
    /// Imagem do item: do índice, de uma listagem da mesma base ou a arte genérica da base.
    pub icon: Option<String>,
    /// Listagens agrupadas por status comparado ao item do usuário (ou o motivo da lista única).
    pub comparison: Option<ListingComparison>,
}

/// Quanto o price check espera pela arte da base (o download roda em segundo plano mesmo assim).
const BASE_ICON_WAIT: std::time::Duration = std::time::Duration::from_secs(3);

/// Parseia o texto do item e consulta índice + trade em paralelo.
pub struct PriceCheckItem {
    parser: Arc<dyn ItemParser>,
    trade: Arc<dyn TradeApiPort>,
    index: Arc<dyn PriceIndexPort>,
}

impl PriceCheckItem {
    pub fn new(
        parser: Arc<dyn ItemParser>,
        trade: Arc<dyn TradeApiPort>,
        index: Arc<dyn PriceIndexPort>,
    ) -> Self {
        Self {
            parser,
            trade,
            index,
        }
    }

    /// `filters = None` → usa os filtros padrão derivados do item e das configurações.
    pub async fn execute(
        &self,
        settings: &Settings,
        text: &str,
        filters: Option<SearchFilters>,
    ) -> Result<PriceCheckOutput, AppError> {
        let league = League::new(settings.league.clone(), settings.game);
        let item = self.parser.parse(league.game, text)?;
        let filters = filters.unwrap_or_else(|| SearchFilters::defaults_for(&item, settings));

        // A arte da base corre junto (raros/mágicos/normais): só é usada se nada melhor aparecer.
        let wants_base_icon = matches!(item.rarity, Rarity::Rare | Rarity::Magic | Rarity::Normal);
        let base_icon = async {
            if !wants_base_icon {
                return None;
            }
            match tokio::time::timeout(BASE_ICON_WAIT, self.index.base_icon(&league, &item.base_type)).await {
                Ok(Ok(icon)) => icon,
                Ok(Err(err)) => {
                    tracing::warn!(%err, "falha ao obter a imagem da base");
                    None
                }
                Err(_) => None,
            }
        };
        let (indexed, rates, trade, base_icon) = tokio::join!(
            self.index.lookup(&league, &item),
            self.index.currency_rates(&league),
            self.trade.price_check(&league, &item, &filters),
            base_icon
        );

        // O índice é "nice to have": falha nele não derruba o price check.
        let indexed = indexed.unwrap_or_else(|err| {
            tracing::warn!(%err, "falha ao consultar índice de preços");
            None
        });
        let rates = rates.unwrap_or_else(|err| {
            tracing::warn!(%err, "falha ao obter cotações; usando só chaos");
            CurrencyRates::default()
        });
        // Reserva do divine pelo próprio índice (mesma conta do PriceCheckPanel), para quando
        // o endpoint de moedas falha.
        let rates = match indexed.as_ref().and_then(|i| i.divine_value.map(|d| (i.chaos_value, d))) {
            Some((chaos, divine)) if divine > 0.0 => rates.with_fallback("divine", chaos / divine),
            _ => rates,
        };

        // Sem a trade, o preço do índice ainda é útil; rate limit segue para a UI (que tem retry).
        let (trade, trade_error) = match trade {
            Ok(t) => (Some(t), None),
            Err(err @ PortError::RateLimited { .. }) => return Err(err.into()),
            Err(err) if indexed.is_some() => {
                tracing::warn!(%err, "trade falhou; usando só o poe.ninja");
                (None, Some(AppError::from(err)))
            }
            Err(err) => return Err(err.into()),
        };
        let summary = trade.as_ref().and_then(|t| summarize(&t.listings, &rates));
        let comparison = trade.as_ref().map(|t| compare_listings(league.game, &item, &t.listings, &rates));
        let icon = indexed
            .as_ref()
            .and_then(|i| i.icon.clone())
            .or_else(|| {
                trade.as_ref().and_then(|t| {
                    t.listings
                        .iter()
                        .find(|l| l.icon.is_some() && l.item.as_ref().is_some_and(|i| i.base_type == item.base_type))
                        .and_then(|l| l.icon.clone())
                })
            })
            .or(base_icon);

        Ok(PriceCheckOutput {
            item,
            filters,
            indexed,
            trade,
            summary,
            trade_error,
            icon,
            comparison,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use async_trait::async_trait;

    use mirrorsight_domain::item::Rarity;
    use mirrorsight_domain::ports::{ItemParseError, PortResult};
    use mirrorsight_domain::GameVersion;

    use super::*;

    struct FakeParser;
    impl ItemParser for FakeParser {
        fn parse(&self, _game: GameVersion, _text: &str) -> Result<Item, ItemParseError> {
            Ok(Item {
                item_class: "Shields".into(),
                rarity: Rarity::Unique,
                name: Some("Aegis Aurora".into()),
                base_type: "Champion Kite Shield".into(),
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

    /// Devolve o resultado configurado uma vez (PortError não é Clone).
    struct FakeTrade(Mutex<Option<PortResult<PriceCheckResult>>>);
    #[async_trait]
    impl TradeApiPort for FakeTrade {
        async fn price_check(&self, _: &League, _: &Item, _: &SearchFilters) -> PortResult<PriceCheckResult> {
            self.0.lock().unwrap().take().expect("uma chamada só")
        }
    }

    struct FakeIndex(Option<IndexedPrice>);
    #[async_trait]
    impl PriceIndexPort for FakeIndex {
        async fn lookup(&self, _: &League, _: &Item) -> PortResult<Option<IndexedPrice>> {
            Ok(self.0.clone())
        }
        async fn currency_rates(&self, _: &League) -> PortResult<CurrencyRates> {
            Ok(CurrencyRates::default())
        }
    }

    fn ninja() -> IndexedPrice {
        IndexedPrice {
            name: "Aegis Aurora".into(),
            chaos_value: 12.0,
            divine_value: None,
            change_24h_percent: None,
            sparkline: vec![],
            icon: None,
        }
    }

    fn found() -> PriceCheckResult {
        PriceCheckResult { query_id: "q".into(), total: 0, listings: vec![], web_url: None }
    }

    async fn run(trade: PortResult<PriceCheckResult>, indexed: Option<IndexedPrice>) -> Result<PriceCheckOutput, AppError> {
        let uc = PriceCheckItem::new(
            Arc::new(FakeParser),
            Arc::new(FakeTrade(Mutex::new(Some(trade)))),
            Arc::new(FakeIndex(indexed)),
        );
        uc.execute(&Settings::default(), "texto", None).await
    }

    fn unknown_item() -> PortError {
        PortError::Other("trade API: Unknown item".into())
    }

    #[tokio::test]
    async fn trade_ok_has_no_trade_error() {
        let out = run(Ok(found()), Some(ninja())).await.unwrap();
        assert!(out.trade.is_some());
        assert!(out.trade_error.is_none());
    }

    #[tokio::test]
    async fn trade_failure_falls_back_to_index() {
        let out = run(Err(unknown_item()), Some(ninja())).await.unwrap();
        assert!(out.trade.is_none());
        assert!(out.summary.is_none());
        assert_eq!(out.indexed.map(|i| i.chaos_value), Some(12.0));
        assert!(matches!(
            out.trade_error,
            Some(AppError::Internal { ref message, reason: None }) if message == "trade API: Unknown item"
        ));
    }

    #[tokio::test]
    async fn trade_failure_without_index_is_an_error() {
        assert!(run(Err(unknown_item()), None).await.is_err());
    }

    /// Índice sem preço, mas com a arte da base.
    struct BaseIconIndex;
    #[async_trait]
    impl PriceIndexPort for BaseIconIndex {
        async fn lookup(&self, _: &League, _: &Item) -> PortResult<Option<IndexedPrice>> {
            Ok(None)
        }
        async fn currency_rates(&self, _: &League) -> PortResult<CurrencyRates> {
            Ok(CurrencyRates::default())
        }
        async fn base_icon(&self, _: &League, base: &str) -> PortResult<Option<String>> {
            Ok(Some(format!("https://web.poecdn.com/{base}.png")))
        }
    }

    struct RareParser;
    impl ItemParser for RareParser {
        fn parse(&self, game: GameVersion, text: &str) -> Result<Item, ItemParseError> {
            let mut item = FakeParser.parse(game, text)?;
            item.rarity = Rarity::Rare;
            item.name = Some("Onslaught Veil".into());
            item.base_type = "Divine Crown".into();
            Ok(item)
        }
    }

    #[tokio::test]
    async fn rare_without_listings_uses_base_icon() {
        let uc = PriceCheckItem::new(
            Arc::new(RareParser),
            Arc::new(FakeTrade(Mutex::new(Some(Ok(found()))))),
            Arc::new(BaseIconIndex),
        );
        let out = uc.execute(&Settings::default(), "texto", None).await.unwrap();
        assert_eq!(out.icon.as_deref(), Some("https://web.poecdn.com/Divine Crown.png"));
    }

    #[tokio::test]
    async fn unique_icon_comes_from_index() {
        let mut price = ninja();
        price.icon = Some("https://web.poecdn.com/aegis.png".into());
        let out = run(Ok(found()), Some(price)).await.unwrap();
        assert_eq!(out.icon.as_deref(), Some("https://web.poecdn.com/aegis.png"));
    }

    #[tokio::test]
    async fn unique_gets_single_list_comparison() {
        use mirrorsight_domain::compare::SingleReason;
        let out = run(Ok(found()), Some(ninja())).await.unwrap();
        assert_eq!(out.comparison, Some(ListingComparison::Single { reason: SingleReason::Rarity }));
    }

    /// Luvas raras com tiers (cópia avançada).
    struct TieredParser;
    impl ItemParser for TieredParser {
        fn parse(&self, game: GameVersion, text: &str) -> Result<Item, ItemParseError> {
            use mirrorsight_domain::item::{AffixSlots, Modifier, ModifierKind, StatRoll};
            let mut item = FakeParser.parse(game, text)?;
            item.rarity = Rarity::Rare;
            item.item_class = "Gloves".into();
            item.affix_slots = Some(AffixSlots { prefixes: 1, suffixes: 2 });
            item.modifiers = vec![Modifier {
                trade_id: "explicit.stat_3299347043".into(),
                text: "+75 to maximum Life".into(),
                kind: ModifierKind::Explicit,
                roll: Some(StatRoll { value: 75.0, min: Some(70.0), max: Some(80.0) }),
                tier: Some(2),
                affix: None,
            }];
            Ok(item)
        }
    }

    #[tokio::test]
    async fn groups_listings_and_uses_index_divine_as_fallback() {
        use mirrorsight_domain::compare::GroupKind;
        use mirrorsight_domain::price::{Currency, Listing, Price};
        let mut price = ninja();
        price.chaos_value = 300.0;
        price.divine_value = Some(2.0); // → 150 c por divine
        let listing = |amount, currency: &str, item| Listing {
            price: Price { amount, currency: Currency(currency.into()) },
            stack_size: None,
            account: "acc".into(),
            online: true,
            indexed_at: String::new(),
            item,
            whisper: None,
            icon: None,
        };
        let same = TieredParser.parse(GameVersion::Poe1, "").ok();
        let result = PriceCheckResult {
            query_id: "q".into(),
            total: 3,
            listings: vec![listing(1.0, "divine", same.clone()), listing(20.0, "chaos", same), listing(5.0, "chaos", None)],
            web_url: None,
        };
        let uc = PriceCheckItem::new(
            Arc::new(TieredParser),
            Arc::new(FakeTrade(Mutex::new(Some(Ok(result))))),
            Arc::new(FakeIndex(Some(price))),
        );
        let out = uc.execute(&Settings::default(), "texto", None).await.unwrap();

        let Some(ListingComparison::Grouped { groups, .. }) = out.comparison else { panic!("esperava agrupado") };
        let equal = groups.iter().find(|g| g.kind == GroupKind::Equal).unwrap();
        assert_eq!(equal.entries.iter().map(|e| e.index).collect::<Vec<_>>(), vec![1, 0]);
        assert_eq!(equal.entries[1].chaos, Some(150.0));
        let no_item = groups.iter().find(|g| g.kind == GroupKind::NoItem).unwrap();
        assert_eq!(no_item.entries.len(), 1);
        assert_eq!(out.summary.and_then(|s| s.divine_chaos), Some(150.0));
    }

    #[tokio::test]
    async fn rate_limit_is_propagated_even_with_index() {
        let err = PortError::RateLimited { retry_after: Duration::from_secs(5) };
        assert!(matches!(run(Err(err), Some(ninja())).await, Err(AppError::RateLimited { .. })));
    }
}
