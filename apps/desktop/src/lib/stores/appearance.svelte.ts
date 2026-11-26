import { backend } from '$lib/backend';
import { settings } from './settings.svelte';

/**
 * Aplica as preferências de aparência (movimento, densidade, escala) à janela atual.
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
}
