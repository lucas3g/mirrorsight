//! Parsers de seção. Cada um tenta consumir uma seção inteira; o primeiro que
//! retorna [`SectionOutcome::Parsed`] vence.

use mirrorsight_domain::item::{Affix, AffixSlots, Influence, Item, ModifierKind};

use crate::catalog::StatCatalog;
use crate::matcher::{ModContext, StatMatcher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionOutcome {
    Parsed,
    Skipped,
}

pub trait SectionParser: Send + Sync {
    fn parse(&self, section: &[&str], item: &mut Item) -> SectionOutcome;
}

/// Ordem importa: parsers mais específicos primeiro; mods por último.
pub fn default_pipeline(catalog: &'static StatCatalog) -> Vec<Box<dyn SectionParser>> {
    vec![
        Box::new(ItemLevelParser),
        Box::new(FlagParser),
        Box::new(RequirementsParser),
        Box::new(SocketsParser),
        Box::new(PropertiesParser),
        Box::new(ModifiersParser { catalog }),
    ]
}

struct ItemLevelParser;
impl SectionParser for ItemLevelParser {
    fn parse(&self, section: &[&str], item: &mut Item) -> SectionOutcome {
        match section {
            [line] => match line.strip_prefix("Item Level: ").and_then(|v| v.trim().parse().ok()) {
                Some(level) => {
                    item.item_level = Some(level);
                    SectionOutcome::Parsed
                }
                None => SectionOutcome::Skipped,
            },
            _ => SectionOutcome::Skipped,
        }
    }
}

/// Seções formadas só por flags: Corrupted, Unidentified, influências, "Fractured Item"…
struct FlagParser;

const INFLUENCE_LINES: &[(&str, Influence)] = &[
    ("Shaper Item", Influence::Shaper),
    ("Elder Item", Influence::Elder),
    ("Crusader Item", Influence::Crusader),
    ("Hunter Item", Influence::Hunter),
    ("Redeemer Item", Influence::Redeemer),
    ("Warlord Item", Influence::Warlord),
    ("Searing Exarch Item", Influence::Searing),
    ("Eater of Worlds Item", Influence::Eater),
];

/// Flags reconhecidas sem efeito no modelo (o tipo do mod já indica fractured etc.).
const INERT_FLAGS: &[&str] = &["Fractured Item", "Synthesised Item", "Mirrored", "Split"];

impl SectionParser for FlagParser {
    fn parse(&self, section: &[&str], item: &mut Item) -> SectionOutcome {
        let known = |l: &&str| {
            matches!(*l, "Corrupted" | "Unidentified")
                || INERT_FLAGS.contains(l)
                || INFLUENCE_LINES.iter().any(|(text, _)| text == l)
        };
        if !section.iter().all(known) {
            return SectionOutcome::Skipped;
        }
        for line in section {
            match *line {
                "Corrupted" => item.corrupted = true,
                "Unidentified" => item.identified = false,
                other => {
                    if let Some((_, influence)) = INFLUENCE_LINES.iter().find(|(t, _)| *t == other) {
                        item.influences.push(*influence);
                    }
                }
            }
        }
        SectionOutcome::Parsed
    }
}

/// Requisitos não são usados no price check; apenas consumimos a seção.
struct RequirementsParser;
impl SectionParser for RequirementsParser {
    fn parse(&self, section: &[&str], _item: &mut Item) -> SectionOutcome {
        match section.first() {
            Some(l) if l.starts_with("Requirements") || l.starts_with("Requires") => SectionOutcome::Parsed,
            _ => SectionOutcome::Skipped,
        }
    }
}

struct SocketsParser;
impl SectionParser for SocketsParser {
    fn parse(&self, section: &[&str], item: &mut Item) -> SectionOutcome {
        match section {
            [line] => match line.strip_prefix("Sockets: ") {
                Some(sockets) => {
                    item.sockets = Some(sockets.trim().to_owned());
                    SectionOutcome::Parsed
                }
                None => SectionOutcome::Skipped,
            },
            _ => SectionOutcome::Skipped,
        }
    }
}

/// Bloco "Chave: valor" (defesas, dano, qualidade, stack size…). Extrai a qualidade.
/// Armas começam com uma linha sem ":" (o tipo da arma), aceita só na primeira posição.
struct PropertiesParser;
impl SectionParser for PropertiesParser {
    fn parse(&self, section: &[&str], item: &mut Item) -> SectionOutcome {
        let is_property = |l: &str| l.split_once(": ").is_some_and(|(k, _)| !k.is_empty() && k.len() <= 40);
        let body = match section {
            [first, rest @ ..] if !is_property(first) && !rest.is_empty() => rest,
            all => all,
        };
        if body.is_empty() || !body.iter().all(|l| is_property(l)) {
            return SectionOutcome::Skipped;
        }
        for line in body {
            if let Some(raw) = line.strip_prefix("Quality: ") {
                item.quality = leading_number(raw);
            } else if let Some(raw) = line.strip_prefix("Item Level: ") {
                item.item_level = leading_number(raw);
            }
        }
        item.properties.extend(crate::properties::extract(body));
        SectionOutcome::Parsed
    }
}

/// "+20% (augmented)" → 20
fn leading_number(raw: &str) -> Option<u32> {
    let digits: String = raw.chars().skip_while(|c| !c.is_ascii_digit()).take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Linhas de mod (normal ou avançado). Consome a seção se ao menos um mod casar;
/// linhas que não casam vão para `unknown_lines`.
struct ModifiersParser {
    catalog: &'static StatCatalog,
}

impl SectionParser for ModifiersParser {
    fn parse(&self, section: &[&str], item: &mut Item) -> SectionOutcome {
        let matcher = StatMatcher::new(self.catalog);
        let mut ctx = ModContext::default();
        let mut found = Vec::new();
        let mut unmatched = Vec::new();
        // Slots contados por cabeçalho, não por linha: um mod híbrido ocupa um slot só.
        let mut slots: Option<AffixSlots> = None;
        let mut i = 0;

        while i < section.len() {
            let line = section[i];
            if let Some(header) = advanced_header(line) {
                if let Some(affix) = header.affix {
                    slots.get_or_insert_with(AffixSlots::default).add(affix);
                }
                ctx = header;
                i += 1;
                continue;
            }
            if is_reminder(line) {
                i += 1;
                continue;
            }
            match matcher.match_at(&section[i..], ctx, &item.item_class) {
                Some(m) => {
                    found.push(m.modifier);
                    i += m.lines;
                }
                None => {
                    unmatched.push(line.to_owned());
                    i += 1;
                }
            }
        }

        if found.is_empty() {
            return SectionOutcome::Skipped;
        }
        item.modifiers.extend(found);
        item.unknown_lines.extend(unmatched);
        if let Some(new) = slots {
            let total = item.affix_slots.get_or_insert_with(AffixSlots::default);
            total.prefixes += new.prefixes;
            total.suffixes += new.suffixes;
        }
        SectionOutcome::Parsed
    }
}

/// `{ Prefix Modifier "Fecund" (Tier: 2) — Life }` → contexto (tipo + tier + afixo).
/// "Master Crafted Suffix Modifier" também conta: o mod de bancada ocupa um slot.
fn advanced_header(line: &str) -> Option<ModContext> {
    let inner = line.strip_prefix('{')?.strip_suffix('}')?.trim();
    let kind = if inner.contains("Implicit") {
        Some(ModifierKind::Implicit)
    } else if inner.contains("Crafted") {
        Some(ModifierKind::Crafted)
    } else if inner.contains("Desecrated") {
        Some(ModifierKind::Desecrated)
    } else {
        None
    };
    let tier = inner
        .split_once("(Tier: ")
        .and_then(|(_, rest)| rest.split(')').next())
        .and_then(|t| t.trim().parse().ok());
    let affix = if inner.contains("Prefix Modifier") {
        Some(Affix::Prefix)
    } else if inner.contains("Suffix Modifier") {
        Some(Affix::Suffix)
    } else {
        None
    };
    Some(ModContext { kind, tier, affix })
}

/// Texto explicativo entre parênteses no modo avançado, ex.: "(Leech ... )".
fn is_reminder(line: &str) -> bool {
    line.starts_with('(') && line.ends_with(')')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_advanced_header() {
        let ctx = advanced_header(r#"{ Prefix Modifier "Fecund" (Tier: 2) — Life }"#).unwrap();
        assert_eq!(ctx, ModContext { kind: None, tier: Some(2), affix: Some(Affix::Prefix) });
        let ctx = advanced_header(r#"{ Master Crafted Suffix Modifier "of Craft" (Rank: 3) — Chaos, Resistance }"#).unwrap();
        assert_eq!(ctx.kind, Some(ModifierKind::Crafted));
        assert_eq!(ctx.affix, Some(Affix::Suffix));
        assert_eq!(ctx.tier, None);
        let implicit = advanced_header("{ Implicit Modifier — Attribute }").unwrap();
        assert_eq!(implicit.kind, Some(ModifierKind::Implicit));
        assert_eq!(implicit.affix, None);
        assert!(advanced_header("+78 to maximum Life").is_none());
    }

    #[test]
    fn reads_quality_number() {
        assert_eq!(leading_number("+20% (augmented)"), Some(20));
    }
}
