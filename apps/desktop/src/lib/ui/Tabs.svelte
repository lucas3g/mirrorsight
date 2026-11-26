<script lang="ts" generics="T extends string">
  import type { Snippet } from 'svelte';

  interface Tab {
    value: T;
    label: string;
    /** Contador mono ao lado do rótulo (ex.: "6/9"). */
    count?: string | number | null;
  }

  /** Abas de 42px com indicador menta de 2px que desliza (só transform). */
  let {
    tabs,
    value,
    onchange,
    ariaLabel,
    aside,
  }: {
    tabs: Tab[];
    value: T;
    onchange: (value: T) => void;
    ariaLabel?: string;
    /** Conteúdo à direita das abas. */
    aside?: Snippet;
  } = $props();

  let buttons: HTMLButtonElement[] = $state([]);
  let indicator = $state({ x: 0, w: 0 });

  function measure() {
    const el = buttons[tabs.findIndex((t) => t.value === value)];
    if (el) indicator = { x: el.offsetLeft, w: el.offsetWidth };
  }

  $effect(() => {
    void value;
    void tabs.map((t) => t.count).join();
    measure();
  });

  $effect(() => {
    const ro = new ResizeObserver(measure);
    buttons.forEach((b) => b && ro.observe(b));
    return () => ro.disconnect();
  });
</script>

<div class="tabs" role="tablist" aria-label={ariaLabel}>
  {#each tabs as tab, i (tab.value)}
    <button
      bind:this={buttons[i]}
      type="button"
      role="tab"
      aria-selected={tab.value === value}
      class:on={tab.value === value}
      onclick={() => tab.value !== value && onchange(tab.value)}
    >
      {tab.label}
      {#if tab.count !== undefined && tab.count !== null}<span class="cnt">{tab.count}</span>{/if}
    </button>
  {/each}
  <span class="ind" style:width="{indicator.w}px" style:transform="translateX({indicator.x}px)"></span>
  {#if aside}<div class="aside">{@render aside()}</div>{/if}
</div>

<style>
  .tabs {
    position: relative;
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 14px;
    border-bottom: 1px solid var(--ms-border);
    flex: none;
  }
  button {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 42px;
    padding: 0 10px;
    font-weight: 650;
    color: var(--ms-text-3);
    transition: color var(--ms-t-fast);
  }
  button:hover { color: var(--ms-text-2); }
  button.on { color: var(--ms-text); }
  button.on .cnt { background: var(--ms-accent-soft); color: var(--ms-accent); }
  .ind {
    position: absolute;
    left: 0;
    bottom: -1px;
    height: 2px;
    border-radius: 2px;
    background: var(--ms-accent);
    transition: transform 400ms var(--ms-spring), width 400ms var(--ms-spring);
  }
  .aside { margin-left: auto; display: flex; align-items: center; gap: 8px; min-width: 0; font-size: 11.5px; color: var(--ms-text-3); }
</style>
