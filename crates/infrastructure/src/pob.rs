//! Builds do Path of Building (`Documents\Path of Building\Builds\*.xml`).
//!
//! Lê só o que a avaliação usa: classe, gemas do grupo principal e os `PlayerStat` que o PoB
//! grava no arquivo (vida, ES, resistências, atributos e requisitos).

use std::path::{Path, PathBuf};

use mirrorsight_domain::evaluation::profile::{BuildFile, BuildItem, BuildItemSet, BuildSkillSet};
use mirrorsight_domain::evaluation::BuildInput;
use mirrorsight_domain::ports::{BuildRepository, PortError, PortResult};

/// Lê os builds salvos pelo PoB numa pasta (padrão: `Documentos\Path of Building\Builds`).
pub struct PobBuildRepository {
    dir: Option<PathBuf>,
}

impl PobBuildRepository {
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self { dir }
    }

    /// Pasta padrão do PoB (respeita a pasta Documentos redirecionada, ex.: OneDrive).
    pub fn default_dir() -> Option<PathBuf> {
        dirs::document_dir().map(|d| d.join("Path of Building").join("Builds"))
    }
}

impl Default for PobBuildRepository {
    fn default() -> Self {
        Self::new(Self::default_dir())
    }
}

impl BuildRepository for PobBuildRepository {
    fn list(&self) -> PortResult<Vec<BuildFile>> {
        let Some(dir) = &self.dir else { return Ok(Vec::new()) };
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        collect(dir, dir, 0, &mut out);
        out.sort_by(|a, b| a.folder.cmp(&b.folder).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        Ok(out)
    }

    fn load(&self, path: &str) -> PortResult<BuildInput> {
        let xml = std::fs::read_to_string(path).map_err(|e| PortError::NotFound(format!("{path}: {e}")))?;
        let name = Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        parse_build(&xml, &name).map_err(PortError::Other)
    }

    fn stamp(&self, path: &str) -> Option<u128> {
        let modified = std::fs::metadata(path).ok()?.modified().ok()?;
        Some(modified.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos())
    }
}

/// Varre a pasta (e subpastas, até 3 níveis) atrás de `.xml`.
fn collect(root: &Path, dir: &Path, depth: u32, out: &mut Vec<BuildFile>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth < 3 {
                collect(root, &path, depth + 1, out);
            }
            continue;
        }
        if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("xml")) {
            continue;
        }
        let folder = path
            .parent()
            .and_then(|p| p.strip_prefix(root).ok())
            .map(|p| p.to_string_lossy().into_owned())
            .filter(|p| !p.is_empty());
        out.push(BuildFile {
            name: path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
            path: path.to_string_lossy().into_owned(),
            folder,
        });
    }
}

fn child<'a, 'input>(node: roxmltree::Node<'a, 'input>, tag: &str) -> Option<roxmltree::Node<'a, 'input>> {
    node.children().find(|n| n.has_tag_name(tag))
}

