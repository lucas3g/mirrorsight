<script lang="ts">
  import { formatAmount } from '$lib/format';
  import { displayAmount } from '$lib/pricing';
  import { settings } from '$lib/stores/settings.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';

  /**
   * Exibe um valor em chaos na moeda preferida do usuário (regra em `displayAmount`:
   * acima de 1 divine mostra divine, exceto quando o próprio item é a Divine Orb).
   */
  let {
    chaos,
    divineRate,
    size = 16,
    itemName,
  }: { chaos: number; divineRate: number | null; size?: number; itemName?: string } = $props();

  const display = $derived(displayAmount(chaos, divineRate, settings.current?.display_currency === 'divine', itemName));
</script>

<span class="value"><CurrencyIcon currency={display.currency} {size} /><b class="mono">{formatAmount(display.amount)}</b></span>

<style>
  .value { display: inline-flex; align-items: center; gap: 5px; }
</style>
