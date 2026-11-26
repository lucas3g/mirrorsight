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
    /// Ctrl+C interceptado: espera bem mais curta, porque todo Ctrl+C no jogo (inclusive o
    /// do chat) passa por ela antes de o Ctrl+C normal ser devolvido.
    pub intercept_poll_interval: Duration,
    pub intercept_poll_limit: Duration,
    /// Quanto esperar o jogo processar o Ctrl+C devolvido antes de abrir o overlay.
    pub resend_settle_limit: Duration,
}

impl Default for ClipboardPolicy {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(48),
            poll_limit: Duration::from_millis(500),
            restore_after: Duration::from_millis(120),
            intercept_poll_interval: Duration::from_millis(15),
            intercept_poll_limit: Duration::from_millis(200),
            resend_settle_limit: Duration::from_millis(250),
        }
    }
}

/// Envia Ctrl+Alt+C ao jogo e aguarda o texto do item aparecer no clipboard.
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

        let text = self.poll_for_item(self.policy.poll_interval, self.policy.poll_limit).await?;

        if restore_clipboard {
            if let Some(prev) = previous {
                tokio::time::sleep(self.policy.restore_after).await;
                self.clipboard.write_text(&prev)?;
            }
        }
        Ok(text)
    }

    /// Ctrl+C físico interceptado no jogo: faz a cópia avançada (tiers e prefix/suffix) e devolve
    /// o texto para o price check; `None` se não havia item sob o cursor.
    ///
    /// Em **todos** os caminhos (item, sem item ou erro) o Ctrl+C simples que o hook engoliu é
    /// reenviado no fim: o clipboard fica com o texto normal do jogo (o que o usuário pediu ao
    /// copiar, para colar no Discord etc.) e a cópia de texto do chat continua funcionando.
    ///
    /// Com item, só retorna depois que o jogo processou o Ctrl+C devolvido (o clipboard troca do
    /// texto avançado para o normal), com limite de [`ClipboardPolicy::resend_settle_limit`]:
    /// assim o overlay, aberto em seguida, não rouba o foco antes e não recebe esse Ctrl+C.
    pub async fn execute_intercepted(&self) -> Result<Option<String>, AppError> {
        let result = self.advanced_copy_or_none().await;
        let resent = self.input.send_plain_copy();
        match (&resent, &result) {
            (Err(err), Ok(_)) => tracing::warn!(%err, "não foi possível reenviar o Ctrl+C ao jogo"),
            (Err(err), Err(_)) => tracing::debug!(%err, "Ctrl+C também não pôde ser reenviado"),
            (Ok(()), _) => {}
        }
        if let (Ok(()), Ok(Some(advanced))) = (&resent, &result) {
            self.wait_clipboard_change(advanced).await;
        }
        result
    }

    /// Devolve ao jogo o Ctrl+C interceptado sem tentar a cópia avançada (ex.: jogo elevado).
    pub fn resend_plain_copy(&self) -> Result<(), AppError> {
        Ok(self.input.send_plain_copy()?)
    }

    /// Cópia avançada. Se não terminar com um item (sem item, falha de input ou do clipboard),
    /// o texto de item apagado do clipboard volta: nada do usuário se perde.
    async fn advanced_copy_or_none(&self) -> Result<Option<String>, AppError> {
        let previous = self.clipboard.read_text()?;
        let cleared = previous.filter(|p| (self.is_item_text)(p));
        if cleared.is_some() {
            self.clipboard.write_text("")?;
        }

        let outcome = self.advanced_copy_steps().await;
        if !matches!(outcome, Ok(Some(_))) {
            if let Some(prev) = &cleared {
                if let Err(err) = self.clipboard.write_text(prev) {
                    tracing::warn!(%err, "não foi possível devolver o conteúdo do clipboard");
                }
            }
        }
        outcome
    }

    async fn advanced_copy_steps(&self) -> Result<Option<String>, AppError> {
        self.input.release_modifiers()?;
        self.input.send_copy()?;
        match self.poll_for_item(self.policy.intercept_poll_interval, self.policy.intercept_poll_limit).await {
            Ok(text) => Ok(Some(text)),
            Err(AppError::NothingCopied) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Espera o clipboard deixar de ser `advanced` (o jogo processou o Ctrl+C devolvido).
    async fn wait_clipboard_change(&self, advanced: &str) {
        let mut elapsed = Duration::ZERO;
        while elapsed <= self.policy.resend_settle_limit {
            match self.clipboard.read_text() {
                Ok(Some(text)) if text == advanced => {}
                Ok(_) | Err(_) => return,
            }
            tokio::time::sleep(self.policy.intercept_poll_interval).await;
            elapsed += self.policy.intercept_poll_interval;
        }
    }

    async fn poll_for_item(&self, interval: Duration, limit: Duration) -> Result<String, AppError> {
        let mut elapsed = Duration::ZERO;
        while elapsed <= limit {
            if let Some(text) = self.clipboard.read_text()? {
                if (self.is_item_text)(&text) {
                    return Ok(text);
                }
            }
            tokio::time::sleep(interval).await;
            elapsed += interval;
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
            self.0.write_text("Item Class: Rings\nRarity: Rare\n{ Prefix Modifier }")
        }
        /// O Ctrl+C simples do jogo: texto sem os cabeçalhos avançados.
        fn send_plain_copy(&self) -> PortResult<()> {
            self.0.write_text("Item Class: Rings\nRarity: Rare")
        }
    }

    /// Jogo que recusa o input (ex.: UIPI): a cópia avançada falha; conta os Ctrl+C reenviados.
    #[derive(Default)]
    struct BlockedGame(std::sync::atomic::AtomicU32);
    impl InputPort for BlockedGame {
        fn release_modifiers(&self) -> PortResult<()> {
            Ok(())
        }
        fn send_copy(&self) -> PortResult<()> {
            Err(mirrorsight_domain::ports::PortError::Other("bloqueado".into()))
        }
        fn send_plain_copy(&self) -> PortResult<()> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    /// Cursor fora de item: a cópia avançada não traz nada; conta os Ctrl+C reenviados.
    #[derive(Default)]
    struct DeadGame(std::sync::atomic::AtomicU32);
    impl InputPort for DeadGame {
        fn release_modifiers(&self) -> PortResult<()> {
            Ok(())
        }
        fn send_copy(&self) -> PortResult<()> {
            Ok(())
        }
        fn send_plain_copy(&self) -> PortResult<()> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
            Arc::new(DeadGame::default()),
            clip,
            is_item,
            ClipboardPolicy::default(),
        );
        assert!(matches!(uc.execute(false).await, Err(AppError::NothingCopied)));
    }

    #[tokio::test(start_paused = true)]
    async fn intercepted_copy_returns_advanced_text_and_leaves_plain_copy_in_clipboard() {
        let clip = Arc::new(FakeClipboard::default());
        clip.write_text("texto do usuário").unwrap();
        let uc = CopyItemFromGame::new(Arc::new(FakeGame(clip.clone())), clip.clone(), is_item, ClipboardPolicy::default());

        let text = uc.execute_intercepted().await.unwrap();

        // O price check recebe os tiers; o clipboard fica com o Ctrl+C normal do jogo.
        assert!(text.is_some_and(|t| t.contains("{ Prefix Modifier }")));
        assert_eq!(clip.read_text().unwrap().as_deref(), Some("Item Class: Rings\nRarity: Rare"));
    }

    #[tokio::test(start_paused = true)]
    async fn intercepted_copy_resends_plain_copy_even_on_error() {
        let clip = Arc::new(FakeClipboard::default());
        let game = Arc::new(BlockedGame::default());
        let uc = CopyItemFromGame::new(game.clone(), clip, is_item, ClipboardPolicy::default());

        assert!(uc.execute_intercepted().await.is_err());
        assert_eq!(game.0.load(std::sync::atomic::Ordering::SeqCst), 1);
        uc.resend_plain_copy().unwrap();
        assert_eq!(game.0.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn failed_advanced_copy_gives_back_the_previous_item_text() {
        let clip = Arc::new(FakeClipboard::default());
        clip.write_text("Item Class: Belts\nRarity: Rare").unwrap();
        let uc = CopyItemFromGame::new(Arc::new(BlockedGame::default()), clip.clone(), is_item, ClipboardPolicy::default());

        assert!(uc.execute_intercepted().await.is_err());
        assert_eq!(clip.read_text().unwrap().as_deref(), Some("Item Class: Belts\nRarity: Rare"));
    }

    /// Copia o item no Ctrl+Alt+C, mas ignora o Ctrl+C devolvido.
    struct DeafGame(Arc<FakeClipboard>);
    impl InputPort for DeafGame {
        fn release_modifiers(&self) -> PortResult<()> {
            Ok(())
        }
        fn send_copy(&self) -> PortResult<()> {
            self.0.write_text("Item Class: Rings\nRarity: Rare")
        }
        fn send_plain_copy(&self) -> PortResult<()> {
            Ok(())
        }
    }

    #[tokio::test(start_paused = true)]
    async fn waits_for_the_resent_copy_only_up_to_the_limit() {
        let clip = Arc::new(FakeClipboard::default());
        let policy = ClipboardPolicy::default();
        let uc = CopyItemFromGame::new(Arc::new(DeafGame(clip.clone())), clip, is_item, policy);

        let start = tokio::time::Instant::now();
        assert!(uc.execute_intercepted().await.unwrap().is_some());
        let waited = start.elapsed();
        assert!(waited >= policy.resend_settle_limit, "{waited:?}");
        assert!(waited <= policy.resend_settle_limit + policy.intercept_poll_interval * 2, "{waited:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn chat_copy_is_given_back_quickly() {
        let clip = Arc::new(FakeClipboard::default());
        let policy = ClipboardPolicy::default();
        let uc = CopyItemFromGame::new(Arc::new(DeadGame::default()), clip, is_item, policy);

        let start = tokio::time::Instant::now();
        assert!(uc.execute_intercepted().await.unwrap().is_none());
        assert!(start.elapsed() < Duration::from_millis(250), "{:?}", start.elapsed());
    }

    #[tokio::test(start_paused = true)]
    async fn intercepted_copy_without_item_resends_plain_copy() {
        let clip = Arc::new(FakeClipboard::default());
        let game = Arc::new(DeadGame::default());
        let uc = CopyItemFromGame::new(game.clone(), clip, is_item, ClipboardPolicy::default());

        assert!(uc.execute_intercepted().await.unwrap().is_none());
        assert_eq!(game.0.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
