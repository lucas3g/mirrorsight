//! Índice de preços poe.ninja, PoE1 e PoE2.
//!
//! - Currency e fragmentos: `GET /{poe1|poe2}/api/economy/exchange/current/overview?league=…&type=…`
//!   `lines[].id` é a mesma tag da trade API; `primaryValue` está na moeda primária
//!   (`core.primary`: chaos no PoE1, divine no PoE2); `core.rates[x]` = quanto de `x` vale 1 primária.
//! - Únicos (só PoE1 por ora): `GET /poe1/api/economy/stash/current/item/overview?league=…&type=Unique…`
//!   com `chaosValue`, `links` e `variant` por linha.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::{StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use mirrorsight_domain::item::{Item, Rarity};
use mirrorsight_domain::ports::{PortResult, PriceIndexPort};
use mirrorsight_domain::price::{CurrencyRates, IndexedPrice};
use mirrorsight_domain::{GameVersion, League};

use crate::http::transport_error;

const SITE: &str = "https://poe.ninja";
const CACHE_TTL: Duration = Duration::from_secs(10 * 60);
const CURRENCY: &str = "Currency";
const FRAGMENT: &str = "Fragment";

// ---------- DTOs ----------

#[derive(Debug, Default, Deserialize)]
struct ExchangeDto {
    core: CoreDto,
    #[serde(default)]
    lines: Vec<ExchangeLineDto>,
    #[serde(default)]
    items: Vec<ExchangeItemDto>,
}

#[derive(Debug, Default, Deserialize)]
struct CoreDto {
    primary: String,
    #[serde(default)]
    rates: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct ExchangeLineDto {
    id: String,
    #[serde(rename = "primaryValue")]
    primary_value: f64,
    #[serde(default)]
    sparkline: Option<SparklineDto>,
}

#[derive(Debug, Clone, Deserialize)]
struct SparklineDto {
    /// Variação acumulada (%) em cada um dos últimos dias (pode ter buracos).
    #[serde(default)]
    data: Vec<Option<f64>>,
}

#[derive(Debug, Deserialize)]
struct ExchangeItemDto {
    id: String,
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct StashDto {
    #[serde(default)]
    lines: Vec<UniqueLine>,
}

#[derive(Debug, Clone, Deserialize)]
struct UniqueLine {
    name: String,
    #[serde(rename = "baseType")]
    base_type: String,
    #[serde(default)]
    variant: Option<String>,
    #[serde(default)]
    links: Option<u32>,
    #[serde(rename = "chaosValue")]
    chaos_value: f64,
    #[serde(rename = "divineValue", default)]
    divine_value: Option<f64>,
    #[serde(rename = "sparkLine", default)]
    spark_line: Option<SparklineDto>,
    #[serde(default)]
    icon: Option<String>,
}

/// Linha do overview `BaseType`: só nome e imagem (o resto do JSON de ~9 MB é ignorado).
#[derive(Debug, Deserialize)]
struct BaseTypeLine {
    #[serde(rename = "baseType", default)]
    base_type: Option<String>,
    name: String,
    #[serde(default)]
    icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct BaseTypeDto {
    #[serde(default)]
    lines: Vec<BaseTypeLine>,
}

/// base → imagem (poecdn). Todas as variantes de uma base usam a mesma arte; fica a primeira.
fn base_icon_map(dto: BaseTypeDto) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in dto.lines {
        let Some(icon) = line.icon.filter(|url| is_https_poecdn(url)) else { continue };
        map.entry(line.base_type.unwrap_or(line.name)).or_insert(icon);
    }
    map
}

// ---------- Modelos normalizados ----------

/// Overview de exchange já normalizado em chaos.
#[derive(Debug, Default)]
struct Exchange {
    /// tag → chaos por unidade.
    chaos_by_tag: HashMap<String, f64>,
    /// nome do item → tag.
    tag_by_name: HashMap<String, String>,
    /// tag → variação acumulada diária (%); `None` = dia sem dado.
    trend_by_tag: HashMap<String, Vec<Option<f64>>>,
}

impl Exchange {
    fn from_dto(dto: ExchangeDto) -> Self {
        let chaos_per_primary = if dto.core.primary == "chaos" {
            1.0
        } else {
            dto.core.rates.get("chaos").copied().unwrap_or(0.0)
        };
        let mut exchange = Self {
            tag_by_name: dto.items.into_iter().map(|i| (i.name, i.id)).collect(),
            ..Self::default()
        };
        if chaos_per_primary > 0.0 {
            exchange.chaos_by_tag.insert(dto.core.primary.clone(), chaos_per_primary);
        }
        for line in dto.lines {
            if chaos_per_primary > 0.0 && line.primary_value > 0.0 {
                exchange.chaos_by_tag.insert(line.id.clone(), line.primary_value * chaos_per_primary);
            }
            if let Some(spark) = line.sparkline {
                exchange.trend_by_tag.insert(line.id, spark.data);
            }
        }
        exchange
    }

