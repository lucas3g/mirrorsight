//! Arquivos JSON locais na pasta de configurações: escrita atômica e os repositórios
//! do histórico (`history.json`) e dos presets de mods (`presets.json`).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use mirrorsight_domain::history::HistoryEntry;
use mirrorsight_domain::ports::{HistoryRepository, PortError, PortResult, PresetRepository};
use mirrorsight_domain::presets::PresetBook;

pub(crate) fn io_err(err: impl std::fmt::Display) -> PortError {
    PortError::Other(err.to_string())
}

/// Grava em `<arquivo>.tmp` e renomeia: um crash no meio nunca deixa o arquivo pela metade.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> PortResult<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(io_err)?;
    }
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp).map_err(io_err)?;
        file.write_all(bytes).map_err(io_err)?;
        file.sync_all().map_err(io_err)?;
    }
    fs::rename(&tmp, path).map_err(io_err)
}

/// Lê um JSON; ausente ou corrompido vira `T::default()` (não pode impedir o app de abrir).
fn read_or_default<T: DeserializeOwned + Default>(path: &Path) -> PortResult<T> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(serde_json::from_str(&raw).unwrap_or_else(|err| {
            tracing::error!(%err, file = %path.display(), "JSON inválido; começando vazio");
            T::default()
        })),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(err) => Err(io_err(err)),
    }
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> PortResult<()> {
    let json = serde_json::to_vec_pretty(value).map_err(io_err)?;
    write_atomic(path, &json)
}

/// Histórico de price checks em `history.json`.
pub struct JsonHistoryRepository {
    path: PathBuf,
}

impl JsonHistoryRepository {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join("history.json"),
        }
    }
}

impl HistoryRepository for JsonHistoryRepository {
    fn load(&self) -> PortResult<Vec<HistoryEntry>> {
        read_or_default(&self.path)
    }

    fn save(&self, entries: &[HistoryEntry]) -> PortResult<()> {
        write_json(&self.path, entries)
    }
}

/// Presets de mods em `presets.json` (`{ "poe1": [...], "poe2": [...] }`).
pub struct JsonPresetRepository {
    path: PathBuf,
}

impl JsonPresetRepository {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join("presets.json"),
        }
    }
}

impl PresetRepository for JsonPresetRepository {
    fn load(&self) -> PortResult<PresetBook> {
        read_or_default(&self.path)
    }

    fn save(&self, book: &PresetBook) -> PortResult<()> {
        write_json(&self.path, book)
    }
}

#[cfg(test)]
mod tests {
    use mirrorsight_domain::history::{HistoryItem, HistorySource};
    use mirrorsight_domain::item::{ModifierKind, Rarity};
    use mirrorsight_domain::presets::{ModPreset, PresetMod};
    use mirrorsight_domain::GameVersion;

    use super::*;

    fn entry() -> HistoryEntry {
        HistoryEntry {
            id: "1".into(),
            game: GameVersion::Poe2,
            league: "Standard".into(),
            item_text: "Rarity: Rare".into(),
            item: HistoryItem {
                name: Some("Doom Grip".into()),
                base_type: "Sorcerer Gloves".into(),
                rarity: Rarity::Rare,
                icon: None,
            },
            filters: None,
            buy_chaos: Some(40.0),
            fair_chaos: Some(50.0),
            sell_chaos: Some(60.0),
            sample: 20,
            source: HistorySource::Trade,
            divine_chaos: None,
            at_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn history_roundtrips_and_starts_empty() {
        let dir = tempfile::tempdir().unwrap();
        let repo = JsonHistoryRepository::new(dir.path());
        assert!(repo.load().unwrap().is_empty());
        repo.save(&[entry()]).unwrap();
        assert_eq!(repo.load().unwrap(), vec![entry()]);
    }

    #[test]
    fn presets_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let repo = JsonPresetRepository::new(dir.path());
        let mut book = PresetBook::default();
        book.upsert(ModPreset {
            id: "p".into(),
            name: "Vida".into(),
            game: GameVersion::Poe1,
            created_ms: 0,
            mods: vec![PresetMod {
                trade_id: "explicit.stat_3299347043".into(),
                text: "+# to maximum Life".into(),
                kind: ModifierKind::Explicit,
                min: Some(70.0),
                max: None,
            }],
        })
        .unwrap();
        repo.save(&book).unwrap();
        assert_eq!(repo.load().unwrap(), book);
    }

    #[test]
    fn corrupted_files_fall_back_to_empty() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("history.json"), "[ quebrado").unwrap();
        fs::write(dir.path().join("presets.json"), "{ quebrado").unwrap();
        assert!(JsonHistoryRepository::new(dir.path()).load().unwrap().is_empty());
        assert_eq!(JsonPresetRepository::new(dir.path()).load().unwrap(), PresetBook::default());
    }
}
