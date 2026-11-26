//! Rate limiter guiado pelos headers da GGG.
//!
//! A trade API informa as regras em `X-Rate-Limit-Ip: 5:10:60,15:60:300,...`
//! (`máx:janela_s:bloqueio_s`) e o estado atual em `X-Rate-Limit-Ip-State: 1:10:0,...`
//! (`hits:janela_s:bloqueio_restante_s`). Estourar uma regra bloqueia o IP pelo tempo
//! indicado — por isso recusamos *antes* de enviar quando a janela local está cheia.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rule {
    max: u32,
    period: Duration,
}

#[derive(Debug, Default)]
struct Bucket {
    rules: Vec<Rule>,
    hits: VecDeque<Instant>,
    blocked_until: Option<Instant>,
}

/// Limites independentes por endpoint ("search", "fetch"…).
#[derive(Debug, Default)]
pub struct RateLimiter {
    buckets: Mutex<HashMap<&'static str, Bucket>>,
}

impl RateLimiter {
    /// Reserva uma requisição. `Err(espera)` se enviar agora arriscaria estourar o limite.
    pub fn try_acquire(&self, key: &'static str, now: Instant) -> Result<(), Duration> {
        let mut buckets = self.lock();
        let bucket = buckets.entry(key).or_default();

        if let Some(until) = bucket.blocked_until {
            if until > now {
                return Err(until - now);
            }
            bucket.blocked_until = None;
        }

        let longest = bucket.rules.iter().map(|r| r.period).max().unwrap_or(Duration::ZERO);
        while bucket.hits.front().is_some_and(|t| now.duration_since(*t) >= longest) {
            bucket.hits.pop_front();
        }

        for rule in &bucket.rules {
            let in_window: Vec<&Instant> = bucket
                .hits
                .iter()
                .filter(|t| now.duration_since(**t) < rule.period)
                .collect();
            // Margem de 1 requisição: o servidor conta também o que o site/outros apps do IP fizeram.
            let budget = rule.max.saturating_sub(1).max(1);
            if u32::try_from(in_window.len()).unwrap_or(u32::MAX) >= budget {
                let oldest = in_window.first().map_or(now, |t| **t);
                return Err((oldest + rule.period).saturating_duration_since(now));
            }
        }

        bucket.hits.push_back(now);
        Ok(())
    }

    /// Atualiza regras e estado a partir dos headers de uma resposta.
    pub fn update(&self, key: &'static str, rules: Option<&str>, state: Option<&str>, now: Instant) {
        let mut buckets = self.lock();
        let bucket = buckets.entry(key).or_default();

        if let Some(parsed) = rules.map(parse_triples).filter(|r| !r.is_empty()) {
            bucket.rules = parsed
                .iter()
                .map(|&(max, period, _)| Rule { max, period: Duration::from_secs(u64::from(period)) })
                .collect();
        }

        let Some(state) = state.map(parse_triples) else { return };
        for (i, &(hits, period, restricted)) in state.iter().enumerate() {
            if restricted > 0 {
                block_until(bucket, now + Duration::from_secs(u64::from(restricted)));
            } else if bucket.rules.get(i).is_some_and(|r| hits >= r.max) {
                // Servidor já vê a janela cheia (outros clientes no mesmo IP): espera a janela.
                block_until(bucket, now + Duration::from_secs(u64::from(period)));
            }
        }
    }

    /// Bloqueio explícito (resposta 429 com `Retry-After`).
    pub fn block(&self, key: &'static str, duration: Duration, now: Instant) {
        let mut buckets = self.lock();
        block_until(buckets.entry(key).or_default(), now + duration);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<&'static str, Bucket>> {
        // Um panic com o lock seguro não corrompe contadores: recupera o estado.
        self.buckets.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn block_until(bucket: &mut Bucket, until: Instant) {
    if bucket.blocked_until.is_none_or(|current| current < until) {
        bucket.blocked_until = Some(until);
    }
}

/// "5:10:60,15:60:300" → [(5,10,60), (15,60,300)]. Entradas malformadas são ignoradas.
fn parse_triples(raw: &str) -> Vec<(u32, u32, u32)> {
    raw.split(',')
        .filter_map(|part| {
            let mut it = part.trim().split(':').map(|n| n.parse::<u32>().ok());
            Some((it.next()??, it.next()??, it.next()??))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = "5:10:60,15:60:300";

    #[test]
    fn parses_header_triples() {
        assert_eq!(parse_triples("5:10:60, 15:60:300,lixo"), vec![(5, 10, 60), (15, 60, 300)]);
    }

    #[test]
    fn allows_until_budget_then_waits_for_window() {
        let limiter = RateLimiter::default();
        let t0 = Instant::now();
        limiter.update("search", Some(RULES), None, t0);

        // Orçamento = máx - 1 = 4 na janela de 10 s.
        for i in 0..4 {
            assert!(limiter.try_acquire("search", t0 + Duration::from_millis(i)).is_ok());
        }
        let wait = limiter.try_acquire("search", t0 + Duration::from_secs(1)).unwrap_err();
        assert!(wait <= Duration::from_secs(9) && wait > Duration::from_secs(8), "{wait:?}");

        // Depois da janela, libera de novo.
        assert!(limiter.try_acquire("search", t0 + Duration::from_secs(11)).is_ok());
    }

    #[test]
    fn server_restriction_blocks() {
        let limiter = RateLimiter::default();
        let t0 = Instant::now();
        limiter.update("search", Some(RULES), Some("5:10:60,5:60:0"), t0);
        let wait = limiter.try_acquire("search", t0).unwrap_err();
        assert_eq!(wait, Duration::from_secs(60));
    }

    #[test]
    fn explicit_block_from_429() {
        let limiter = RateLimiter::default();
        let t0 = Instant::now();
        limiter.block("fetch", Duration::from_secs(30), t0);
        assert!(limiter.try_acquire("fetch", t0 + Duration::from_secs(29)).is_err());
        assert!(limiter.try_acquire("fetch", t0 + Duration::from_secs(31)).is_ok());
    }

    #[test]
    fn endpoints_are_independent() {
        let limiter = RateLimiter::default();
        let t0 = Instant::now();
        limiter.block("search", Duration::from_secs(30), t0);
        assert!(limiter.try_acquire("fetch", t0).is_ok());
    }
}