    fn price_of(&self, name: &str, divine_chaos: Option<f64>) -> Option<IndexedPrice> {
        let tag = self.tag_by_name.get(name)?;
        let chaos = *self.chaos_by_tag.get(tag)?;
        let trend = self.trend_by_tag.get(tag).cloned().unwrap_or_default();
        Some(indexed(name, chaos, divine_chaos.filter(|d| *d > 0.0).map(|d| chaos / d), &trend, None))
    }
}

/// Só URLs do poecdn passam pela CSP do app; o resto vira placeholder na UI.
pub(crate) fn is_https_poecdn(url: &str) -> bool {
    url.starts_with("https://web.poecdn.com/")
}

/// Monta o preço de referência a partir da variação acumulada diária.
fn indexed(
    name: &str,
    chaos_value: f64,
    divine_value: Option<f64>,
    trend: &[Option<f64>],
    icon: Option<String>,
) -> IndexedPrice {
    // Variação acumulada → série de preço relativa (100 = início da semana).
    // Dia sem dado repete o último valor conhecido, para os pontos não se deslocarem;
    // buracos antes do primeiro dado são omitidos (não há valor para repetir).
    let mut last_known = None;
    let sparkline: Vec<f64> = trend
        .iter()
        .filter_map(|pct| {
            last_known = pct.map(|p| 100.0 + p).or(last_known);
            last_known
        })
        .collect();
    // 24h só com os dois últimos dias presentes; com buraco seria a variação de mais dias.
    let change_24h_percent = match trend {
        [.., Some(prev), Some(last)] => Some(((100.0 + last) / (100.0 + prev) - 1.0) * 100.0),
        _ => None,
    };
    let icon = icon.filter(|url| is_https_poecdn(url));
    IndexedPrice { name: name.to_owned(), chaos_value, divine_value, change_24h_percent, sparkline, icon }
}

/// Escolhe a linha do único que corresponde ao item: mesmo nome/base e mesma faixa de links
/// (poe.ninja separa 5L e 6L); sem variante tem preferência. Sem a faixa do item, cai só na
/// linha sem links — nunca no preço de outra faixa (ex.: um 0-4L não leva o preço do 6L).
fn pick_unique<'a>(lines: &'a [UniqueLine], item: &Item) -> Option<&'a UniqueLine> {
    let plain = item.name.as_deref()?;
    // O poe.ninja lista Foulborn com o prefixo no nome ("Foulborn Aegis Aurora").
    let name = if item.foulborn { format!("Foulborn {plain}") } else { plain.to_owned() };
    let links = item.max_links();
    let wanted = (links >= 5).then_some(links);
    let same_item = |l: &&UniqueLine| l.name == name && l.base_type == item.base_type;
    let same_links = |l: &&UniqueLine| l.links.filter(|n| *n >= 5) == wanted;
    let unlinked = |l: &&UniqueLine| l.links.is_none();

