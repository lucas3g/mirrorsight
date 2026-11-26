//! Janela overlay: transparente, sem borda, sempre no topo, aberta no centro do jogo e arrastável.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use mirrorsight_application::AppError;

use crate::state::AppState;

pub const LABEL: &str = "overlay";

/// Largura lógica do painel Aurora compacto (colunas 400px | resto).
const WIDTH: f64 = 820.0;
/// Fração da altura do monitor ocupada pelo overlay.
const HEIGHT_RATIO: f64 = 0.86;

/// Pedido de price check enviado ao frontend do overlay.
#[derive(Clone, Serialize)]
pub struct PriceCheckRequest {
    pub text: String,
    /// Abre já fixado (hotkey "price check fixado").
    pub locked: bool,
}

pub const EVENT_REQUEST: &str = "overlay:price-check";
pub const EVENT_ERROR: &str = "overlay:error";
/// Hotkey "mostrar overlay" com o painel fechado: o frontend reabre o último price check.
pub const EVENT_REOPEN: &str = "overlay:reopen";

/// Já houve algum price check nesta sessão (há o que reabrir com "mostrar overlay").
static HAS_ITEM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn has_item() -> bool {
    HAS_ITEM.load(std::sync::atomic::Ordering::Relaxed)
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

/// Como restaurar o overlay depois de expandir para o card do olho.
#[derive(Clone, Copy)]
struct PeekRestore {
    /// Tamanho do painel antes de crescer.
    size: PhysicalSize<u32>,
    /// Quanto a janela andou para a esquerda ao crescer (0 se cresceu para a direita).
    shift: i32,
}

/// Estado do peek em andamento. A posição não é guardada: o usuário pode ter arrastado
/// a janela, então a restauração parte da posição atual.
#[derive(Default)]
pub struct OverlayGeometry(Mutex<Option<PeekRestore>>);

impl OverlayGeometry {
    fn set(&self, value: Option<PeekRestore>) {
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }
    fn take(&self) -> Option<PeekRestore> {
        self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take()
    }
}

/// Para onde o overlay cresceu para mostrar o card do item listado.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PeekSide {
    Right,
    Left,
    /// Sem espaço no monitor: o card fica por cima do próprio painel.
    Inside,
}

