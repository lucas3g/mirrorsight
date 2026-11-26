use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use mirrorsight_domain::history::{self, HistoryEntry};
use mirrorsight_domain::ports::HistoryRepository;

use crate::AppError;

/// Histórico de price checks. Cada operação lê, altera e salva sob o mesmo lock:
/// o overlay e a janela principal gravam ao mesmo tempo sem apagar a mudança do outro.
pub struct HistoryService {
    repo: Arc<dyn HistoryRepository>,
    lock: Mutex<()>,
}

impl HistoryService {
    pub fn new(repo: Arc<dyn HistoryRepository>) -> Self {
        Self {
            repo,
            lock: Mutex::new(()),
        }
    }

    fn guard(&self) -> MutexGuard<'_, ()> {
        self.lock.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn list(&self) -> Result<Vec<HistoryEntry>, AppError> {
        let _guard = self.guard();
        Ok(self.repo.load()?)
    }

    /// Registra no topo e descarta os mais antigos além do limite.
    pub fn record(&self, entry: HistoryEntry) -> Result<Vec<HistoryEntry>, AppError> {
        let _guard = self.guard();
        let mut entries = self.repo.load()?;
        history::push(&mut entries, entry);
        self.repo.save(&entries)?;
        Ok(entries)
    }

    pub fn remove(&self, id: &str) -> Result<Vec<HistoryEntry>, AppError> {
        let _guard = self.guard();
        let mut entries = self.repo.load()?;
        entries.retain(|e| e.id != id);
        self.repo.save(&entries)?;
        Ok(entries)
    }

    pub fn clear(&self) -> Result<(), AppError> {
        let _guard = self.guard();
        Ok(self.repo.save(&[])?)
    }
}

#[cfg(test)]
mod tests {
    use mirrorsight_domain::history::{HistoryItem, HistorySource, HISTORY_LIMIT};
    use mirrorsight_domain::item::Rarity;
    use mirrorsight_domain::ports::PortResult;
    use mirrorsight_domain::GameVersion;

    use super::*;

    #[derive(Default)]
    struct MemoryRepo(Mutex<Vec<HistoryEntry>>);
    impl HistoryRepository for MemoryRepo {
        fn load(&self) -> PortResult<Vec<HistoryEntry>> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn save(&self, entries: &[HistoryEntry]) -> PortResult<()> {
            *self.0.lock().unwrap() = entries.to_vec();
            Ok(())
        }
    }

    fn entry(id: usize) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            game: GameVersion::Poe1,
            league: "Standard".into(),
            item_text: "Rarity: Rare".into(),
            item: HistoryItem {
                name: Some("Doom Grip".into()),
                base_type: "Sorcerer Gloves".into(),
                rarity: Rarity::Rare,
                icon: None,
            },
            filters: None,
            buy_chaos: Some(40.0),
            fair_chaos: Some(50.0),
            sell_chaos: Some(60.0),
            sample: 20,
            source: HistorySource::Trade,
            divine_chaos: Some(200.0),
            at_ms: 0,
        }
    }

    #[test]
    fn record_caps_at_limit_and_clear_empties() {
        let service = HistoryService::new(Arc::new(MemoryRepo::default()));
        for i in 0..(HISTORY_LIMIT + 3) {
            service.record(entry(i)).unwrap();
        }
        let list = service.list().unwrap();
        assert_eq!(list.len(), HISTORY_LIMIT);
        assert_eq!(list[0].id, (HISTORY_LIMIT + 2).to_string());

        assert_eq!(service.remove(&list[0].id).unwrap().len(), HISTORY_LIMIT - 1);
        service.clear().unwrap();
        assert!(service.list().unwrap().is_empty());
    }
}
