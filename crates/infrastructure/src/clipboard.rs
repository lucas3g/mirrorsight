//! Clipboard do sistema via `arboard`.

use mirrorsight_domain::ports::{ClipboardPort, PortError, PortResult};

#[derive(Default)]
pub struct SystemClipboard;

fn open() -> PortResult<arboard::Clipboard> {
    arboard::Clipboard::new().map_err(|e| PortError::Other(format!("clipboard indisponível: {e}")))
}

impl ClipboardPort for SystemClipboard {
    fn read_text(&self) -> PortResult<Option<String>> {
        match open()?.get_text() {
            Ok(text) => Ok(Some(text)),
            // Clipboard vazio ou com conteúdo não-texto (imagem, arquivos).
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(PortError::Other(format!("falha ao ler o clipboard: {e}"))),
        }
    }

    fn write_text(&self, text: &str) -> PortResult<()> {
        open()?
            .set_text(text)
            .map_err(|e| PortError::Other(format!("falha ao escrever no clipboard: {e}")))
    }
}
