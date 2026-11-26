import { commands } from '$lib/stores/commands.svelte';
import { priceCheck } from '$lib/stores/priceCheck.svelte';

/** Popovers (presets, confirmações…) tratam o próprio Esc/Enter. */
const insidePopover = (t: EventTarget | null) => t instanceof Element && !!t.closest('[data-popover]');

/**
 * Atalhos do painel de price check (janela principal e overlay): Esc fecha.
 * A busca é automática após editar filtros (ver `AUTO_SEARCH_MS` no store).
 * Com a paleta Ctrl+K ou um popover aberto, nada vaza para o price check.
 */
export function handlePriceCheckKeys(e: KeyboardEvent): void {
  if (commands.open || e.defaultPrevented || insidePopover(e.target)) return;
  if (e.key === 'Escape') priceCheck.close();
}

/** Ctrl+K abre/fecha a paleta. Devolve `true` se tratou o evento. */
export function handleCommandKey(e: KeyboardEvent): boolean {
  if (e.ctrlKey && !e.altKey && !e.shiftKey && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    commands.toggle();
    return true;
  }
  return false;
}
