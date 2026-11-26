//! Ícone na bandeja do sistema. O app vive aqui: fechar a janela só a esconde,
//! porque as hotkeys precisam do processo rodando enquanto você joga.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager};

/// Evento para o frontend da janela principal abrir a aba de configurações.
pub const EVENT_OPEN_SETTINGS: &str = "main:open-settings";
const MAIN: &str = "main";

pub fn build(app: &App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir Mirrorsight", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &settings, &PredefinedMenuItem::separator(app)?, &quit])?;

    let mut tray = TrayIconBuilder::with_id("mirrorsight")
        .tooltip("Mirrorsight — Ctrl+D sobre um item no jogo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app, false),
            "settings" => show_main(app, true),
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
    Ok(())
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
