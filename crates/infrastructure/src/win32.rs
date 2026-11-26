//! Integração com o Windows: janela do jogo, input sintético e foco.
//!
//! Único módulo com `unsafe` no projeto. Cada chamada FFI tem um `// SAFETY:`.
#![allow(unsafe_code)]

use std::sync::Mutex;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentProcess, GetCurrentThreadId, OpenProcess, OpenProcessToken,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, FindWindowW, GetClassNameW, GetClientRect, GetForegroundWindow, GetWindowTextW,
    GetWindowThreadProcessId,
    IsIconic, IsWindow, SetForegroundWindow, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOOWNERZORDER,
    SWP_NOZORDER, SW_RESTORE,
};

use mirrorsight_domain::ports::{GameWindowPort, InputPort, PortError, PortResult, WindowRect};

/// Classe da janela do cliente (PoE1 e PoE2).
const GAME_CLASS: &str = "POEWindowClass";
const GAME_TITLES: &[&str] = &["Path of Exile", "Path of Exile 2"];
const VK_C: VIRTUAL_KEY = VIRTUAL_KEY(0x43);

/// Janela do jogo. Lembra o último HWND visto em foco para devolver o foco depois do overlay.
#[derive(Default)]
pub struct Win32GameWindow {
    /// HWND como `isize`: o ponteiro bruto não é `Send`.
    last_game: Mutex<Option<isize>>,
}

impl Win32GameWindow {
    fn remember(&self, hwnd: HWND) {
        *self.last_game.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(hwnd.0 as isize);
    }

    fn last(&self) -> Option<HWND> {
        let raw = (*self.last_game.lock().unwrap_or_else(std::sync::PoisonError::into_inner))?;
        let hwnd = HWND(raw as *mut _);
        // SAFETY: IsWindow aceita qualquer valor e só consulta se o handle ainda existe.
        unsafe { IsWindow(Some(hwnd)) }.as_bool().then_some(hwnd)
    }
}

impl GameWindowPort for Win32GameWindow {
    fn is_game_focused(&self) -> bool {
        // SAFETY: sem pré-condições; pode retornar HWND nulo.
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_invalid() || !is_game_window(hwnd) {
            return false;
        }
        self.remember(hwnd);
        true
    }

    fn focus_game(&self) -> PortResult<()> {
        let hwnd = self.last().ok_or(PortError::Known(mirrorsight_domain::error::Reason::GameWindowNotFound))?;
        force_foreground(hwnd);
        Ok(())
    }

    fn has_elevation_mismatch(&self) -> bool {
        let Some(hwnd) = self.last() else { return false };
        let mut pid = 0_u32;
        // SAFETY: hwnd válido (checado em `last`), `pid` é um u32 local.
        unsafe { GetWindowThreadProcessId(hwnd, Some(&raw mut pid)) };
        if pid == 0 {
            return false;
        }
        // SAFETY: OpenProcess com acesso mínimo; o handle é fechado em `process_elevated`.
        let game = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
        let Ok(game) = game else {
            // Acesso negado a um processo do mesmo usuário indica que ele roda elevado.
            return !current_process_elevated();
        };
        let game_elevated = process_elevated(game, true);
        game_elevated && !current_process_elevated()
    }

    fn game_rect(&self) -> Option<WindowRect> {
        let hwnd = self.last().or_else(find_game_window)?;
        // SAFETY: hwnd válido (IsWindow/FindWindowW); RECT/POINT são locais.
        unsafe {
            if IsIconic(hwnd).as_bool() {
                return None;
            }
            let mut rect = RECT::default();
            GetClientRect(hwnd, &raw mut rect).ok()?;
            let mut origin = POINT { x: rect.left, y: rect.top };
            if !ClientToScreen(hwnd, &raw mut origin).as_bool() {
                return None;
            }
            let width = u32::try_from(rect.right - rect.left).ok().filter(|w| *w > 0)?;
            let height = u32::try_from(rect.bottom - rect.top).ok().filter(|h| *h > 0)?;
            Some(WindowRect { x: origin.x, y: origin.y, width, height })
        }
    }
}

/// Procura o cliente pela classe (quando nenhum price check lembrou o HWND ainda).
fn find_game_window() -> Option<HWND> {
    let class: Vec<u16> = GAME_CLASS.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `class` é uma string UTF-16 terminada em zero que vive até o fim da chamada.
    let hwnd = unsafe { FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) }.ok()?;
    (!hwnd.is_invalid()).then_some(hwnd)
}

fn window_text(hwnd: HWND) -> String {
    let mut buf = [0_u16; 128];
    // SAFETY: buffer local com o tamanho informado pelo slice.
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..usize::try_from(len).unwrap_or(0)])
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0_u16; 64];
    // SAFETY: buffer local com o tamanho informado pelo slice.
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..usize::try_from(len).unwrap_or(0)])
}

