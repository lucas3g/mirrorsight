<script lang="ts">
  import type { Item } from '$lib/types';
  import { RARITY_COLOR } from '$lib/format';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';

  /**
   * Imagem do item: ícone do poe.ninja → ícone de currency → placeholder neutro com o nome.
   * Ocupa toda a caixa do pai; `size` só ajusta o conteúdo (ícone de currency e texto).
   */
  let {
    item,
    icon = null,
    size = 'lg',
  }: { item: Pick<Item, 'name' | 'base_type' | 'rarity'>; icon?: string | null; size?: 'lg' | 'sm' } = $props();

  const name = $derived(item.name ?? item.base_type);
  /** URL que falhou ao carregar: cai no placeholder sem tentar de novo. */
  let failed = $state<string | null>(null);
  const showImage = $derived(!!icon && failed !== icon);
</script>

<div class="ia {size}">
  {#if showImage && icon}
    <img src={icon} alt={name} title={name} loading="lazy" decoding="async" draggable="false" onerror={() => (failed = icon)} />
  {:else if item.rarity === 'currency'}
    <CurrencyIcon currency={item.base_type} size={size === 'lg' ? 40 : 30} />
  {:else}
    <span class="ph" style:color={RARITY_COLOR[item.rarity]} title={name}>{name}</span>
  {/if}
</div>

<style>
  .ia {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    overflow: hidden;
  }
  img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.7));
  }
  .ph {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    overflow: hidden;
    padding: 0 3px;
    text-align: center;
    font-size: 10px;
    font-weight: 650;
    line-height: 1.2;
    overflow-wrap: anywhere;
  }
  .sm .ph { font-size: 8.5px; padding: 0 2px; }
  .lg img { padding: 4px; }
  .sm img { padding: 2px; }
</style>
