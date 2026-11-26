<script lang="ts">
  import type { ItemQuality } from '$lib/types';
  import { evalMsgs as em } from '$lib/i18n/messages/evaluation';

  /**
   * Nota de qualidade do item (0–100 + grau), independente da build.
   * Sem nota (dados insuficientes) mostra "— Inconclusiva" em vez de um número.
   * `short`: só a nota (overlay, linhas estreitas).
   */
  let { quality, short = false }: { quality: ItemQuality; short?: boolean } = $props();

  const score = $derived(quality.score === null ? null : Math.round(quality.score));
  const label = $derived(quality.grade ? em.grade[quality.grade] : em.inconclusive);
  const title = $derived(`${em.qualityTitle}: ${score ?? '—'} · ${label} · ${em.confidence[quality.confidence]}`);
</script>

<span class="q {quality.grade ?? 'na'}" {title}>
  <b class="mono">{score ?? '—'}</b>{#if !short}<i>{label}</i>{/if}
</span>

<style>
  .q {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 20px;
    padding: 0 7px;
    border-radius: var(--ms-r-sm);
    font-size: 11px;
    white-space: nowrap;
    flex: none;
    background: var(--ms-surface-3);
    color: var(--ms-text-2);
  }
  b { font-weight: 750; }
  i { font-style: normal; font-weight: 600; opacity: 0.85; }
  .excellent { background: var(--ms-success-soft); color: var(--ms-success); }
  .good { background: var(--ms-info-soft); color: var(--ms-info); }
  .average { background: var(--ms-surface-3); color: var(--ms-text); }
  .weak { background: var(--ms-danger-soft); color: var(--ms-danger); }
  .na { background: transparent; box-shadow: inset 0 0 0 1px var(--ms-border-strong); color: var(--ms-text-3); }
</style>
