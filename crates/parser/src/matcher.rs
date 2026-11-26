//! Casamento de linhas de mod com stats da trade API.
//!
//! Para cada linha (ou bloco de até [`MAX_SPAN`] linhas, para stats multi-linha)
//! gera variantes do template e procura no catálogo, na ordem:
//!
//! 1. texto exato;
//! 2. `-#` → `+#`;
//! 3. sem sinal (`+#`/`-#` → `#`), como o PoE2 escreve seus templates;
//! 4. `reduced`→`increased` / `less`→`more` (valor negado).
//!
//! Cada uma também com o sufixo ` (Local)` — testado primeiro em armas/armaduras.

use mirrorsight_domain::item::{Affix, ModOrigin, Modifier, ModifierKind, StatRoll};

use crate::catalog::StatCatalog;
use crate::mod_line::{strip_markers, tokenize};

/// Maior número de linhas que um único stat ocupa no texto do item.
const MAX_SPAN: usize = 3;

/// Classes cujos mods de defesa/dano são "locais" (aplicam ao próprio item).
const LOCAL_CLASS_HINTS: &[&str] = &[
    "Sword", "Axe", "Mace", "Bow", "Claw", "Dagger", "Wand", "Staff", "Stave", "Sceptre",
    "Crossbow", "Flail", "Spear", "Body Armour", "Helmet", "Glove", "Boot", "Shield",
    "Buckler", "Focus", "Quiver",
];

const NEGATING_SWAPS: &[(&str, &str)] = &[("reduced", "increased"), ("less", "more")];

/// Contexto vindo do cabeçalho avançado `{ Prefix Modifier "X" (Tier: 2) — ... }`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModContext {
    pub kind: Option<ModifierKind>,
    pub tier: Option<u32>,
    pub affix: Option<Affix>,
    /// "(Rank: 3)" dos mods de bancada.
    pub rank: Option<u32>,
    /// Nome entre aspas: `"Fecund"`.
    pub mod_name: Option<String>,
    pub origin: Option<ModOrigin>,
}

pub struct StatMatcher<'a> {
    catalog: &'a StatCatalog,
}

/// Um mod reconhecido e quantas linhas ele consumiu.
pub struct Matched {
    pub modifier: Modifier,
    pub lines: usize,
}

struct Candidate {
    template: String,
    negate: bool,
}

impl<'a> StatMatcher<'a> {
    pub const fn new(catalog: &'a StatCatalog) -> Self {
        Self { catalog }
    }

    /// Tenta casar `lines[0..]` (1 até MAX_SPAN linhas). `item_class` decide a preferência por stats locais.
    pub fn match_at(&self, lines: &[&str], ctx: &ModContext, item_class: &str) -> Option<Matched> {
        let prefer_local = LOCAL_CLASS_HINTS.iter().any(|h| item_class.contains(h));
        (1..=MAX_SPAN.min(lines.len()))
            .find_map(|span| self.match_span(&lines[..span], ctx, prefer_local, item_class))
    }

    fn match_span(&self, lines: &[&str], ctx: &ModContext, prefer_local: bool, item_class: &str) -> Option<Matched> {
        let mut marker_kind = None;
        let texts: Vec<&str> = lines
            .iter()
            .map(|l| {
                let (text, kind) = strip_markers(l);
                marker_kind = marker_kind.or(kind);
                text
            })
            .collect();
        let kind = marker_kind.or(ctx.kind).unwrap_or(ModifierKind::Explicit);
        let tokens = tokenize(&texts.join("\n"));

        for candidate in candidates(&tokens.template, prefer_local, item_class) {
            let Some(id) = self.lookup(kind, &candidate.template) else { continue };
            let roll = tokens.roll().map(|r| if candidate.negate { negate(r) } else { r });
            return Some(Matched {
                modifier: Modifier {
                    trade_id: id.to_owned(),
                    text: tokens.display.clone(),
                    kind,
                    roll,
                    tier: ctx.tier,
                    affix: ctx.affix,
                    rank: ctx.rank,
                    mod_name: ctx.mod_name.clone(),
                    origin: ctx.origin,
                },
                lines: lines.len(),
            });
        }
        None
    }

    /// Procura no grupo do tipo. Fractured/crafted/desecrated caem para explicit quando o
    /// stat não existe no próprio grupo (a trade API não lista todos — ex.: chaos res crafted).
    fn lookup(&self, kind: ModifierKind, template: &str) -> Option<&'a str> {
        self.catalog.lookup(kind, template).or_else(|| match kind {
            ModifierKind::Fractured | ModifierKind::Crafted | ModifierKind::Desecrated => {
                self.catalog.lookup(ModifierKind::Explicit, template)
            }
            _ => None,
        })
    }
}

