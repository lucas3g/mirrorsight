import { backend, describeError, toAppError } from '$lib/backend';
import type { Settings } from '$lib/types';
import { toasts } from './toasts.svelte';

/**
 * Configurações com salvamento otimista: aplica na hora e reverte se o backend
 * rejeitar (a validação de verdade mora no domínio Rust).
 */
class SettingsStore {
  current = $state<Settings | null>(null);
  loadError = $state<string | null>(null);

  async load(): Promise<void> {
    try {
      this.current = await backend.getSettings();
      this.loadError = null;
    } catch (err) {
      this.loadError = describeError(toAppError(err));
    }
  }

  async update(patch: Partial<Settings>): Promise<boolean> {
    if (!this.current) return false;
    const previous = $state.snapshot(this.current) as Settings;
    const next: Settings = { ...previous, ...patch };
    this.current = next;
    try {
      await backend.saveSettings(next);
      return true;
    } catch (err) {
      this.current = previous;
      toasts.push('error', describeError(toAppError(err)));
      return false;
    }
  }
}

export const settings = new SettingsStore();
