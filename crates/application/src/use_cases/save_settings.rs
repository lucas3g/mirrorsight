use std::sync::Arc;

use mirrorsight_domain::hotkey::validate_bindings;
use mirrorsight_domain::ports::SettingsRepository;
use mirrorsight_domain::settings::Settings;

use crate::AppError;

/// Valida e persiste as configurações.
pub struct SaveSettings {
    repo: Arc<dyn SettingsRepository>,
}

impl SaveSettings {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, settings: &Settings) -> Result<(), AppError> {
        validate_bindings(&settings.hotkeys)?;
        if !(0.5..=2.0).contains(&settings.ui_scale) {
            return Err(AppError::InvalidSettings {
                message: "a escala da UI deve estar entre 50% e 200%".into(),
            });
        }
        if settings.mod_range_percent > 50 {
            return Err(AppError::InvalidSettings {
                message: "o range dos mods deve ficar entre 0% e 50%".into(),
            });
        }
        self.repo.save(settings)?;
        Ok(())
    }
}