fn candidates(template: &str, prefer_local: bool, item_class: &str) -> Vec<Candidate> {
    let mut base = vec![Candidate { template: template.to_owned(), negate: false }];
    if template.contains("-#") {
        base.push(Candidate { template: template.replace("-#", "+#"), negate: false });
    }
    if template.contains("+#") || template.contains("-#") {
        base.push(Candidate { template: template.replace("+#", "#").replace("-#", "#"), negate: false });
    }
    for (from, to) in NEGATING_SWAPS {
        if template.contains(from) {
            base.push(Candidate { template: template.replacen(from, to, 1), negate: true });
        }
    }

    let local = |c: &Candidate| Candidate { template: format!("{} (Local)", c.template), negate: c.negate };
    // Alguns stats trazem a classe no texto: "+#% Chance to Block (Shields)".
    let by_class: Vec<Candidate> = base
        .iter()
        .map(|c| Candidate { template: format!("{} ({item_class})", c.template), negate: c.negate })
        .collect();
    let (first, second): (Vec<Candidate>, Vec<Candidate>) = if prefer_local {
        (base.iter().map(local).collect(), base)
    } else {
        let locals = base.iter().map(local).collect();
        (base, locals)
    };
    first.into_iter().chain(second).chain(by_class).collect()
}

fn negate(roll: StatRoll) -> StatRoll {
    StatRoll {
        value: -roll.value,
        min: roll.max.map(|v| -v),
        max: roll.min.map(|v| -v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirrorsight_domain::GameVersion;

    fn matcher() -> StatMatcher<'static> {
        StatMatcher::new(StatCatalog::embedded(GameVersion::Poe1))
    }

    fn one(line: &str, class: &str) -> Modifier {
        matcher()
            .match_at(&[line], &ModContext::default(), class)
            .unwrap_or_else(|| panic!("não casou: {line}"))
            .modifier
    }

    #[test]
    fn matches_plain_explicit() {
        let m = one("+78 to maximum Life", "Gloves");
        assert_eq!(m.trade_id, "explicit.stat_3299347043");
        assert_eq!(m.kind, ModifierKind::Explicit);
        assert_eq!(m.roll.unwrap().value, 78.0);
    }

    #[test]
    fn marker_selects_group() {
        let m = one("+16% to Fire Resistance (crafted)", "Gloves");
        assert_eq!(m.trade_id, "crafted.stat_3372524247");
        assert_eq!(m.kind, ModifierKind::Crafted);
        assert_eq!(m.text, "+16% to Fire Resistance");
    }

    #[test]
    fn crafted_missing_from_group_falls_back_to_explicit() {
        let m = one("+16% to Chaos Resistance (crafted)", "Gloves");
        assert_eq!(m.trade_id, "explicit.stat_2923486259");
        assert_eq!(m.kind, ModifierKind::Crafted);
    }

    #[test]
    fn prefers_local_on_armour_and_global_elsewhere() {
        assert_eq!(one("+45 to maximum Energy Shield", "Gloves").trade_id, "explicit.stat_4052037485");
        assert_eq!(one("+45 to maximum Energy Shield", "Amulets").trade_id, "explicit.stat_3489782002");
    }

    #[test]
    fn reduced_maps_to_increased_with_negative_value() {
        let m = one("10% reduced Movement Speed", "Boots");
        assert_eq!(m.trade_id, "explicit.stat_2250533757");
        assert_eq!(m.roll.unwrap().value, -10.0);
    }

    #[test]
    fn negative_resistance_uses_plus_template() {
        let m = one("-12% to Fire Resistance", "Rings");
        assert_eq!(m.trade_id, "explicit.stat_3372524247");
        assert_eq!(m.roll.unwrap().value, -12.0);
    }

    #[test]
    fn advanced_line_keeps_range_and_tier() {
        let ctx = ModContext { tier: Some(2), affix: Some(Affix::Prefix), ..ModContext::default() };
        let m = matcher()
            .match_at(&["+78(70-79) to maximum Life (fractured)"], &ctx, "Gloves")
            .unwrap()
            .modifier;
        assert_eq!(m.kind, ModifierKind::Fractured);
        assert!(m.trade_id.starts_with("fractured."));
        assert_eq!(m.tier, Some(2));
        assert_eq!(m.affix, Some(Affix::Prefix));
        let roll = m.roll.unwrap();
        assert_eq!((roll.min, roll.max), (Some(70.0), Some(79.0)));
        assert_eq!(m.text, "+78 to maximum Life");
    }

    #[test]
    fn unknown_line_does_not_match() {
        assert!(matcher().match_at(&["A sombra da forja"], &ModContext::default(), "Gloves").is_none());
    }
}
