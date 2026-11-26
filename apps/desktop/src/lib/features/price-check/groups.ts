import type { GroupKind, SingleReason } from '$lib/types';
import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';

/** Classe do selo (`.gtag.<cls>`), traço do ícone (viewBox 12) e rótulo de cada grupo. */
export const GROUP_STYLE: Record<GroupKind, { cls: string; path: string; label: () => string }> = {
  worse: { cls: 'worse', path: 'M6 2v8M2.5 6.5L6 10l3.5-3.5', label: () => pm.groupWorse },
  equal: { cls: 'equal', path: 'M2.5 4.5h7M2.5 7.5h7', label: () => pm.groupEqual },
  better: { cls: 'better', path: 'M6 10V2M2.5 5.5L6 2l3.5 3.5', label: () => pm.groupBetter },
  not_comparable: { cls: 'nc', path: 'M3 6h6', label: () => pm.groupNotComparable },
  no_item: { cls: 'null', path: 'M3 3l6 6M9 3l-6 6', label: () => pm.groupNoItem },
};

/** Cor do grupo (texto "· decidiu" e valor que decidiu, no card do olho). */
export const GROUP_COLOR: Record<GroupKind, string> = {
  worse: 'var(--ms-warning)',
  equal: 'var(--ms-accent)',
  better: 'var(--ms-info)',
  not_comparable: 'var(--ms-text-2)',
  no_item: 'var(--ms-text-3)',
};

/** Aviso acima da lista única. */
export function singleListText(reason: SingleReason): string {
  switch (reason.kind) {
    case 'poe2':
      return pm.singleListPoe2;
    case 'rarity':
      return pm.singleListRarity;
    case 'class':
      return pm.singleListClass(reason.class);
    case 'no_status':
      return pm.singleListNoStatus;
  }
}
