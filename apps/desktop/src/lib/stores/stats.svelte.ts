import { backend } from '$lib/backend';
import { KIND_ORDER, normalize } from '$lib/format';
import type { CatalogStat, GameVersion, ModifierKind } from '$lib/types';

/** Máximo de resultados exibidos na busca "Adicionar mod". */
const SEARCH_LIMIT = 50;

interface Indexed {
  stat: CatalogStat;
  key: string;
}

export interface StatGroup {
  kind: ModifierKind;
  stats: CatalogStat[];
}

/**
 * Catálogo de stats por jogo, carregado só na primeira abertura de "Adicionar mod"
 * (o do PoE1 tem ~14 mil entradas).
 */
class StatsStore {
  loading = $state(false);
  error = $state(false);
  private cache = new Map<GameVersion, Indexed[]>();
  /** Muda quando um catálogo termina de carregar (dispara as buscas derivadas). */
  version = $state(0);

  async ensure(game: GameVersion): Promise<void> {
    if (this.cache.has(game)) return;
    this.loading = true;
    this.error = false;
    try {
      const list = await backend.listStats(game);
      this.cache.set(game, list.map((stat) => ({ stat, key: normalize(stat.text) })));
      this.version++;
    } catch {
      this.error = true;
    } finally {
      this.loading = false;
    }
  }

  /** Tipos presentes no catálogo (o chip "Pseudo" só aparece se houver pseudo). */
  kinds(game: GameVersion): Set<ModifierKind> {
    void this.version;
    return new Set((this.cache.get(game) ?? []).map((e) => e.stat.kind));
  }

  /** Busca por texto (sem acentos), filtrada por tipo, agrupada na ordem dos grupos. */
  search(game: GameVersion, query: string, kind: ModifierKind | 'all'): StatGroup[] {
    void this.version;
    const all = this.cache.get(game) ?? [];
    const terms = normalize(query.trim()).split(/\s+/).filter(Boolean);
    if (terms.length === 0) return [];
    const hits: CatalogStat[] = [];
    for (const e of all) {
      if (kind !== 'all' && e.stat.kind !== kind) continue;
      if (terms.every((t) => e.key.includes(t))) {
        hits.push(e.stat);
        if (hits.length >= SEARCH_LIMIT) break;
      }
    }
    return KIND_ORDER.map((k) => ({ kind: k, stats: hits.filter((s) => s.kind === k) })).filter((g) => g.stats.length > 0);
  }
}

export const stats = new StatsStore();
