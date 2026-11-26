//! Presets de mods salvos pelo usuário, separados por jogo.

use serde::{Deserialize, Serialize};

use crate::item::ModifierKind;
use crate::{DomainError, GameVersion};

/// Tamanho máximo do nome de um preset (em caracteres).
pub const PRESET_NAME_MAX: usize = 40;

/// Um mod ligado no momento em que o preset foi salvo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetMod {
    pub trade_id: String,
    pub text: String,
    pub kind: ModifierKind,
    /// `None` = mantém o mínimo/máximo calculado para o item.
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModPreset {
    pub id: String,
    pub name: String,
    pub game: GameVersion,
    /// Epoch em milissegundos.
    pub created_ms: i64,
    pub mods: Vec<PresetMod>,
}

impl ModPreset {
    /// Nome sem espaços nas pontas, 1 a [`PRESET_NAME_MAX`] caracteres, e ao menos um mod.
    pub fn validated(mut self) -> Result<Self, DomainError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(DomainError::InvalidPreset("dê um nome ao preset".into()));
        }
        if name.chars().count() > PRESET_NAME_MAX {
            return Err(DomainError::InvalidPreset(format!(
                "o nome do preset pode ter até {PRESET_NAME_MAX} caracteres"
            )));
        }
        if self.mods.is_empty() {
            return Err(DomainError::InvalidPreset("ligue ao menos um mod para salvar o preset".into()));
        }
        self.name = name.to_owned();
        Ok(self)
    }
}

/// Todos os presets, uma lista por jogo (mods e ids de stats não se misturam entre jogos).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PresetBook {
    pub poe1: Vec<ModPreset>,
    pub poe2: Vec<ModPreset>,
}

impl PresetBook {
    pub fn for_game(&self, game: GameVersion) -> &[ModPreset] {
        match game {
            GameVersion::Poe1 => &self.poe1,
            GameVersion::Poe2 => &self.poe2,
        }
    }

    fn for_game_mut(&mut self, game: GameVersion) -> &mut Vec<ModPreset> {
        match game {
            GameVersion::Poe1 => &mut self.poe1,
            GameVersion::Poe2 => &mut self.poe2,
        }
    }

    /// Cria ou atualiza (pelo `id`). Preset novo entra no fim da lista do seu jogo.
    pub fn upsert(&mut self, preset: ModPreset) -> Result<(), DomainError> {
        let preset = preset.validated()?;
        let (id, game) = (preset.id.clone(), preset.game);
        // Mesmo id em outro jogo não deveria existir; garante unicidade global.
        self.remove_where(|p| p.id == id && p.game != game);
        let list = self.for_game_mut(game);
        match list.iter().position(|p| p.id == id) {
            Some(index) => list[index] = preset,
            None => list.push(preset),
        }
        Ok(())
    }

    /// Remove pelo `id`. `true` se algo foi removido.
    pub fn remove(&mut self, id: &str) -> bool {
        self.remove_where(|p| p.id == id)
    }

    fn remove_where(&mut self, pred: impl Fn(&ModPreset) -> bool) -> bool {
        let before = self.poe1.len() + self.poe2.len();
        self.poe1.retain(|p| !pred(p));
        self.poe2.retain(|p| !pred(p));
        before != self.poe1.len() + self.poe2.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset(id: &str, name: &str, game: GameVersion, mods: usize) -> ModPreset {
        ModPreset {
            id: id.into(),
            name: name.into(),
            game,
            created_ms: 0,
            mods: (0..mods)
                .map(|i| PresetMod {
                    trade_id: format!("explicit.stat_{i}"),
                    text: "+# to maximum Life".into(),
                    kind: ModifierKind::Explicit,
                    min: Some(70.0),
                    max: None,
                })
                .collect(),
        }
    }

    #[test]
    fn validates_name_and_mods() {
        let mut book = PresetBook::default();
        assert!(book.upsert(preset("a", "   ", GameVersion::Poe1, 1)).is_err());
        assert!(book.upsert(preset("a", &"x".repeat(41), GameVersion::Poe1, 1)).is_err());
        assert!(book.upsert(preset("a", "Vida + res", GameVersion::Poe1, 0)).is_err());
        assert!(book.upsert(preset("a", "  Vida + res  ", GameVersion::Poe1, 2)).is_ok());
        assert_eq!(book.poe1[0].name, "Vida + res");
    }

    #[test]
    fn upsert_updates_by_id_and_keeps_games_apart() {
        let mut book = PresetBook::default();
        book.upsert(preset("a", "Um", GameVersion::Poe1, 1)).unwrap();
        book.upsert(preset("b", "Dois", GameVersion::Poe2, 1)).unwrap();
        book.upsert(preset("a", "Um renomeado", GameVersion::Poe1, 3)).unwrap();

        assert_eq!(book.for_game(GameVersion::Poe1).len(), 1);
        assert_eq!(book.for_game(GameVersion::Poe1)[0].name, "Um renomeado");
        assert_eq!(book.for_game(GameVersion::Poe1)[0].mods.len(), 3);
        assert_eq!(book.for_game(GameVersion::Poe2).len(), 1);

        assert!(book.remove("b"));
        assert!(!book.remove("b"));
        assert!(book.for_game(GameVersion::Poe2).is_empty());
    }
}
