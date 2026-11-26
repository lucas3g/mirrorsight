import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';

import type {
  AppError,
  CatalogStat,
  GameVersion,
  HistoryEntry,
  HotkeyBinding,
  ModPreset,
  PriceCheckOutput,
  SearchFilters,
  Settings,
} from '$lib/types';
import type { Backend, PeekSide, PriceCheckRequest, WindowRole } from './port';

/** Nomes dos eventos emitidos por `src-tauri/src/overlay.rs`. */
const EVENT_REQUEST = 'overlay:price-check';
const EVENT_ERROR = 'overlay:error';
const EVENT_REOPEN = 'overlay:reopen';
/** Emitido por `src-tauri/src/tray.rs`. */
const EVENT_OPEN_SETTINGS = 'main:open-settings';

/** Adapter real: comandos definidos em `src-tauri/src/commands.rs`. */
export class TauriBackend implements Backend {
  readonly kind = 'tauri' as const;

  getSettings(): Promise<Settings> {
    return invoke<Settings>('get_settings');
  }

  saveSettings(settings: Settings): Promise<void> {
    return invoke('save_settings', { settings });
  }

  validateHotkeys(bindings: HotkeyBinding[]): Promise<void> {
    return invoke('validate_hotkeys', { bindings });
  }

  listLeagues(game: GameVersion): Promise<string[]> {
    return invoke<string[]>('list_leagues', { game });
  }

  priceCheckText(text: string, filters?: SearchFilters): Promise<PriceCheckOutput> {
    return invoke<PriceCheckOutput>('price_check_text', { text, filters: filters ?? null });
  }

  openExternal(url: string): Promise<void> {
    return openUrl(url);
  }

  setUiScale(scale: number): Promise<void> {
    return getCurrentWebview().setZoom(scale);
  }

  listStats(game: GameVersion): Promise<CatalogStat[]> {
    return invoke<CatalogStat[]>('list_stats', { game });
  }

  getHistory(): Promise<HistoryEntry[]> {
    return invoke<HistoryEntry[]>('get_history');
  }

  recordHistory(entry: HistoryEntry): Promise<HistoryEntry[]> {
    return invoke<HistoryEntry[]>('record_history', { entry });
  }

  removeHistory(id: string): Promise<HistoryEntry[]> {
    return invoke<HistoryEntry[]>('remove_history', { id });
  }

  clearHistory(): Promise<void> {
    return invoke('clear_history');
  }

  listPresets(game: GameVersion): Promise<ModPreset[]> {
    return invoke<ModPreset[]>('list_presets', { game });
  }

  upsertPreset(preset: ModPreset): Promise<ModPreset[]> {
    return invoke<ModPreset[]>('upsert_preset', { preset });
  }

  deletePreset(id: string): Promise<void> {
    return invoke('delete_preset', { id });
  }

  windowRole(): WindowRole {
    return getCurrentWindow().label === 'overlay' ? 'overlay' : 'main';
  }

  hideOverlay(): Promise<void> {
    return invoke('hide_overlay');
  }

  startWindowDrag(): Promise<void> {
    return getCurrentWindow().startDragging();
  }

  setOverlayPeek(open: boolean, extra: number): Promise<PeekSide> {
    return invoke<PeekSide>('overlay_peek', { open, extra });
  }

  onPriceCheckRequest(handler: (request: PriceCheckRequest) => void): Promise<() => void> {
    return listen<PriceCheckRequest>(EVENT_REQUEST, (e) => handler(e.payload));
  }

  onPriceCheckError(handler: (error: AppError) => void): Promise<() => void> {
    return listen<AppError>(EVENT_ERROR, (e) => handler(e.payload));
  }

  onOverlayReopen(handler: () => void): Promise<() => void> {
    return listen(EVENT_REOPEN, () => handler());
  }

  onOpenSettings(handler: () => void): Promise<() => void> {
    return listen(EVENT_OPEN_SETTINGS, () => handler());
  }

  readonly hasWindowControls = true;

  minimizeWindow(): Promise<void> {
    return getCurrentWindow().minimize();
  }

  toggleMaximizeWindow(): Promise<void> {
    return getCurrentWindow().toggleMaximize();
  }

  closeWindow(): Promise<void> {
    return getCurrentWindow().close();
  }

  isWindowMaximized(): Promise<boolean> {
    return getCurrentWindow().isMaximized();
  }

  onWindowResized(handler: () => void): Promise<() => void> {
    return getCurrentWindow().onResized(() => handler());
  }
}
