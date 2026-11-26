//! Utilidades de stat: chave de template, hash da trade e posição na faixa.

use super::knowledge::Range;

/// Chave normalizada de um texto de mod: números e faixas viram `#`, sinais somem, minúsculas.
/// "+(40-55) to Strength" e "+48 to Strength" → "# to strength".
/// Igual ao `templateKey` de `tools/gamedata/update-gamedata.mjs`.
pub fn template_key(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Faixa "(a-b)" (aceita negativos e decimais).
        if c == '(' {
            if let Some(end) = range_end(&chars, i) {
                out.push('#');
                i = end;
                continue;
            }
        }
        let prev_word = i > 0 && (chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '#');
        if c.is_ascii_digit() && !prev_word {
            while i < chars.len() && (chars[i].is_ascii_digit() || (chars[i] == '.' && next_is_digit(&chars, i))) {
                i += 1;
            }
            out.push('#');
            continue;
        }
        out.push(c);
        i += 1;
    }
    // Sinais colados ao número não distinguem o stat ("+#" ≡ "-#" ≡ "#").
    let unsigned = out.replace("+#", "#").replace("-#", "#");
    unsigned.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn next_is_digit(chars: &[char], i: usize) -> bool {
    chars.get(i + 1).is_some_and(char::is_ascii_digit)
}

/// Se `chars[start..]` começa com "(a-b)", devolve o índice após o ")".
fn range_end(chars: &[char], start: usize) -> Option<usize> {
    let close = chars[start..].iter().position(|c| *c == ')')? + start;
    let inner: String = chars[start + 1..close].iter().collect();
    let split = inner.char_indices().skip(1).find(|(_, c)| *c == '-')?.0;
    inner[..split].trim().parse::<f64>().ok()?;
    inner[split + 1..].trim().parse::<f64>().ok()?;
    Some(close + 1)
}

/// "explicit.stat_3299347043" → "3299347043". Pseudo e propriedades não têm hash.
pub fn stat_hash(trade_id: &str) -> Option<&str> {
    let (group, rest) = trade_id.split_once('.')?;
    if matches!(group, "pseudo" | "property") {
        return None;
    }
    rest.strip_prefix("stat_")
}

/// Posição de `value` na faixa, de 0 a 1. `None` se a faixa não tem amplitude (mod fixo).
pub fn percentile(value: f64, (min, max): Range) -> Option<f64> {
    let span = max - min;
    if span.abs() <= f64::EPSILON {
        None
    } else {
        Some(((value - min) / span).clamp(0.0, 1.0))
    }
}

/// O valor (ou o seu oposto — "reduced" vira negativo no parser) cabe na faixa?
/// Devolve o valor no sentido da faixa. Tolera arredondamento de 0.5.
pub fn fit(value: f64, (min, max): Range) -> Option<f64> {
    const TOLERANCE: f64 = 0.5;
    let inside = |v: f64| v >= min - TOLERANCE && v <= max + TOLERANCE;
    if inside(value) {
        Some(value)
    } else if inside(-value) {
        Some(-value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_key_normalizes_ranges_numbers_and_signs() {
        assert_eq!(template_key("+(40-55) to Strength"), "# to strength");
        assert_eq!(template_key("+48 to Strength"), "# to strength");
        assert_eq!(template_key("(-15--10)% to Fire Resistance"), "#% to fire resistance");
        assert_eq!(template_key("-12% to Fire Resistance"), "#% to fire resistance");
        assert_eq!(template_key("Adds (10-15) to (20-25) Fire Damage"), "adds # to # fire damage");
        assert_eq!(template_key("Adds 12 to 21 Fire Damage"), "adds # to # fire damage");
        assert_eq!(template_key("(1-1.5)% of Fire Damage Leeched as Life"), "#% of fire damage leeched as life");
        assert_eq!(template_key("1.2% of Fire Damage Leeched as Life"), "#% of fire damage leeched as life");
        // Dígitos dentro de palavras não são números.
        assert_eq!(template_key("Grants Level 20 Summon Arbalists"), "grants level # summon arbalists");
    }

    #[test]
    fn stat_hash_from_trade_id() {
        assert_eq!(stat_hash("explicit.stat_3299347043"), Some("3299347043"));
        assert_eq!(stat_hash("crafted.stat_1"), Some("1"));
        assert_eq!(stat_hash("pseudo.pseudo_total_life"), None);
        assert_eq!(stat_hash("property.es"), None);
    }

    #[test]
    fn percentile_and_fit() {
        assert_eq!(percentile(75.0, (70.0, 80.0)), Some(0.5));
        assert_eq!(percentile(1.0, (1.0, 1.0)), None);
        assert_eq!(fit(-15.0, (10.0, 20.0)), Some(15.0));
        assert_eq!(fit(30.0, (10.0, 20.0)), None);
        assert_eq!(fit(20.4, (10.0, 20.0)), Some(20.4));
    }
}
