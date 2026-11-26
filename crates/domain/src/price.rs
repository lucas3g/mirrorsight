use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::item::Item;

/// Moeda em que um preço é expresso (tag da trade API: "chaos", "divine", "exalted"...).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency(pub String);

impl Currency {
    pub fn chaos() -> Self {
        Self("chaos".into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: Currency,
}

/// Uma listagem retornada pela trade API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Listing {
    pub price: Price,
    pub stack_size: Option<u32>,
    pub account: String,
    pub online: bool,
    /// ISO-8601 retornado pela API.
    pub indexed_at: String,
    /// Snapshot do item listado (mods, propriedades), quando a API o fornece.
    #[serde(default)]
    pub item: Option<Item>,
    /// Mensagem de whisper pronta, fornecida pela trade API.
    #[serde(default)]
    pub whisper: Option<String>,
    /// Imagem do item listado (URL do poecdn). Em raros/mágicos é a arte da base.
    #[serde(default)]
    pub icon: Option<String>,
}

/// Resultado agregado de um price check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceCheckResult {
    pub query_id: String,
    pub total: u32,
    pub listings: Vec<Listing>,
    /// Link da mesma busca no site oficial (montado pelo adapter de trade).
    #[serde(default)]
    pub web_url: Option<String>,
}

/// Preço de referência vindo de um índice (poe.ninja).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexedPrice {
    pub name: String,
    pub chaos_value: f64,
    pub divine_value: Option<f64>,
    pub change_24h_percent: Option<f64>,
    pub sparkline: Vec<f64>,
    /// Imagem do item (URL do poecdn), quando o índice a fornece.
    #[serde(default)]
    pub icon: Option<String>,
}

/// Quantos chaos vale uma unidade de cada moeda. `chaos` vale 1 sempre.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrencyRates {
    chaos_per_unit: HashMap<String, f64>,
}

impl Default for CurrencyRates {
    fn default() -> Self {
        Self::from_pairs([])
    }
}

impl CurrencyRates {
    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, f64)>) -> Self {
        let mut chaos_per_unit: HashMap<String, f64> = pairs
            .into_iter()
            .filter(|(_, v)| v.is_finite() && *v > 0.0)
            .collect();
        chaos_per_unit.insert("chaos".into(), 1.0);
        Self { chaos_per_unit }
    }

    /// Cotação de reserva: só entra se a moeda ainda não tem cotação (ex.: divine do índice
    /// quando o endpoint de moedas do poe.ninja falhou).
    #[must_use]
    pub fn with_fallback(mut self, tag: &str, chaos_per_unit: f64) -> Self {
        if chaos_per_unit.is_finite() && chaos_per_unit > 0.0 {
            self.chaos_per_unit.entry(tag.to_owned()).or_insert(chaos_per_unit);
        }
        self
    }

    pub fn chaos_per(&self, currency: &Currency) -> Option<f64> {
        self.chaos_per_unit.get(&currency.0).copied()
    }

    /// Converte um preço para chaos; `None` se a moeda não tem cotação conhecida.
    pub fn to_chaos(&self, price: &Price) -> Option<f64> {
        self.chaos_per(&price.currency).map(|rate| price.amount * rate)
    }
}

/// Número de faixas do histograma de distribuição de preços.
pub const HISTOGRAM_BUCKETS: usize = 12;
/// Abaixo disso a amostra é pequena demais para sugerir preço (Compra/Justo/Venda).
pub const MIN_SUGGEST_SAMPLE: usize = 5;
/// Quantas listagens mais baratas entram na estimativa.
const ESTIMATE_WINDOW: usize = 10;

