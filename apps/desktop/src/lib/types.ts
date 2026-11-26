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
  /** Prefixo/sufixo, quando a origem informa (cópia avançada ou mods da trade API). */
  affix?: Affix | null;
}

export type Affix = 'prefix' | 'suffix';

/** Slots de afixo ocupados no item. */
export interface AffixSlots {
  prefixes: number;
  suffixes: number;
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
  affix_slots?: AffixSlots | null;
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
  trade_error: AppError | null;
  /** Listagens agrupadas por status comparado ao seu item (PoE1), ou o motivo da lista única. */
  comparison?: ListingComparison | null;
}

// ---------- Comparação por status (crates/domain/src/compare.rs) ----------

export type GroupKind = 'worse' | 'equal' | 'better' | 'not_comparable' | 'no_item';
export type Criterion = 'tier' | 'percentile' | 'slots';

export interface CompareScore {
  avg_tier: number;
  /** 0–1. */
  avg_percentile: number;
  free_valued: number;
}

export interface CompareDetail {
  common: number;
  /** trade_ids do anúncio em comum com o seu item. */
  common_ids: string[];
  listing: CompareScore;
  yours: CompareScore;
  /** Critério que decidiu o grupo; null em "igual". */
  decided: Criterion | null;
}

export interface GroupEntry {
  /** Posição em `trade.listings`. */
  index: number;
  chaos: number | null;
  detail: CompareDetail | null;
}

export interface ListingGroup {
  kind: GroupKind;
  entries: GroupEntry[];
}

export interface ItemStatus {
  avg_tier: number;
  avg_percentile: number;
  free_valued: number;
  valued: Affix;
  mods: { trade_id: string; tier: number; percentile: number }[];
}

export type SingleReason =
  | { kind: 'poe2' }
  | { kind: 'rarity' }
  | { kind: 'class'; class: string }
  | { kind: 'no_status' };

export type ListingComparison =
  | { mode: 'single'; reason: SingleReason }
  | { mode: 'grouped'; you: ItemStatus; groups: ListingGroup[] };

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
  /** Idioma do cliente do jogo (como o PoE copia o item). */
  language: string;
  /** Idioma da interface (schema v3); `null` = automático (idioma do sistema). */
  ui_language: string | null;
  restore_clipboard: boolean;
  /** Ctrl+C sobre um item no jogo também abre o price check (com tiers e prefix/suffix). */
  intercept_copy: boolean;
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
  | { code: 'network'; data: { message: string; reason?: ErrorReason } }
  | { code: 'invalid-settings'; data: { message: string; reason?: ErrorReason } }
  | { code: 'internal'; data: { message: string; reason?: ErrorReason } };

/**
 * Motivo estruturado de um erro (`Reason` em crates/domain/src/error.rs,
 * `#[serde(tag = "key", content = "args")]`). A UI traduz pelo `key`; `message` vira fallback.
 */
export type ErrorReason =
  | { key: 'invalid-hotkey'; args: { hotkey: string } }
  | { key: 'reserved-hotkey'; args: { hotkey: string } }
  | { key: 'duplicate-hotkey'; args: { hotkey: string } }
  | { key: 'preset-name-empty' }
  | { key: 'preset-name-too-long'; args: { max: number } }
  | { key: 'preset-no-mods' }
  | { key: 'ui-scale-range' }
  | { key: 'mod-range-range' }
  | { key: 'unknown-ui-language'; args: { lang: string } }
  | { key: 'game-elevated' }
  | { key: 'input-blocked' }
  | { key: 'clipboard-unavailable' }
  | { key: 'clipboard-read' }
  | { key: 'clipboard-write' }
  | { key: 'game-window-not-found' }
  | { key: 'timeout' }
  | { key: 'unexpected-response' }
  | { key: 'trade-unavailable' };
