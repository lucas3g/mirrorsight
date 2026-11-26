<script lang="ts">
  import type { Snippet } from 'svelte';
  import { portal } from './portal';

  /**
   * Popover posicionado sob `anchor` (no `<body>`, via portal). Fecha com Esc ou clique fora;
   * o Esc não vaza para o price check (`data-popover` + `stopPropagation`).
   */
  let {
    anchor,
    onclose,
    align = 'start',
    width = 300,
    label,
    children,
  }: {
    anchor: HTMLElement | undefined;
    onclose: () => void;
    align?: 'start' | 'end';
    width?: number;
    label?: string;
    children: Snippet;
  } = $props();

  let box: HTMLElement | undefined = $state();
  let pos = $state({ left: 0, top: 0 });

  function place() {
    if (!anchor) return;
    const r = anchor.getBoundingClientRect();
    const w = Math.min(width, window.innerWidth - 16);
    const left = align === 'end' ? r.right - w : r.left;
    pos = { left: Math.max(8, Math.min(left, window.innerWidth - w - 8)), top: r.bottom + 6 };
  }

  $effect(() => {
    place();
    const onResize = () => place();
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  });

  function onPointerDown(e: PointerEvent) {
    const t = e.target as Node;
    if (box?.contains(t) || anchor?.contains(t)) return;
    onclose();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window onpointerdowncapture={onPointerDown} onkeydowncapture={onKeydown} />

<div
  class="popover"
  data-popover
  role="dialog"
  aria-label={label}
  use:portal
  bind:this={box}
  style:left="{pos.left}px"
  style:top="{pos.top}px"
  style:width="{Math.min(width, window.innerWidth - 16)}px"
>
  {@render children()}
</div>
