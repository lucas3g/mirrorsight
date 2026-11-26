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
  /** Rank de bancada (mods crafted). */
  rank?: number | null;
  /** Nome do afixo ("Fecund"). */
  mod_name?: string | null;
  origin?: ModOrigin | null;
}

export type EldritchLevel = 'lesser' | 'greater' | 'grand' | 'exceptional' | 'exquisite' | 'perfect';

/** Origem do mod além do tipo (crates/domain/src/item.rs). */
export type ModOrigin =
  | { kind: 'eldritch'; searing: boolean; level: EldritchLevel }
  | { kind: 'corruption' }
  | { kind: 'veiled' }
  | { kind: 'influence'; influence: Influence }
  | { kind: 'unique' };

export interface ItemFlags {
  fractured: boolean;
  synthesised: boolean;
  mirrored: boolean;
  split: boolean;
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
  flags?: ItemFlags;
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
  /** Propriedades a 20% de qualidade, como a trade compara ("ar", "es", "pdps"…). */
  max_quality?: Record<string, number>;
  /** As propriedades do item mostram o valor a 20% de qualidade (item ainda aceita qualidade). */
  at_max_quality?: boolean;
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
  /** Corrompidos/espelhados tirados da lista por ficarem abaixo do mínimo de um filtro de propriedade. */
  hidden_below_min?: number;
  /** Listagens agrupadas por status comparado ao seu item (PoE1), ou o motivo da lista única. */
  comparison?: ListingComparison | null;
}

// ---------- Comparação por status (crates/domain/src/compare.rs) ----------

export type GroupKind = 'worse' | 'equal' | 'better' | 'not_comparable' | 'no_item';
// ---------- Avaliação de itens (crates/domain/src/evaluation) ----------

export type Grade = 'excellent' | 'good' | 'average' | 'weak';
export type Confidence = 'low' | 'medium' | 'high';
export type Verdict = 'better' | 'worse' | 'equivalent';

export type InconclusiveReason =
  | 'unique_not_in_database'
  | 'unique_without_data'
  | 'mod_not_in_database'
  | 'out_of_range'
  | 'no_range'
  | 'foulborn';

/** Como a qualidade do mod foi (ou não) determinada. */
export type ModStatus =
  | { kind: 'exact' }
  | { kind: 'from_database' }
  | { kind: 'fixed' }
  | { kind: 'variant' }
  /** Mutação Foulborn de valor fixo (troca um mod do único). */
  | { kind: 'mutated' }
  | { kind: 'inconclusive'; reason: InconclusiveReason };

/** Avaliação de um mod (independe da build). */
export interface ModEvaluation {
  trade_id: string;
  text: string;
  kind: ModifierKind;
  origin: ModOrigin | null;
  value: number | null;
  /** Faixa usada (tier, único ou bancada). */
  min: number | null;
  max: number | null;
  tier: number | null;
  tier_count: number | null;
  /** Posição do roll na faixa, 0–1. */
  roll: number | null;
  global_roll: number | null;
  /** Nota do mod 0–1; null = fora da nota (fixo, variante ou inconclusivo). */
  quality: number | null;
  importance: number;
  status: ModStatus;
  variant: string | null;
  /** A faixa do jogo difere da base de conhecimento. */
  range_mismatch: boolean;
  tags: string[];
}

export type CraftBlock = 'corrupted' | 'mirrored' | 'split';

export interface CraftingPotential {
  open_prefixes: number | null;
  open_suffixes: number | null;
  valued: Affix | null;
  crafted: number;
  fractured_high_tier: number;
  ilvl_allows_top_tier: boolean | null;
  blocked: CraftBlock[];
  /** 0–1. */
  score: number;
}

export interface SocketInfo {
  sockets: number;
  links: number;
  max: number;
  weight: number;
}

export interface UniqueMatch {
  name: string;
  /** Está na base de conhecimento (com mods). */
  found: boolean;
  version: string | null;
  variant: string | null;
  legacy: boolean;
  variable_mods: number;
  fixed_mods: number;
}

/** Qualidade do item — comercial, sem depender da build. */
export interface ItemQuality {
  /** 0–100; null = inconclusiva. */
  score: number | null;
  grade: Grade | null;
  confidence: Confidence;
  mods_score: number | null;
  mods: ModEvaluation[];
  crafting: CraftingPotential | null;
  unique: UniqueMatch | null;
  sockets: SocketInfo | null;
}

export interface StatDelta {
  key: string;
  text: string;
  yours: number;
  theirs: number;
  weight: number;
  /** Positivo = ganho. */
  contribution: number;
}

/** Efeito da troca numa resistência ("fire"…) ou atributo ("str"…). */
export interface NeedChange {
  stat: string;
  short_before: number;
  short_after: number;
  contribution: number;
}

/** Upgrade em relação ao seu item, para a build. */
export interface UpgradeVerdict {
  verdict: Verdict;
  delta: number;
  /** `delta` relativo ao valor do seu item. */
  relative: number;
  /** Valor do seu item na mesma conta: contribuição ÷ reference = % do seu item. */
  reference: number;
  gains: StatDelta[];
  losses: StatDelta[];
  synergy: number;
  needs: NeedChange[];
  relevant_loss: boolean;
  confidence: Confidence;
}

/** Build usado na relevância (genérico quando nenhum build do PoB está escolhido). */
export interface BuildSummary {
  name: string | null;
  generic: boolean;
  class: string | null;
  main_skill: string | null;
  focus: string[];
  /** Títulos do item set e do skill set usados. */
  item_set: string | null;
  skill_set: string | null;
  /** Resistências/atributos estimados (item set diferente do ativo no PoB). */
  needs_estimated: boolean;
}

/** Item set ou skill set de um build do PoB. */
export interface SetOption {
  id: string;
  title: string;
}

/** Resumo do build com a escolha de sets, e os sets disponíveis. */
export interface BuildDetails {
  summary: BuildSummary;
  item_sets: SetOption[];
  skill_sets: SetOption[];
  /** Sets ativos quando o build foi salvo no PoB. */
  active_item_set: string | null;
  active_skill_set: string | null;
}

/** Build salvo do Path of Building. */
export interface BuildFile {
  name: string;
  path: string;
  folder: string | null;
}

/** Avaliação de um anúncio comparável (card do olho). */
export interface CompareDetail {
  upgrade: UpgradeVerdict;
  quality: ItemQuality;
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

/** Seu item (faixa "Seu item"). */
export interface ItemStatus {
  quality: ItemQuality;
  build: BuildSummary;
  /** Origem da base de conhecimento ("PoB v2.69.0 · 2026-11-26"). */
  data_source: string;
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
  /** Build do Path of Building usado na relevância do upgrade; `null` = pesos genéricos. */
  build_path: string | null;
  /** Item set e skill set do build (ids do PoB); `null` = o ativo no PoB. */
  build_item_set: string | null;
  build_skill_set: string | null;
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
