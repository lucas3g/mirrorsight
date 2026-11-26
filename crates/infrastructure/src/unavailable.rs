//! Adapters "nulos" (Null Object) usados enquanto os reais não existem.
//! Permitem montar o composition root completo desde a fase 0.

use async_trait::async_trait;

use mirrorsight_domain::item::Item;
use mirrorsight_domain::ports::{PortError, PortResult, PriceIndexPort, TradeApiPort};
use mirrorsight_domain::price::{CurrencyRates, IndexedPrice, PriceCheckResult};
use mirrorsight_domain::search::SearchFilters;
use mirrorsight_domain::League;

pub struct UnavailableTradeApi;

#[async_trait]
impl TradeApiPort for UnavailableTradeApi {
    async fn price_check(
        &self,
        _league: &League,
        _item: &Item,
        _filters: &SearchFilters,
    ) -> PortResult<PriceCheckResult> {
        Err(PortError::Other("trade API ainda não implementada (fase 4)".into()))
    }
}

pub struct EmptyPriceIndex;

#[async_trait]
impl PriceIndexPort for EmptyPriceIndex {
    async fn lookup(&self, _league: &League, _item: &Item) -> PortResult<Option<IndexedPrice>> {
        Ok(None)
    }

    async fn currency_rates(&self, _league: &League) -> PortResult<CurrencyRates> {
        Ok(CurrencyRates::default())
    }
}
