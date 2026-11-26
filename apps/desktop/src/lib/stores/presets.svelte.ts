import { backend, describeError, toAppError } from '$lib/backend';
import type { GameVersion, ModPreset, StatFilter } from '$lib/types';
import { toasts } from './toasts.svelte';

function newId(): string {
  return typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

/** Presets de mods do jogo atual (salvos em `presets.json`, separados por PoE1/PoE2). */
class PresetsStore {
  list = $state<ModPreset[]>([]);
  game = $state<GameVersion>('poe1');

  async load(game: GameVersion): Promise<void> {
    this.game = game;
    try {
      const list = await backend.listPresets(game);
      if (this.game === game) this.list = list;
    } catch {
      this.list = [];
    }
  }

  /** Salva os mods ligados com nome. `false` se o backend recusar (nome vazio etc.). */
  async save(name: string, stats: StatFilter[]): Promise<boolean> {
    const preset: ModPreset = {
      id: newId(),
      name,
      game: this.game,
      created_ms: Date.now(),
      mods: stats
        .filter((s) => s.enabled)
        .map((s) => ({ trade_id: s.trade_id, text: s.text, kind: s.kind, min: s.min, max: s.max })),
    };
    return this.upsert(preset, `Preset “${name.trim()}” salvo`);
  }

  async rename(preset: ModPreset, name: string): Promise<boolean> {
    return this.upsert({ ...$state.snapshot(preset), name }, 'Preset renomeado');
  }

  async remove(preset: ModPreset): Promise<void> {
    try {
      await backend.deletePreset(preset.id);
      this.list = this.list.filter((p) => p.id !== preset.id);
      toasts.push('info', `Preset “${preset.name}” excluído`);
    } catch (err) {
      toasts.push('error', describeError(toAppError(err)));
    }
  }

  private async upsert(preset: ModPreset, message: string): Promise<boolean> {
    try {
      const list = await backend.upsertPreset(preset);
      if (preset.game === this.game) this.list = list;
      toasts.push('success', message);
      return true;
    } catch (err) {
      toasts.push('error', describeError(toAppError(err)));
      return false;
    }
  }
}

export const presets = new PresetsStore();
