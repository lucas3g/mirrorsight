<script lang="ts">
  import { backend } from '$lib/backend';
  import type { Item } from '$lib/types';
  import { INFLUENCE_LABEL, RARITY_COLOR, maxLinks } from '$lib/format';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import ItemArt from '$lib/ui/ItemArt.svelte';
  import Sockets from '$lib/ui/Sockets.svelte';

  /** Cartão do item: imagem, raridade, nome/base, propriedades e ações (fixar, trade, fechar). */
  let {
    item,
    webUrl = null,
    icon = null,
  }: { item: Item | null; webUrl?: string | null; icon?: string | null } = $props();

  const links = $derived(item?.sockets ? maxLinks(item.sockets) : 0);
  const hasTiers = $derived(item?.modifiers.some((m) => m.tier !== null) ?? false);
  const fractured = $derived(item?.modifiers.some((m) => m.kind === 'fractured') ?? false);

  function togglePin() {
    priceCheck.pinned = !priceCheck.pinned;
    toasts.push('info', priceCheck.pinned ? pm.pinnedToast : pm.unpinnedToast);
  }
</script>

<section class="card hero rarity" style:--rar={item ? RARITY_COLOR[item.rarity] : 'var(--ms-border-strong)'} aria-label={pm.itemAria}>
  <div class="top">
    <div class="art">
      {#if item}<ItemArt {item} {icon} size="lg" />{:else}<Icon name="item" size={28} />{/if}
    </div>

    {#if item}
      <div class="ident">
        <span class="label rar">{pm.rarity[item.rarity]} · {item.item_class}</span>
        <h2>{item.name ?? item.base_type}</h2>
        {#if item.name}<p class="base">{item.base_type}</p>{/if}
        <div class="props">
          {#if item.item_level}<span class="prop">iLvl <b>{item.item_level}</b></span>{/if}
          {#if item.quality}<span class="prop">{pm.quality} <b>+{item.quality}%</b></span>{/if}
          {#if item.sockets}<Sockets sockets={item.sockets} />{#if links >= 2}<span class="prop"><b>{links}L</b></span>{/if}{/if}
          {#each item.influences as influence (influence)}<span class="prop inf">{INFLUENCE_LABEL[influence]}</span>{/each}
          {#if fractured}<span class="prop frac">Fractured</span>{/if}
          {#if item.corrupted}<span class="prop bad">{pm.corrupted}</span>{/if}
          {#if !item.identified}<span class="prop">{pm.unidentified}</span>{/if}
          {#if hasTiers}<span class="prop" title={pm.advancedCopy}>Tiers <b>Ctrl+Alt+C</b></span>{/if}
        </div>
      </div>
    {:else}
      <div class="ident">
        <div class="sk" style="width:110px;height:11px"></div>
        <div class="sk" style="width:200px;height:20px;margin-top:8px"></div>
        <div class="sk" style="width:150px;height:13px;margin-top:6px"></div>
        <div class="sk" style="width:220px;height:23px;margin-top:10px"></div>
      </div>
    {/if}

    <div class="actions">
      <button class="ib" class:on={priceCheck.pinned} onclick={togglePin} title={pm.pinPanel} aria-pressed={priceCheck.pinned}>
        <Icon name="pin" />
      </button>
      {#if webUrl}
        <button class="ib" onclick={() => backend.openExternal(webUrl)} title={pm.openTrade}><Icon name="ext" /></button>
      {/if}
      <button class="ib" onclick={() => priceCheck.close()} title={pm.closeEsc}><Icon name="x" /></button>
    </div>
  </div>
</section>

<style>
  .hero { padding: 16px 16px 14px; overflow: hidden; flex: none; }
  .top { display: flex; gap: 14px; align-items: flex-start; }
  .art {
    width: 58px;
    height: 76px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border-strong);
    color: var(--rar);
  }
  .ident { flex: 1; min-width: 0; }
  .rar { color: var(--rar); }
  h2 {
    margin-top: 2px;
    font-size: 19px;
    font-weight: 750;
    line-height: 1.15;
    letter-spacing: -0.01em;
    color: var(--rar);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .base { margin-top: 2px; font-size: 13.5px; font-weight: 600; color: var(--ms-text); }
  .props { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 10px; }
  .props :global(.chip) { height: 23px; border-radius: var(--ms-r-sm); padding: 0 8px; font-size: 11.5px; }
  .inf { color: var(--ms-success); }
  .frac { color: var(--ms-fractured); }
  .actions { display: flex; gap: 2px; margin: -4px -4px 0 0; }
</style>
