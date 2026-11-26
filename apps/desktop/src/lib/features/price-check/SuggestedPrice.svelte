<script lang="ts">
  import type { PriceSummary, SuggestedLevel } from '$lib/types';
  import { formatAmount } from '$lib/format';
  import { LEVELS, LEVEL_META, buyoutNote, displayAmount, suggestedFor } from '$lib/pricing';
  import { copySuggestedNote } from '$lib/stores/commands.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';

  /**
   * Preço sugerido em três níveis (Compra p25 / Justo p50 / Venda p75), dica do nível
   * e cópia da nota `~b/o`. `compact` = faixa do overlay.
   */
  let { summary, loading = false, compact = false }: { summary: PriceSummary | null; loading?: boolean; compact?: boolean } = $props();

  const preferDivine = $derived(settings.current?.display_currency === 'divine');
  const sample = $derived(summary?.sample_size ?? 0);
  const ready = $derived(!!summary?.suggested);

  function shown(level: SuggestedLevel) {
    const chaos = suggestedFor(summary, level);
    if (chaos === null || !summary) return null;
    return displayAmount(chaos, summary.divine_chaos, preferDivine);
  }

  const selected = $derived(shown(priceCheck.level));
  const note = $derived.by(() => {
    const chaos = suggestedFor(summary, priceCheck.level);
    return chaos !== null && summary ? buyoutNote(chaos, summary.divine_chaos, preferDivine) : null;
  });
  const sentence = $derived(
    selected ? LEVEL_META[priceCheck.level].sentence(`${formatAmount(selected.amount)} ${selected.currency === 'divine' ? 'div' : 'c'}`) : '',
  );

  function relax() {
    priceCheck.relax(10);
    void priceCheck.search();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
    e.preventDefault();
    const i = LEVELS.indexOf(priceCheck.level) + (e.key === 'ArrowRight' ? 1 : -1);
    priceCheck.level = LEVELS[(i + LEVELS.length) % LEVELS.length]!;
  }
</script>

<div class="sugg" class:compact>
  <div class="head">
    <span class="label">{pm.suggestedPrice}</span>
    {#if !compact && ready}<span class="sub">{pm.percentilesOf(sample)}</span>{/if}
  </div>

  <div class="levels" role="radiogroup" aria-label={pm.levelAria} tabindex="-1" onkeydown={onKeydown}>
    {#each LEVELS as level (level)}
      {@const meta = LEVEL_META[level]}
      {@const value = shown(level)}
      <button
        role="radio"
        aria-checked={priceCheck.level === level}
        tabindex={priceCheck.level === level ? 0 : -1}
        class="lvl"
        class:on={ready && priceCheck.level === level}
        onclick={() => (priceCheck.level = level)}
      >
        <span class="nm">{meta.name} <small class="mono">{meta.tag}</small></span>
        {#if loading}
          <span class="sk" style="height:{compact ? 16 : 22}px;width:70%"></span>
        {:else if value}
          <span class="v mono">{formatAmount(value.amount)}<CurrencyIcon currency={value.currency} size={compact ? 14 : 16} /></span>
        {:else}
          <span class="v mono none">—</span>
        {/if}
        {#if !compact}<span class="hint">{meta.hint}</span>{/if}
      </button>
    {/each}
  </div>

  {#if !loading}
    {#if ready && note}
      {#if !compact}
        <p class="tip"><Icon name="bolt" size={13} />{sentence}</p>
      {/if}
      <button class="btn btn-sm copy" onclick={copySuggestedNote} title={pm.copyNote}>
        <Icon name="copy" size={13} /><span class="mono">{note}</span>
      </button>
    {:else if summary}
      <div class="warn-box few">
        <Icon name="warning" size={14} />
        <span>{pm.tooFew(sample)}</span>
        <button class="btn btn-ghost btn-sm" onclick={relax}>{pm.relax10}</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .sugg {
    margin-top: 14px;
    padding: 12px;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-0);
    border: 1px solid var(--ms-border);
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px 14px;
    align-items: center;
  }
  .head { grid-column: 1 / -1; display: flex; align-items: baseline; gap: 6px; }
  .sub { font-size: 11.5px; color: var(--ms-text-3); }
  .levels { grid-column: 1 / -1; display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
  .lvl {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    padding: 9px 10px;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border);
    text-align: left;
    transition: background var(--ms-t-fast), border-color var(--ms-t-fast), transform 200ms var(--ms-spring);
  }
  .lvl:hover { border-color: var(--ms-border-strong); }
  .lvl:active { transform: scale(0.97); }
  .lvl.on { background: var(--ms-suggest-soft); border-color: var(--ms-suggest); }
  .nm { font-size: 12px; font-weight: 700; color: var(--ms-text-2); }
  .nm small { font-size: 10px; font-weight: 600; color: var(--ms-text-3); }
  .on .nm { color: var(--ms-text); }
  .v { display: flex; align-items: center; gap: 6px; font-size: 20px; font-weight: 800; color: var(--ms-text); }
  .on .v { color: var(--ms-suggest); }
  .v.none { color: var(--ms-text-3); }
  .hint { font-size: 11.5px; color: var(--ms-text-3); }
  .tip { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--ms-text-2); min-width: 0; }
  .tip :global(svg) { color: var(--ms-suggest); flex: none; }
  .copy { justify-self: end; }
  .copy .mono { font-size: 11.5px; }
  .few { grid-column: 1 / -1; }
  .few span { flex: 1; }

  /* Faixa compacta do overlay: rótulo | 3 níveis | nota. */
  .compact {
    margin: 0;
    border-radius: 0;
    border-width: 1px 0;
    grid-template-columns: auto minmax(0, 1fr) auto;
    padding: 8px 14px;
  }
  .compact .head { grid-column: auto; }
  .compact .levels { grid-column: auto; gap: 6px; }
  .compact .lvl { flex-direction: row; align-items: center; justify-content: space-between; padding: 5px 8px; }
  .compact .v { font-size: 16px; }
  .compact .few { grid-column: 1 / -1; padding: 5px 8px; }
  .compact .copy { grid-column: auto; }
</style>