    let mod_words = mod_word_stems(item);
    let candidates: Vec<&UniqueLine> = lines.iter().filter(same_item).collect();
    // Prefere o variant cujas palavras aparecem nos mods do item; empate → sem variant.
    let best = |filter: &dyn Fn(&&UniqueLine) -> bool| {
        candidates
            .iter()
            .copied()
            .filter(|l| filter(l))
            .rev() // max_by_key devolve o último empatado; assim fica o primeiro.
            .max_by_key(|l| (variant_score(l.variant.as_deref(), &mod_words), l.variant.is_none()))
    };
    best(&same_links).or_else(|| best(&unlinked))
}

/// Radical (4 primeiras letras, minúsculas) de cada palavra com 3+ letras.
fn stems(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|w| w.chars().count() >= 3)
        .map(|w| w.to_lowercase().chars().take(4).collect())
}

fn mod_word_stems(item: &Item) -> std::collections::HashSet<String> {
    item.modifiers.iter().flat_map(|m| stems(&m.text).collect::<Vec<_>>()).collect()
}

/// Quantas palavras do variant ("No Reflect") aparecem nos mods ("Damage cannot be Reflected").
fn variant_score(variant: Option<&str>, mod_words: &std::collections::HashSet<String>) -> usize {
    variant.map_or(0, |v| stems(v).filter(|s| mod_words.contains(s)).count())
}

/// Categoria de únicos do poe.ninja para a classe do item (sem diferenciar maiúsculas:
/// "Warstaves" precisa bater com "stave").
fn unique_category(item_class: &str) -> Option<&'static str> {
    const WEAPON: &[&str] = &[
        "sword", "axe", "mace", "bow", "claw", "dagger", "wand", "staff", "stave", "sceptre", "fishing rod",
    ];
    const ARMOUR: &[&str] = &["body armour", "helmet", "glove", "boot", "shield", "quiver"];
    const ACCESSORY: &[&str] = &["amulet", "ring", "belt", "trinket"];

    let class = item_class.to_lowercase();
    let has = |list: &[&str]| list.iter().any(|k| class.contains(k));
    if class.contains("flask") {
        Some("UniqueFlask")
    } else if class.contains("jewel") {
        Some("UniqueJewel")
    } else if has(ACCESSORY) {
        Some("UniqueAccessory")
    } else if has(ARMOUR) {
        Some("UniqueArmour")
    } else if has(WEAPON) {
        Some("UniqueWeapon")
    } else if class == "maps" {
        Some("UniqueMap")
    } else {
        None
    }
}

// ---------- Adapter ----------

