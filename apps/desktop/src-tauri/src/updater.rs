//! Atualização automática via GitHub Releases (`tauri-plugin-updater`).
//!
//! Estratégia (o app é um overlay usado durante o jogo, então nunca fecha sozinho no meio da sessão):
//! - **Ao abrir:** se houver versão nova, baixa e instala na hora. O instalador NSIS (modo passivo)
//!   mostra só uma barra de progresso e reabre o app com os mesmos argumentos.
//! - **Durante a sessão:** a cada [`CHECK_INTERVAL`] verifica de novo; se houver versão nova, só
//!   baixa e avisa a UI/bandeja ([`EVENT_READY`]). A instalação espera o clique do usuário.
//!
//! O manifesto (`latest.json`) e a assinatura são publicados pela Action `Release`.
//! Falhas de rede vão só para o log: atualizar nunca pode atrapalhar o price check.

use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};
use tokio::sync::Mutex;

use crate::state::AppState;
use crate::tray;

/// Evento para a janela principal: atualização baixada e pronta para instalar.
pub const EVENT_READY: &str = "updater:ready";
const MAIN: &str = "main";

/// Espera antes da checagem de abertura (rede e janela ainda subindo).
const STARTUP_DELAY: Duration = Duration::from_secs(5);
/// Intervalo das checagens durante a sessão.
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// Versão nova já baixada, à espera do "Reiniciar".
#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
}

/// Resultado de uma checagem (comando `check_for_update`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum UpdateCheck {
    UpToDate,
    Ready(UpdateInfo),
}

struct Pending {
    update: Update,
    bytes: Vec<u8>,
}

/// Estado compartilhado: a atualização pendente. O `Mutex` assíncrono também serializa as
/// checagens (automática e manual nunca baixam a mesma versão duas vezes).
#[derive(Default)]
pub struct UpdaterState {
    pending: Mutex<Option<Pending>>,
}

fn info(update: &Update) -> UpdateInfo {
    UpdateInfo { version: update.version.clone(), notes: update.body.clone() }
}

fn auto_update_enabled(app: &AppHandle) -> bool {
    // Sem conseguir ler as configurações, vale o padrão (ligado).
    app.state::<AppState>().settings_repo.load().map_or(true, |s| s.auto_update)
}

/// Agenda a checagem de abertura e as periódicas. Chamar no `setup`.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_DELAY).await;
        if auto_update_enabled(&app) {
            if let Err(err) = install_on_startup(&app).await {
                tracing::warn!(%err, "falha ao verificar atualização na abertura");
            }
        }

        let mut ticker = tokio::time::interval(CHECK_INTERVAL);
        ticker.tick().await; // o primeiro tick é imediato: a abertura já foi tratada acima
        loop {
            ticker.tick().await;
            if !auto_update_enabled(&app) {
                continue;
            }
            if let Err(err) = check_and_download(&app).await {
                tracing::warn!(%err, "falha ao verificar atualização");
            }
        }
    });
}

/// Abertura do app: o usuário ainda não está usando o overlay, então instala direto.
async fn install_on_startup(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
    let state = app.state::<UpdaterState>();
    let _guard = state.pending.lock().await;
    let Some(update) = app.updater()?.check().await? else {
        tracing::debug!("nenhuma atualização disponível");
        return Ok(());
    };
    tracing::info!(version = %update.version, "atualização encontrada na abertura: instalando");
    update.download_and_install(|_, _| {}, || {}).await?;
    // No Windows o `install` já encerrou o processo (o instalador reabre o app).
    app.restart();
}

/// Verifica e, se houver versão nova, baixa em segundo plano e avisa a UI e a bandeja.
/// Não instala: isso fica para o clique do usuário ([`install_pending`]).
pub async fn check_and_download(app: &AppHandle) -> tauri_plugin_updater::Result<UpdateCheck> {
    let state = app.state::<UpdaterState>();
    let mut pending = state.pending.lock().await;

    let Some(update) = app.updater()?.check().await? else {
        // Pode já haver uma baixada antes (o manifesto não "volta" de versão, mas por garantia).
        return Ok(pending.as_ref().map_or(UpdateCheck::UpToDate, |p| UpdateCheck::Ready(info(&p.update))));
    };
    if let Some(current) = pending.as_ref() {
        if current.update.version == update.version {
            return Ok(UpdateCheck::Ready(info(&current.update)));
        }
    }

    tracing::info!(version = %update.version, "baixando atualização em segundo plano");
    let bytes = update.download(|_, _| {}, || {}).await?;
    let ready = info(&update);
    *pending = Some(Pending { update, bytes });
    drop(pending);

    tray::set_update_ready(app, &ready.version);
    let _ = app.emit_to(MAIN, EVENT_READY, &ready);
    Ok(UpdateCheck::Ready(ready))
}

/// Atualização já baixada, se houver (para a UI que abriu depois do evento).
pub async fn pending_info(app: &AppHandle) -> Option<UpdateInfo> {
    app.state::<UpdaterState>().pending.lock().await.as_ref().map(|p| info(&p.update))
}

/// Instala a atualização baixada e reinicia. Sem nada pendente, não faz nada.
pub async fn install_pending(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
    let state = app.state::<UpdaterState>();
    let pending = state.pending.lock().await;
    let Some(p) = pending.as_ref() else {
        return Ok(());
    };
    tracing::info!(version = %p.update.version, "instalando atualização");
    p.update.install(&p.bytes)?;
    drop(pending);
    // No Windows o `install` já encerrou o processo (o instalador reabre o app).
    app.restart();
}