fn is_game_window(hwnd: HWND) -> bool {
    class_name(hwnd) == GAME_CLASS || GAME_TITLES.contains(&window_text(hwnd).as_str())
}

/// Traz a janela para frente mesmo quando outro processo detém o foco
/// (o Windows bloqueia `SetForegroundWindow` de processos em segundo plano).
pub fn force_foreground(hwnd: HWND) {
    // SAFETY: todas as chamadas recebem handles/IDs obtidos do próprio sistema;
    // AttachThreadInput é desfeito no fim do bloco.
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let foreground = GetForegroundWindow();
        let fg_thread = GetWindowThreadProcessId(foreground, None);
        let our_thread = GetCurrentThreadId();
        let attached = fg_thread != 0 && fg_thread != our_thread && AttachThreadInput(our_thread, fg_thread, true).as_bool();
        let _ = SetForegroundWindow(hwnd);
        let _ = BringWindowToTop(hwnd);
        if attached {
            let _ = AttachThreadInput(our_thread, fg_thread, false);
        }
    }
}

/// Versão para handles vindos do Tauri (`isize`).
pub fn force_foreground_raw(hwnd: isize) {
    force_foreground(HWND(hwnd as *mut _));
}

/// Posição e tamanho numa só chamada (sem estado intermediário visível), sem mexer no
/// Z-order nem no foco.
pub fn set_bounds_raw(hwnd: isize, x: i32, y: i32, width: u32, height: u32) {
    let (w, h) = (i32::try_from(width).unwrap_or(i32::MAX), i32::try_from(height).unwrap_or(i32::MAX));
    // SAFETY: handle vindo do Tauri para uma janela do próprio processo.
    let _ = unsafe {
        SetWindowPos(HWND(hwnd as *mut _), None, x, y, w, h, SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER)
    };
}

fn current_process_elevated() -> bool {
    // SAFETY: pseudo-handle do próprio processo; não precisa ser fechado.
    process_elevated(unsafe { GetCurrentProcess() }, false)
}

/// Lê `TokenElevation`. `close` fecha o handle do processo ao final.
fn process_elevated(process: HANDLE, close: bool) -> bool {
    let mut token = HANDLE::default();
    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned = 0_u32;
    // SAFETY: ponteiros para variáveis locais com o tamanho correto; handles fechados abaixo.
    unsafe {
        let ok = OpenProcessToken(process, TOKEN_QUERY, &raw mut token).is_ok()
            && GetTokenInformation(
                token,
                TokenElevation,
                Some((&raw mut elevation).cast()),
                u32::try_from(std::mem::size_of::<TOKEN_ELEVATION>()).unwrap_or(0),
                &raw mut returned,
            )
            .is_ok();
        if !token.is_invalid() {
            let _ = CloseHandle(token);
        }
        if close {
            let _ = CloseHandle(process);
        }
        ok && elevation.TokenIsElevated != 0
    }
}

/// Envio de teclas via `SendInput`.
#[derive(Default)]
pub struct Win32Input;

fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(inputs: &[INPUT]) -> PortResult<()> {
    let size = i32::try_from(std::mem::size_of::<INPUT>()).unwrap_or(0);
    // SAFETY: slice de INPUT válidos; `size` é o tamanho da struct, como a API exige.
    let sent = unsafe { SendInput(inputs, size) };
    if usize::try_from(sent).unwrap_or(0) == inputs.len() {
        Ok(())
    } else {
        // Acontece quando o jogo roda como administrador e nós não (UIPI).
        Err(PortError::Known(mirrorsight_domain::error::Reason::InputBlocked))
    }
}

impl InputPort for Win32Input {
    fn release_modifiers(&self) -> PortResult<()> {
        let held: Vec<INPUT> = [VK_LCONTROL, VK_RCONTROL, VK_LSHIFT, VK_RSHIFT, VK_LMENU, VK_RMENU]
            .into_iter()
            // SAFETY: consulta de estado de tecla, sem pré-condições.
            .filter(|vk| unsafe { GetAsyncKeyState(i32::from(vk.0)) } < 0)
            .map(|vk| key(vk, true))
            .collect();
        if held.is_empty() { Ok(()) } else { send(&held) }
    }

    /// Ctrl+Alt+C: cópia avançada (traz faixas de roll e tiers dos mods).
    fn send_copy(&self) -> PortResult<()> {
        send(&[
            key(VK_CONTROL, false),
            key(VK_MENU, false),
            key(VK_C, false),
            key(VK_C, true),
            key(VK_MENU, true),
            key(VK_CONTROL, true),
        ])
    }
}