type CacheKey = (GameVersion, String, &'static str);
type Cache<T> = Mutex<HashMap<CacheKey, (Instant, Arc<T>)>>;

/// A arte das bases quase não muda: o overview (~9 MB) é baixado no máximo uma vez por dia.
const BASE_ICON_TTL: Duration = Duration::from_secs(24 * 60 * 60);
/// Download que falhou (mapa vazio) tenta de novo depois disso.
const BASE_ICON_RETRY: Duration = Duration::from_secs(5 * 60);

/// `None` enquanto o download roda; `Some(mapa)` quando termina (vazio = falhou).
type BaseIconRx = tokio::sync::watch::Receiver<Option<Arc<HashMap<String, String>>>>;

pub struct PoeNinjaIndex {
    http: reqwest::Client,
    exchange_cache: Cache<Exchange>,
    /// Serializa os downloads de exchange: `lookup` e `currency_rates` rodam juntos e,
    /// com o cache vazio, fariam dois GETs iguais da Currency.
    exchange_fetch: tokio::sync::Mutex<()>,
    unique_cache: Cache<Vec<UniqueLine>>,
    /// Por liga: quando o download começou e o mapa base → imagem (preenchido ao terminar).
    base_icons: Mutex<HashMap<(GameVersion, String), (Instant, BaseIconRx)>>,
}

impl PoeNinjaIndex {
    pub fn new(http: reqwest::Client) -> Self {
        Self {
            http,
            exchange_cache: Mutex::default(),
            exchange_fetch: tokio::sync::Mutex::default(),
            unique_cache: Mutex::default(),
            base_icons: Mutex::default(),
        }
    }

    /// Mapa base → imagem da liga. O download roda numa task própria: se quem pediu
    /// desistir de esperar (timeout do price check), ele termina e fica para o próximo.
    async fn base_icon_map(&self, league: &League) -> Arc<HashMap<String, String>> {
        let key = (league.game, league.id.clone());
        let mut rx = {
            let mut downloads = self.base_icons.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let fresh = downloads.get(&key).filter(|(at, rx)| {
                let ttl = match rx.borrow().as_ref() {
                    Some(map) if map.is_empty() => BASE_ICON_RETRY,
                    _ => BASE_ICON_TTL,
                };
                at.elapsed() < ttl
            });
            if let Some((_, rx)) = fresh {
                rx.clone()
            } else {
                let (tx, rx) = tokio::sync::watch::channel(None);
                downloads.insert(key, (Instant::now(), rx.clone()));
                let (http, url) = (self.http.clone(), stash_url(league, "BaseType"));
                tokio::spawn(async move {
                    let _ = tx.send(Some(download_base_icons(http, url).await));
                });
                rx
            }
        };
        // Espera o download (desta chamada ou de um price check anterior) terminar.
        let map = match rx.wait_for(Option::is_some).await {
            Ok(done) => done.clone().unwrap_or_default(),
            Err(_) => Arc::default(),
        };
        map
    }

    async fn exchange(&self, league: &League, kind: &'static str) -> PortResult<Arc<Exchange>> {
        let key = (league.game, league.id.clone(), kind);
        if let Some(hit) = cached(&self.exchange_cache, &key) {
            return Ok(hit);
        }
        let _fetching = self.exchange_fetch.lock().await;
        // Quem esperou o lock encontra o resultado de quem baixou primeiro.
        if let Some(hit) = cached(&self.exchange_cache, &key) {
            return Ok(hit);
        }
        let dto: ExchangeDto = self.get_or_default(exchange_url(league, kind)).await?;
        Ok(store(&self.exchange_cache, key, Exchange::from_dto(dto)))
    }

    async fn uniques(&self, league: &League, category: &'static str) -> PortResult<Arc<Vec<UniqueLine>>> {
        let key = (league.game, league.id.clone(), category);
        if let Some(hit) = cached(&self.unique_cache, &key) {
            return Ok(hit);
        }
        let dto: StashDto = self.get_or_default(stash_url(league, category)).await?;
        Ok(store(&self.unique_cache, key, dto.lines))
    }

    /// GET JSON; 404 (liga/categoria sem dados) vira valor vazio em vez de erro.
    async fn get_or_default<T: DeserializeOwned + Default>(&self, url: Url) -> PortResult<T> {
        let response = self.http.get(url).send().await.map_err(|e| transport_error(&e))?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(T::default());
        }
        response
            .error_for_status()
            .map_err(|e| transport_error(&e))?
            .json()
            .await
            .map_err(|e| transport_error(&e))
    }

    async fn divine_chaos(&self, league: &League) -> PortResult<Option<f64>> {
        Ok(self.exchange(league, CURRENCY).await?.chaos_by_tag.get("divine").copied())
    }
}

/// Imagem da base. Itens mágicos trazem os afixos no nome ("Hale Sorcerer Gloves of the Whale"):
/// sem o nome exato, vale a base mais longa contida nele, inteira entre espaços
/// ("Sorcerer Gloves", não "Gloves" de outra base).
fn find_base_icon(map: &HashMap<String, String>, base_type: &str) -> Option<String> {
    if let Some(icon) = map.get(base_type) {
        return Some(icon.clone());
    }
    let padded = format!(" {base_type} ");
    map.iter()
        .filter(|(base, _)| padded.contains(&format!(" {base} ")))
        .max_by_key(|(base, _)| base.len())
        .map(|(_, icon)| icon.clone())
}

