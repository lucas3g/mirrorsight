//! Adapters que implementam os ports de `mirrorsight-domain`.
//!
//! - `settings`    — repositório JSON com escrita atômica.               ✅
//! - `json_file`   — escrita atômica + histórico e presets em JSON.      ✅
//! - `trade`       — trade API (PoE1/PoE2) + rate limiter por headers.  ✅
//! - `ninja`       — índice poe.ninja (cotações e preço de currency).   ✅
//! - `unavailable` — adapters nulos (testes / modo offline).            ✅
//! - `win32`       — janela do jogo, SendInput, foco, elevação.         ✅
//! - `clipboard`   — clipboard do sistema.                              ✅
//! - `game_log`    — tail do Client.txt.                                ⏳
//!
//! `unsafe` só é permitido dentro de `win32`, com comentário `// SAFETY:`.

pub mod clipboard;
pub mod http;
pub mod json_file;
pub mod ninja;
pub mod pob;
pub mod rate_limit;
pub mod settings;
pub mod trade;
pub mod unavailable;
#[cfg(windows)]
pub mod win32;
