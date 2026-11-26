//! Hotkeys globais. Ficam registradas **só enquanto o jogo está em foco**: fora dele,
//! Ctrl+D etc. continuam funcionando normalmente nos outros programas.

use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use mirrorsight_application::AppError;
use mirrorsight_domain::hotkey::HotkeyAction;

use crate::overlay::{self, PriceCheckRequest};
use crate::state::AppState;

const WATCH_INTERVAL: Duration = Duration::from_millis(150);

/// Atalhos atualmente registrados (`None` = nada registrado).
#[derive(Default)]
pub struct HotkeyManager {
    registered: Mutex<Option<Vec<(Shortcut, HotkeyAction)>>>,
}

impl HotkeyManager {
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Vec<(Shortcut, HotkeyAction)>>> {
        self.registered.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// `true` se o estado registrado não corresponde ao foco atual do jogo.
    fn needs_sync(&self, game_focused: bool) -> bool {
        self.lock().is_some() != game_focused
    }

    /// Registra/desregistra conforme o foco do jogo.
    /// Precisa rodar na thread principal: `RegisterHotKey` só aceita a janela da própria thread.
    fn sync(&self, app: &AppHandle, game_focused: bool) {
        let mut registered = self.lock();
        match (game_focused, registered.is_some()) {
            (true, false) => {
                let list = register_from_settings(app);
                tracing::info!(count = list.len(), "jogo em foco: hotkeys registradas");
                *registered = Some(list);
            }
            (false, true) => {
                let _ = app.global_shortcut().unregister_all();
                tracing::info!("jogo sem foco: hotkeys liberadas");
                *registered = None;
            }
            _ => {}
        }
    }

    /// Após salvar configurações: força novo registro com as teclas atualizadas.
    pub fn invalidate(&self, app: &AppHandle) {
        let mut registered = self.lock();
        if registered.take().is_some() {
            let _ = app.global_shortcut().unregister_all();
        }
    }

    fn action_for(&self, shortcut: &Shortcut) -> Option<HotkeyAction> {
        self.lock()
            .as_ref()?
            .iter()
            .find(|(s, _)| s == shortcut)
            .map(|(_, action)| *action)
    }
}

/// "Ctrl + Shift + D" (formato do domínio) → "Ctrl+Shift+D" (formato do plugin).
fn to_shortcut(hotkey: &str) -> Option<Shortcut> {
    hotkey.replace(' ', "").parse().ok()
}

fn register_from_settings(app: &AppHandle) -> Vec<(Shortcut, HotkeyAction)> {
    let state = app.state::<AppState>();
    let bindings = state.settings_repo.load().map(|s| s.hotkeys).unwrap_or_default();
    bindings
        .into_iter()
        .filter_map(|b| match to_shortcut(b.hotkey.as_str()) {
            Some(shortcut) if app.global_shortcut().register(shortcut).is_ok() => Some((shortcut, b.action)),
            _ => {
                tracing::warn!(hotkey = %b.hotkey, "não foi possível registrar a hotkey");
                None
            }
        })
        .collect()
}

/// Thread que acompanha o foco do jogo; o (des)registro é feito na thread principal.
pub fn spawn_focus_watcher(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(WATCH_INTERVAL);
        let focused = app.state::<AppState>().game_window.is_game_focused();
        if app.state::<HotkeyManager>().needs_sync(focused) {
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || handle.state::<HotkeyManager>().sync(&handle, focused));
        }
    });
}

/// Handler do plugin: só reage ao "pressionar".
pub fn on_shortcut(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state != ShortcutState::Pressed {
        return;
    }
    let Some(action) = app.state::<HotkeyManager>().action_for(shortcut) else { return };
    tracing::info!(?action, "hotkey acionada");
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = dispatch(&app, action).await {
            tracing::error!(%err, ?action, "falha ao executar hotkey");
        }
    });
}

