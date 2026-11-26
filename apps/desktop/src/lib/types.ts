/**
 * Espelho dos DTOs serializados pelo backend Rust (serde, snake_case).
 * Mudou um struct em `crates/domain` ou `crates/application`? Atualize aqui.
 */

export type GameVersion = 'poe1' | 'poe2';

export type Rarity =
  | 'normal'
  | 'magic'
  | 'rare'
  | 'unique'
  | 'gem'
  | 'currency'
  | 'divinationcard'
  | 'other';

export type ModifierKind =
  | 'implicit'
  | 'explicit'
  | 'crafted'
  | 'fractured'
  | 'enchant'
  | 'rune'
  | 'desecrated'
  | 'pseudo'
  /** Defesas e dano da base (Armour, ES, DPS…): filtros próprios da busca, não stats. */
  | 'property';

export type Influence = 'shaper' | 'elder' | 'crusader' | 'hunter' | 'redeemer' | 'warlord' | 'searing' | 'eater';

export interface StatRoll {
  value: number;
  min: number | null;
  max: number | null;
}

export interface Modifier {
  trade_id: string;
  text: string;
  kind: ModifierKind;
  roll: StatRoll | null;
  /** Disponível quando o item foi copiado com Ctrl+Alt+C. */
  tier: number | null;
}

export interface Item {
  item_class: string;
  rarity: Rarity;
  name: string | null;
  base_type: string;
  item_level: number | null;
  quality: number | null;
  corrupted: boolean;
  identified: boolean;
  /** Formato do jogo: "B-B-R G". */
  sockets: string | null;
  influences: Influence[];
  modifiers: Modifier[];
  /** Defesas e dano da base, como filtros (tipo 'property'). */
  properties?: Modifier[];
  unknown_lines: string[];
}

export interface Price {
  amount: number;
  /** Tag da trade API: "chaos", "divine", "exalted"… */
  currency: string;
}

export interface Listing {
  price: Price;
  stack_size: number | null;
  account: string;
  online: boolean;
  indexed_at: string;
  item: Item | null;
  whisper: string | null;
  /** Imagem do item listado (poecdn); em raros/mágicos, a arte da base. */
  icon?: string | null;
}

export interface PriceCheckResult {
  query_id: string;
  total: number;
  listings: Listing[];
  web_url: string | null;
}

export interface IndexedPrice {
  name: string;
  chaos_value: number;
  divine_value: number | null;
  change_24h_percent: number | null;
  sparkline: number[];
  /** Imagem do item (poecdn), quando o poe.ninja a fornece. */
  icon: string | null;
}

export interface PriceSummary {
  estimate_chaos: number;
  min_chaos: number;
  median_chaos: number;
  max_chaos: number;
  sample_size: number;
  histogram: number[];
  /** 0 (nenhuma) a 3 (alta). */
  confidence: number;
  divine_chaos: number | null;
  /** Percentis das listagens; `null` com menos de 5 listagens. */
  suggested: SuggestedPrice | null;
}

/** Preço sugerido: Compra = p25, Justo = mediana, Venda = p75 (em chaos). */
export interface SuggestedPrice {
  buy_chaos: number;
  fair_chaos: number;
  sell_chaos: number;
}

export type SuggestedLevel = 'buy' | 'fair' | 'sell';

export interface StatFilter {
  trade_id: string;
  text: string;
  kind: ModifierKind;
  roll: StatRoll | null;
  tier: number | null;
  enabled: boolean;
  min: number | null;
  max: number | null;
}

export interface SearchFilters {
  online_only: boolean;
  stats: StatFilter[];
  item_level_min: number | null;
  quality_min: number | null;
  links_min: number | null;
}

export interface PriceCheckOutput {
  item: Item;
  filters: SearchFilters;
  indexed: IndexedPrice | null;
  trade: PriceCheckResult | null;
  /** Imagem do item: poe.ninja, listagem da mesma base ou arte genérica da base. */
  icon?: string | null;
  summary: PriceSummary | null;
  /** Motivo da ausência de listagens quando a trade falhou e o poe.ninja respondeu. */
  trade_error: string | null;
}

export type HotkeyAction = 'price-check' | 'price-check-locked' | 'toggle-overlay' | 'open-settings';

export interface HotkeyBinding {
  action: HotkeyAction;
  /** Formato normalizado do domínio: "Ctrl + Shift + D". */
  hotkey: string;
}

export type DisplayCurrency = 'chaos' | 'divine';
export type Density = 'compact' | 'normal';

export interface Settings {
  schema_version: number;
  game: GameVersion;
  league: string;
  language: string;
  restore_clipboard: boolean;
  launch_on_startup: boolean;
  client_log_path: string | null;
  hotkeys: HotkeyBinding[];
  show_offline: boolean;
  mod_range_percent: number;
  default_filter_kinds: ModifierKind[];
  display_currency: DisplayCurrency;
  density: Density;
  ui_scale: number;
  reduce_motion: boolean;
  /** Nível do preço sugerido que vem selecionado (schema v2). */
  suggested_level: SuggestedLevel;
}

/** Stat do catálogo da trade API (busca "Adicionar mod"). */
export interface CatalogStat {
  trade_id: string;
  /** Template com # no lugar dos números. */
  text: string;
  kind: ModifierKind;
}

export interface HistoryItem {
  name: string | null;
  base_type: string;
  rarity: Rarity;
  /** Imagem do item (poe.ninja); ausente em entradas antigas. */
  icon?: string | null;
}

export type HistorySource = 'trade' | 'ninja';

export interface HistoryEntry {
  id: string;
  game: GameVersion;
  league: string;
  item_text: string;
  item: HistoryItem;
  filters: SearchFilters | null;
  buy_chaos: number | null;
  fair_chaos: number | null;
  sell_chaos: number | null;
  sample: number;
  source: HistorySource;
  divine_chaos: number | null;
  /** Epoch em ms. */
  at_ms: number;
}

export interface PresetMod {
  trade_id: string;
  text: string;
  kind: ModifierKind;
  min: number | null;
  max: number | null;
}

export interface ModPreset {
  id: string;
  name: string;
  game: GameVersion;
  created_ms: number;
  mods: PresetMod[];
}

/** `AppError` do Rust: `#[serde(tag = "code", content = "data")]`. */
export type AppError =
  | { code: 'nothing-copied' }
  | { code: 'item-not-recognized'; data: { detail: string } }
  | { code: 'rate-limited'; data: { retry_after_ms: number } }
  | { code: 'network'; data: { message: string } }
  | { code: 'invalid-settings'; data: { message: string } }
  | { code: 'internal'; data: { message: string } };
