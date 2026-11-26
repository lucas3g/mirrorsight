<script lang="ts" generics="T extends string">
  import type { Snippet } from 'svelte';

  interface Option {
    value: T;
    label: string;
  }

  let {
    options,
    value = $bindable(),
    size = 'sm',
    tone = 'accent',
    pill = false,
    ariaLabel,
    onchange,
    item,
  }: {
    options: Option[];
    value: T;
    size?: 'sm' | 'lg';
    /** `suggest` = variante lavanda (preço sugerido). */
    tone?: 'accent' | 'suggest';
    /** Cantos totalmente arredondados (seletor de jogo da barra de título). */
    pill?: boolean;
    ariaLabel?: string;
    onchange?: (value: T) => void;
    /** Conteúdo customizado do botão (ex.: ícone de currency + texto). */
    item?: Snippet<[Option]>;
  } = $props();

  let buttons: HTMLButtonElement[] = $state([]);
  let indicator = $state({ x: 0, w: 0 });

  function measure() {
    const idx = options.findIndex((o) => o.value === value);
    const el = buttons[idx];
    if (el) indicator = { x: el.offsetLeft, w: el.offsetWidth };
  }

  $effect(() => {
    void value;
    void options.length;
    measure();
  });

  $effect(() => {
    const ro = new ResizeObserver(measure);
    buttons.forEach((b) => b && ro.observe(b));
    return () => ro.disconnect();
  });

  function select(v: T) {
    if (v === value) return;
    value = v;
    onchange?.(v);
  }
</script>

<div class="seg {tone}" class:lg={size === 'lg'} class:pill role="radiogroup" aria-label={ariaLabel}>
  <span class="ind" style:width="{indicator.w}px" style:transform="translateX({indicator.x}px)"></span>
  {#each options as opt, i (opt.value)}
    <button
      bind:this={buttons[i]}
      type="button"
      role="radio"
      aria-checked={opt.value === value}
      onclick={() => select(opt.value)}
    >
      {#if item}{@render item(opt)}{:else}{opt.label}{/if}
    </button>
  {/each}
</div>

<style>
  .seg {
    position: relative;
    display: inline-flex;
    flex: none;
    padding: 3px;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-0);
    border: 1px solid var(--ms-border);
  }
  button {
    position: relative;
    z-index: 1;
    height: 26px;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 650;
    white-space: nowrap;
    color: var(--ms-text-3);
    border-radius: 7px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: color 200ms;
  }
  button:hover { color: var(--ms-text-2); }
  .accent button[aria-checked='true'] { color: var(--ms-on-accent); }
  .suggest button[aria-checked='true'] { color: #160d33; }
  .lg button { height: 30px; padding: 0 14px; font-size: 12.5px; }
  .ind {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 0;
    border-radius: 7px;
    background: var(--ms-accent);
    transition: transform 380ms var(--ms-spring), width 380ms var(--ms-spring);
  }
  .suggest .ind { background: var(--ms-suggest); }
  .pill,
  .pill button,
  .pill .ind { border-radius: 99px; }
  .pill { background: var(--ms-surface-2); }
  .pill button { height: 22px; padding: 0 11px; font-size: 11.5px; font-weight: 700; }
</style>