/// Move e redimensiona numa única operação: em dois passos (posição, depois tamanho) o
/// painel "pula" sob o mouse, o olho perde o hover e o card entra em loop de abrir/fechar.
fn set_bounds(win: &WebviewWindow, pos: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> tauri::Result<()> {
    #[cfg(windows)]
    if let Ok(hwnd) = win.hwnd() {
        mirrorsight_infrastructure::win32::set_bounds_raw(hwnd.0 as isize, pos.x, pos.y, size.width, size.height);
        return Ok(());
    }
    win.set_size(size)?;
    win.set_position(pos)
}

/// Alarga o overlay em `extra` px **físicos** para o card do olho (`open`), ou restaura.
/// O frontend manda o valor já multiplicado por `devicePixelRatio` (DPI do monitor × escala
/// da interface), senão o card é cortado com a escala acima de 100%.
/// Cresce para a direita; sem espaço, para a esquerda (o frontend compensa o deslocamento
/// para o painel ficar parado na tela).
pub fn peek(app: &AppHandle, open: bool, extra: f64) -> tauri::Result<PeekSide> {
    let Some(win) = window(app) else { return Ok(PeekSide::Inside) };
    let geometry = app.state::<OverlayGeometry>();

    if !open {
        if let Some(restore) = geometry.take() {
            let pos = win.outer_position()?;
            set_bounds(&win, PhysicalPosition::new(pos.x + restore.shift, pos.y), restore.size)?;
        }
        return Ok(PeekSide::Inside);
    }

    // Já expandido: não cresce de novo.
    if geometry.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_some() {
        return Ok(PeekSide::Inside);
    }
    let pos = win.outer_position()?;
    let size = win.inner_size()?;
    let Some(monitor) = win.current_monitor()? else { return Ok(PeekSide::Inside) };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let extra = extra.max(0.0).ceil() as u32;
    let extra_i = i32::try_from(extra).unwrap_or(i32::MAX);
    let width_i = i32::try_from(size.width).unwrap_or(i32::MAX);
    let monitor_left = monitor.position().x;
    let monitor_right = monitor_left + i32::try_from(monitor.size().width).unwrap_or(i32::MAX);
    let grown = PhysicalSize::new(size.width + extra, size.height);

    if pos.x + width_i + extra_i <= monitor_right {
        win.set_size(grown)?;
        geometry.set(Some(PeekRestore { size, shift: 0 }));
        Ok(PeekSide::Right)
    } else if pos.x - extra_i >= monitor_left {
        set_bounds(&win, PhysicalPosition::new(pos.x - extra_i, pos.y), grown)?;
        geometry.set(Some(PeekRestore { size, shift: extra_i }));
        Ok(PeekSide::Left)
    } else {
        Ok(PeekSide::Inside)
    }
}

/// Área de referência para centralizar, em pixels físicos.
struct Area {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
}

/// A janela do jogo; sem jogo (fechado/minimizado), o monitor do cursor.
fn target_area(app: &AppHandle, win: &WebviewWindow) -> tauri::Result<Option<Area>> {
    if let Some(rect) = app.state::<AppState>().game_window.game_rect() {
        let (x, y) = (f64::from(rect.x), f64::from(rect.y));
        let (w, h) = (f64::from(rect.width), f64::from(rect.height));
        let scale = app
            .monitor_from_point(x + w / 2.0, y + h / 2.0)?
            .or(win.primary_monitor()?)
            .map_or(1.0, |m| m.scale_factor());
        return Ok(Some(Area { x, y, w, h, scale }));
    }
    let cursor = app.cursor_position()?;
    let monitor = app.monitor_from_point(cursor.x, cursor.y)?.or(win.primary_monitor()?);
    Ok(monitor.map(|m| Area {
        x: f64::from(m.position().x),
        y: f64::from(m.position().y),
        w: f64::from(m.size().width),
        h: f64::from(m.size().height),
        scale: m.scale_factor(),
    }))
}

/// Centraliza na janela do Path of Exile (ou no monitor do cursor), mostra e foca.
/// Arrastar vale só até fechar: cada abertura volta ao centro do jogo.
pub fn show_centered(app: &AppHandle) -> tauri::Result<()> {
    let Some(win) = window(app) else { return Ok(()) };

    if let Some(area) = target_area(app, &win)? {
        let width = (WIDTH * area.scale).min(area.w);
        let height = (area.h * HEIGHT_RATIO).floor();
        let x = area.x + (area.w - width) / 2.0;
        let y = area.y + (area.h - height) / 2.0;

        // Um peek esquecido não pode ser "restaurado" por cima da geometria nova.
        app.state::<OverlayGeometry>().set(None);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            win.set_size(PhysicalSize::new(width as u32, height as u32))?;
            win.set_position(PhysicalPosition::new(x as i32, y as i32))?;
        }
    }

    win.show()?;
    win.set_always_on_top(true)?;
    focus(&win);
    Ok(())
}

/// Foco "na marra": o Windows não deixa um processo em segundo plano roubar o foco.
fn focus(win: &WebviewWindow) {
    let _ = win.set_focus();
    #[cfg(windows)]
    if let Ok(hwnd) = win.hwnd() {
        mirrorsight_infrastructure::win32::force_foreground_raw(hwnd.0 as isize);
    }
}

pub fn hide(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = window(app) {
        win.hide()?;
    }
    Ok(())
}

pub fn is_visible(app: &AppHandle) -> bool {
    window(app).and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

pub fn request(app: &AppHandle, request: PriceCheckRequest) -> tauri::Result<()> {
    HAS_ITEM.store(true, std::sync::atomic::Ordering::Relaxed);
    show_centered(app)?;
    app.emit_to(LABEL, EVENT_REQUEST, request)
}

/// Mostra o overlay com o último price check (sem nova busca).
pub fn reopen(app: &AppHandle) -> tauri::Result<()> {
    show_centered(app)?;
    app.emit_to(LABEL, EVENT_REOPEN, ())
}

pub fn error(app: &AppHandle, err: &AppError) -> tauri::Result<()> {
    show_centered(app)?;
    app.emit_to(LABEL, EVENT_ERROR, err)
}
