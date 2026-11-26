<script lang="ts">
  import { parseSockets } from '$lib/format';

  /** Chip com os sockets ("B-B-R G"): grupos ligados juntos, separados por espaço. */
  let { sockets }: { sockets: string } = $props();
  const groups = $derived(parseSockets(sockets));
  const total = $derived(groups.reduce((n, g) => n + g.colors.length, 0));
</script>

<span class="chip" title="Sockets: {sockets}">
  <span class="mono">{total}S</span>
  <span class="groups">
    {#each groups as group, gi (gi)}
      <span class="group">
        {#each group.colors as color, si (si)}
          {#if si > 0}<i class="link"></i>{/if}<i class="socket" style:background={color}></i>
        {/each}
      </span>
    {/each}
  </span>
</span>

<style>
  .groups { display: inline-flex; gap: 5px; }
  .group { display: inline-flex; align-items: center; }
  .socket { width: 7px; height: 7px; border-radius: 50%; display: block; }
  .link { width: 3px; height: 2px; background: var(--ms-text-3); display: block; }
</style>
