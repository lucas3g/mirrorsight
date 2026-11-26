import { backend } from '$lib/backend';
import { i18n } from '$lib/i18n/core.svelte';
import { common } from '$lib/i18n/messages/common';
import { settings } from './settings.svelte';

/**
 * Aplica as preferências de aparência (movimento, densidade, escala, idioma) à janela atual.
 * Chamar durante a inicialização de um componente raiz (registra `$effect`s).
 */
export function useAppearance(): void {
  $effect(() => {
    const s = settings.current;
    if (!s) return;
    const root = document.documentElement;
    root.classList.toggle('reduce-motion', s.reduce_motion);
    root.classList.toggle('density-compact', s.density === 'compact');
  });
  $effect(() => {
    const scale = settings.current?.ui_scale;
    if (scale) void backend.setUiScale(scale);
  });
  $effect(() => {
    document.documentElement.lang = i18n.locale;
  });
  // A bandeja é do processo: só a janela principal a traduz.
  if (backend.windowRole() === 'main') {
    $effect(() => {
      const s = settings.current;
      if (!s) return;
      const hotkey = s.hotkeys.find((b) => b.action === 'price-check')?.hotkey ?? 'Ctrl + D';
      backend
        .setTrayLabels({ open: common.tray.open, settings: common.tray.settings, quit: common.tray.quit, tooltip: common.tray.tooltip(hotkey) })
        .catch(() => {});
    });
  }
}
