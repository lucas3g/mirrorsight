<script lang="ts">
  import type { GroupKind } from '$lib/types';
  import { GROUP_STYLE } from './groups';

  /** Selo do grupo (Pior/Igual/Melhor/…): cor, ícone e texto — não depende só da cor. */
  let { kind }: { kind: GroupKind } = $props();
  const style = $derived(GROUP_STYLE[kind]);
</script>

<span class="gtag {style.cls}">
  <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
    <path d={style.path} fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
  </svg>
  {style.label()}
</span>

<style>
  .gtag {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 20px;
    padding: 0 8px;
    border-radius: 99px;
    border: 1px solid;
    font-size: 10.5px;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    white-space: nowrap;
    flex: none;
  }
  .worse { color: var(--ms-warning); background: var(--ms-warning-soft); border-color: var(--ms-warning-line); }
  .equal { color: var(--ms-accent); background: var(--ms-accent-soft); border-color: var(--ms-accent-line); }
  .better { color: var(--ms-info); background: var(--ms-info-soft); border-color: var(--ms-info-line); }
  .nc { color: var(--ms-text-2); background: var(--ms-surface-2); border-color: var(--ms-border-strong); }
  .null { color: var(--ms-text-3); background: var(--ms-surface-2); border-color: var(--ms-border); }
</style>