/// Extrai do XML do PoB o que a avaliação usa.
pub fn parse_build(xml: &str, name: &str) -> Result<BuildInput, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML do PoB inválido: {e}"))?;
    let root = doc.root_element();
    if !root.has_tag_name("PathOfBuilding") {
        return Err("o arquivo não é um build do Path of Building".into());
    }
    let build = child(root, "Build").ok_or("build sem <Build>")?;

    let mut stats = std::collections::HashMap::new();
    for stat in build.children().filter(|n| n.has_tag_name("PlayerStat")) {
        if let (Some(key), Some(value)) = (stat.attribute("stat"), stat.attribute("value").and_then(|v| v.parse::<f64>().ok())) {
            stats.insert(key.to_owned(), value);
        }
    }

    // Grupo principal: `mainSocketGroup` (1 = primeiro) entre os <Skill> de cada skill set.
    let main_index = build
        .attribute("mainSocketGroup")
        .and_then(|v| v.parse::<usize>().ok())
        .and_then(|n| n.checked_sub(1))
        .unwrap_or(0);
    let skills_node = child(root, "Skills");
    let mut skill_sets = Vec::new();
    let mut active_skill_set = None;
    let mut main_skill_gems = Vec::new();
    let mut main_active = None;
    if let Some(skills) = skills_node {
        let active = skills.attribute("activeSkillSet");
        let sets: Vec<_> = skills.children().filter(|n| n.has_tag_name("SkillSet")).collect();
        if sets.is_empty() {
            // Formato antigo: <Skill> direto em <Skills>.
            (main_skill_gems, main_active) = main_group(skills, Some(main_index));
        } else {
            // O PoB só grava o grupo principal (`mainSocketGroup`) do skill set ativo; nos
            // outros sets o mesmo índice pode apontar para uma aura, então eles usam a heurística.
            let active_index = sets.iter().position(|s| s.attribute("id") == active).unwrap_or(0);
            for (i, set) in sets.iter().enumerate() {
                let id = set.attribute("id").map_or_else(|| (i + 1).to_string(), str::to_owned);
                let title = set.attribute("title").filter(|t| !t.is_empty()).map_or_else(|| format!("Skill set {id}"), str::to_owned);
                let (gems, main) = main_group(*set, (i == active_index).then_some(main_index));
                skill_sets.push(BuildSkillSet { id: id.clone(), title, main_skill_gems: gems, main_active: main });
            }
            let active_set = skill_sets.iter().find(|s| Some(s.id.as_str()) == active).or_else(|| skill_sets.first());
            if let Some(set) = active_set {
                active_skill_set = Some(set.id.clone());
                main_skill_gems.clone_from(&set.main_skill_gems);
                main_active = set.main_active;
            }
        }
    }
    let (item_sets, active_item_set) = item_sets(root);

    let attr = |key: &str| build.attribute(key).filter(|v| !v.is_empty() && *v != "None").map(str::to_owned);
    Ok(BuildInput {
        name: name.to_owned(),
        class: attr("className"),
        ascendancy: attr("ascendClassName"),
        level: build.attribute("level").and_then(|v| v.parse().ok()),
        main_skill_gems,
        main_active,
        stats,
        item_sets,
        active_item_set,
        skill_sets,
        active_skill_set,
    })
}

fn enabled(n: &roxmltree::Node<'_, '_>) -> bool {
    !matches!(n.attribute("enabled"), Some("false"))
}