async fn download_base_icons(http: reqwest::Client, url: Url) -> Arc<HashMap<String, String>> {
    let result = async {
        let response = http.get(url).send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(BaseTypeDto::default());
        }
        response.error_for_status()?.json::<BaseTypeDto>().await
    }
    .await;
    match result {
        Ok(dto) => {
            let map = base_icon_map(dto);
            tracing::info!(bases = map.len(), "imagens das bases carregadas do poe.ninja");
            Arc::new(map)
        }
        Err(err) => {
            tracing::warn!(%err, "falha ao baixar as imagens das bases do poe.ninja");
            Arc::default()
        }
    }
}

fn cached<T>(cache: &Cache<T>, key: &CacheKey) -> Option<Arc<T>> {
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(key)
        .filter(|(at, _)| at.elapsed() < CACHE_TTL)
        .map(|(_, v)| v.clone())
}

fn store<T>(cache: &Cache<T>, key: CacheKey, value: T) -> Arc<T> {
    let value = Arc::new(value);
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(key, (Instant::now(), value.clone()));
    value
}

#[async_trait]
impl PriceIndexPort for PoeNinjaIndex {
    async fn lookup(&self, league: &League, item: &Item) -> PortResult<Option<IndexedPrice>> {
        // A cotação do divine é só um extra: sem ela o preço em chaos continua valendo.
        let divine = self.divine_chaos(league).await.unwrap_or_else(|err| {
            tracing::warn!(%err, "falha ao obter cotação do divine no poe.ninja");
            None
        });

        if item.rarity == Rarity::Unique {
            let Some(category) = unique_category(&item.item_class).filter(|_| league.game == GameVersion::Poe1)
            else {
                return Ok(None);
            };
            let lines = self.uniques(league, category).await?;
            return Ok(pick_unique(&lines, item).map(|line| {
                let trend: Vec<Option<f64>> = line.spark_line.as_ref().map(|s| s.data.clone()).unwrap_or_default();
                let divine_value = line.divine_value.or_else(|| divine.filter(|d| *d > 0.0).map(|d| line.chaos_value / d));
                indexed(&line.name, line.chaos_value, divine_value, &trend, line.icon.clone())
            }));
        }

        for kind in exchange_kinds(league.game, item) {
            if let Some(price) = self.exchange(league, kind).await?.price_of(&item.base_type, divine) {
                return Ok(Some(price));
            }
        }
        Ok(None)
    }

    async fn currency_rates(&self, league: &League) -> PortResult<CurrencyRates> {
        let exchange = self.exchange(league, CURRENCY).await?;
        Ok(CurrencyRates::from_pairs(exchange.chaos_by_tag.iter().map(|(k, v)| (k.clone(), *v))))
    }

    async fn base_icon(&self, league: &League, base_type: &str) -> PortResult<Option<String>> {
        // O poe.ninja só tem o overview de bases no PoE1.
        if league.game != GameVersion::Poe1 {
            return Ok(None);
        }
        let map = self.base_icon_map(league).await;
        Ok(find_base_icon(&map, base_type))
    }
}

/// Categorias do exchange do poe.ninja onde procurar um item não-único, em ordem.
/// Cada categoria é baixada uma vez e cacheada, então tentar várias é barato.
fn exchange_kinds(game: GameVersion, item: &Item) -> &'static [&'static str] {
    const POE1_CURRENCY: &[&str] = &[CURRENCY, "Essence", "Fossil", "Oil", "Omen", "Tattoo", FRAGMENT];
    if item.item_class.contains("Fragment") {
        return &[FRAGMENT];
    }
    match (game, item.rarity) {
        (GameVersion::Poe1, Rarity::DivinationCard) => &["DivinationCard"],
        (GameVersion::Poe1, Rarity::Currency) => POE1_CURRENCY,
        (GameVersion::Poe2, Rarity::Currency) => &[CURRENCY, FRAGMENT],
        _ => &[],
    }
}

fn realm(game: GameVersion) -> &'static str {
    match game {
        GameVersion::Poe1 => "poe1",
        GameVersion::Poe2 => "poe2",
    }
}

