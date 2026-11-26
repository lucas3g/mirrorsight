use std::fs;
use std::path::{Path, PathBuf};

use mirrorsight_domain::ports::{PortResult, SettingsRepository};
use mirrorsight_domain::settings::Settings;

use crate::json_file::{io_err, write_atomic};

/// Persiste [`Settings`] em JSON. Escrita atômica: grava em `.tmp` e renomeia.
pub struct JsonSettingsRepository {
    path: PathBuf,
}

impl JsonSettingsRepository {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join("settings.json"),
        }
    }
}

impl SettingsRepository for JsonSettingsRepository {
    fn load(&self) -> PortResult<Settings> {
        match fs::read_to_string(&self.path) {
            Ok(raw) => serde_json::from_str::<Settings>(&raw).map(Settings::migrate).or_else(|err| {
                // Arquivo corrompido não pode impedir o app de abrir.
                tracing::error!(%err, "settings.json inválido; usando padrões");
                Ok(Settings::default())
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(err) => Err(io_err(err)),
        }
    }

    fn save(&self, settings: &Settings) -> PortResult<()> {
        let json = serde_json::to_vec_pretty(settings).map_err(io_err)?;
        write_atomic(&self.path, &json)
    }
}

#[cfg(test)]
mod tests {
    use mirrorsight_domain::settings::{SuggestedLevel, SETTINGS_SCHEMA_VERSION};

    use super::*;

    #[test]
    fn returns_defaults_when_missing_and_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let repo = JsonSettingsRepository::new(dir.path());
        assert_eq!(repo.load().unwrap(), Settings::default());

        let settings = Settings {
            league: "Mercenaries".into(),
            suggested_level: SuggestedLevel::Sell,
            ..Settings::default()
        };
        repo.save(&settings).unwrap();
        assert_eq!(repo.load().unwrap(), settings);
    }

    #[test]
    fn corrupted_file_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("settings.json"), "{ quebrado").unwrap();
        let repo = JsonSettingsRepository::new(dir.path());
        assert_eq!(repo.load().unwrap(), Settings::default());
    }

    #[test]
    fn v1_file_is_migrated_on_load() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("settings.json"), r#"{ "schema_version": 1, "league": "Keepers" }"#).unwrap();
        let loaded = JsonSettingsRepository::new(dir.path()).load().unwrap();
        assert_eq!(loaded.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(loaded.league, "Keepers");
        assert_eq!(loaded.suggested_level, SuggestedLevel::Fair);
    }
}
