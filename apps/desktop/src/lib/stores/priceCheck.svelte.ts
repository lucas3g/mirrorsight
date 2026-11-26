
import { backend, toAppError } from '$lib/backend';
import type {
  AppError,
  CatalogStat,
  HistoryEntry,
  ModifierKind,
  ModPreset,
  PriceCheckOutput,
  SearchFilters,
  StatFilter,
  SuggestedLevel,
} from '$lib/types';
import { divineRateOf, itemIconOf } from '$lib/pricing';
import { history } from './history.svelte';
import { settings } from './settings.svelte';

export type ViewMode = 'full' | 'quick';

export type Phase =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'ready' }
  | { kind: 'rate-limited'; until: number; totalMs: number }
  | { kind: 'error'; error: AppError };

function newId(): string {
  return typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}


/** Espera (ms) após a última edição de filtro antes de buscar automaticamente. */
export const AUTO_SEARCH_MS = 600;

/** Arredonda para baixo na precisão do valor de origem: inteiros ficam inteiros, decimais (APS 1.45) mantêm 2 casas. */
function floorLike(value: number, origin: number): number {
  return Number.isInteger(origin) ? Math.floor(value) : Math.floor(value * 100) / 100;
}

/**
 * Máquina de estados do price check. Guarda o texto do item para refazer a busca
 * com filtros editados e descarta respostas atrasadas (requestId).
 *
 * Além dos filtros, guarda quantas linhas vêm do item (`itemRows`; as seguintes foram
 * adicionadas pela busca). As linhas aparecem na ordem do item, agrupadas por tipo.
 */
class PriceCheckStore {
  phase = $state<Phase>({ kind: 'idle' });
  output = $state<PriceCheckOutput | null>(null);
  filters = $state<SearchFilters | null>(null);
  /** Filtros mudaram desde a última busca. */
  dirty = $state(false);
  mode = $state<ViewMode>('full');
  pinned = $state(false);

  /**
   * Quantas linhas do início de `filters.stats` são do item; as demais vieram de "Adicionar mod".
   * Por posição, não por `trade_id`: um mod adicionado pode repetir um stat do item.
   */
  itemRows = $state(0);
  /** Margem (%) aplicada aos mínimos; começa em `mod_range_percent`. */
  margin = $state(10);
  /** Nível do preço sugerido selecionado para o item atual. */
  level = $state<SuggestedLevel>('fair');
  /** Preset aplicado ao item atual. */
  presetId = $state<string | null>(null);

  private text = '';
  /** Id da entrada de histórico do item atual: novas buscas atualizam a mesma entrada. */
  private historyId = '';
  private requestId = 0;
  private retryTimer: ReturnType<typeof setTimeout> | undefined;
  /** Busca automática agendada após editar filtros (debounce). */
  private autoTimer: ReturnType<typeof setTimeout> | undefined;
  /** Estado do painel no momento em que foi fechado (para `restoreLast`). */
  private last: {
    text: string;
    output: PriceCheckOutput | null;
    filters: SearchFilters | null;
    itemRows: number;
    presetId: string | null;
    dirty: boolean;
    mode: ViewMode;
  } | null = null;

  get itemText(): string {
    return this.text;
  }

  /**
   * Filtros mudaram: marca como alterados e agenda a busca para `AUTO_SEARCH_MS` depois
   * da última edição (cada edição reinicia o timer). Uma busca em andamento é descartada,
   * para a resposta antiga não sobrescrever o que acabou de ser editado.
   */
  private markDirty(): void {
    this.dirty = true;
    if (!this.text || !this.filters) return;
    let delay = AUTO_SEARCH_MS;
    if (this.phase.kind === 'loading') this.requestId++;
    if (this.phase.kind === 'rate-limited') {
      clearTimeout(this.retryTimer);
      delay = Math.max(delay, this.phase.until - Date.now());
    }
    clearTimeout(this.autoTimer);
    this.autoTimer = setTimeout(() => void this.search(), delay);
  }

  /** Novo item: descarta resultado anterior. `filters` ausente = filtros padrão do backend. */
  async check(text: string, mode: ViewMode = 'full', filters?: SearchFilters): Promise<void> {
    this.text = text;
    this.mode = mode;
    this.output = null;
    this.filters = null;
    this.itemRows = 0;
    this.presetId = null;
    this.dirty = false;
    this.historyId = newId();
    this.margin = settings.current?.mod_range_percent ?? 10;
    this.level = settings.current?.suggested_level ?? 'fair';
    await this.run(filters);
  }

