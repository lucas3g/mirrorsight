use serde::{Deserialize, Serialize};

use crate::hotkey::{Hotkey, HotkeyAction, HotkeyBinding};
use crate::item::ModifierKind;
use crate::GameVersion;

/// Versão atual do schema de configuração; incrementar a cada mudança incompatível e migrar.
pub const SETTINGS_SCHEMA_VERSION: u32 = 3;

/// Idiomas da interface (os mesmos dos READMEs). `None` em `ui_language` = idioma do sistema.
pub const UI_LANGUAGES: [&str; 7] = ["en", "pt-BR", "es", "de", "zh-CN", "ja", "ko"];

/// Nível do preço sugerido que vem selecionado e vai para a nota `~b/o`.
/// Compra = p25, Justo = mediana, Venda = p75 das listagens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SuggestedLevel {
    Buy,
    #[default]
    Fair,
    Sell,
}

/// Moeda em que a UI exibe preços convertidos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DisplayCurrency {
    #[default]
    Chaos,
    Divine,
}

/// Densidade das listas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    Compact,
    #[default]
    Normal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub game: GameVersion,
    pub league: String,
    /// Idioma do cliente do jogo (como o PoE copia o item).
    pub language: String,
    /// v3: idioma da interface; `None` = automático (idioma do sistema).
    pub ui_language: Option<String>,
    pub restore_clipboard: bool,
    pub launch_on_startup: bool,
    pub client_log_path: Option<String>,
    pub hotkeys: Vec<HotkeyBinding>,
    pub show_offline: bool,
    /// Mínimo pré-preenchido = valor rolado − este percentual.
    pub mod_range_percent: u8,
    /// Tipos de mod que já vêm marcados no filtro.
    pub default_filter_kinds: Vec<ModifierKind>,
    pub display_currency: DisplayCurrency,
    pub density: Density,
    pub ui_scale: f32,
    pub reduce_motion: bool,
    /// v2: nível padrão do preço sugerido.
    pub suggested_level: SuggestedLevel,
}

impl Settings {
    /// Sobe um arquivo antigo para o schema atual. Campos novos já chegam com o
    /// padrão via `#[serde(default)]`; aqui só ficam ajustes que dependem da versão.
    #[must_use]
    pub fn migrate(mut self) -> Self {
        if self.schema_version < 2 {
            // v1 → v2: `suggested_level` entrou (padrão `Fair`).
            self.schema_version = 2;
        }
        if self.schema_version < 3 {
            // v2 → v3: `ui_language` entrou (padrão `None` = automático).
            self.schema_version = 3;
        }
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        let bind = |action, key: &str| HotkeyBinding {
            action,
            hotkey: Hotkey::parse(key).unwrap_or_else(|_| unreachable!("hotkey padrão válida")),
        };
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            game: GameVersion::Poe1,
            league: "Standard".into(),
            language: "en".into(),
            ui_language: None,
            restore_clipboard: true,
            launch_on_startup: false,
            client_log_path: None,
            hotkeys: vec![
                bind(HotkeyAction::PriceCheck, "Ctrl + D"),
                bind(HotkeyAction::PriceCheckLocked, "Ctrl + Alt + D"),
                bind(HotkeyAction::ToggleOverlay, "Shift + Space"),
                bind(HotkeyAction::OpenSettings, "Ctrl + Shift + O"),
            ],
            show_offline: false,
            mod_range_percent: 10,
            default_filter_kinds: vec![ModifierKind::Explicit, ModifierKind::Fractured],
            display_currency: DisplayCurrency::Chaos,
            density: Density::Normal,
            ui_scale: 1.0,
            reduce_motion: false,
            suggested_level: SuggestedLevel::Fair,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_file_migrates_to_v2_with_fair_level() {
        let raw = r#"{ "schema_version": 1, "league": "Mercenaries", "mod_range_percent": 15 }"#;
        let settings = serde_json::from_str::<Settings>(raw).unwrap().migrate();
        assert_eq!(settings.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(settings.suggested_level, SuggestedLevel::Fair);
        assert_eq!(settings.league, "Mercenaries");
        assert_eq!(settings.mod_range_percent, 15);
    }

    #[test]
    fn v2_file_migrates_to_v3_with_automatic_ui_language() {
        let raw = r#"{ "schema_version": 2, "language": "pt" }"#;
        let settings = serde_json::from_str::<Settings>(raw).unwrap().migrate();
        assert_eq!(settings.schema_version, 3);
        assert_eq!(settings.ui_language, None);
        assert_eq!(settings.language, "pt");
    }
}
