use std::sync::Arc;
use std::time::Duration;

use mirrorsight_domain::ports::{ClipboardPort, InputPort};

use crate::AppError;

/// Parâmetros de polling do clipboard (valores do APT, ajustáveis em testes).
#[derive(Debug, Clone, Copy)]
pub struct ClipboardPolicy {
    pub poll_interval: Duration,
    pub poll_limit: Duration,
    pub restore_after: Duration,
}

impl Default for ClipboardPolicy {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(48),
            poll_limit: Duration::from_millis(500),
            restore_after: Duration::from_millis(120),
        }
    }
}

/// Envia Ctrl+C ao jogo e aguarda o texto do item aparecer no clipboard.
pub struct CopyItemFromGame {
    input: Arc<dyn InputPort>,
    clipboard: Arc<dyn ClipboardPort>,
    is_item_text: fn(&str) -> bool,
    policy: ClipboardPolicy,
}

impl CopyItemFromGame {
    pub fn new(
        input: Arc<dyn InputPort>,
        clipboard: Arc<dyn ClipboardPort>,
        is_item_text: fn(&str) -> bool,
        policy: ClipboardPolicy,
    ) -> Self {
        Self {
            input,
            clipboard,
            is_item_text,
            policy,
        }
    }

    pub async fn execute(&self, restore_clipboard: bool) -> Result<String, AppError> {
        let previous = self.clipboard.read_text()?;
        if previous.as_deref().is_some_and(self.is_item_text) {
            // Evita aceitar uma cópia antiga como se fosse a nova.
            self.clipboard.write_text("")?;
        }

        self.input.release_modifiers()?;
        self.input.send_copy()?;

        let text = self.poll_for_item().await?;

        if restore_clipboard {
            if let Some(prev) = previous {
                tokio::time::sleep(self.policy.restore_after).await;
                self.clipboard.write_text(&prev)?;
            }
        }
        Ok(text)
    }

    async fn poll_for_item(&self) -> Result<String, AppError> {
        let mut elapsed = Duration::ZERO;
        while elapsed <= self.policy.poll_limit {
            if let Some(text) = self.clipboard.read_text()? {
                if (self.is_item_text)(&text) {
                    return Ok(text);
                }
            }
            tokio::time::sleep(self.policy.poll_interval).await;
            elapsed += self.policy.poll_interval;
        }
        Err(AppError::NothingCopied)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use mirrorsight_domain::ports::PortResult;

    use super::*;

    #[derive(Default)]
    struct FakeClipboard(Mutex<Option<String>>);
    impl ClipboardPort for FakeClipboard {
        fn read_text(&self) -> PortResult<Option<String>> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn write_text(&self, text: &str) -> PortResult<()> {
            *self.0.lock().unwrap() = Some(text.to_owned());
            Ok(())
        }
    }

    /// Simula o jogo: ao receber Ctrl+C, escreve o item no clipboard.
    struct FakeGame(Arc<FakeClipboard>);
    impl InputPort for FakeGame {
        fn release_modifiers(&self) -> PortResult<()> {
            Ok(())
        }
        fn send_copy(&self) -> PortResult<()> {
            self.0.write_text("Item Class: Rings\nRarity: Rare")
        }
    }

    struct DeadGame;
    impl InputPort for DeadGame {
        fn release_modifiers(&self) -> PortResult<()> {
            Ok(())
        }
        fn send_copy(&self) -> PortResult<()> {
            Ok(())
        }
    }

    fn is_item(t: &str) -> bool {
        t.starts_with("Item Class: ")
    }

    #[tokio::test(start_paused = true)]
    async fn copies_and_restores_clipboard() {
        let clip = Arc::new(FakeClipboard::default());
        clip.write_text("texto do usuário").unwrap();
        let uc = CopyItemFromGame::new(
            Arc::new(FakeGame(clip.clone())),
            clip.clone(),
            is_item,
            ClipboardPolicy::default(),
        );

        let text = uc.execute(true).await.unwrap();

        assert!(text.starts_with("Item Class: Rings"));
        assert_eq!(clip.read_text().unwrap().as_deref(), Some("texto do usuário"));
    }

    #[tokio::test(start_paused = true)]
    async fn times_out_when_game_does_not_copy() {
        let clip = Arc::new(FakeClipboard::default());
        let uc = CopyItemFromGame::new(
            Arc::new(DeadGame),
            clip,
            is_item,
            ClipboardPolicy::default(),
        );
        assert!(matches!(uc.execute(false).await, Err(AppError::NothingCopied)));
    }
}