  /** Refaz o price check de uma entrada do histórico, com os filtros salvos. */
  async reopen(entry: HistoryEntry): Promise<void> {
    await this.check(entry.item_text, 'full', entry.filters ? ($state.snapshot(entry.filters) as SearchFilters) : undefined);
  }

  /**
   * Reabre o último price check fechado, como estava (sem nova busca, salvo se os filtros
   * tinham edições pendentes ou o resultado não chegou a carregar). `false` = nada para reabrir.
   */
  restoreLast(): boolean {
    const last = this.last;
    if (!last) return false;
    this.text = last.text;
    this.mode = last.mode;
    this.output = last.output;
    this.filters = last.filters;
    this.itemRows = last.itemRows;
    this.presetId = last.presetId;
    this.dirty = last.dirty;
    if (!last.output || !last.filters) {
      void this.run(last.filters ?? undefined);
    } else {
      this.phase = { kind: 'ready' };
      if (last.dirty) void this.search();
    }
    return true;
  }

  /** Refaz a busca do item atual com os filtros editados. */
  async search(): Promise<void> {
    if (!this.text || !this.filters) return;
    await this.run($state.snapshot(this.filters) as SearchFilters);
  }

  /** Falha antes do price check (ex.: nada foi copiado do jogo). */
  fail(error: AppError): void {
    this.requestId++;
    clearTimeout(this.retryTimer);
    clearTimeout(this.autoTimer);
    this.text = '';
    this.mode = 'full';
    this.output = null;
    this.filters = null;
    this.phase = { kind: 'error', error };
  }

  close(): void {
    this.requestId++;
    clearTimeout(this.retryTimer);
    clearTimeout(this.autoTimer);
    if (this.text) {
      this.last = {
        text: this.text,
        output: this.output,
        filters: this.filters ? ($state.snapshot(this.filters) as SearchFilters) : null,
        itemRows: this.itemRows,
        presetId: this.presetId,
        dirty: this.dirty,
        mode: this.mode,
      };
    }
    this.phase = { kind: 'idle' };
    this.output = null;
    this.filters = null;
    this.itemRows = 0;
    this.presetId = null;
    this.dirty = false;
    this.pinned = false;
  }

  setStat(index: number, patch: Partial<StatFilter>): void {
    const stat = this.filters?.stats[index];
    if (!stat) return;
    Object.assign(stat, patch);
    this.markDirty();
  }

  setAllStats(enabled: boolean): void {
    this.filters?.stats.forEach((s) => (s.enabled = enabled));
    this.markDirty();
  }

  /** Liga só os mods que satisfazem `pred` (ex.: "Só T1–T2"). */
  enableWhere(pred: (s: StatFilter) => boolean): void {
    this.filters?.stats.forEach((s) => (s.enabled = pred(s)));
    this.markDirty();
  }

  /** Marca/desmarca todos os mods de um grupo. */
  setGroup(kind: ModifierKind, enabled: boolean): void {
    this.filters?.stats.forEach((s) => {
      if (s.kind === kind) s.enabled = enabled;
    });
    this.markDirty();
  }

  /** Afrouxa os mínimos em `percent`% para ampliar a busca. */
  relax(percent: number): void {
    const factor = percent / 100;
    this.filters?.stats.forEach((s) => {
      if (s.min !== null) s.min = floorLike(s.min - Math.abs(s.min) * factor, s.min);
    });
    this.markDirty();
  }

  /** Margem: mínimo dos mods ligados = valor rolado − pct% (em valor absoluto). */
  setMargin(pct: number): void {
    this.margin = pct;
    const factor = pct / 100;
    this.filters?.stats.forEach((s) => {
      if (s.enabled && s.roll) s.min = floorLike(s.roll.value - Math.abs(s.roll.value) * factor, s.roll.value);
    });
    this.markDirty();
  }

  setFilter<K extends 'online_only' | 'item_level_min' | 'quality_min' | 'links_min'>(key: K, value: SearchFilters[K]): void {
    if (!this.filters) return;
    this.filters[key] = value;
    this.markDirty();
  }

  /** Adiciona um mod do catálogo: entra ligado, sem mín/máx, no fim do seu grupo. Devolve o índice. */
  addStat(stat: CatalogStat): number | null {
    if (!this.filters) return null;
    this.filters.stats.push({
      trade_id: stat.trade_id,
      text: stat.text,
      kind: stat.kind,
      roll: null,
      tier: null,
      enabled: true,
      min: null,
      max: null,
    });
    const index = this.filters.stats.length - 1;
    this.markDirty();
    return index;
  }

