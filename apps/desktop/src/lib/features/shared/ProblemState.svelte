<script lang="ts">
  import type { Phase } from '$lib/stores/priceCheck.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { describeError } from '$lib/backend';
  import StateMessage from './StateMessage.svelte';
  import CountdownRing from './CountdownRing.svelte';

  /** Estados de falha comuns a todos os painéis de preço. */
  let { phase }: { phase: Extract<Phase, { kind: 'error' } | { kind: 'rate-limited' }> } = $props();
</script>

{#if phase.kind === 'rate-limited'}
  <StateMessage title="Muitas buscas seguidas" text="A Trade API pediu uma pausa. A busca volta sozinha quando o contador zerar.">
    {#snippet visual()}<CountdownRing until={phase.until} totalMs={phase.totalMs} />{/snippet}
  </StateMessage>
{:else if phase.error.code === 'network'}
  <StateMessage icon="wifi" tone="danger" title="Sem conexão com o trade" text={describeError(phase.error)}>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.search()}>Tentar novamente</button>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.close()}>Fechar</button>
  </StateMessage>
{:else if phase.error.code === 'item-not-recognized' || phase.error.code === 'nothing-copied'}
  <StateMessage icon="question" tone="danger" title="Item não reconhecido" text={describeError(phase.error)}>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.close()}>Fechar</button>
  </StateMessage>
{:else}
  <StateMessage icon="bolt" tone="danger" title="Não foi possível concluir" text={describeError(phase.error)}>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.search()}>Tentar novamente</button>
    <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.close()}>Fechar</button>
  </StateMessage>
{/if}
