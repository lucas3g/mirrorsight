<script lang="ts">
  import { backend } from '$lib/backend';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import BulkPanel from '$lib/features/bulk/BulkPanel.svelte';
  import QuickPriceCard from '$lib/features/quick/QuickPriceCard.svelte';
  import ProblemState from '$lib/features/shared/ProblemState.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import IdleView from './IdleView.svelte';
  import PriceCheckPanel from './PriceCheckPanel.svelte';

  /** Escolhe a variante: ocioso, falha sem item, quick, bulk (currency) ou completo. */
  const variant = $derived.by(() => {
    const phase = priceCheck.phase;
    if (phase.kind === 'idle') return 'idle';
    // Sem item parseado não há o que mostrar além do problema.
    if (phase.kind === 'error' && !priceCheck.output) return 'problem';
    if (priceCheck.mode === 'quick') return 'quick';
    if (priceCheck.output?.item.rarity === 'currency') return 'bulk';
    return 'full';
  });

  const compact = backend.windowRole() === 'overlay';
</script>

<div class="view" class:center={variant === 'quick' || variant === 'problem'} class:pad={!compact && variant !== 'full'}>
  {#if variant === 'idle'}
    <IdleView />
  {:else if variant === 'problem' && priceCheck.phase.kind === 'error'}
    <div class="panel problem enter-up">
      <button class="ib close" onclick={() => priceCheck.close()} title={pm.closeEsc}><Icon name="x" /></button>
      <ProblemState phase={priceCheck.phase} />
    </div>
  {:else if variant === 'quick'}
    <QuickPriceCard />
  {:else if variant === 'bulk'}
    <BulkPanel />
  {:else}
    <PriceCheckPanel {compact} />
  {/if}
</div>

<style>
  .view { height: 100%; display: flex; justify-content: center; align-items: stretch; min-height: 0; min-width: 0; container: pcview / inline-size; }
  .view > :global(*) { min-width: 0; }
  .view > :global(.pc) { flex: 1; }
  .pad { padding: 14px; }
  .center { align-items: center; }
  .problem { width: min(420px, 100%); }
  .close { position: absolute; right: 10px; top: 10px; z-index: 2; }
</style>
