<script lang="ts">
  import { onMount } from 'svelte';
  import { backend } from '$lib/backend';
  import { handleCommandKey, handlePriceCheckKeys } from '$lib/features/price-check/keyboard';
  import CommandPalette from '$lib/features/commands/CommandPalette.svelte';
  import AppShell from '$lib/shell/AppShell.svelte';
  import { useAppearance } from '$lib/stores/appearance.svelte';
  import { history } from '$lib/stores/history.svelte';
  import { nav } from '$lib/stores/nav.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Toasts from '$lib/ui/Toasts.svelte';

  /** Janela principal: só a montagem (casca, paleta, toasts) e os atalhos globais. */
  let shiftOnPaste = false;

  onMount(() => {
    void settings.load();
    void history.load();
    const subscription = backend.onOpenSettings(() => nav.openSettings());
    return () => void subscription.then((unsubscribe) => unsubscribe());
  });

  useAppearance();

  // Presets do jogo atual (recarrega ao trocar de jogo).
  $effect(() => {
    const game = settings.current?.game;
    if (game) void presets.load(game);
  });

  const isEditable = (t: EventTarget | null) =>
    t instanceof HTMLElement && (t.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(t.tagName));

  function onPaste(e: ClipboardEvent) {
    // Vale só para esta colagem: Shift+Insert ou o menu de contexto depois abrem o modo completo.
    const quick = shiftOnPaste;
    shiftOnPaste = false;
    if (isEditable(e.target)) return;
    const text = e.clipboardData?.getData('text/plain') ?? '';
    if (!text.trim()) return;
    e.preventDefault();
    nav.go('price');
    void priceCheck.check(text, quick ? 'quick' : 'full');
  }

  function onKeydown(e: KeyboardEvent) {
    if (handleCommandKey(e)) return;
    if (e.key.toLowerCase() === 'v' && e.ctrlKey) shiftOnPaste = e.shiftKey;
    if (nav.screen === 'price') handlePriceCheckKeys(e);
  }

  /** O overlay grava histórico e presets por fora: recarrega quando a janela volta ao foco. */
  function onFocus() {
    void history.load();
    const game = settings.current?.game;
    if (game) void presets.load(game);
  }
</script>

<svelte:window onpaste={onPaste} onkeydown={onKeydown} onfocus={onFocus} />

<AppShell />
<CommandPalette role="main" />
<Toasts />
