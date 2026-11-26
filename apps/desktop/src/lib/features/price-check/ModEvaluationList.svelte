<script lang="ts">
  import type { ModEvaluation } from '$lib/types';
  import { KIND_LABEL } from '$lib/format';
  import { evalMsgs as em } from '$lib/i18n/messages/evaluation';
  import ModText from '$lib/features/shared/ModText.svelte';

  /**
   * Avaliação de cada mod, compacta (uma linha): tipo, texto, tier e o resultado — o % do roll
   * nos mods que variam (≈ quando a faixa veio da base de dados), ou por que o mod fica fora da
   * nota (não varia, versão, Foulborn, sem dados). Uma legenda curta explica os que aparecem.
   * `hitIds`: mods em comum com o seu item (destacados).
   */
  let { mods, hitIds }: { mods: ModEvaluation[]; hitIds: Set<string> } = $props();

  type Shown = 'roll' | 'fixed' | 'variant' | 'mutated' | 'inconclusive';
  /** O que a coluna mostra: o % do roll quando o mod tem nota; senão o motivo. */
  function shownOf(m: ModEvaluation): Shown {
    switch (m.status.kind) {
      case 'exact':
      case 'from_database':
        return m.roll !== null || m.quality !== null ? 'roll' : 'fixed';
      default:
        return m.status.kind;
    }
  }
  const rollOf = (m: ModEvaluation) => m.roll ?? m.quality ?? 0;
  const rollClass = (v: number) => (v >= 0.8 ? 'hi' : v >= 0.4 ? 'mid' : 'lo');
  /** Legenda só com o que aparece neste item, na ordem da coluna. */
  const legend = $derived.by(() => {
    const kinds = new Set(mods.map(shownOf));
    const db = mods.some((m) => shownOf(m) === 'roll' && m.status.kind === 'from_database');
    const keys = (['roll', 'db', 'fixed', 'variant', 'mutated', 'inconclusive'] as const).filter((k) =>
      k === 'db' ? db : kinds.has(k),
    );
    return keys.map((k) => em.legend[k]);
  });
</script>

<div class="list">
  {#each mods as m, i (i)}
    <div class="row" class:hit={hitIds.has(m.trade_id)}>
      <span class="k t-{m.kind}">{KIND_LABEL[m.kind].slice(0, 4)}</span>
      <span class="txt"><ModText text={m.text} /></span>
      <span class="tl mono" class:t1={m.tier === 1} class:t2={m.tier === 2}>{m.tier !== null ? `T${m.tier}` : ''}</span>
      {#if shownOf(m) === 'roll'}
        <span class="st roll mono {rollClass(rollOf(m))}">{m.status.kind === 'from_database' ? '≈' : ''}{Math.round(rollOf(m) * 100)}%</span>
      {:else}
        <span class="st {shownOf(m)}">{em.status[m.status.kind]}</span>
      {/if}
    </div>
  {/each}
  {#if legend.length > 0}
    <ul class="legend">
      {#each legend as line (line)}<li>{line}</li>{/each}
    </ul>
  {/if}
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 26px minmax(0, 1fr) 20px 54px;
    align-items: center;
    gap: 6px;
    padding: 1.5px 0;
    font-size: 11.5px;
    line-height: 1.25;
    color: var(--ms-text-2);
  }
  .row.hit .txt { color: var(--ms-text); }
  .txt { --num-color: var(--ms-text-2); }
  .row.hit .txt { --num-color: var(--ms-accent-2); }
  .k { font-size: 9px; font-weight: 800; text-transform: uppercase; color: var(--ms-explicit); }
  .k.t-implicit { color: var(--ms-implicit); }
  .k.t-fractured { color: var(--ms-fractured); }
  .k.t-crafted { color: var(--ms-crafted); }
  .k.t-enchant { color: var(--ms-enchant); }
  .txt { min-width: 0; }
  .tl { font-size: 9.5px; font-weight: 800; text-align: center; color: var(--ms-text-3); }
  .tl.t1 { color: var(--ms-accent); }
  .tl.t2 { color: var(--ms-info); }
  .st { font-size: 9.5px; font-weight: 700; text-align: right; white-space: nowrap; }
  .st.roll { font-size: 11px; font-weight: 800; }
  .st.roll.hi { color: var(--ms-success); }
  .st.roll.mid { color: var(--ms-text); }
  .st.roll.lo { color: var(--ms-warning); }
  .st.fixed { color: var(--ms-text-3); }
  .st.variant { color: var(--ms-suggest); }
  .st.mutated { color: var(--ms-unique); }
  .st.inconclusive { color: var(--ms-warning); }
  .legend {
    list-style: none;
    margin-top: 5px;
    padding-top: 4px;
    border-top: 1px dashed var(--ms-border);
    font-size: 9.5px;
    line-height: 1.4;
    color: var(--ms-text-3);
  }
</style>
