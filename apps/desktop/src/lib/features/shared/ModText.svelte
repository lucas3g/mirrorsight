<script lang="ts">
  import { splitNumbers } from '$lib/format';

  /** Texto de mod com os números em destaque (sem {@html}). */
  let { text }: { text: string } = $props();
  const parts = $derived(splitNumbers(text));
</script>

{#each parts as part, i (i)}{#if part.numeric}<b class="n">{part.text}</b>{:else}{part.text}{/if}{/each}

<style>
  .n { font-family: var(--ms-mono); font-weight: 700; color: var(--num-color, var(--ms-accent-2)); }
</style>
