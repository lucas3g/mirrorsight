<script lang="ts">
  import type { Phase } from '$lib/stores/priceCheck.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { describeError } from '$lib/backend';
  import StateMessage from './StateMessage.svelte';
  import CountdownRing from './CountdownRing.svelte';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';

  /** Estados de falha comuns a todos os painéis de preço. */
  let { phase }: { phase: Extract<Phase, { kind: 'error' } | { kind: 'rate-limited' }> } = $props();
</script>

{#if phase.kind === 'rate-limited'}
  <StateMessage title={pm.tooManyTitle} text={pm.tooManyText}>
    {#snippet visual()}<CountdownRing until={phase.until} totalMs={phase.totalMs} />{/snippet}
  </StateMessage>
{:else if phase.error.code === 'network'}
  <StateMessage icon="wifi" tone="danger" title={pm.noConnectionTitle} text={describeError(phase.error)}>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.search()}>{pm.retry}</button>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.close()}>{pm.close}</button>
  </StateMessage>
{:else if phase.error.code === 'item-not-recognized' || phase.error.code === 'nothing-copied'}
  <StateMessage icon="question" tone="danger" title={pm.notRecognizedTitle} text={describeError(phase.error)}>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.close()}>{pm.close}</button>
  </StateMessage>
{:else}
  <StateMessage icon="bolt" tone="danger" title={pm.couldNotFinish} text={describeError(phase.error)}>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.search()}>{pm.retry}</button>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.close()}>{pm.close}</button>
  </StateMessage>
{/if}
