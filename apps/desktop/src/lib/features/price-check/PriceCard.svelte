<script lang="ts">
  import type { IndexedPrice, PriceSummary } from '$lib/types';
  import { formatAmount } from '$lib/format';
  import { displayAmount, ninjaDivineRate } from '$lib/pricing';
  import { settings } from '$lib/stores/settings.svelte';
  import ChaosValue from '$lib/features/shared/ChaosValue.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import Sparkline from '$lib/ui/Sparkline.svelte';
  import SuggestedPrice from './SuggestedPrice.svelte';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';

  /** Cartão de preço: estimativa, confiança, mín/mediana/máx, poe.ninja e preço sugerido. */
  let {
    summary,
    indexed = null,
    loading,
  }: { summary: PriceSummary | null; indexed?: IndexedPrice | null; loading: boolean } = $props();

  const big = $derived(summary ? displayAmount(summary.estimate_chaos, summary.divine_chaos, settings.current?.display_currency === 'divine') : null);
  const ninjaChange = $derived(indexed?.change_24h_percent ?? null);
  /** Cotação do divine derivada do próprio poe.ninja, para quando não há listagens. */
  const ninjaDivine = $derived(ninjaDivineRate(indexed));
</script>

<section class="card price" aria-label={pm.priceAria}>
  {#if loading}<div class="loadbar"></div>{/if}
  <div class="row">
    <div class="main">
      <span class="label">{pm.estimatedPrice}</span>
      {#if loading}
        <div class="sk" style="width:180px;height:40px;margin-top:6px"></div>
        <div class="sk" style="width:220px;height:12px;margin-top:10px"></div>
      {:else if summary && big}
        <div class="big">
          <b class="mono">{formatAmount(big.amount)}</b>
          <CurrencyIcon currency={big.currency} size={26} />
          <span class="unit">{big.currency}</span>
        </div>
        <div class="eq">
          {#if big.currency === 'divine'}≈ <b class="mono">{formatAmount(summary.estimate_chaos)}</b> chaos ·{/if}
          {#if summary.divine_chaos}1 div = <b class="mono">{formatAmount(summary.divine_chaos)}</b> c{/if}
        </div>
        <div class="conf">
          <span class="bars" aria-hidden="true">
            {#each [1, 2, 3] as level (level)}
              <i class:on={summary.confidence >= level} class:low={summary.confidence === 1 && level === 1}></i>
            {/each}
          </span>
          {pm.confidence(summary.confidence)} · <b class="mono">{summary.sample_size}</b> {pm.listingsUnit(summary.sample_size)}
        </div>
      {:else if indexed}
        <div class="big ninja-big">
          <ChaosValue chaos={indexed.chaos_value} divineRate={ninjaDivine} size={26} />
        </div>
        <div class="eq">{pm.ninjaPriceNoListings}</div>
      {:else}
        <div class="big"><b class="mono none">—</b></div>
        <div class="eq">{pm.noPricedListings}</div>
      {/if}
    </div>

    <div class="rng">
      {#each [[pm.min, summary?.min_chaos], [pm.median, summary?.median_chaos], [pm.max, summary?.max_chaos]] as [name, value] (name)}
        <div>
          <span>{name}</span>
          {#if loading}<span class="sk" style="width:48px;height:12px"></span>
          {:else if typeof value === 'number'}<ChaosValue chaos={value} divineRate={summary?.divine_chaos ?? null} size={14} />
          {:else}<b class="mono">—</b>{/if}
        </div>
      {/each}
    </div>
  </div>

  {#if indexed && !loading}
    <div class="ninja" title={pm.ninjaReference}>
      poe.ninja <ChaosValue chaos={indexed.chaos_value} divineRate={summary?.divine_chaos ?? ninjaDivine} size={14} />
      <Sparkline values={indexed.sparkline} width={64} height={16} color={ninjaChange !== null && ninjaChange < 0 ? 'var(--ms-danger)' : 'var(--ms-success)'} />
      {#if ninjaChange !== null}
        <span class="chg" class:up={ninjaChange >= 0} class:down={ninjaChange < 0}>{ninjaChange >= 0 ? '+' : ''}{formatAmount(ninjaChange)}%</span>
      {/if}
    </div>
  {/if}

  <SuggestedPrice {summary} {loading} />
</section>

<style>
  .price { padding: 16px; overflow: hidden; flex: none; }
  .row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 16px; align-items: start; }
  .big { display: flex; align-items: center; gap: 10px; margin-top: 6px; }
  .big b { font: 800 40px/1 var(--ms-mono); letter-spacing: -0.03em; color: var(--ms-price); }
  .big .none { color: var(--ms-text-3); }
  .ninja-big :global(.value) { gap: 10px; }
  .ninja-big :global(b) { font: 800 40px/1 var(--ms-mono); letter-spacing: -0.03em; color: var(--ms-price); }
  .unit { font-size: 13px; font-weight: 650; color: var(--ms-text-2); }
  .eq { margin-top: 6px; display: flex; align-items: center; gap: 5px; flex-wrap: wrap; font-size: 12px; color: var(--ms-text-2); }
  .eq b { color: var(--ms-text); }
  .conf { display: flex; align-items: center; gap: 6px; margin-top: 10px; font-size: 12px; color: var(--ms-text-2); }
  .conf b { color: var(--ms-text); }
  .bars { display: flex; gap: 3px; margin-right: 2px; }
  .bars i {
    width: 18px;
    height: 5px;
    border-radius: 3px;
    background: var(--ms-surface-4);
    transform-origin: left;
    animation: ms-grow-x 500ms var(--ms-spring) both;
  }
  .bars i:nth-child(2) { animation-delay: 80ms; }
  .bars i:nth-child(3) { animation-delay: 160ms; }
  .bars i.on { background: var(--ms-success); }
  .bars i.low { background: var(--ms-warning); }
  .rng { display: grid; gap: 6px; min-width: 170px; }
  .rng div {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 6px 10px;
    border-radius: 7px;
    background: var(--ms-surface-2);
    font-size: 11.5px;
    color: var(--ms-text-3);
  }
  .rng :global(b) { font: 700 12.5px var(--ms-mono); color: var(--ms-text); }
  .ninja { display: flex; align-items: center; gap: 6px; margin-top: 12px; font-size: 11.5px; color: var(--ms-text-3); }
  .ninja :global(b) { color: var(--ms-text-2); }
  @media (max-width: 560px) { .row { grid-template-columns: 1fr; } }
</style>
