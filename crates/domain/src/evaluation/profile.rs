//! Perfil da build: quanto cada tag de mod vale para o personagem e o que ele precisa manter
//! (resistências no cap, requisitos de atributo). Vem de um build do Path of Building; sem build,
//! usa os pesos genéricos das regras.
//!
//! Um build do PoB pode ter vários *item sets* e *skill sets*. O usuário escolhe quais usar
//! ([`BuildChoice`]); o padrão é o set ativo quando o build foi salvo. O PoB só grava os totais
//! (resistências, atributos) do item set ativo: para outro set, eles são **estimados** trocando os
//! valores fixos dos itens de um set pelos do outro.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::knowledge::{GameKnowledge, Rules};
use super::stat::template_key;

/// Um build salvo do Path of Building.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildFile {
    /// Nome do arquivo sem extensão.
    pub name: String,
    /// Caminho completo (é o que vai para `Settings::build_path`).
    pub path: String,
    /// Subpasta dentro da pasta de builds, quando houver.
    pub folder: Option<String>,
}

/// Um item set do PoB: título e o texto de cada item equipado (formato do PoB, com `{range:x}`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildItemSet {
    pub id: String,
    pub title: String,
    pub items: Vec<BuildItem>,
}

/// Item equipado num slot do PoB ("Gloves", "Ring 1", "Weapon 2"…), com o texto do PoB.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildItem {
    pub slot: String,
    pub text: String,
}

/// O que um item equipado do set escolhido dá de resistência e atributo (para a troca).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquippedItem {
    pub slot: String,
    /// Nome do item (raros/únicos) ou a base.
    pub name: String,
    /// Valores fixos por necessidade ("fire", "str"…).
    pub needs: HashMap<String, f64>,
}

/// Slots do PoB que um item dessa classe ocupa.
pub fn slots_for_class(item_class: &str) -> &'static [&'static str] {
    match item_class {
        "Helmets" => &["Helmet"],
        "Body Armours" => &["Body Armour"],
        "Gloves" => &["Gloves"],
        "Boots" => &["Boots"],
        "Belts" => &["Belt"],
        "Amulets" => &["Amulet"],
        "Rings" => &["Ring 1", "Ring 2"],
        "Shields" | "Quivers" | "Foci" | "Bucklers" => &["Weapon 2"],
        "Claws" | "Daggers" | "Rune Daggers" | "Wands" | "One Hand Swords" | "Thrusting One Hand Swords"
        | "One Hand Axes" | "One Hand Maces" | "Sceptres" => &["Weapon 1", "Weapon 2"],
        "Bows" | "Staves" | "Warstaves" | "Two Hand Swords" | "Two Hand Axes" | "Two Hand Maces" | "Fishing Rods" => {
            &["Weapon 1"]
        }
        _ => &[],
    }
}

/// Nome do item no texto do PoB: a linha depois de "Rarity:" (raros/únicos têm nome; os
/// demais trazem só a base).
fn pob_item_name(text: &str) -> String {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let first = lines.next().unwrap_or_default();
    if first.starts_with("Rarity:") {
        lines.next().unwrap_or_default().to_owned()
    } else {
        first.to_owned()
    }
}

/// Um skill set do PoB, já reduzido ao grupo principal.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildSkillSet {
    pub id: String,
    pub title: String,
    pub main_skill_gems: Vec<String>,
    pub main_active: Option<u32>,
}

/// Quais sets do build usar (`None` = o ativo no PoB).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildChoice {
    pub item_set: Option<String>,
    pub skill_set: Option<String>,
}

/// Opção de set para a tela de configurações.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetOption {
    pub id: String,
    pub title: String,
}

/// O que a tela de configurações mostra de um build: o resumo e os sets disponíveis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildDetails {
    pub summary: BuildSummary,
    pub item_sets: Vec<SetOption>,
    pub skill_sets: Vec<SetOption>,
    /// Sets ativos quando o build foi salvo no PoB.
    pub active_item_set: Option<String>,
    pub active_skill_set: Option<String>,
}