/// Gemas do grupo principal de um conjunto de <Skill>.
///
/// Com `main_index` (set ativo), o grupo que o PoB marcou. Sem ele (outros sets), o grupo ligado
/// marcado para o "Full DPS"; senão o grupo ligado com mais gemas (a skill principal costuma
/// ser o 6L), o primeiro em caso de empate.
fn main_group(container: roxmltree::Node<'_, '_>, main_index: Option<usize>) -> (Vec<String>, Option<u32>) {
    let skills: Vec<_> = container.children().filter(|n| n.has_tag_name("Skill")).collect();
    let gem_count = |s: &roxmltree::Node<'_, '_>| s.children().filter(|g| g.has_tag_name("Gem") && enabled(g)).count();
    let heuristic = || {
        skills
            .iter()
            .filter(|s| enabled(s) && gem_count(s) > 0)
            .find(|s| s.attribute("includeInFullDPS") == Some("true"))
            .or_else(|| {
                skills
                    .iter()
                    .filter(|s| enabled(s))
                    .fold(None, |best: Option<&roxmltree::Node<'_, '_>>, s| match best {
                        Some(b) if gem_count(b) >= gem_count(s) => Some(b),
                        _ => Some(s),
                    })
            })
    };
    let main = main_index
        .and_then(|i| skills.get(i))
        .filter(|s| enabled(s))
        .or_else(heuristic);
    let gems = main
        .map(|s| {
            s.children()
                .filter(|g| g.has_tag_name("Gem") && enabled(g))
                .filter_map(|g| g.attribute("nameSpec").or_else(|| g.attribute("skillId")))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let active = main.and_then(|s| s.attribute("mainActiveSkill")).and_then(|v| v.parse().ok());
    (gems, active)
}

/// Item sets: título e o texto de cada item equipado (sem a arma reserva nem frascos).
fn item_sets(root: roxmltree::Node<'_, '_>) -> (Vec<BuildItemSet>, Option<String>) {
    let Some(items) = child(root, "Items") else { return (Vec::new(), None) };
    let texts: std::collections::HashMap<&str, String> = items
        .children()
        .filter(|n| n.has_tag_name("Item"))
        .filter_map(|n| {
            let id = n.attribute("id")?;
            let text: String = n.children().filter(roxmltree::Node::is_text).filter_map(|t| t.text()).collect();
            Some((id, text.trim().to_owned()))
        })
        .collect();
    let sets: Vec<BuildItemSet> = items
        .children()
        .filter(|n| n.has_tag_name("ItemSet"))
        .enumerate()
        .map(|(i, set)| {
            let id = set.attribute("id").map_or_else(|| (i + 1).to_string(), str::to_owned);
            let title = set.attribute("title").filter(|t| !t.is_empty()).map_or_else(|| format!("Item set {id}"), str::to_owned);
            let items = set
                .children()
                .filter(|n| n.has_tag_name("Slot"))
                .filter(|slot| {
                    let name = slot.attribute("name").unwrap_or_default();
                    !name.contains("Swap") && !name.starts_with("Flask")
                })
                .filter_map(|slot| {
                    let id = slot.attribute("itemId").filter(|id| *id != "0")?;
                    Some(BuildItem { slot: slot.attribute("name")?.to_owned(), text: texts.get(id)?.clone() })
                })
                .collect();
            BuildItemSet { id, title, items }
        })
        .collect();
    let active = items.attribute("activeItemSet").map(str::to_owned).or_else(|| sets.first().map(|s| s.id.clone()));
    (sets, active)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<PathOfBuilding>
  <Build level="90" className="Witch" ascendClassName="Elementalist" mainSocketGroup="2" targetVersion="3_0">
    <PlayerStat stat="Life" value="4200"/>
    <PlayerStat stat="EnergyShield" value="310.5"/>
    <PlayerStat stat="FireResist" value="75"/>
    <PlayerStat stat="FireResistOverCap" value="12"/>
    <PlayerStat stat="Str" value="120"/>
    <PlayerStat stat="ReqStr" value="111"/>
  </Build>
  <Skills activeSkillSet="2">
    <SkillSet id="1"><Skill enabled="true"><Gem nameSpec="Cleave" enabled="true"/></Skill></SkillSet>
    <SkillSet id="2">
      <Skill enabled="true" mainActiveSkill="1"><Gem nameSpec="Clarity" enabled="true"/></Skill>
      <Skill enabled="true" mainActiveSkill="1">
        <Gem nameSpec="Spell Echo" enabled="true"/>
        <Gem nameSpec="Fireball" enabled="true"/>
        <Gem nameSpec="Desativada" enabled="false"/>
      </Skill>
    </SkillSet>
  </Skills>
  <Items activeItemSet="2">
    <Item id="1">
Rarity: RARE
Doom Grip
Iron Gauntlets
+30% to Fire Resistance
      <ModRange range="0.5" id="1"/>
    </Item>
    <Item id="2">
Rarity: RARE
Gale Stride
Iron Greaves
{range:0.5}+(10-20)% to all Elemental Resistances
    </Item>
    <Item id="3">
Rarity: MAGIC
Divine Life Flask
    </Item>
    <ItemSet id="1" title="Leveling">
      <Slot name="Gloves" itemId="1"/>
      <Slot name="Weapon 1 Swap" itemId="2"/>
      <Slot name="Flask 1" itemId="3"/>
      <Slot name="Boots" itemId="0"/>
    </ItemSet>
    <ItemSet id="2">
      <Slot name="Boots" itemId="2"/>
    </ItemSet>
  </Items>
</PathOfBuilding>"#;

    #[test]
    fn reads_class_stats_and_main_skill_of_active_set() {
        let b = parse_build(XML, "Meu build").unwrap();
        assert_eq!(b.name, "Meu build");
        assert_eq!(b.class.as_deref(), Some("Witch"));
        assert_eq!(b.ascendancy.as_deref(), Some("Elementalist"));
        assert_eq!(b.level, Some(90));
        assert_eq!(b.main_skill_gems, ["Spell Echo", "Fireball"]);
        assert_eq!(b.main_active, Some(1));
        assert_eq!(b.stats["EnergyShield"], 310.5);
        assert_eq!(b.stats["FireResistOverCap"], 12.0);
    }

    #[test]
    fn reads_item_sets_and_skill_sets() {
        let b = parse_build(XML, "x").unwrap();
        assert_eq!(b.active_skill_set.as_deref(), Some("2"));
        assert_eq!(b.skill_sets.len(), 2);
        // Skill set 1 não é o ativo: usa a heurística (aqui, o único grupo).
        assert_eq!(b.skill_sets[0].main_skill_gems, ["Cleave"]);
        assert_eq!(b.active_item_set.as_deref(), Some("2"));
        let titles: Vec<&str> = b.item_sets.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, ["Leveling", "Item set 2"]);
        // Arma reserva e frasco ficam de fora; slot vazio (itemId 0) também.
        assert_eq!(b.item_sets[0].items.len(), 1);
        assert_eq!(b.item_sets[0].items[0].slot, "Gloves");
        assert!(b.item_sets[0].items[0].text.contains("+30% to Fire Resistance"));
        assert!(b.item_sets[1].items[0].text.contains("{range:0.5}+(10-20)% to all Elemental Resistances"));
    }

    #[test]
    fn inactive_skill_set_does_not_reuse_the_main_group_index() {
        // mainSocketGroup="2" vale para o set ativo (2). No set 1 o grupo 2 é uma aura: a skill
        // principal tem de vir do grupo com mais gemas (ou do marcado para o Full DPS).
        let xml = r#"<PathOfBuilding>
  <Build mainSocketGroup="2"/>
  <Skills activeSkillSet="2">
    <SkillSet id="1" title="Leveling">
      <Skill enabled="true"><Gem nameSpec="Rolling Magma"/><Gem nameSpec="Arcane Surge"/><Gem nameSpec="Elemental Proliferation"/></Skill>
      <Skill enabled="true"><Gem nameSpec="Clarity"/></Skill>
    </SkillSet>
    <SkillSet id="2" title="Mapas">
      <Skill enabled="true"><Gem nameSpec="Herald of Ash"/></Skill>
      <Skill enabled="true"><Gem nameSpec="Fireball"/><Gem nameSpec="Spell Echo"/></Skill>
    </SkillSet>
    <SkillSet id="3" title="Bosses">
      <Skill enabled="true"><Gem nameSpec="Vaal Haste"/><Gem nameSpec="Enlighten"/><Gem nameSpec="Empower"/></Skill>
      <Skill enabled="true" includeInFullDPS="true"><Gem nameSpec="Vaal Fireball"/></Skill>
    </SkillSet>
  </Skills>
</PathOfBuilding>"#;
        let b = parse_build(xml, "x").unwrap();
        assert_eq!(b.main_skill_gems, ["Fireball", "Spell Echo"]);
        assert_eq!(b.skill_sets[0].main_skill_gems[0], "Rolling Magma");
        assert_eq!(b.skill_sets[1].main_skill_gems[0], "Fireball");
        assert_eq!(b.skill_sets[2].main_skill_gems, ["Vaal Fireball"]);
    }

    #[test]
    fn rejects_non_pob_xml() {
        assert!(parse_build("<Outro/>", "x").is_err());
        assert!(parse_build("não é xml", "x").is_err());
    }

    #[test]
    fn lists_xml_files_recursively() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("B.xml"), XML).unwrap();
        std::fs::create_dir(dir.path().join("Liga")).unwrap();
        std::fs::write(dir.path().join("Liga").join("a.xml"), XML).unwrap();
        std::fs::write(dir.path().join("notas.txt"), "x").unwrap();
        let repo = PobBuildRepository::new(Some(dir.path().to_path_buf()));
        let list = repo.list().unwrap();
        let names: Vec<(&str, Option<&str>)> = list.iter().map(|b| (b.name.as_str(), b.folder.as_deref())).collect();
        assert_eq!(names, [("B", None), ("a", Some("Liga"))]);
        let loaded = repo.load(&list[0].path).unwrap();
        assert_eq!(loaded.name, "B");
        assert!(repo.load("C:/nao/existe.xml").is_err());
    }

    /// Lê os builds reais da máquina (só quando existirem). `cargo test -- --ignored`
    #[test]
    #[ignore = "depende dos builds do PoB na máquina"]
    fn reads_local_builds() {
        let repo = PobBuildRepository::default();
        for file in repo.list().unwrap() {
            let b = repo.load(&file.path).unwrap_or_else(|e| panic!("{}: {e}", file.path));
            let k = mirrorsight_gamedata::EmbeddedKnowledge::poe1();
            let summary = mirrorsight_domain::evaluation::BuildProfile::from_input(&b, k).summary();
            println!(
                "{} → {:?} {:?} {:?} · {} item sets ({:?} ativo) · {} skill sets",
                file.name,
                summary.class,
                summary.main_skill,
                summary.focus,
                b.item_sets.len(),
                summary.item_set,
                b.skill_sets.len()
            );
        }
    }
}
