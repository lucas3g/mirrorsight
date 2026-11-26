//! Tokenização de uma linha de mod: separa marcadores, números e faixas de roll.
//!
//! Aceita os dois formatos do jogo:
//! - normal (Ctrl+C):          `+78 to maximum Life (fractured)`
//! - avançado (Ctrl+Alt+C):    `+78(70-79) to maximum Life (fractured)`

use mirrorsight_domain::item::{ModifierKind, StatRoll};

/// Sufixos que o jogo acrescenta à linha indicando a origem do mod.
const KIND_MARKERS: &[(&str, ModifierKind)] = &[
    (" (implicit)", ModifierKind::Implicit),
    (" (crafted)", ModifierKind::Crafted),
    (" (fractured)", ModifierKind::Fractured),
    (" (enchant)", ModifierKind::Enchant),
    (" (rune)", ModifierKind::Rune),
    (" (desecrated)", ModifierKind::Desecrated),
];

/// Anotações sem efeito no stat (modo avançado).
const NOISE_SUFFIXES: &[&str] = &[" — Unscalable Value", " (augmented)"];

/// Remove marcadores do fim da linha e devolve o tipo indicado, se houver.
pub fn strip_markers(line: &str) -> (&str, Option<ModifierKind>) {
    let mut text = line.trim();
    let mut kind = None;
    loop {
        if let Some((rest, k)) = KIND_MARKERS
            .iter()
            .find_map(|(suffix, k)| text.strip_suffix(suffix).map(|rest| (rest, *k)))
        {
            text = rest.trim_end();
            kind = kind.or(Some(k));
            continue;
        }
        if let Some(rest) = NOISE_SUFFIXES.iter().find_map(|s| text.strip_suffix(s)) {
            text = rest.trim_end();
            continue;
        }
        return (text, kind);
    }
}

/// Resultado da tokenização.
#[derive(Debug, Clone, PartialEq)]
pub struct Tokenized {
    /// Texto com cada número trocado por `#` (sinais mantidos), ex.: `+# to maximum Life`.
    pub template: String,
    /// Texto para exibição: números mantidos, faixas `(70-79)` removidas.
    pub display: String,
    pub values: Vec<f64>,
    /// Faixa de cada número, quando o item veio do modo avançado.
    pub ranges: Vec<Option<(f64, f64)>>,
}

impl Tokenized {
    /// Roll agregado (média, como a trade API faz para stats com vários números).
    pub fn roll(&self) -> Option<StatRoll> {
        if self.values.is_empty() {
            return None;
        }
        let count = self.values.len() as f64;
        let value = self.values.iter().sum::<f64>() / count;
        let ranges: Option<Vec<(f64, f64)>> = self.ranges.iter().copied().collect();
        let (min, max) = match ranges {
            Some(r) if !r.is_empty() => (
                Some(r.iter().map(|(lo, _)| lo).sum::<f64>() / count),
                Some(r.iter().map(|(_, hi)| hi).sum::<f64>() / count),
            ),
            _ => (None, None),
        };
        Some(StatRoll { value, min, max })
    }
}

/// Troca números por `#`, coletando valores e faixas.
pub fn tokenize(text: &str) -> Tokenized {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Tokenized {
        template: String::with_capacity(text.len()),
        display: String::with_capacity(text.len()),
        values: Vec::new(),
        ranges: Vec::new(),
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let starts_number = c.is_ascii_digit() && (i == 0 || !is_word_char(chars[i - 1]));
        if !starts_number {
            out.template.push(c);
            out.display.push(c);
            i += 1;
            continue;
        }

        let (number, end) = read_number(&chars, i);
        // Um "-" colado antes do número (e não parte de uma palavra) é o sinal.
        let negative = i > 0 && chars[i - 1] == '-' && (i < 2 || !is_word_char(chars[i - 2]));
        out.values.push(if negative { -number } else { number });
        out.template.push('#');
        out.display.extend(&chars[i..end]);

        let (range, after_range) = read_range(&chars, end);
        out.ranges.push(range);
        i = after_range;
    }
    out
}

const fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '#'
}

/// Lê `123` ou `1.5` a partir de `start`. Retorna (valor, índice após o número).
fn read_number(chars: &[char], start: usize) -> (f64, usize) {
    let mut end = start;
    while end < chars.len() && chars[end].is_ascii_digit() {
        end += 1;
    }
    if end + 1 < chars.len() && chars[end] == '.' && chars[end + 1].is_ascii_digit() {
        end += 1;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
    }
    let raw: String = chars[start..end].iter().collect();
    (raw.parse().unwrap_or(0.0), end)
}

/// Lê uma faixa `(lo-hi)` (aceita negativos: `(-15--10)`) logo após o número.
fn read_range(chars: &[char], start: usize) -> (Option<(f64, f64)>, usize) {
    if chars.get(start) != Some(&'(') {
        return (None, start);
    }
    let Some(close) = chars[start..].iter().position(|c| *c == ')').map(|p| start + p) else {
        return (None, start);
    };
    let inner: String = chars[start + 1..close].iter().collect();
    match parse_range(&inner) {
        Some(range) => (Some(range), close + 1),
        None => (None, start),
    }
}

fn parse_range(inner: &str) -> Option<(f64, f64)> {
    // O separador é o primeiro "-" que não está no início (o do início é sinal do mínimo).
    let split = inner.char_indices().skip(1).find(|(_, c)| *c == '-')?.0;
    let lo: f64 = inner[..split].trim().parse().ok()?;
    let hi: f64 = inner[split + 1..].trim().parse().ok()?;
    Some((lo.min(hi), lo.max(hi)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_kind_and_noise_markers() {
        assert_eq!(strip_markers("+78 to maximum Life (fractured)"), ("+78 to maximum Life", Some(ModifierKind::Fractured)));
        assert_eq!(strip_markers("+16% to Chaos Resistance (crafted)"), ("+16% to Chaos Resistance", Some(ModifierKind::Crafted)));
        assert_eq!(strip_markers("+45% to Fire Resistance"), ("+45% to Fire Resistance", None));
    }

    #[test]
    fn tokenizes_simple_line() {
        let t = tokenize("+78 to maximum Life");
        assert_eq!(t.template, "+# to maximum Life");
        assert_eq!(t.values, vec![78.0]);
        assert_eq!(t.roll().unwrap().min, None);
    }

    #[test]
    fn tokenizes_advanced_ranges_and_averages() {
        let t = tokenize("Adds 12(9-15) to 21(18-24) Physical Damage to Attacks");
        assert_eq!(t.template, "Adds # to # Physical Damage to Attacks");
        assert_eq!(t.display, "Adds 12 to 21 Physical Damage to Attacks");
        let roll = t.roll().unwrap();
        assert_eq!(roll.value, 16.5);
        assert_eq!(roll.min, Some(13.5));
        assert_eq!(roll.max, Some(19.5));
    }

    #[test]
    fn handles_negative_values_and_ranges() {
        let t = tokenize("-12(-15--10)% to Fire Resistance");
        assert_eq!(t.template, "-#% to Fire Resistance");
        assert_eq!(t.values, vec![-12.0]);
        assert_eq!(t.ranges, vec![Some((-15.0, -10.0))]);
    }

    #[test]
    fn decimals_and_words_with_digits() {
        let t = tokenize("1.5% of Physical Attack Damage Leeched as Life");
        assert_eq!(t.template, "#% of Physical Attack Damage Leeched as Life");
        assert_eq!(t.values, vec![1.5]);
    }
}
