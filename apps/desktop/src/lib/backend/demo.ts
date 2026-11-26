import type {
  AppError,
  BuildDetails,
  BuildFile,
  CatalogStat,
  ErrorReason,
  GameVersion,
  HistoryEntry,
  HotkeyBinding,
  ModPreset,
  PriceCheckOutput,
  SearchFilters,
  Settings,
} from '$lib/types';
import type { Backend, PeekSide, PriceCheckRequest, TrayLabels, WindowRole } from './port';
import {
  DEMO_FEW,
  DEMO_GLOVES,
  DEMO_NETWORK,
  DEMO_RATE_LIMIT,
  DEMO_BUILDS,
  DEMO_STATS,
  demoBuildDetails,
  currencyOutput,
  defaultFilters,
  rareOutput,
} from './demo-fixtures';

const STORAGE_KEY = 'mirrorsight.demo.settings';
const HISTORY_KEY = 'mirrorsight.demo.history';
const PRESETS_KEY = 'mirrorsight.demo.presets';
/** Espelho de `HISTORY_LIMIT` (crates/domain/src/history.rs). */
const HISTORY_LIMIT = 50;
/** Espelho de `PRESET_NAME_MAX` (crates/domain/src/presets.rs). */
const PRESET_NAME_MAX = 40;

const DEFAULT_SETTINGS: Settings = {
  schema_version: 3,
  game: 'poe1',
  league: 'Standard',
  language: 'en',
  ui_language: null,
  restore_clipboard: true,
  intercept_copy: true,
  launch_on_startup: false,
  client_log_path: null,
  hotkeys: [
    { action: 'price-check', hotkey: 'Ctrl + D' },
    { action: 'price-check-locked', hotkey: 'Ctrl + Alt + D' },
    { action: 'toggle-overlay', hotkey: 'Shift + Space' },
    { action: 'open-settings', hotkey: 'Ctrl + Shift + O' },
  ],
  show_offline: false,
  mod_range_percent: 10,
  default_filter_kinds: ['explicit', 'fractured'],
  display_currency: 'chaos',
  density: 'normal',
  ui_scale: 1,
  reduce_motion: false,
  suggested_level: 'fair',
  build_path: null,
  build_item_set: null,
  build_skill_set: null,
};

/** Espelho simplificado de `RESERVED` em `crates/domain/src/hotkey.rs` (só para o modo demo). */
const RESERVED = ['ctrl + c', 'ctrl + v', 'ctrl + a', 'ctrl + f', 'ctrl + enter', 'home', 'delete', 'enter', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'];

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));
const fail = (err: AppError): never => {
  throw err;
};

/** Mesmo formato do Rust: a UI traduz pelo `reason`. */
const invalid = (reason: ErrorReason): AppError => ({ code: 'invalid-settings', data: { message: reason.key, reason } });

function readJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

type PresetBook = Record<GameVersion, ModPreset[]>;

/**
 * Adapter usado fora do Tauri (navegador / `npm run dev`). Simula latência,
 * rate limit e erros para exercitar todos os estados da UI. Histórico e presets
 * ficam no `localStorage` (no app real, `history.json` / `presets.json`).
 */
export class DemoBackend implements Backend {
  readonly kind = 'demo' as const;
  private rateLimitArmed = true;

  async getSettings(): Promise<Settings> {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return structuredClone(DEFAULT_SETTINGS);
    try {
      // Arquivos v1 não têm `suggested_level`: o padrão cobre a migração.
      return { ...DEFAULT_SETTINGS, ...(JSON.parse(raw) as Partial<Settings>), schema_version: 2 };
    } catch {
      return structuredClone(DEFAULT_SETTINGS);
    }
  }

  async saveSettings(settings: Settings): Promise<void> {
    await this.validateHotkeys(settings.hotkeys);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }

  async validateHotkeys(bindings: HotkeyBinding[]): Promise<void> {
    const seen = new Set<string>();
    for (const { hotkey } of bindings) {
      const key = hotkey.toLowerCase();
      if (RESERVED.includes(key)) fail(invalid({ key: 'reserved-hotkey', args: { hotkey } }));
      if (seen.has(key)) fail(invalid({ key: 'duplicate-hotkey', args: { hotkey } }));
      seen.add(key);
    }
  }

  async listLeagues(game: GameVersion): Promise<string[]> {
    await delay(200);
    return game === 'poe2'
      ? ['Runes of Aldur', 'HC Runes of Aldur', 'Standard', 'Hardcore']
      : ['Covens', 'Hardcore Covens', 'Standard', 'Hardcore'];
  }

  async priceCheckText(text: string, filters?: SearchFilters): Promise<PriceCheckOutput> {
    await delay(650);
    if (text.includes(DEMO_NETWORK)) fail({ code: 'network', data: { message: 'connection reset' } });
    if (text.includes(DEMO_RATE_LIMIT) && this.rateLimitArmed) {
      this.rateLimitArmed = false;
      fail({ code: 'rate-limited', data: { retry_after_ms: 6000 } });
    }
    if (!/^Rarity:/m.test(text)) fail({ code: 'item-not-recognized', data: { detail: 'texto não é um item' } });

    if (/^Rarity: Currency/m.test(text)) return currencyOutput();

    const settings = await this.getSettings();
    const found = DEMO_BUILDS.find((b) => b.file.path === settings.build_path);
    const build = found ? { ...found, details: demoBuildDetails(found, settings.build_item_set, settings.build_skill_set) } : null;
    return rareOutput(
      filters ??
        defaultFilters(DEMO_GLOVES, settings.mod_range_percent, settings.default_filter_kinds, settings.show_offline),
      text.includes(DEMO_FEW),
      build,
    );
  }

