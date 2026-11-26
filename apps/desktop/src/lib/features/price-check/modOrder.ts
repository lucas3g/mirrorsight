import type { ModifierKind } from '$lib/types';
import { KIND_ORDER } from '$lib/format';

/**
 * Stat de um mod sem o tipo: "fractured.stat_1" → "stat_1". Fractured, explícito e crafted do
 * mesmo stat ocupam a mesma posição ao alinhar um anúncio com o seu item. Implícitos, enchants,
 * propriedades e pseudo mantêm o prefixo: o mesmo hash noutro grupo é outro mod.
 */
export const statKey = (tradeId: string): string => tradeId.replace(/^(?:explicit|fractured|crafted)\./, '');

/** Índices na ordem de exibição do editor: agrupados por tipo (`KIND_ORDER`), na ordem do item. */
export function displayIndices(mods: readonly { kind: ModifierKind }[]): number[] {
  return KIND_ORDER.flatMap((kind) => mods.flatMap((m, i) => (m.kind === kind ? [i] : [])));
}

/**
 * Reordena os mods de um anúncio na ordem dos stats do seu item (`reference`, já na ordem de
 * exibição). Os que o seu item não tem vão para o fim, na ordem original do anúncio.
 */
export function alignTo<T extends { trade_id: string }>(mods: readonly T[], reference: readonly string[]): T[] {
  const position = new Map<string, number>();
  reference.forEach((id, i) => {
    const key = statKey(id);
    if (!position.has(key)) position.set(key, i);
  });
  return mods
    .map((m, i) => ({ m, i, pos: position.get(statKey(m.trade_id)) ?? Number.POSITIVE_INFINITY }))
    .sort((a, b) => a.pos - b.pos || a.i - b.i)
    .map(({ m }) => m);
}
