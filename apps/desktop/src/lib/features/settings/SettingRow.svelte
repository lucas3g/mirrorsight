<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Linha de configuração (`.srow`: um card por ajuste). `keywords` alimenta a busca de ajustes. */
  let {
    label,
    description,
    keywords = '',
    wrap = false,
    extra,
    children,
  }: {
    label: string;
    description?: string;
    keywords?: string;
    wrap?: boolean;
    /** Conteúdo abaixo da descrição (ex.: mensagem de validação). */
    extra?: Snippet;
    children: Snippet;
  } = $props();
</script>

<div class="srow row" class:wrap data-search="{label} {description ?? ''} {keywords}">
  <div class="lbl">
    <b>{label}</b>
    {#if description}<span>{description}</span>{/if}
    {#if extra}{@render extra()}{/if}
  </div>
  <div class="control">{@render children()}</div>
</div>

<style>
  .row { min-height: 60px; transition: border-color var(--ms-t-fast), box-shadow var(--ms-t-fast); }
  .row.wrap { flex-wrap: wrap; }
  /* Destaque da busca: borda e anel menta, sem fade. */
  .row:global(.hl) { border-color: var(--ms-accent); box-shadow: 0 0 0 3px var(--ms-accent-soft); }
  .lbl { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  b { font-weight: 650; font-size: 13px; }
  span { font-size: 11.5px; color: var(--ms-text-3); }
  .control { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
</style>
