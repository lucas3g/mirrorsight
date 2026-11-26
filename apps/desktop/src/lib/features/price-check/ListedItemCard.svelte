<script lang="ts">
  import type { Item, Listing } from '$lib/types';
  import { INFLUENCE_LABEL, KIND_LABEL, RARITY_COLOR, formatAmount, relativeAge } from '$lib/format';
  import Sockets from '$lib/ui/Sockets.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import ModText from '$lib/features/shared/ModText.svelte';
  import { portal } from '$lib/ui/portal';
  import { common } from '$lib/i18n/messages/common';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';

  /** Detalhe do item de uma listagem, exibido ao passar o mouse no olho. */
  let {
    listing,
    item,
    matchedIds,
    left,
    top,
  }: { listing: Listing; item: Item; matchedIds: Set<string>; left: number; top: number } = $props();

  const hasMatches = $derived(item.modifiers.some((m) => matchedIds.has(m.trade_id)));
  const listedLine = $derived.by(() => {
    const age = relativeAge(listing.indexed_at);
    return age === common.now ? pm.listedNow : pm.listedAgo(age);
  });
</script>

<div class="card" use:portal style:left="{left}px" style:top="{top}px" style:--rar={RARITY_COLOR[item.rarity]} role="tooltip">
  <header>
    <div class="name">{item.name ?? item.base_type}</div>
    <div class="base">
      {item.name ? `${item.base_type} · ` : ''}{item.item_class}{item.item_level ? ` · iLvl ${item.item_level}` : ''}
    </div>
    <div class="chips">
      {#each item.influences as influence (influence)}<span class="chip inf">{INFLUENCE_LABEL[influence]}</span>{/each}
      {#if item.modifiers.some((m) => m.kind === 'fractured')}<span class="chip frac">Fractured</span>{/if}
      {#if item.corrupted}<span class="chip corr">Corrupted</span>{/if}
      {#if item.sockets}<Sockets sockets={item.sockets} />{/if}
      {#if item.quality}<span class="chip mono">Q {item.quality}%</span>{/if}
    </div>
  </header>

  {#if item.modifiers.length > 0}
    <section>
      {#each item.modifiers as mod, i (i)}
        <div class="mod" class:match={matchedIds.has(mod.trade_id)}>
          <span class="tag t-{mod.kind}">{KIND_LABEL[mod.kind].slice(0, 4)}</span>
          <span class="txt">{#if matchedIds.has(mod.trade_id)}<span class="hit"></span>{/if}<ModText text={mod.text} /></span>
          {#if mod.tier !== null}<span class="tier" class:t1={mod.tier === 1} class:t2={mod.tier === 2}>T{mod.tier}</span>{:else}<span></span>{/if}
        </div>
      {/each}
      {#if hasMatches}<p class="legend"><span class="hit"></span>{pm.matchesFilters}</p>{/if}
    </section>
  {/if}

  <footer>
    <span class="p mono"><CurrencyIcon currency={listing.price.currency} size={22} />{formatAmount(listing.price.amount)}</span>
    <span class="s">
      <span class="dot" class:on={listing.online} class:afk={!listing.online}></span>
      {listing.account}<br />{listedLine}
    </span>
  </footer>
</div>

<style>
  .card {
    position: fixed;
    z-index: 60;
    width: 330px;
    border-radius: 12px;
    overflow: hidden;
    pointer-events: none;
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border-strong);
    box-shadow: 0 30px 60px -20px #000, 0 0 0 1px #000;
    transform-origin: right center;
    animation: peek-in 350ms var(--ms-spring) both;
  }
  .card::before { content: ''; position: absolute; inset: 0 0 auto; height: 3px; background: var(--rar); }
  @keyframes peek-in { from { transform: scale(0.92) translateX(10px); } }
  header { padding: 14px 14px 10px; border-bottom: 1px solid var(--ms-border); }
  .name { font-size: 15px; font-weight: 700; color: var(--rar); }
  .base { font-size: 12px; color: var(--ms-text-2); margin: 1px 0 8px; }
  .chips { display: flex; flex-wrap: wrap; gap: 4px; }
  section { padding: 9px 14px; border-bottom: 1px solid var(--ms-border); }
  .mod {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 3px 0;
    font-size: 12.5px;
    line-height: 1.3;
    color: var(--ms-text-2);
  }
  .mod.match .txt { color: var(--ms-text); }
  .tier { height: 18px; min-width: 24px; font-size: 9.5px; }
  .hit {
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-right: 6px;
    vertical-align: 1px;
    border-radius: 2px;
    background: var(--ms-accent);
  }
  .legend { font-size: 10.5px; color: var(--ms-text-3); margin-top: 6px; }
  footer { display: flex; align-items: center; gap: 10px; padding: 10px 14px; background: var(--ms-surface-1); }
  .mod .txt { --num-color: var(--ms-text-2); }
  .mod.match .txt { --num-color: var(--ms-accent-2); }
  .p { display: flex; align-items: center; gap: 6px; font-size: 16px; font-weight: 700; color: var(--ms-price); }
  .s { flex: 1; text-align: right; font-size: 11px; color: var(--ms-text-2); line-height: 1.4; }
  .s .dot { margin-right: 5px; }
</style>
