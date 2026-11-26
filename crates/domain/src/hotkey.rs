use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::DomainError;

/// Ações que podem ser disparadas por hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HotkeyAction {
    PriceCheck,
    PriceCheckLocked,
    ToggleOverlay,
    OpenSettings,
}

/// Combinação normalizada, ex.: `"Ctrl + D"`, `"Shift + Space"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hotkey(String);

/// Teclas que o jogo usa e que nunca podem ser capturadas pelo overlay.
const RESERVED: &[&str] = &[
    "Ctrl + C",
    "Ctrl + V",
    "Ctrl + A",
    "Ctrl + F",
    "Ctrl + Enter",
    "Home",
    "Delete",
    "Enter",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
];

impl Hotkey {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let parts: Vec<&str> = raw
            .split('+')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        if parts.is_empty() {
            return Err(DomainError::InvalidHotkey(raw.to_owned()));
        }
        Ok(Self(parts.join(" + ")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_reserved(&self) -> bool {
        RESERVED.iter().any(|r| r.eq_ignore_ascii_case(&self.0))
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Hotkey {
    type Error = DomainError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Hotkey> for String {
    fn from(value: Hotkey) -> Self {
        value.0
    }
}

/// Vínculo ação → tecla.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub action: HotkeyAction,
    pub hotkey: Hotkey,
}

/// Valida um conjunto de bindings: sem teclas reservadas e sem duplicatas.
pub fn validate_bindings(bindings: &[HotkeyBinding]) -> Result<(), DomainError> {
    let mut seen = HashSet::new();
    for binding in bindings {
        if binding.hotkey.is_reserved() {
            return Err(DomainError::ReservedHotkey(binding.hotkey.to_string()));
        }
        if !seen.insert(binding.hotkey.as_str().to_ascii_lowercase()) {
            return Err(DomainError::DuplicateHotkey(binding.hotkey.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bind(action: HotkeyAction, key: &str) -> HotkeyBinding {
        HotkeyBinding {
            action,
            hotkey: Hotkey::parse(key).expect("hotkey válida"),
        }
    }

    #[test]
    fn normalizes_spacing() {
        assert_eq!(Hotkey::parse("ctrl+d").unwrap().as_str(), "ctrl + d");
    }

    #[test]
    fn rejects_reserved() {
        let err = validate_bindings(&[bind(HotkeyAction::PriceCheck, "ctrl+c")]).unwrap_err();
        assert!(matches!(err, DomainError::ReservedHotkey(_)));
    }

    #[test]
    fn rejects_duplicates() {
        let err = validate_bindings(&[
            bind(HotkeyAction::PriceCheck, "Ctrl + D"),
            bind(HotkeyAction::OpenSettings, "ctrl+d"),
        ])
        .unwrap_err();
        assert!(matches!(err, DomainError::DuplicateHotkey(_)));
    }
}
