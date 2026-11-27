//! Composition root: monta as dependências concretas e expõe os comandos à UI.
//! Nenhuma regra de negócio aqui — só wiring.

mod commands;
mod hotkeys;
mod overlay;
mod state;
mod tray;
mod updater;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

/// Argumento usado quando o Windows inicia o app junto com o sistema: abre só na bandeja.
pub const MINIMIZED_ARG: &str = "--minimized";

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mirrorsight=debug".into()),
        )
        .init();

    let result = tauri::Builder::default()
        // Primeiro plugin: uma segunda execução só traz a janela existente para frente.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| tray::show_main(app, false)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![MINIMIZED_ARG])))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(hotkeys::on_shortcut)
                .build(),
        )
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            app.manage(state::AppState::build(config_dir));
            app.manage(hotkeys::HotkeyManager::default());
            app.manage(overlay::OverlayGeometry::default());
            app.manage(updater::UpdaterState::default());
            tray::build(app)?;
            updater::spawn(app.handle().clone());
            hotkeys::spawn_focus_watcher(app.handle().clone());
            hotkeys::spawn_copy_interceptor(app.handle().clone());

            if std::env::args().any(|a| a == MINIMIZED_ARG) {
                if let Some(main) = app.get_webview_window("main") {
                    main.hide()?;
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Fechar a janela principal só esconde: o app continua na bandeja com as hotkeys ativas.
            if let (WindowEvent::CloseRequested { api, .. }, "main") = (event, window.label()) {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::set_tray_labels,
            commands::validate_hotkeys,
            commands::list_leagues,
            commands::price_check_text,
            commands::hide_overlay,
            commands::overlay_peek,
            commands::list_stats,
            commands::get_history,
            commands::record_history,
            commands::remove_history,
            commands::clear_history,
            commands::list_presets,
            commands::upsert_preset,
            commands::delete_preset,
            commands::list_pob_builds,
            commands::pob_build_details,
            commands::update_status,
            commands::check_for_update,
            commands::install_update,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        tracing::error!(%err, "falha ao iniciar o Mirrorsight");
        std::process::exit(1);
    }
}
