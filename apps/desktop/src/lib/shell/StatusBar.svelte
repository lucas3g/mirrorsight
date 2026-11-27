<script lang="ts">
  import { APP_VERSION } from '$lib/version';
  import { formatAmount } from '$lib/format';
  import { shellMsgs } from '$lib/i18n/messages/shell';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { updater } from '$lib/stores/updater.svelte';
  import Icon from '$lib/ui/Icon.svelte';

  /** Barra de status (30px): estado da Trade API, cotação do divine e atalhos. */
  let now = $state(Date.now());
  /** Última cotação vista: continua visível depois que o painel fecha. */
  let lastDivine = $state<number | null>(null);

  $effect(() => {
    const rate = priceCheck.output?.summary?.divine_chaos;
    if (rate) lastDivine = rate;
  });

  const m = $derived(shellMsgs.status);
  const phase = $derived(priceCheck.phase);

  $effect(() => {
    if (phase.kind !== 'rate-limited') return;
    const timer = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(timer);
  });

  const api = $derived.by(() => {
    if (phase.kind === 'rate-limited') {
      return { tone: 'warn', text: m.apiRateLimit(Math.max(0, Math.ceil((phase.until - now) / 1000))) };
    }
    if (phase.kind === 'error' && phase.error.code === 'network') return { tone: 'bad', text: m.apiError };
    if (phase.kind === 'loading') return { tone: 'ok', text: m.apiLoading };
    return { tone: 'ok', text: m.apiOk };
  });

  const key = (action: string, fallback: string) =>
    (settings.current?.hotkeys.find((h) => h.action === action)?.hotkey ?? fallback).replace(/\s*\+\s*/g, ' ');
</script>

<footer class="status">
  <span class="it"><span class="led {api.tone}"></span>{api.text}</span>
  {#if lastDivine}
    <span class="it">poe.ninja · 1 div = <b class="mono">{formatAmount(lastDivine)}</b> c</span>
  {/if}
  <span class="grow"></span>
  <span class="it keys">
    <kbd class="kbd">{key('price-check', 'Ctrl + D')}</kbd> {m.keyPriceCheck}
    <kbd class="kbd">{key('price-check-locked', 'Ctrl + Alt + D')}</kbd> {m.keyLocked}
    <kbd class="kbd">Ctrl K</kbd> {m.keyCommands}
    <kbd class="kbd">Esc</kbd> {m.keyClose}
  </span>
  {#if updater.ready}
    <button class="upd" title={m.updateChipTitle} disabled={updater.phase.kind === 'installing'}
      onclick={() => void updater.install()}>
      <span class="ico" class:spin={updater.phase.kind === 'installing'}><Icon name="reset" size={12} /></span>
      <span class="mono">v{updater.ready.version}</span> {m.updateChip}
    </button>
  {/if}
  <span class="ver mono">v{APP_VERSION}</span>
</footer>

<style>
  .status {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    gap: 18px;
    min-width: 0;
    padding: 0 14px;
    border-top: 1px solid var(--ms-border);
    background: var(--ms-surface-0);
    font-size: 11.5px;
    color: var(--ms-text-2);
    white-space: nowrap;
    overflow: hidden;
  }
  .it { display: flex; align-items: center; gap: 7px; }
  .it b { color: var(--ms-text); }
  .keys { gap: 5px; color: var(--ms-text-3); }
  .keys .kbd { margin-left: 6px; }
  .grow { flex: 1; }
  .ver { color: var(--ms-text-3); }
  /* Atualização baixada: chip menta clicável ("vX pronta · Reiniciar"). */
  .upd {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 20px;
    padding: 0 9px 0 7px;
    border: 1px solid var(--ms-accent-line);
    border-radius: 999px;
    background: var(--ms-accent-soft);
    color: var(--ms-accent);
    font: inherit;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: transform var(--ms-t-fast) var(--ms-ease);
  }
  .upd:hover:not(:disabled) { transform: translateY(-1px); }
  .upd:focus-visible { outline: none; box-shadow: var(--ms-focus); }
  .upd:disabled { cursor: progress; }
  .ico { display: inline-flex; }
  .spin { animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .led { position: relative; width: 7px; height: 7px; border-radius: 50%; background: var(--ms-success); }
  .led::after {
    content: '';
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    border: 1.5px solid currentColor;
    color: var(--ms-success);
    animation: ping 2s var(--ms-ease) infinite;
  }
  .led.warn { background: var(--ms-warning); }
  .led.warn::after { color: var(--ms-warning); }
  .led.bad { background: var(--ms-danger); }
  .led.bad::after { color: var(--ms-danger); }
  /* Anel que cresce e some pela borda (cor), sem animar opacidade. */
  @keyframes ping {
    0% { transform: scale(0.4); }
    70%, 100% { transform: scale(1.4); border-color: transparent; }
  }
  @media (max-width: 900px) { .keys { display: none; } }
</style>
