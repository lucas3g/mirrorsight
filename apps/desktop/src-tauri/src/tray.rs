//! Ícone na bandeja do sistema. O app vive aqui: fechar a janela só a esconde,
//! porque as hotkeys precisam do processo rodando enquanto você joga.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use std::sync::{Mutex, PoisonError};

use tauri::{App, AppHandle, Emitter, Manager, Wry};

/// Evento para o frontend da janela principal abrir a aba de configurações.
pub const EVENT_OPEN_SETTINGS: &str = "main:open-settings";
const MAIN: &str = "main";

const TRAY_ID: &str = "mirrorsight";

/// Itens do menu da bandeja, guardados para o frontend trocar o idioma depois.
pub struct TrayMenu {
    menu: Menu<Wry>,
    open: MenuItem<Wry>,
    settings: MenuItem<Wry>,
    quit: MenuItem<Wry>,
    /// "Reiniciar para atualizar": só entra no menu quando há atualização baixada.
    update: MenuItem<Wry>,
    update_state: Mutex<UpdateItem>,
}

#[derive(Default)]
struct UpdateItem {
    /// Texto traduzido, sem a versão.
    label: Option<String>,
    /// Versão pronta para instalar; `Some` = o item já está no menu.
    version: Option<String>,
}

impl UpdateItem {
    fn text(&self) -> String {
        let label = self.label.as_deref().unwrap_or("Restart to update");
        match &self.version {
            Some(v) => format!("{label} (v{v})"),
            None => label.to_owned(),
        }
    }
}

/// Textos traduzidos do menu da bandeja (vindos do frontend).
#[derive(Debug, serde::Deserialize)]
pub struct TrayLabels {
    pub open: String,
    pub settings: String,
    pub quit: String,
    pub tooltip: String,
    /// "Reiniciar para atualizar" (a versão é acrescentada aqui).
    pub update: String,
}

/// Aplica os textos no idioma escolhido pelo usuário.
pub fn set_labels(app: &AppHandle, labels: &TrayLabels) -> tauri::Result<()> {
    if let Some(menu) = app.try_state::<TrayMenu>() {
        menu.open.set_text(&labels.open)?;
        menu.settings.set_text(&labels.settings)?;
        menu.quit.set_text(&labels.quit)?;
        let text = {
            let mut item = menu.update_state.lock().unwrap_or_else(PoisonError::into_inner);
            item.label = Some(labels.update.clone());
            item.text()
        };
        menu.update.set_text(text)?;
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_tooltip(Some(&labels.tooltip))?;
    }
    Ok(())
}

pub fn build(app: &App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Mirrorsight", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let update = MenuItem::with_id(app, "update", "Restart to update", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &settings, &PredefinedMenuItem::separator(app)?, &quit])?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Mirrorsight — Ctrl+D")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app, false),
            "settings" => show_main(app, true),
            "update" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = crate::updater::install_pending(&app).await {
                        tracing::error!(%err, "falha ao instalar a atualização");
                    }
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle(), false);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    app.manage(TrayMenu { menu, open, settings, quit, update, update_state: Mutex::default() });
    Ok(())
}

/// Mostra "Reiniciar para atualizar (vX)" no topo do menu da bandeja.
pub fn set_update_ready(app: &AppHandle, version: &str) {
    let Some(tray) = app.try_state::<TrayMenu>() else { return };
    let (text, first_time) = {
        let mut item = tray.update_state.lock().unwrap_or_else(PoisonError::into_inner);
        let first_time = item.version.is_none();
        item.version = Some(version.to_owned());
        (item.text(), first_time)
    };
    let result = tray.update.set_text(text).and_then(|()| {
        if first_time {
            tray.menu.insert(&tray.update, 0)?;
            tray.menu.insert(&PredefinedMenuItem::separator(app)?, 1)?;
        }
        Ok(())
    });
    if let Err(err) = result {
        tracing::warn!(%err, "não foi possível mostrar a atualização na bandeja");
    }
}

/// Traz a janela principal para frente (opcionalmente já na aba de configurações).
pub fn show_main(app: &AppHandle, open_settings: bool) {
    if let Some(main) = app.get_webview_window(MAIN) {
        let _ = main.unminimize();
        let _ = main.show();
        let _ = main.set_focus();
        if open_settings {
            let _ = app.emit_to(MAIN, EVENT_OPEN_SETTINGS, ());
        }
    }
}
