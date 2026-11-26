<script lang="ts">
  /** Renderiza "Ctrl + Shift + D" como teclas. */
  let { combo }: { combo: string } = $props();
  const keys = $derived(combo.split('+').map((k) => k.trim()).filter(Boolean));
</script>

<span class="caps">
  {#each keys as key, i (i)}<kbd class="kbd">{key}</kbd>{/each}
</span>

<style>
  .caps { display: inline-flex; gap: 3px; align-items: center; }
</style>
