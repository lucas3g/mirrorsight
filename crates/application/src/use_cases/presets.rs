use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use mirrorsight_domain::ports::PresetRepository;
use mirrorsight_domain::presets::ModPreset;
use mirrorsight_domain::GameVersion;

use crate::AppError;

/// Presets de mods por jogo. Leitura-alteração-gravação sob lock (duas janelas gravam).
pub struct PresetService {
    repo: Arc<dyn PresetRepository>,
    lock: Mutex<()>,
}

impl PresetService {
    pub fn new(repo: Arc<dyn PresetRepository>) -> Self {
        Self {
            repo,
            lock: Mutex::new(()),
        }
    }

    fn guard(&self) -> MutexGuard<'_, ()> {
        self.lock.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn list(&self, game: GameVersion) -> Result<Vec<ModPreset>, AppError> {
        let _guard = self.guard();
        Ok(self.repo.load()?.for_game(game).to_vec())
    }

    /// Cria ou atualiza pelo `id`. Devolve a lista atualizada do jogo do preset.
    /// Nome/mods inválidos viram `AppError::InvalidSettings` (código que a UI já trata).
    pub fn upsert(&self, preset: ModPreset) -> Result<Vec<ModPreset>, AppError> {
        let _guard = self.guard();
        let game = preset.game;
        let mut book = self.repo.load()?;
        book.upsert(preset)?;
        self.repo.save(&book)?;
        Ok(book.for_game(game).to_vec())
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let _guard = self.guard();
        let mut book = self.repo.load()?;
        if book.remove(id) {
            self.repo.save(&book)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mirrorsight_domain::item::ModifierKind;
    use mirrorsight_domain::ports::PortResult;
    use mirrorsight_domain::presets::{PresetBook, PresetMod};

    use super::*;

    #[derive(Default)]
    struct MemoryRepo(Mutex<PresetBook>);
    impl PresetRepository for MemoryRepo {
        fn load(&self) -> PortResult<PresetBook> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn save(&self, book: &PresetBook) -> PortResult<()> {
            *self.0.lock().unwrap() = book.clone();
            Ok(())
        }
    }

    fn preset(id: &str, name: &str) -> ModPreset {
        ModPreset {
            id: id.into(),
            name: name.into(),
            game: GameVersion::Poe1,
            created_ms: 0,
            mods: vec![PresetMod {
                trade_id: "explicit.stat_3299347043".into(),
                text: "+# to maximum Life".into(),
                kind: ModifierKind::Explicit,
                min: Some(70.0),
                max: None,
            }],
        }
    }

    #[test]
    fn upsert_updates_by_id_and_delete_removes() {
        let service = PresetService::new(Arc::new(MemoryRepo::default()));
        service.upsert(preset("a", "Vida")).unwrap();
        let list = service.upsert(preset("a", "Vida + res")).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Vida + res");
        assert!(service.list(GameVersion::Poe2).unwrap().is_empty());

        service.delete("a").unwrap();
        assert!(service.list(GameVersion::Poe1).unwrap().is_empty());
    }

    #[test]
    fn invalid_preset_maps_to_invalid_settings() {
        let service = PresetService::new(Arc::new(MemoryRepo::default()));
        let err = service.upsert(preset("a", "  ")).unwrap_err();
        assert!(matches!(err, AppError::InvalidSettings { .. }));
    }
}