  async openExternal(url: string): Promise<void> {
    window.open(url, '_blank', 'noopener');
  }

  async setTrayLabels(_labels: TrayLabels): Promise<void> {
    // Sem bandeja no navegador.
  }

  async setUiScale(_scale: number): Promise<void> {
    // No navegador o usuário controla o zoom (Ctrl + / Ctrl -).
  }

  async listStats(_game: GameVersion): Promise<CatalogStat[]> {
    await delay(120);
    return structuredClone(DEMO_STATS);
  }

  async getHistory(): Promise<HistoryEntry[]> {
    return readJson<HistoryEntry[]>(HISTORY_KEY, []);
  }

  async recordHistory(entry: HistoryEntry): Promise<HistoryEntry[]> {
    const entries = [entry, ...(await this.getHistory()).filter((e) => e.id !== entry.id)].slice(0, HISTORY_LIMIT);
    localStorage.setItem(HISTORY_KEY, JSON.stringify(entries));
    return entries;
  }

  async removeHistory(id: string): Promise<HistoryEntry[]> {
    const entries = (await this.getHistory()).filter((e) => e.id !== id);
    localStorage.setItem(HISTORY_KEY, JSON.stringify(entries));
    return entries;
  }

  async clearHistory(): Promise<void> {
    localStorage.setItem(HISTORY_KEY, '[]');
  }

  private presetBook(): PresetBook {
    return { poe1: [], poe2: [], ...readJson<Partial<PresetBook>>(PRESETS_KEY, {}) };
  }

  async listPresets(game: GameVersion): Promise<ModPreset[]> {
    return this.presetBook()[game];
  }

  /** Mesmas regras de `ModPreset::validated` no domínio Rust. */
  async upsertPreset(preset: ModPreset): Promise<ModPreset[]> {
    const name = preset.name.trim();
    if (!name) fail(invalid({ key: 'preset-name-empty' }));
    if ([...name].length > PRESET_NAME_MAX) {
      fail(invalid({ key: 'preset-name-too-long', args: { max: PRESET_NAME_MAX } }));
    }
    if (preset.mods.length === 0) fail(invalid({ key: 'preset-no-mods' }));

    const book = this.presetBook();
    const clean = { ...preset, name };
    const other: GameVersion = preset.game === 'poe1' ? 'poe2' : 'poe1';
    book[other] = book[other].filter((p) => p.id !== preset.id);
    const list = book[preset.game];
    const index = list.findIndex((p) => p.id === preset.id);
    if (index >= 0) list[index] = clean;
    else list.push(clean);
    localStorage.setItem(PRESETS_KEY, JSON.stringify(book));
    return list;
  }

  async listPobBuilds(): Promise<BuildFile[]> {
    return DEMO_BUILDS.map((b) => b.file);
  }

  async pobBuildDetails(path: string, itemSet: string | null, skillSet: string | null): Promise<BuildDetails> {
    const found = DEMO_BUILDS.find((b) => b.file.path === path);
    if (!found) return fail({ code: 'internal', data: { message: `build não encontrado: ${path}` } });
    return demoBuildDetails(found, itemSet, skillSet);
  }

  async deletePreset(id: string): Promise<void> {
    const book = this.presetBook();
    book.poe1 = book.poe1.filter((p) => p.id !== id);
    book.poe2 = book.poe2.filter((p) => p.id !== id);
    localStorage.setItem(PRESETS_KEY, JSON.stringify(book));
  }

  /** `?overlay` na URL simula a janela overlay no navegador. */
  windowRole(): WindowRole {
    return new URLSearchParams(location.search).has('overlay') ? 'overlay' : 'main';
  }

  async hideOverlay(): Promise<void> {}

  async startWindowDrag(): Promise<void> {}

  async setOverlayPeek(_open: boolean, _extra: number): Promise<PeekSide> {
    return 'inside';
  }

  async onPriceCheckRequest(_handler: (request: PriceCheckRequest) => void): Promise<() => void> {
    return () => {};
  }

  async onPriceCheckError(_handler: (error: AppError) => void): Promise<() => void> {
    return () => {};
  }

  async onOverlayReopen(_handler: () => void): Promise<() => void> {
    return () => {};
  }

  async onOpenSettings(_handler: () => void): Promise<() => void> {
    return () => {};
  }

  /** No navegador quem controla a janela é o próprio navegador. */
  readonly hasWindowControls = false;
  async minimizeWindow(): Promise<void> {}
  async toggleMaximizeWindow(): Promise<void> {}
  async closeWindow(): Promise<void> {}
  async isWindowMaximized(): Promise<boolean> {
    return false;
  }
  async onWindowResized(_handler: () => void): Promise<() => void> {
    return () => {};
  }
}