/// Dados crus lidos do build (o adapter do PoB preenche; o domínio interpreta).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildInput {
    pub name: String,
    pub class: Option<String>,
    pub ascendancy: Option<String>,
    pub level: Option<u32>,
    /// Gemas do grupo principal (skill + suportes), pelo nome.
    pub main_skill_gems: Vec<String>,
    /// Qual gema ativa do grupo é a principal (1 = a primeira ativa), como no PoB.
    #[serde(default)]
    pub main_active: Option<u32>,
    /// `PlayerStat` do PoB: "Life", "EnergyShield", "FireResist", "FireResistOverCap", "Str", "ReqStr"…
    /// Valem para o item set ativo.
    pub stats: HashMap<String, f64>,
    #[serde(default)]
    pub item_sets: Vec<BuildItemSet>,
    #[serde(default)]
    pub active_item_set: Option<String>,
    #[serde(default)]
    pub skill_sets: Vec<BuildSkillSet>,
    #[serde(default)]
    pub active_skill_set: Option<String>,
}

impl BuildInput {
    /// Resumo do perfil com a escolha de sets, mais os sets disponíveis.
    pub fn details(&self, choice: &BuildChoice, knowledge: &dyn GameKnowledge) -> BuildDetails {
        let options = |sets: Vec<(&String, &String)>| {
            sets.into_iter().map(|(id, title)| SetOption { id: id.clone(), title: title.clone() }).collect()
        };
        BuildDetails {
            summary: BuildProfile::from_input_with(self, choice, knowledge).summary(),
            item_sets: options(self.item_sets.iter().map(|s| (&s.id, &s.title)).collect()),
            skill_sets: options(self.skill_sets.iter().map(|s| (&s.id, &s.title)).collect()),
            active_item_set: self.active_item_set.clone(),
            active_skill_set: self.active_skill_set.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResistState {
    /// Resistência efetiva (já limitada pelo cap).
    pub value: f64,
    /// Quanto passa do cap.
    pub overcap: f64,
    pub cap: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeState {
    pub value: f64,
    pub required: f64,
}

/// Restrições do personagem que uma troca de item pode quebrar.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildNeeds {
    /// "fire" | "cold" | "lightning" | "chaos".
    pub resists: HashMap<String, ResistState>,
    /// "str" | "dex" | "int".
    pub attributes: HashMap<String, AttributeState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildProfile {
    pub name: Option<String>,
    /// Sem build carregado (pesos genéricos).
    pub generic: bool,
    pub class: Option<String>,
    pub main_skill: Option<String>,
    /// Tags de mod que a skill principal usa ("caster", "fire"…).
    pub skill_tags: Vec<String>,
    pub tag_weights: HashMap<String, f64>,
    pub needs: Option<BuildNeeds>,
    /// Títulos dos sets usados.
    pub item_set: Option<String>,
    pub skill_set: Option<String>,
    /// Resistências/atributos estimados (item set diferente do ativo no PoB).
    pub needs_estimated: bool,
    /// Itens equipados no item set escolhido: a troca tira o do slot, não o item copiado.
    #[serde(default)]
    pub equipped: Vec<EquippedItem>,
}

/// O que o front mostra do perfil.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildSummary {
    pub name: Option<String>,
    pub generic: bool,
    pub class: Option<String>,
    pub main_skill: Option<String>,
    /// Tags mais valorizadas (para explicar a relevância).
    pub focus: Vec<String>,
    #[serde(default)]
    pub item_set: Option<String>,
    #[serde(default)]
    pub skill_set: Option<String>,
    #[serde(default)]
    pub needs_estimated: bool,
}

const ELEMENTS: [(&str, &str); 4] = [("fire", "Fire"), ("cold", "Cold"), ("lightning", "Lightning"), ("chaos", "Chaos")];
const ATTRIBUTES: [(&str, &str); 3] = [("str", "Str"), ("dex", "Dex"), ("int", "Int")];

impl BuildProfile {
    /// Sem build: só os pesos genéricos.
    pub fn generic(rules: &Rules) -> Self {
        Self {
            name: None,
            generic: true,
            class: None,
            main_skill: None,
            skill_tags: Vec::new(),
            tag_weights: rules.tag_weights.clone(),
            needs: None,
            item_set: None,
            skill_set: None,
            needs_estimated: false,
            equipped: Vec::new(),
        }
    }

    /// Interpreta o build com os sets ativos no PoB.
    pub fn from_input(input: &BuildInput, knowledge: &dyn GameKnowledge) -> Self {
        Self::from_input_with(input, &BuildChoice::default(), knowledge)
    }

    /// Interpreta o build: tags da skill principal, crit, pool de vida/ES e necessidades,
    /// com o item set e o skill set escolhidos.
    pub fn from_input_with(input: &BuildInput, choice: &BuildChoice, knowledge: &dyn GameKnowledge) -> Self {
        let rules = knowledge.rules();
        let b = &rules.build;
        let mut weights = rules.tag_weights.clone();

        // Skill set: o escolhido (se existir) ou o ativo.
        let skill_id = choice.skill_set.as_ref().or(input.active_skill_set.as_ref());
        let skill_set = skill_id.and_then(|id| input.skill_sets.iter().find(|s| &s.id == id));
        let (gems, main_active) = match skill_set {
            Some(set) if choice.skill_set.is_some() => (&set.main_skill_gems, set.main_active),
            _ => (&input.main_skill_gems, input.main_active),
        };

        // Tags de mod da skill principal: a n-ésima gema ativa (não suporte) do grupo principal.
        // A lista segue a do PoB para o `mainActiveSkill` apontar para a mesma gema: toda gema
        // ligada que não é suporte (mesmo as que a base não conhece), e gema Vaal conta duas
        // vezes (a skill Vaal e a skill base).
        let empty: &[String] = &[];
        let mut actives: Vec<(String, &[String])> = Vec::new();
        for gem in gems {
            let tags = knowledge.gem_tags(gem);
            if tags.is_some_and(|t| t.iter().any(|x| x == "support")) {
                continue;
            }
            actives.push((gem.clone(), tags.unwrap_or(empty)));
            if let Some(base) = gem.strip_prefix("Vaal ") {
                actives.push((base.to_owned(), knowledge.gem_tags(base).unwrap_or(empty)));
            }
        }
        let pick = main_active.and_then(|n| usize::try_from(n).ok()?.checked_sub(1)).unwrap_or(0);
        let main = actives.get(pick).or_else(|| actives.first());
        let main_skill = main.map(|(gem, _)| gem.clone());
        let mut skill_tags: Vec<String> = Vec::new();
        for gem_tag in main.map(|(_, tags)| *tags).unwrap_or_default() {
            for mod_tag in b.gem_tag_map.get(gem_tag).into_iter().flatten() {
                if !skill_tags.contains(mod_tag) {
                    skill_tags.push(mod_tag.clone());
                }
            }
        }
        if let Some(dominant) = b.dominant_tags.iter().find(|t| skill_tags.contains(t)) {
            skill_tags = vec![dominant.clone()];
        }

        let stat = |k: &str| input.stats.get(k).copied();
        if stat("CritChance").is_some_and(|c| c >= b.crit_threshold) {
            let w = weights.entry("critical".to_owned()).or_insert(rules.default_weight);
            *w *= b.crit_boost;
        }
        // Pool dominante: vida ou escudo de energia.
        let life = stat("Life").unwrap_or(0.0);
        let es = stat("EnergyShield").unwrap_or(0.0);
        if life > 0.0 || es > 0.0 {
            let (main, other) = if es > life { ("energy_shield", "life") } else { ("life", "energy_shield") };
            if let Some(w) = weights.get_mut(main) {
                *w *= b.pool_boost;
            }
            if let Some(w) = weights.get_mut(other) {
                *w *= b.pool_damp;
            }
        }

        let mut needs = BuildNeeds::default();
        for (key, pob) in ELEMENTS {
            if let Some(value) = stat(&format!("{pob}Resist")) {
                let overcap = stat(&format!("{pob}ResistOverCap")).unwrap_or(0.0);
                let cap = stat(&format!("{pob}ResistMax")).unwrap_or(rules.needs.res_cap);
                needs.resists.insert(key.to_owned(), ResistState { value, overcap, cap });
            }
        }
        for (key, pob) in ATTRIBUTES {
            if let (Some(value), Some(required)) = (stat(pob), stat(&format!("Req{pob}"))) {
                needs.attributes.insert(key.to_owned(), AttributeState { value, required });
            }
        }
        let has_needs = !needs.resists.is_empty() || !needs.attributes.is_empty();

        // Item set: os totais do PoB são do set ativo; outro set troca os valores fixos dos itens.
        let find_items = |id: Option<&String>| id.and_then(|id| input.item_sets.iter().find(|s| &s.id == id));
        let active_items = find_items(input.active_item_set.as_ref());
        let chosen_items = find_items(choice.item_set.as_ref()).or(active_items);
        let mut needs_estimated = false;
        if let (Some(active), Some(chosen)) = (active_items, chosen_items) {
            if active.id != chosen.id && has_needs {
                let texts = |set: &BuildItemSet| set.items.iter().map(|i| i.text.clone()).collect::<Vec<_>>();
                let before = flat_needs(&texts(active), rules);
                let after = flat_needs(&texts(chosen), rules);
                let shift = |key: &str| after.get(key).copied().unwrap_or(0.0) - before.get(key).copied().unwrap_or(0.0);
                for (key, state) in &mut needs.resists {
                    let total = state.value + state.overcap + shift(key);
                    state.value = total.min(state.cap);
                    state.overcap = (total - state.cap).max(0.0);
                }
                for (key, state) in &mut needs.attributes {
                    state.value += shift(key);
                }
                needs_estimated = true;
            }
        }

        Self {
            name: Some(input.name.clone()),
            generic: false,
            class: input.ascendancy.clone().or_else(|| input.class.clone()),
            main_skill,
            skill_tags,
            tag_weights: weights,
            needs: has_needs.then_some(needs),
            item_set: chosen_items.map(|s| s.title.clone()),
            skill_set: skill_set.map(|s| s.title.clone()),
            needs_estimated,
            equipped: chosen_items
                .map(|set| {
                    set.items
                        .iter()
                        .map(|i| EquippedItem {
                            slot: i.slot.clone(),
                            name: pob_item_name(&i.text),
                            needs: flat_needs(std::slice::from_ref(&i.text), rules),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    /// Item do build que a troca substituiria: o do slot da classe; em anéis/armas, o de mesmo
    /// nome que o seu item, senão o primeiro slot ocupado. `None` = o build não tem item ali.
    pub fn equipped_for(&self, item_class: &str, name: &str) -> Option<&EquippedItem> {
        let slots = slots_for_class(item_class);
        let in_slots: Vec<&EquippedItem> = self.equipped.iter().filter(|e| slots.contains(&e.slot.as_str())).collect();
        in_slots.iter().find(|e| e.name.eq_ignore_ascii_case(name)).or_else(|| in_slots.first()).copied()
    }

    /// Relevância de um mod com essas tags para a build: o maior peso entre as tags. Em mods de
    /// dano, sobe quando casa com a skill principal e cai quando é de uma família que ela não usa
    /// (fogo num build de gelo, ataque num build de feitiço). Resistência a fogo não é dano:
    /// não sobe só porque a skill é de fogo.
    pub fn relevance(&self, tags: &[String], rules: &Rules) -> f64 {
        let base = tags
            .iter()
            .filter_map(|t| self.tag_weights.get(t))
            .copied()
            .fold(None, |acc: Option<f64>, w| Some(acc.map_or(w, |a| a.max(w))))
            .unwrap_or(rules.default_weight);
        if self.skill_tags.is_empty() || !tags.iter().any(|t| rules.damage_tags.contains(t)) {
            return base;
        }
        let off = rules.exclusive_families.iter().any(|family| {
            let build_uses = family.iter().any(|f| self.skill_tags.contains(f));
            let mod_in_family: Vec<&String> = tags.iter().filter(|t| family.contains(t)).collect();
            build_uses && !mod_in_family.is_empty() && !mod_in_family.iter().any(|t| self.skill_tags.contains(t))
        });
        if off {
            base * rules.offtype_factor
        } else if tags.iter().any(|t| self.skill_tags.contains(t)) {
            base * rules.build.skill_boost
        } else {
            base
        }
    }

    pub fn summary(&self) -> BuildSummary {
        let mut weighted: Vec<(&String, &f64)> = self.tag_weights.iter().filter(|(_, w)| **w > 1.0).collect();
        weighted.sort_by(|a, b| b.1.total_cmp(a.1).then(a.0.cmp(b.0)));
        // Tags da skill primeiro, depois as defesas/recursos de maior peso.
        let mut focus: Vec<String> = self.skill_tags.clone();
        for (tag, _) in weighted {
            if !focus.contains(tag) {
                focus.push(tag.clone());
            }
        }
        focus.truncate(6);
        BuildSummary {
            name: self.name.clone(),
            generic: self.generic,
            class: self.class.clone(),
            main_skill: self.main_skill.clone(),
            focus,
            item_set: self.item_set.clone(),
            skill_set: self.skill_set.clone(),
            needs_estimated: self.needs_estimated,
        }
    }
}

/// Soma dos valores fixos de resistência ("fire"…) e atributo ("str"…) nos textos dos itens
/// (formato do PoB). Faixas `(a-b)` usam o `{range:x}` da linha (0.5 sem ele); aliases como
/// "all elemental resistances" contam para cada elemento.
pub fn flat_needs(items: &[String], rules: &Rules) -> HashMap<String, f64> {
    let mut out: HashMap<String, f64> = HashMap::new();
    let needs = rules.needs.res_stats.iter().chain(rules.needs.attr_stats.iter());
    let targets: Vec<(&String, &Vec<String>)> = needs.collect();
    for item in items {
        // Únicos com variantes: o PoB grava as linhas de todas, marcadas com `{variant:a,b}`.
        let selected = item
            .lines()
            .find_map(|l| l.trim().strip_prefix("Selected Variant:"))
            .and_then(|v| v.trim().parse::<u32>().ok());
        for line in item.lines().filter(|l| variant_applies(l, selected)) {
            let Some((key, value)) = resolved_line(line) else { continue };
            let keys = rules.stat_aliases.iter().find(|a| a.from == key).map_or_else(|| vec![key.clone()], |a| a.to.clone());
            for k in keys {
                for (need, templates) in &targets {
                    if templates.contains(&k) {
                        *out.entry((*need).clone()).or_default() += value;
                    }
                }
            }
        }
    }
    out
}

/// A linha vale na variante selecionada do item? Sem `{variant:…}` (ou sem variante
/// selecionada conhecida), vale sempre.
fn variant_applies(line: &str, selected: Option<u32>) -> bool {
    let Some(selected) = selected else { return true };
    let mut text = line.trim();
    while let Some(rest) = text.strip_prefix('{') {
        let Some((marker, after)) = rest.split_once('}') else { return true };
        if let Some(list) = marker.strip_prefix("variant:") {
            return list.split(',').any(|v| v.trim().parse::<u32>().ok() == Some(selected));
        }
        text = after;
    }
    true
}

/// "{tags:x}{range:0.25}+(40-60)% to Fire Resistance" → ("#% to fire resistance", 45).
fn resolved_line(line: &str) -> Option<(String, f64)> {
    let mut text = line.trim();
    let mut range = 0.5;
    while let Some(rest) = text.strip_prefix('{') {
        let (marker, after) = rest.split_once('}')?;
        if let Some(r) = marker.strip_prefix("range:").and_then(|v| v.parse::<f64>().ok()) {
            range = r;
        }
        text = after;
    }
    // Troca cada "(a-b)" pelo valor rolado.
    let mut resolved = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('(') {
        let Some(close) = rest[open..].find(')').map(|c| open + c) else { break };
        let inner = &rest[open + 1..close];
        let split = inner.char_indices().skip(1).find(|(_, c)| *c == '-').map(|(i, _)| i);
        let value = split.and_then(|i| Some((inner[..i].trim().parse::<f64>().ok()?, inner[i + 1..].trim().parse::<f64>().ok()?)));
        resolved.push_str(&rest[..open]);
        match value {
            Some((lo, hi)) => resolved.push_str(&format!("{}", (lo + (hi - lo) * range).round())),
            None => resolved.push_str(&rest[open..=close]),
        }
        rest = &rest[close + 1..];
    }
    resolved.push_str(rest);
    let numbers: Vec<f64> = number_values(&resolved);
    if numbers.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let value = numbers.iter().sum::<f64>() / numbers.len() as f64;
    Some((template_key(&resolved), value))
}

/// Números do texto, com sinal ("-12% to Fire Resistance" → -12).
fn number_values(text: &str) -> Vec<f64> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let prev_word = i > 0 && chars[i - 1].is_ascii_alphanumeric();
        if chars[i].is_ascii_digit() && !prev_word {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            let negative = start > 0 && chars[start - 1] == '-';
            if let Ok(v) = raw.parse::<f64>() {
                out.push(if negative { -v } else { v });
            }
            continue;
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::test_support::fake_knowledge;

    fn fire_caster() -> BuildInput {
        BuildInput {
            name: "Fireball Elementalist".into(),
            class: Some("Witch".into()),
            ascendancy: Some("Elementalist".into()),
            level: Some(90),
            main_skill_gems: vec!["Spell Echo".into(), "Fireball".into()],
            main_active: Some(1),
            stats: [
                ("Life", 4000.0),
                ("EnergyShield", 300.0),
                ("FireResist", 75.0),
                ("FireResistOverCap", 10.0),
                ("ColdResist", 70.0),
                ("Str", 100.0),
                ("ReqStr", 98.0),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn derives_skill_tags_pool_and_needs() {
        let k = fake_knowledge();
        let p = BuildProfile::from_input(&fire_caster(), &k);
        assert_eq!(p.main_skill.as_deref(), Some("Fireball"));
        assert!(p.skill_tags.contains(&"fire".to_owned()));
        assert!(p.skill_tags.contains(&"caster".to_owned()));
        let rules = k.rules();
        assert!(p.tag_weights["life"] > rules.tag_weights["life"]);
        let tags = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        // Dano de fogo sobe; resistência a fogo não.
        let fire_damage = tags(&["damage", "elemental_damage", "fire"]);
        assert!(p.relevance(&fire_damage, rules) > rules.generic_weight(&fire_damage));
        let fire_res = tags(&["elemental", "fire", "resistance"]);
        assert!((p.relevance(&fire_res, rules) - rules.generic_weight(&fire_res)).abs() < 1e-9);
        let needs = p.needs.unwrap();
        assert_eq!(needs.resists["fire"].overcap, 10.0);
        assert_eq!(needs.resists["cold"].value, 70.0);
        assert_eq!(needs.attributes["str"].required, 98.0);
    }

    #[test]
    fn off_type_damage_is_damped() {
        let k = fake_knowledge();
        let rules = k.rules();
        let p = BuildProfile::from_input(&fire_caster(), &k);
        let tags = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let fire = p.relevance(&tags(&["damage", "elemental", "fire"]), rules);
        let cold = p.relevance(&tags(&["damage", "elemental", "cold"]), rules);
        let attack = p.relevance(&tags(&["damage", "attack"]), rules);
        assert!(fire > cold);
        assert!(cold < rules.generic_weight(&tags(&["damage", "elemental", "cold"])));
        assert!(attack < fire);
        // Mods que não são de dano não sofrem corte.
        let life = p.relevance(&tags(&["resource", "life"]), rules);
        assert!(life >= rules.tag_weights["life"]);
    }

    fn with_sets() -> BuildInput {
        let mut input = fire_caster();
        input.active_item_set = Some("1".into());
        input.item_sets = vec![
            BuildItemSet {
                id: "1".into(),
                title: "End Game".into(),
                items: vec![BuildItem { slot: "Gloves".into(), text: "Rarity: RARE\nX\nGloves\n+40% to Fire Resistance\n+20 to Strength".into() }],
            },
            BuildItemSet {
                id: "2".into(),
                title: "Leveling".into(),
                items: vec![BuildItem { slot: "Boots".into(), text: "Rarity: RARE\nY\nBoots\n{range:0.5}+(10-20)% to all Elemental Resistances".into() }],
            },
        ];
        input.active_skill_set = Some("1".into());
        input.skill_sets = vec![
            BuildSkillSet { id: "1".into(), title: "Mapas".into(), main_skill_gems: vec!["Fireball".into()], main_active: Some(1) },
            BuildSkillSet { id: "2".into(), title: "Vazio".into(), main_skill_gems: vec![], main_active: None },
        ];
        input
    }

    #[test]
    fn other_item_set_estimates_resists_and_attributes() {
        let k = fake_knowledge();
        let input = with_sets();
        // Set ativo: valores do PoB como estão.
        let active = BuildProfile::from_input(&input, &k);
        assert!(!active.needs_estimated);
        assert_eq!(active.item_set.as_deref(), Some("End Game"));
        // Outro set: fogo 75 + 10 acima − 40 + 15 = 60; força 100 − 20 = 80.
        let choice = BuildChoice { item_set: Some("2".into()), skill_set: None };
        let other = BuildProfile::from_input_with(&input, &choice, &k);
        assert!(other.needs_estimated);
        assert_eq!(other.item_set.as_deref(), Some("Leveling"));
        let needs = other.needs.unwrap();
        assert_eq!((needs.resists["fire"].value, needs.resists["fire"].overcap), (60.0, 0.0));
        assert_eq!(needs.resists["cold"].value, 75.0);
        assert_eq!(needs.attributes["str"].value, 80.0);
    }

    #[test]
    fn skill_set_choice_changes_main_skill() {
        let k = fake_knowledge();
        let input = with_sets();
        let empty = BuildChoice { item_set: None, skill_set: Some("2".into()) };
        let p = BuildProfile::from_input_with(&input, &empty, &k);
        assert_eq!(p.main_skill, None);
        assert_eq!(p.skill_set.as_deref(), Some("Vazio"));
        let details = input.details(&BuildChoice::default(), &k);
        assert_eq!(details.item_sets.len(), 2);
        assert_eq!(details.active_skill_set.as_deref(), Some("1"));
        assert_eq!(details.summary.main_skill.as_deref(), Some("Fireball"));
    }

    #[test]
    fn equipped_item_of_the_slot_is_known() {
        let k = fake_knowledge();
        let p = BuildProfile::from_input(&with_sets(), &k);
        let gloves = p.equipped_for("Gloves", "Qualquer").unwrap();
        assert_eq!(gloves.name, "X");
        assert_eq!(gloves.needs["fire"], 40.0);
        assert!(p.equipped_for("Helmets", "X").is_none());
    }

    #[test]
    fn main_active_index_counts_unknown_and_vaal_gems_like_pob() {
        let k = fake_knowledge();
        let mut input = fire_caster();
        // PoB: [Gema Desconhecida, Vaal Fireball (+ Fireball), Spell Echo] → 3 = Fireball.
        input.main_skill_gems = vec!["Gema Desconhecida".into(), "Vaal Fireball".into(), "Spell Echo".into()];
        input.main_active = Some(3);
        assert_eq!(BuildProfile::from_input(&input, &k).main_skill.as_deref(), Some("Fireball"));
        input.main_active = Some(1);
        let p = BuildProfile::from_input(&input, &k);
        assert_eq!(p.main_skill.as_deref(), Some("Gema Desconhecida"));
        assert!(p.skill_tags.is_empty());
    }

    #[test]
    fn flat_needs_reads_pob_item_text() {
        let k = fake_knowledge();
        let items = vec![
            "{tags:resistance}{range:1}+(17-23)% to Chaos Resistance\n-12% to Cold Resistance\n+45 to Dexterity\nLevelReq: 59".into(),
        ];
        let flat = flat_needs(&items, k.rules());
        assert_eq!(flat["chaos"], 23.0);
        assert_eq!(flat["cold"], -12.0);
        assert_eq!(flat["dex"], 45.0);
        assert!(!flat.contains_key("fire"));
    }

    #[test]
    fn flat_needs_counts_only_the_selected_variant() {
        let k = fake_knowledge();
        let items = vec![
            "Rarity: UNIQUE\nX\nBelt\nVariant: Pre 3.0.0\nVariant: Current\nSelected Variant: 2\n{variant:1}+(20-30)% to Fire Resistance\n{variant:2}{range:1}+(30-40)% to Fire Resistance\n+10 to Strength".into(),
        ];
        let flat = flat_needs(&items, k.rules());
        assert_eq!(flat["fire"], 40.0);
        assert_eq!(flat["str"], 10.0);
    }

    #[test]
    fn generic_profile_uses_rule_weights() {
        let k = fake_knowledge();
        let p = BuildProfile::generic(k.rules());
        assert!(p.generic);
        assert!(p.needs.is_none());
        assert_eq!(p.summary().name, None);
    }
}