/// Resumo estatístico das listagens, tudo em chaos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceSummary {
    /// Mediana das listagens mais baratas: robusta contra price-fixing e outliers caros.
    pub estimate_chaos: f64,
    pub min_chaos: f64,
    pub median_chaos: f64,
    pub max_chaos: f64,
    /// Listagens consideradas (com moeda conhecida).
    pub sample_size: u32,
    /// Contagem por faixa entre `min_chaos` e `max_chaos`.
    pub histogram: Vec<u32>,
    /// 0 (nenhuma) a 3 (alta), conforme o tamanho da amostra.
    pub confidence: u8,
    /// Cotação do divine em chaos, para exibir equivalências.
    pub divine_chaos: Option<f64>,
    /// Preço sugerido em três níveis; `None` com menos de [`MIN_SUGGEST_SAMPLE`] listagens.
    #[serde(default)]
    pub suggested: Option<SuggestedPrice>,
}

/// Percentis das listagens convertidas para chaos: Compra = p25, Justo = p50, Venda = p75.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SuggestedPrice {
    pub buy_chaos: f64,
    pub fair_chaos: f64,
    pub sell_chaos: f64,
}

impl SuggestedPrice {
    /// `sorted` precisa estar ordenado. `None` se a amostra for pequena demais.
    pub fn from_sorted(sorted: &[f64]) -> Option<Self> {
        (sorted.len() >= MIN_SUGGEST_SAMPLE).then(|| Self {
            buy_chaos: percentile(sorted, 0.25),
            fair_chaos: percentile(sorted, 0.5),
            sell_chaos: percentile(sorted, 0.75),
        })
    }
}

/// Calcula o resumo de preço. `None` quando nenhuma listagem tem moeda conhecida.
pub fn summarize(listings: &[Listing], rates: &CurrencyRates) -> Option<PriceSummary> {
    let mut values: Vec<f64> = listings
        .iter()
        .filter_map(|l| rates.to_chaos(&l.price))
        .filter(|v| v.is_finite() && *v >= 0.0)
        .collect();
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);

    let min = values[0];
    let max = values[values.len() - 1];
    let window = &values[..values.len().min(ESTIMATE_WINDOW)];

    Some(PriceSummary {
        estimate_chaos: median(window),
        min_chaos: min,
        median_chaos: median(&values),
        max_chaos: max,
        sample_size: u32::try_from(values.len()).unwrap_or(u32::MAX),
        histogram: histogram(&values, min, max),
        confidence: confidence(values.len()),
        divine_chaos: rates.chaos_per(&Currency("divine".into())),
        suggested: SuggestedPrice::from_sorted(&values),
    })
}

/// Percentil `q` (0–1) com interpolação linear (rank = q·(n−1)).
/// `sorted` precisa estar ordenado e não vazio; para `q = 0.5` coincide com a mediana.
pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    let last = sorted.len().saturating_sub(1);
    #[allow(clippy::cast_precision_loss)]
    let rank = q.clamp(0.0, 1.0) * last as f64;
    // `rank` está em [0, last]: os casts são seguros.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lo = rank.floor() as usize;
    let hi = (lo + 1).min(last);
    #[allow(clippy::cast_precision_loss)]
    let frac = rank - lo as f64;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

