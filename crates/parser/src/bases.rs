//! Bases que a trade API identifica por `type` + `discriminator` (gemas transfiguradas).
//! Tabelas geradas por `node tools/stats/update-stats.mjs` a partir de `/api/trade/data/items`.

use std::collections::HashMap;
use std::sync::OnceLock;

use mirrorsight_domain::GameVersion;

const POE1_TSV: &str = include_str!("../data/bases-poe1.tsv");
const POE2_TSV: &str = include_str!("../data/bases-poe2.tsv");

/// Texto exibido no jogo → (type, discriminator).
type Table = HashMap<&'static str, (&'static str, &'static str)>;

fn load(tsv: &'static str) -> Table {
    tsv.lines()
        .filter_map(|line| {
            let mut cols = line.split('\t');
            Some((cols.next()?, (cols.next()?, cols.next()?)))
        })
        .collect()
}

fn table(game: GameVersion) -> &'static Table {
    static POE1: OnceLock<Table> = OnceLock::new();
    static POE2: OnceLock<Table> = OnceLock::new();
    match game {
        GameVersion::Poe1 => POE1.get_or_init(|| load(POE1_TSV)),
        GameVersion::Poe2 => POE2.get_or_init(|| load(POE2_TSV)),
    }
}

/// `"Kinetic Blast of Clustering"` → `Some(("Kinetic Blast", "alt_x"))`.
pub fn discriminated(game: GameVersion, text: &str) -> Option<(&'static str, &'static str)> {
    table(game).get(text).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_transfigured_gem() {
        assert_eq!(
            discriminated(GameVersion::Poe1, "Kinetic Blast of Clustering"),
            Some(("Kinetic Blast", "alt_x"))
        );
        assert_eq!(discriminated(GameVersion::Poe1, "Kinetic Blast"), None);
    }
}
