//! Comandos Tauri: finos — validam, chamam um caso de uso, devolvem DTO/erro.

use tauri::{AppHandle, State};

use mirrorsight_application::use_cases::PriceCheckOutput;
use mirrorsight_application::AppError;
use mirrorsight_domain::history::HistoryEntry;
use mirrorsight_domain::hotkey::{validate_bindings, HotkeyBinding};
use mirrorsight_domain::ports::ItemParseError;
use mirrorsight_domain::presets::ModPreset;
use mirrorsight_domain::search::SearchFilters;
use mirrorsight_domain::settings::Settings;
use mirrorsight_domain::GameVersion;
use mirrorsight_parser::catalog::{CatalogStat, StatCatalog};

use crate::hotkeys::HotkeyManager;
use crate::overlay;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, AppError> {
    Ok(state.settings_repo.load()?)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    hotkeys: State<'_, HotkeyManager>,
    settings: Settings,
) -> Result<(), AppError> {
    state.save_settings.execute(&settings)?;
    // Teclas podem ter mudado: o watcher registra de novo no próximo ciclo.
    hotkeys.invalidate(&app);
    apply_autostart(&app, settings.launch_on_startup);
    Ok(())
}

/// Sincroniza "Iniciar com o Windows" com o registro do sistema. Falha aqui não
/// invalida o salvamento das demais configurações.
fn apply_autostart(app: &AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    if launcher.is_enabled().ok() == Some(enabled) {
        return;
    }
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    if let Err(err) = result {
        tracing::warn!(%err, enabled, "não foi possível alterar a inicialização com o Windows");
    }
}

/// Alarga/restaura o overlay para o card do item listado (olho). `extra` em px lógicos.
#[tauri::command]
pub fn overlay_peek(app: AppHandle, open: bool, extra: f64) -> Result<overlay::PeekSide, AppError> {
    overlay::peek(&app, open, extra).map_err(|e| AppError::Internal { message: e.to_string() })
}

/// Fecha o overlay e devolve o foco ao jogo.
#[tauri::command]
pub fn hide_overlay(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    overlay::hide(&app).map_err(|e| AppError::Internal { message: e.to_string() })?;
    // Jogo fechado ou nunca visto: não há a quem devolver o foco — não é erro.
    let _ = state.game_window.focus_game();
    Ok(())
}

/// Valida um conjunto de hotkeys com as mesmas regras do domínio (feedback imediato na UI).
#[tauri::command]
pub fn validate_hotkeys(bindings: Vec<HotkeyBinding>) -> Result<(), AppError> {
    Ok(validate_bindings(&bindings)?)
}

/// Ligas de trade disponíveis (a primeira é a liga de desafio atual).
#[tauri::command]
pub async fn list_leagues(state: State<'_, AppState>, game: GameVersion) -> Result<Vec<String>, AppError> {
    state.list_leagues.execute(game).await
}

/// Price check a partir do texto do item. `filters` ausente → filtros padrão.
#[tauri::command]
pub async fn price_check_text(
    state: State<'_, AppState>,
    text: String,
    filters: Option<SearchFilters>,
) -> Result<PriceCheckOutput, AppError> {
    if text.trim().is_empty() {
        return Err(ItemParseError::NotAnItem.into());
    }
    let settings = state.settings_repo.load()?;
    state.price_check.execute(&settings, &text, filters).await
}

/// Catálogo de stats do jogo, para a busca "Adicionar mod" (carregado uma vez por jogo).
#[tauri::command]
pub fn list_stats(game: GameVersion) -> Vec<CatalogStat> {
    StatCatalog::embedded(game).entries().to_vec()
}

/// Histórico local de price checks (mais recentes primeiro).
#[tauri::command]
pub fn get_history(state: State<'_, AppState>) -> Result<Vec<HistoryEntry>, AppError> {
    state.history.list()
}

#[tauri::command]
pub fn record_history(state: State<'_, AppState>, entry: HistoryEntry) -> Result<Vec<HistoryEntry>, AppError> {
    state.history.record(entry)
}

#[tauri::command]
pub fn remove_history(state: State<'_, AppState>, id: String) -> Result<Vec<HistoryEntry>, AppError> {
    state.history.remove(&id)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), AppError> {
    state.history.clear()
}

/// Presets de mods do jogo.
#[tauri::command]
pub fn list_presets(state: State<'_, AppState>, game: GameVersion) -> Result<Vec<ModPreset>, AppError> {
    state.presets.list(game)
}

/// Cria ou atualiza; devolve a lista atualizada do jogo do preset.
#[tauri::command]
pub fn upsert_preset(state: State<'_, AppState>, preset: ModPreset) -> Result<Vec<ModPreset>, AppError> {
    state.presets.upsert(preset)
}

#[tauri::command]
pub fn delete_preset(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.presets.delete(&id)
}
