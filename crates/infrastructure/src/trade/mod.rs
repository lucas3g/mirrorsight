//! Adapter da trade API oficial (pathofexile.com) — PoE1 e PoE2.
//!
//! Fluxo: `POST search` (IDs das listagens, ordenadas por preço) → `GET fetch` em
//! páginas de 10 → [`Listing`]s. Cada endpoint tem seu limite; ver [`RateLimiter`].

mod dto;
pub mod query;

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::{RequestBuilder, Response, StatusCode, Url};

use mirrorsight_domain::item::Item;
use mirrorsight_domain::ports::{ItemParser, LeaguesPort, PortError, PortResult, TradeApiPort};
use mirrorsight_domain::price::{Listing, PriceCheckResult};
use mirrorsight_domain::search::SearchFilters;
use mirrorsight_domain::{GameVersion, League};

use crate::http::transport_error;
use crate::rate_limit::RateLimiter;
use dto::{ErrorResponse, FetchResponse, LeaguesResponse, SearchResponse};

const SITE: &str = "https://www.pathofexile.com";
const SEARCH: &str = "search";
const FETCH: &str = "fetch";
/// A API aceita no máximo 10 IDs por fetch.
const FETCH_PAGE: usize = 10;
/// Listagens buscadas por price check (2 fetches): base da estimativa e do histograma.
const MAX_LISTINGS: usize = 20;
/// Usado quando um 429 vem sem `Retry-After`.
const DEFAULT_RETRY: Duration = Duration::from_secs(60);

pub struct PoeTradeApi {
    http: reqwest::Client,
    parser: Arc<dyn ItemParser>,
    limiter: RateLimiter,
}

impl PoeTradeApi {
    pub fn new(http: reqwest::Client, parser: Arc<dyn ItemParser>) -> Self {
        Self { http, parser, limiter: RateLimiter::default() }
    }

    /// Envia respeitando o rate limit e converte respostas de erro.
    async fn send(&self, key: &'static str, request: RequestBuilder) -> PortResult<Response> {
        self.limiter
            .try_acquire(key, Instant::now())
            .map_err(|retry_after| PortError::RateLimited { retry_after })?;

        let response = request.send().await.map_err(|e| transport_error(&e))?;
        let headers = response.headers();
        let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
        self.limiter
            .update(key, header("x-rate-limit-ip"), header("x-rate-limit-ip-state"), Instant::now());

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = header("retry-after")
                .and_then(|v| v.parse().ok())
                .map_or(DEFAULT_RETRY, Duration::from_secs);
            self.limiter.block(key, retry_after, Instant::now());
            return Err(PortError::RateLimited { retry_after });
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<ErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or_else(|_| format!("HTTP {status}"));
            return Err(PortError::Other(format!("trade API: {message}")));
        }
        Ok(response)
    }

    async fn fetch_page(&self, game: GameVersion, query_id: &str, ids: &[String]) -> PortResult<Vec<Listing>> {
        let mut url = api_url(game, &["fetch", &ids.join(",")]);
        url.query_pairs_mut().append_pair("query", query_id);
        let response = self.send(FETCH, self.http.get(url)).await?;
        let body: FetchResponse = response.json().await.map_err(|e| transport_error(&e))?;
        Ok(body
            .result
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.into_listing(game, self.parser.as_ref()))
            .collect())
    }
}

#[async_trait]
impl TradeApiPort for PoeTradeApi {
    async fn price_check(&self, league: &League, item: &Item, filters: &SearchFilters) -> PortResult<PriceCheckResult> {
        let game = league.game;
        let body = query::build(game, item, filters);
        tracing::info!(query = %body, "busca enviada ao trade");
        let response = self
            .send(SEARCH, self.http.post(search_url(game, &league.id)).json(&body))
            .await
            .inspect_err(|err| {
                if !matches!(err, PortError::RateLimited { .. }) {
                    tracing::warn!(query = %body, %err, "busca rejeitada pela trade API");
                }
            })?;
        let search: SearchResponse = response.json().await.map_err(|e| transport_error(&e))?;
        let web_url = Some(web_url(game, &league.id, &search.id));

        let mut listings = Vec::new();
        for page in search.result.iter().take(MAX_LISTINGS).collect::<Vec<_>>().chunks(FETCH_PAGE) {
            let ids: Vec<String> = page.iter().map(|s| (*s).clone()).collect();
            match self.fetch_page(game, &search.id, &ids).await {
                Ok(mut found) => listings.append(&mut found),
                // Já temos listagens: melhor mostrar menos do que falhar o price check inteiro.
                Err(err) if !listings.is_empty() => {
                    tracing::warn!(%err, "fetch parcial; usando listagens já obtidas");
                    break;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(PriceCheckResult { query_id: search.id, total: search.total, listings, web_url })
    }
}

#[async_trait]
impl LeaguesPort for PoeTradeApi {
    async fn leagues(&self, game: GameVersion) -> PortResult<Vec<String>> {
        let response = self
            .http
            .get(api_url(game, &["data", "leagues"]))
            .send()
            .await
            .map_err(|e| transport_error(&e))?
            .error_for_status()
            .map_err(|e| transport_error(&e))?;
        let body: LeaguesResponse = response.json().await.map_err(|e| transport_error(&e))?;
        let realm = match game {
            GameVersion::Poe1 => "pc",
            GameVersion::Poe2 => "poe2",
        };
        Ok(body
            .result
            .into_iter()
            .filter(|l| l.realm.as_deref().is_none_or(|r| r == realm))
            .map(|l| l.id)
            .collect())
    }
}

/// `…/api/trade` (PoE1) ou `…/api/trade2` (PoE2) + segmentos codificados.
fn api_url(game: GameVersion, segments: &[&str]) -> Url {
    let root = match game {
        GameVersion::Poe1 => "trade",
        GameVersion::Poe2 => "trade2",
    };
    let mut url = Url::parse(SITE).unwrap_or_else(|_| unreachable!("URL base constante"));
    if let Ok(mut path) = url.path_segments_mut() {
        path.push("api").push(root).extend(segments);
    }
    url
}

fn search_url(game: GameVersion, league: &str) -> Url {
    match game {
        GameVersion::Poe1 => api_url(game, &["search", league]),
        GameVersion::Poe2 => api_url(game, &["search", "poe2", league]),
    }
}

/// A mesma busca no site oficial, para o botão "Trade".
fn web_url(game: GameVersion, league: &str, query_id: &str) -> String {
    let mut url = Url::parse(SITE).unwrap_or_else(|_| unreachable!("URL base constante"));
    if let Ok(mut path) = url.path_segments_mut() {
        match game {
            GameVersion::Poe1 => path.extend(["trade", "search", league, query_id]),
            GameVersion::Poe2 => path.extend(["trade2", "search", "poe2", league, query_id]),
        };
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_encode_league_and_split_games() {
        assert_eq!(
            search_url(GameVersion::Poe1, "Hardcore Covens").as_str(),
            "https://www.pathofexile.com/api/trade/search/Hardcore%20Covens"
        );
        assert_eq!(
            search_url(GameVersion::Poe2, "Runes of Aldur").as_str(),
            "https://www.pathofexile.com/api/trade2/search/poe2/Runes%20of%20Aldur"
        );
        assert_eq!(
            web_url(GameVersion::Poe1, "Covens", "abc"),
            "https://www.pathofexile.com/trade/search/Covens/abc"
        );
    }
}