  /** A linha `index` veio de "Adicionar mod" (não é do item). */
  isAdded(index: number): boolean {
    return index >= this.itemRows;
  }

  /** Remove um mod adicionado pela busca (os do item não podem ser removidos). */
  removeStat(index: number): void {
    const stat = this.filters?.stats[index];
    if (!this.filters || !stat || !this.isAdded(index)) return;
    this.filters.stats.splice(index, 1);
    this.markDirty();
  }

  /**
   * Aplica um preset: liga os mods do item que estão nele (com o mín/máx salvo; vazio
   * mantém o calculado) e desliga os demais. Devolve quantos mods do preset faltam no item.
   */
  applyPreset(preset: ModPreset): number {
    const stats = this.filters?.stats;
    if (!stats) return 0;
    for (const s of stats) {
      const saved = preset.mods.find((m) => m.trade_id === s.trade_id);
      s.enabled = !!saved;
      if (saved?.min !== null && saved?.min !== undefined) s.min = saved.min;
      if (saved?.max !== null && saved?.max !== undefined) s.max = saved.max;
    }
    this.presetId = preset.id;
    this.markDirty();
    return preset.mods.filter((m) => !stats.some((s) => s.trade_id === m.trade_id)).length;
  }

  private async run(filters: SearchFilters | undefined): Promise<void> {
    const id = ++this.requestId;
    clearTimeout(this.retryTimer);
    clearTimeout(this.autoTimer);
    this.phase = { kind: 'loading' };
    try {
      const out = await backend.priceCheckText(this.text, filters);
      if (id !== this.requestId) return;
      const first = this.output === null;
      this.output = out;
      // Cópia independente: editar filtros não pode alterar `output`.
      this.filters = JSON.parse(JSON.stringify(out.filters)) as SearchFilters;
      this.syncDisplayState(out, first);
      this.dirty = false;
      this.phase = { kind: 'ready' };
      void this.recordHistory(out);
    } catch (err) {
      if (id !== this.requestId) return;
      const error = toAppError(err);
      if (error.code === 'rate-limited') {
        const totalMs = error.data.retry_after_ms;
        this.phase = { kind: 'rate-limited', until: Date.now() + totalMs, totalMs };
        this.retryTimer = setTimeout(() => void this.run(filters), totalMs);
      } else {
        this.phase = { kind: 'error', error };
      }
    }
  }

  /**
   * No primeiro resultado do item, conta as linhas do item: as do início cujo stat é do item
   * (reaberto do histórico, as adicionadas vêm depois).
   */
  private syncDisplayState(out: PriceCheckOutput, first: boolean): void {
    const n = out.filters.stats.length;
    const itemIds = new Set([...out.item.modifiers, ...(out.item.properties ?? [])].map((m) => m.trade_id));
    if (first) {
      const extra = out.filters.stats.findIndex((s) => !itemIds.has(s.trade_id));
      this.itemRows = extra === -1 ? n : extra;
    }
  }

  private async recordHistory(out: PriceCheckOutput): Promise<void> {
    const current = settings.current;
    if (!current) return;
    const isCurrency = out.item.rarity === 'currency';
    // Currency e itens sem trade (a busca falhou) usam o poe.ninja como referência.
    const ninjaOnly = isCurrency || !out.trade;
    const suggested = out.summary?.suggested ?? null;
    await history.record({
      id: this.historyId || newId(),
      game: current.game,
      league: current.league,
      item_text: this.text,
      item: { name: out.item.name, base_type: out.item.base_type, rarity: out.item.rarity, icon: itemIconOf(out) },
      filters: isCurrency ? null : (JSON.parse(JSON.stringify(out.filters)) as SearchFilters),
      buy_chaos: ninjaOnly ? null : (suggested?.buy_chaos ?? null),
      fair_chaos: ninjaOnly ? (out.indexed?.chaos_value ?? out.summary?.estimate_chaos ?? null) : (suggested?.fair_chaos ?? null),
      sell_chaos: ninjaOnly ? null : (suggested?.sell_chaos ?? null),
      sample: out.summary?.sample_size ?? 0,
      source: ninjaOnly ? 'ninja' : 'trade',
      divine_chaos: divineRateOf(out),
      at_ms: Date.now(),
    });
  }
}

export const priceCheck = new PriceCheckStore();
