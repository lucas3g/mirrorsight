<script lang="ts">
  import type { ItemQuality, Listing } from '$lib/types';
  import { formatAmount, relativeAge } from '$lib/format';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import QualityChip from './QualityChip.svelte';

  let {
    listing,
    index,
    best,
    peeking,
    compact = false,
    quality = null,
    onpeek,
    onunpeek,
    onwhisper,
  }: {
    listing: Listing;
    index: number;
    best: boolean;
    peeking: boolean;
    /** Overlay: linhas de 34px, com iLvl, só o olho. */
    compact?: boolean;
    /** Qualidade do item anunciado (listagem agrupada); independe do grupo/upgrade. */
    quality?: ItemQuality | null;
    onpeek: (row: HTMLElement) => void;
    onunpeek: () => void;
    onwhisper: (listing: Listing) => void;
  } = $props();

  let row: HTMLElement | undefined = $state();
</script>

<div class="listing" class:best class:peeking class:compact bind:this={row} style:--i={index}>
  <span class="price">
    <b class="mono">{formatAmount(listing.price.amount)}</b><CurrencyIcon currency={listing.price.currency} size={16} />
    {#if best && !compact}<span class="cheap">{pm.cheapest}</span>{/if}
  </span>
  {#if compact}
    <span class="ilvl mono" title="Item level">{listing.item?.item_level ?? '—'}</span>
  {/if}
  <span class="seller">
    <span class="dot" class:on={listing.online} class:afk={!listing.online} title={listing.online ? 'online' : 'offline/afk'}></span>
    <span class="name">{listing.account}</span>
    {#if quality}<span class="qc"><QualityChip {quality} short={compact} /></span>{/if}
  </span>
  <span class="age mono">{relativeAge(listing.indexed_at)}</span>
  <span class="act">
    {#if listing.item}
      <button
        class="ib sm"
        class:on={peeking}
        aria-label={pm.viewListedItem}
        title={pm.viewItem}
        onmouseenter={() => row && onpeek(row)}
        onmouseleave={onunpeek}
        onfocus={() => row && onpeek(row)}
        onblur={onunpeek}
      >
        <Icon name="eye" size={16} />
      </button>
    {/if}
    {#if listing.whisper && !compact}
      <button class="ib sm" title={pm.copyWhisper} aria-label={pm.copyWhisperFor(listing.account)} onclick={() => onwhisper(listing)}>
        <Icon name="chat" size={15} />
      </button>
    {/if}
  </span>
</div>

<style>
  .listing {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr) 70px 64px;
    align-items: center;
    gap: 8px;
    height: var(--ms-row-h);
    padding: 0 14px;
    transition: background var(--ms-t-fast);
    animation: ms-row-up 450ms var(--ms-spring) both;
    animation-delay: calc(var(--i) * 30ms);
  }
  /* Overlay: preço · iLvl · vendedor · listado · olho (cabeçalho em MarketCard `.clh`). */
  .listing.compact { grid-template-columns: 88px 34px minmax(0, 1fr) 52px 28px; height: 34px; padding: 0 12px; }
  .ilvl { font-size: 12px; font-weight: 650; color: var(--ms-text-2); text-align: center; }
  .listing.best { background: var(--ms-surface-2); }
  .listing:hover,
  .peeking { background: var(--ms-surface-2); }
  .price { display: flex; align-items: center; gap: 6px; min-width: 0; }
  .price b { font-size: 13px; font-weight: 700; color: var(--ms-price); }
  .cheap { font-size: 9.5px; font-weight: 700; letter-spacing: 0.04em; color: var(--ms-suggest); white-space: nowrap; }
  .seller { display: flex; align-items: center; gap: 7px; min-width: 0; font-size: 12px; color: var(--ms-text-2); }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .qc { margin-left: auto; display: inline-flex; }
  .age { font-size: 11.5px; color: var(--ms-text-3); text-align: right; }
  .act { display: flex; justify-content: flex-end; gap: 2px; }
</style>
