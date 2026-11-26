import { backend, describeError, toAppError } from '$lib/backend';
import { historyMsgs } from '$lib/i18n/messages/history';
import type { HistoryEntry } from '$lib/types';
import { toasts } from './toasts.svelte';

/** Espelho de `HISTORY_LIMIT` (crates/domain/src/history.rs). */
export const HISTORY_LIMIT = 50;

/**
 * Histórico local dos price checks. Quem grava é o backend (lock no Rust): o overlay
 * registra por fora, então a janela principal recarrega no foco e ao abrir a tela.
 */
class HistoryStore {
  entries = $state<HistoryEntry[]>([]);

  async load(): Promise<void> {
    try {
      this.entries = await backend.getHistory();
    } catch {
      // Histórico é secundário: falhar aqui não pode atrapalhar o price check.
    }
  }

  async record(entry: HistoryEntry): Promise<void> {
    try {
      this.entries = await backend.recordHistory(entry);
    } catch (err) {
      console.warn('histórico não gravado', err);
    }
  }

  async remove(id: string): Promise<void> {
    try {
      this.entries = await backend.removeHistory(id);
    } catch (err) {
      toasts.push('error', describeError(toAppError(err)));
    }
  }

  async clear(): Promise<void> {
    try {
      await backend.clearHistory();
      this.entries = [];
      toasts.push('success', historyMsgs.cleared);
    } catch (err) {
      toasts.push('error', describeError(toAppError(err)));
    }
  }

  recent(n: number): HistoryEntry[] {
    return this.entries.slice(0, n);
  }
}

export const history = new HistoryStore();
