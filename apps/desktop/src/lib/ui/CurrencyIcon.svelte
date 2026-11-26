<script lang="ts">
  import { currencyIconUrl, currencyName } from '$lib/currency';

  /** `currency`: tag da trade API ("chaos") ou nome do item ("Divine Orb"). */
  let { currency, size = 16 }: { currency: string; size?: number } = $props();

  const src = $derived(currencyIconUrl(currency));
  const name = $derived(currencyName(currency));
</script>

{#if src}
  <img class="cur" {src} width={size} height={size} alt={name} title={name} draggable="false" />
{:else}
  <span class="cur fallback" style:width="{size}px" style:height="{size}px" title={name}>
    {name.slice(0, 1).toUpperCase()}
  </span>
{/if}

<style>
  .cur {
    display: inline-block;
    flex: none;
    object-fit: contain;
    vertical-align: -4px;
    filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.7));
  }
  .fallback {
    display: inline-grid;
    place-items: center;
    border-radius: 50%;
    background: var(--ms-surface-3);
    color: var(--ms-text-2);
    font: 700 9px var(--ms-mono);
  }
</style>