async fn dispatch(app: &AppHandle, action: HotkeyAction) -> tauri::Result<()> {
    match action {
        HotkeyAction::PriceCheck => price_check(app, false).await,
        HotkeyAction::PriceCheckLocked => price_check(app, true).await,
        HotkeyAction::ToggleOverlay => {
            if overlay::is_visible(app) {
                overlay::hide(app)?;
                let _ = app.state::<AppState>().game_window.focus_game();
                Ok(())
            } else if overlay::has_item() {
                // Antes só mostrava a janela, que fica vazia (transparente) com o painel fechado.
                overlay::reopen(app)
            } else {
                // Nada para reabrir ainda: abre a janela principal.
                crate::tray::show_main(app, false);
                Ok(())
            }
        }
        HotkeyAction::OpenSettings => {
            crate::tray::show_main(app, true);
            Ok(())
        }
    }
}

/// Intercepta o Ctrl+C físico com o jogo em foco (opção `intercept_copy`): faz a cópia
/// avançada (tiers e prefix/suffix) e abre o overlay como o atalho de price check. O Ctrl+C
/// simples é sempre devolvido ao jogo, então o clipboard e o chat seguem como antes. Erros vão
/// só para o log: abrir o overlay a cada Ctrl+C no chat seria intrusivo.
pub fn spawn_copy_interceptor(app: AppHandle) {
    #[cfg(windows)]
    {
        use mirrorsight_infrastructure::win32;

        let enabled = app.state::<AppState>().settings_repo.load().map(|s| s.intercept_copy).unwrap_or(true);
        win32::set_copy_hook_enabled(enabled);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        if !win32::spawn_copy_hook(move || {
            let _ = tx.send(());
        }) {
            return;
        }
        tauri::async_runtime::spawn(async move {
            while rx.recv().await.is_some() {
                intercepted_copy(&app).await;
                win32::finish_copy();
            }
        });
    }
    #[cfg(not(windows))]
    let _ = app;
}

/// Segue a configuração salva (chamado ao salvar as configurações).
pub fn set_copy_interception(enabled: bool) {
    #[cfg(windows)]
    mirrorsight_infrastructure::win32::set_copy_hook_enabled(enabled);
    #[cfg(not(windows))]
    let _ = enabled;
}

#[cfg(windows)]
async fn intercepted_copy(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.game_window.has_elevation_mismatch() {
        // Defesa: o hook já deixa passar o Ctrl+C quando o jogo roda elevado (o UIPI bloquearia
        // a cópia e o reenvio). Se chegar aqui mesmo assim, devolve o Ctrl+C (melhor esforço).
        tracing::warn!("Ctrl+C interceptado sem price check: o jogo roda como administrador");
        if let Err(err) = state.copy_item.resend_plain_copy() {
            tracing::warn!(%err, "não foi possível reenviar o Ctrl+C ao jogo");
        }
        return;
    }
    // `execute_intercepted` reenvia o Ctrl+C simples em todos os caminhos, inclusive no erro, e
    // só volta depois que o jogo o processou: o overlay abaixo não rouba esse Ctrl+C.
    match state.copy_item.execute_intercepted().await {
        Ok(Some(text)) => {
            tracing::info!(chars = text.len(), "item copiado pelo Ctrl+C");
            if let Err(err) = overlay::request(app, PriceCheckRequest { text, locked: false }) {
                tracing::error!(%err, "falha ao abrir o overlay");
            }
        }
        Ok(None) => tracing::debug!("Ctrl+C sem item sob o cursor: reenviado ao jogo"),
        Err(err) => tracing::warn!(%err, "falha no Ctrl+C interceptado"),
    }
}

/// Copia o item sob o cursor (Ctrl+Alt+C no jogo) e abre o overlay com ele.
async fn price_check(app: &AppHandle, locked: bool) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    if state.game_window.has_elevation_mismatch() {
        return overlay::error(app, &AppError::known(mirrorsight_domain::error::Reason::GameElevated));
    }

    let restore = state.settings_repo.load().map(|s| s.restore_clipboard).unwrap_or(true);
    match state.copy_item.execute(restore).await {
        Ok(text) => {
            tracing::info!(chars = text.len(), "item copiado do jogo");
            overlay::request(app, PriceCheckRequest { text, locked })
        }
        Err(err) => {
            tracing::warn!(%err, "não foi possível copiar o item");
            overlay::error(app, &err)
        }
    }
}