fn url_with(segments: &[&str], league: &str, kind: &str) -> Url {
    let mut url = Url::parse(SITE).unwrap_or_else(|_| unreachable!("URL base constante"));
    if let Ok(mut path) = url.path_segments_mut() {
        path.extend(segments);
    }
    url.query_pairs_mut().append_pair("league", league).append_pair("type", kind);
    url
}

fn exchange_url(league: &League, kind: &str) -> Url {
    url_with(&[realm(league.game), "api", "economy", "exchange", "current", "overview"], &league.id, kind)
}

fn stash_url(league: &League, category: &str) -> Url {
    url_with(&[realm(league.game), "api", "economy", "stash", "current", "item", "overview"], &league.id, category)
}

#[cfg(test)]
mod tests {
    use super::*;

    const POE1: &str = r#"{"core":{"items":[],"rates":{"divine":0.005634},"primary":"chaos","secondary":"divine"},
      "lines":[{"id":"divine","primaryValue":177.5,"sparkline":{"totalChange":-15.8,"data":[3.3,0.03,-11.24,-15.8]}},
               {"id":"alch","primaryValue":0.25}],
      "items":[{"id":"divine","name":"Divine Orb"},{"id":"alch","name":"Orb of Alchemy"}]}"#;

    const POE2: &str = r#"{"core":{"rates":{"exalted":513.2,"chaos":7.55},"primary":"divine"},
      "lines":[{"id":"divine","primaryValue":1},{"id":"exalted","primaryValue":0.00195}],
      "items":[{"id":"divine","name":"Divine Orb"}]}"#;

    const UNIQUES: &str = r#"{"lines":[
      {"name":"Belly of the Beast","baseType":"Full Wyrmscale","links":6,"chaosValue":60,"divineValue":0.34},
      {"name":"Belly of the Beast","baseType":"Full Wyrmscale","links":5,"chaosValue":9},
      {"name":"Belly of the Beast","baseType":"Full Wyrmscale","chaosValue":4,"sparkLine":{"data":[0,null,-20]},
       "icon":"https://web.poecdn.com/gen/image/x.png"},
      {"name":"Foulborn Belly of the Beast","baseType":"Full Wyrmscale","chaosValue":99}
    ]}"#;

    fn exchange(json: &str) -> Exchange {
        Exchange::from_dto(serde_json::from_str(json).unwrap())
    }

    fn belly(sockets: Option<&str>) -> Item {
        Item {
            item_class: "Body Armours".into(),
            rarity: Rarity::Unique,
            name: Some("Belly of the Beast".into()),
            base_type: "Full Wyrmscale".into(),
            item_level: None,
            quality: None,
            corrupted: false,
            identified: true,
            sockets: sockets.map(Into::into),
            influences: vec![],
            modifiers: vec![],
            properties: Vec::new(),
            unknown_lines: vec![],
            foulborn: false,
            discriminator: None,
            trade_type: None,
            map_tier: None,
            affix_slots: None,
            flags: Default::default(),
        }
    }

    #[test]
    fn poe1_values_are_already_in_chaos() {
        let o = exchange(POE1);
        assert_eq!(o.chaos_by_tag["chaos"], 1.0);
        let price = o.price_of("Divine Orb", Some(177.5)).unwrap();
        assert_eq!(price.chaos_value, 177.5);
        assert_eq!(price.divine_value, Some(1.0));
        let change = price.change_24h_percent.unwrap();
        assert!((change + 5.13).abs() < 0.01, "{change}");
    }

    #[test]
    fn poe2_converts_divine_primary_to_chaos() {
        let o = exchange(POE2);
        assert_eq!(o.chaos_by_tag["divine"], 7.55);
        assert!((o.chaos_by_tag["exalted"] - 0.00195 * 7.55).abs() < 1e-9);
    }

    #[test]
    fn unknown_item_has_no_price() {
        assert!(exchange(POE1).price_of("Mirror of Kalandra", None).is_none());
    }

    #[test]
    fn unique_matches_links_bracket() {
        let lines: StashDto = serde_json::from_str(UNIQUES).unwrap();
        let pick = |s| pick_unique(&lines.lines, &belly(s)).map(|l| l.chaos_value);
        assert_eq!(pick(Some("R-R-R-G-G-B")), Some(60.0));
        assert_eq!(pick(Some("R-R-R-G-G B")), Some(9.0));
        assert_eq!(pick(Some("R-R G")), Some(4.0));
        assert_eq!(pick(None), Some(4.0));
    }

    #[test]
    fn foulborn_matches_prefixed_name_and_variant_by_mods() {
        use mirrorsight_domain::item::{Modifier, ModifierKind};
        let lines: StashDto = serde_json::from_str(
            r#"{"lines":[
              {"name":"Aegis Aurora","baseType":"Champion Kite Shield","chaosValue":5},
              {"name":"Foulborn Aegis Aurora","baseType":"Champion Kite Shield","variant":"Mana Shield","chaosValue":30},
              {"name":"Foulborn Aegis Aurora","baseType":"Champion Kite Shield","variant":"No Reflect","chaosValue":80}
            ]}"#,
        )
        .unwrap();
        let mut aegis = belly(None);
        aegis.item_class = "Shields".into();
        aegis.name = Some("Aegis Aurora".into());
        aegis.base_type = "Champion Kite Shield".into();
        assert_eq!(pick_unique(&lines.lines, &aegis).map(|l| l.chaos_value), Some(5.0));

        aegis.foulborn = true;
        aegis.modifiers = vec![Modifier {
            trade_id: "explicit.stat_2670993553".into(),
            text: "Damage cannot be Reflected".into(),
            kind: ModifierKind::Explicit,
            roll: None,
            tier: None,
            affix: None,
            ..Default::default()
        }];
        assert_eq!(pick_unique(&lines.lines, &aegis).map(|l| l.chaos_value), Some(80.0));
    }

    #[test]
    fn exchange_kinds_by_item() {
        let mut card = belly(None);
        card.rarity = Rarity::DivinationCard;
        card.item_class = "Divination Cards".into();
        assert_eq!(exchange_kinds(GameVersion::Poe1, &card), &["DivinationCard"]);
        card.rarity = Rarity::Currency;
        card.item_class = "Stackable Currency".into();
        assert!(exchange_kinds(GameVersion::Poe1, &card).contains(&"Essence"));
        card.item_class = "Map Fragments".into();
        assert_eq!(exchange_kinds(GameVersion::Poe1, &card), &[FRAGMENT]);
    }

    #[test]
    fn base_icon_map_keeps_first_poecdn_icon_per_base() {
        let dto: BaseTypeDto = serde_json::from_str(
            r#"{"lines":[
              {"name":"Divine Crown","baseType":"Divine Crown","variant":"Shaper","icon":"https://web.poecdn.com/gen/image/a/HelmetStrInt9.png"},
              {"name":"Divine Crown","baseType":"Divine Crown","icon":"https://web.poecdn.com/gen/image/b/HelmetStrInt9.png"},
              {"name":"Vaal Gauntlets","icon":"http://evil.example/x.png"},
              {"name":"Sorcerer Gloves"}
            ]}"#,
        )
        .unwrap();
        let map = base_icon_map(dto);
        assert_eq!(map.len(), 1);
        assert_eq!(map["Divine Crown"], "https://web.poecdn.com/gen/image/a/HelmetStrInt9.png");
    }

    #[test]
    fn magic_item_finds_base_icon_inside_affixed_name() {
        let map: HashMap<String, String> = [("Sorcerer Gloves", "sg.png"), ("Gloves", "x.png"), ("Ring", "r.png")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        assert_eq!(find_base_icon(&map, "Sorcerer Gloves").as_deref(), Some("sg.png"));
        assert_eq!(find_base_icon(&map, "Hale Sorcerer Gloves of the Whale").as_deref(), Some("sg.png"));
        // "Ring" não casa dentro de "Ringmail" (palavra inteira).
        assert!(find_base_icon(&map, "Studded Ringmail").is_none());
    }

    #[test]
    fn unique_icon_is_kept_only_from_poecdn() {
        let lines: StashDto = serde_json::from_str(UNIQUES).unwrap();
        let line = pick_unique(&lines.lines, &belly(None)).unwrap();
        let price = indexed(&line.name, line.chaos_value, None, &[], line.icon.clone());
        assert_eq!(price.icon.as_deref(), Some("https://web.poecdn.com/gen/image/x.png"));

        let other = indexed("x", 1.0, None, &[], Some("https://example.com/x.png".into()));
        assert!(other.icon.is_none());
        assert!(exchange(POE1).price_of("Divine Orb", None).unwrap().icon.is_none());
    }

    #[test]
    fn unique_categories() {
        assert_eq!(unique_category("Body Armours"), Some("UniqueArmour"));
        assert_eq!(unique_category("Rings"), Some("UniqueAccessory"));
        assert_eq!(unique_category("Two Hand Swords"), Some("UniqueWeapon"));
        assert_eq!(unique_category("Life Flasks"), Some("UniqueFlask"));
        assert_eq!(unique_category("Abyss Jewels"), Some("UniqueJewel"));
        assert_eq!(unique_category("Stackable Currency"), None);
        assert_eq!(unique_category("Warstaves"), Some("UniqueWeapon"));
        assert_eq!(unique_category("Trinkets"), Some("UniqueAccessory"));
        assert_eq!(unique_category("Maps"), Some("UniqueMap"));
    }

    #[test]
    fn unique_never_takes_price_of_other_links_bracket() {
        let only_6l: StashDto = serde_json::from_str(
            r#"{"lines":[{"name":"Belly of the Beast","baseType":"Full Wyrmscale","links":6,"chaosValue":60}]}"#,
        )
        .unwrap();
        assert!(pick_unique(&only_6l.lines, &belly(Some("R-R G"))).is_none());
        assert!(pick_unique(&only_6l.lines, &belly(Some("R-R-R-G-G B"))).is_none());

        // 5L sem linha de 5L: cai na linha sem links, preferindo a sem variante.
        let unlinked: StashDto = serde_json::from_str(
            r#"{"lines":[
              {"name":"Belly of the Beast","baseType":"Full Wyrmscale","links":6,"chaosValue":60},
              {"name":"Belly of the Beast","baseType":"Full Wyrmscale","variant":"x","chaosValue":7},
              {"name":"Belly of the Beast","baseType":"Full Wyrmscale","chaosValue":4}
            ]}"#,
        )
        .unwrap();
        let pick = pick_unique(&unlinked.lines, &belly(Some("R-R-R-G-G B"))).map(|l| l.chaos_value);
        assert_eq!(pick, Some(4.0));
    }

    #[test]
    fn sparkline_holes_do_not_fake_24h_change() {
        let with_hole = indexed("x", 1.0, None, &[Some(0.0), None, Some(-20.0)], None);
        assert_eq!(with_hole.change_24h_percent, None);
        assert_eq!(with_hole.sparkline, vec![100.0, 100.0, 80.0]);

        let leading_hole = indexed("x", 1.0, None, &[None, Some(0.0), Some(-20.0)], None);
        assert_eq!(leading_hole.sparkline, vec![100.0, 80.0]);
        let change = leading_hole.change_24h_percent.unwrap();
        assert!((change + 20.0).abs() < 1e-9, "{change}");

        let last_missing = indexed("x", 1.0, None, &[Some(0.0), Some(-20.0), None], None);
        assert_eq!(last_missing.change_24h_percent, None);
        assert_eq!(last_missing.sparkline, vec![100.0, 80.0, 80.0]);
    }

    #[test]
    fn urls_encode_league() {
        let league = League::new("Runes of Aldur", GameVersion::Poe2);
        assert_eq!(
            exchange_url(&league, CURRENCY).as_str(),
            "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Runes+of+Aldur&type=Currency"
        );
        assert_eq!(
            stash_url(&League::new("Covens", GameVersion::Poe1), "UniqueArmour").as_str(),
            "https://poe.ninja/poe1/api/economy/stash/current/item/overview?league=Covens&type=UniqueArmour"
        );
    }
}