/// `values` precisa estar ordenado e não vazio.
fn median(values: &[f64]) -> f64 {
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn histogram(values: &[f64], min: f64, max: f64) -> Vec<u32> {
    let mut buckets = vec![0_u32; HISTOGRAM_BUCKETS];
    let span = max - min;
    for v in values {
        let idx = if span <= f64::EPSILON {
            0
        } else {
            // Índice sempre em [0, BUCKETS-1]; o cast é seguro após o clamp.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let raw = (((v - min) / span) * HISTOGRAM_BUCKETS as f64).floor() as usize;
            raw.min(HISTOGRAM_BUCKETS - 1)
        };
        buckets[idx] += 1;
    }
    buckets
}

const fn confidence(sample: usize) -> u8 {
    match sample {
        0..=2 => 0,
        3..=7 => 1,
        8..=19 => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(amount: f64, currency: &str) -> Listing {
        Listing {
            price: Price {
                amount,
                currency: Currency(currency.into()),
            },
            stack_size: None,
            account: "acc".into(),
            online: true,
            indexed_at: String::new(),
            item: None,
            whisper: None,
            icon: None,
        }
    }

    fn rates() -> CurrencyRates {
        CurrencyRates::from_pairs([("divine".into(), 200.0)])
    }

    #[test]
    fn empty_or_unknown_currency_yields_none() {
        assert!(summarize(&[], &rates()).is_none());
        assert!(summarize(&[listing(1.0, "mystery")], &rates()).is_none());
    }

    #[test]
    fn converts_currencies_and_computes_stats() {
        let listings = [
            listing(40.0, "chaos"),
            listing(0.25, "divine"), // 50c
            listing(60.0, "chaos"),
            listing(3.0, "mystery"), // ignorada
        ];
        let s = summarize(&listings, &rates()).unwrap();
        assert_eq!(s.sample_size, 3);
        assert_eq!(s.min_chaos, 40.0);
        assert_eq!(s.max_chaos, 60.0);
        assert_eq!(s.median_chaos, 50.0);
        assert_eq!(s.estimate_chaos, 50.0);
        assert_eq!(s.histogram.iter().sum::<u32>(), 3);
        assert_eq!(s.divine_chaos, Some(200.0));
        assert_eq!(s.confidence, 1);
    }

    #[test]
    fn estimate_ignores_expensive_tail() {
        let mut listings: Vec<Listing> = (0..10).map(|i| listing(10.0 + f64::from(i), "chaos")).collect();
        listings.extend((0..20).map(|_| listing(500.0, "chaos")));
        let s = summarize(&listings, &rates()).unwrap();
        assert!(s.estimate_chaos < 20.0, "estimativa {}", s.estimate_chaos);
        assert_eq!(s.confidence, 3);
    }

    #[test]
    fn percentile_of_known_sample() {
        let v = [10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&v, 0.25), 20.0);
        assert_eq!(percentile(&v, 0.5), 30.0);
        assert_eq!(percentile(&v, 0.75), 40.0);
        assert_eq!(percentile(&v, 0.0), 10.0);
        assert_eq!(percentile(&v, 1.0), 50.0);
    }

    #[test]
    fn percentile_interpolates_with_even_sample() {
        let v = [10.0, 20.0, 30.0, 40.0];
        // rank 0.75 → 10 + 0.75 · 10
        assert_eq!(percentile(&v, 0.25), 17.5);
        assert_eq!(percentile(&v, 0.5), median(&v));
        assert_eq!(percentile(&v, 0.75), 32.5);
    }

    #[test]
    fn suggested_none_below_5() {
        let listings: Vec<Listing> = (1..=4).map(|i| listing(f64::from(i) * 10.0, "chaos")).collect();
        assert!(summarize(&listings, &rates()).unwrap().suggested.is_none());
    }

    #[test]
    fn suggested_converts_currencies_first() {
        let listings = [
            listing(0.5, "divine"), // 100c
            listing(20.0, "chaos"),
            listing(0.1, "divine"), // 20c
            listing(60.0, "chaos"),
            listing(40.0, "chaos"),
        ];
        // Ordenado em chaos: 20, 20, 40, 60, 100.
        let s = summarize(&listings, &rates()).unwrap().suggested.unwrap();
        assert_eq!(s.buy_chaos, 20.0);
        assert_eq!(s.fair_chaos, 40.0);
        assert_eq!(s.sell_chaos, 60.0);
    }

    #[test]
    fn fallback_rate_only_fills_missing_currency() {
        let known = rates().with_fallback("divine", 150.0);
        assert_eq!(known.chaos_per(&Currency("divine".into())), Some(200.0));
        let missing = CurrencyRates::default().with_fallback("divine", 150.0).with_fallback("exalted", f64::NAN);
        assert_eq!(missing.chaos_per(&Currency("divine".into())), Some(150.0));
        assert_eq!(missing.chaos_per(&Currency("exalted".into())), None);
    }

    #[test]
    fn identical_prices_go_to_first_bucket() {
        let s = summarize(&[listing(5.0, "chaos"), listing(5.0, "chaos")], &rates()).unwrap();
        assert_eq!(s.histogram[0], 2);
    }
}
