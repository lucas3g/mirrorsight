use std::sync::Arc;

use mirrorsight_domain::hotkey::validate_bindings;
use mirrorsight_domain::ports::SettingsRepository;
use mirrorsight_domain::error::Reason;
use mirrorsight_domain::settings::{Settings, UI_LANGUAGES};

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
            return Err(AppError::invalid(Reason::UiScaleRange));
        }
        if settings.mod_range_percent > 50 {
            return Err(AppError::invalid(Reason::ModRangeRange));
        }
        if let Some(lang) = &settings.ui_language {
            if !UI_LANGUAGES.contains(&lang.as_str()) {
                return Err(AppError::invalid(Reason::UnknownUiLanguage { lang: lang.clone() }));
            }
        }
        self.repo.save(settings)?;
        Ok(())
    }
}
