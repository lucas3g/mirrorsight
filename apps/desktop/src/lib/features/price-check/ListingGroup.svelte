<script lang="ts">
  import type { GroupEntry, GroupKind, Listing } from '$lib/types';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import ChaosValue from '$lib/features/shared/ChaosValue.svelte';
  import GroupTag from './GroupTag.svelte';
  import ListingRow from './ListingRow.svelte';
  import { GROUP_PREVIEW } from './groups';

  /**
   * Um grupo da listagem agrupada (PoE1): cabeçalho com selo, contagem e menor preço do grupo
   * (na moeda de exibição do usuário), e as linhas.
   *
   * Sem destaque `.best`/"MAIS BARATO" aqui, de propósito (design aprovado, prancheta 1): o
   * mais barato de cada grupo já é o "mín." do cabeçalho; destacar só o mais barato da lista
   * inteira, que pode estar no grupo "Pior", induziria a comparar preços entre grupos.
   */
  let {
    kind,
    entries,
    listings,
    divineRate,
    start,
    compact,
    peekIndex,
    onpeek,
    onunpeek,
    onwhisper,
  }: {
    kind: GroupKind;
    entries: GroupEntry[];
    /** `trade.listings`: as entradas apontam para cá pelo índice original. */
    listings: Listing[];
    /** Chaos por divine (resumo, com reserva do poe.ninja). */
    divineRate: number | null;
    /** Posição da primeira linha na lista inteira (atraso contínuo da animação). */
    start: number;
    compact: boolean;
    peekIndex: number | null;
    onpeek: (entry: GroupEntry, row: HTMLElement) => void;
    onunpeek: () => void;
    onwhisper: (listing: Listing) => void;
  } = $props();

  let expanded = $state(false);
  // Nova busca: o grupo volta a mostrar só os primeiros.
  $effect(() => {
    void entries;
    expanded = false;
  });
  const shown = $derived(expanded ? entries : entries.slice(0, GROUP_PREVIEW));
  const hidden = $derived(entries.length - shown.length);

  const min = $derived(entries.find((e) => e.chaos !== null)?.chaos ?? null);
  const hint = $derived(kind === 'not_comparable' ? pm.groupNotComparableHint : kind === 'no_item' ? pm.groupNoItemHint : null);
</script>

<div class="group" class:wide={!compact}>
  <div class="gh">
    <GroupTag {kind} />
    <span class="cnt">{entries.length}</span>
    {#if min !== null}<span class="gmin mono">{pm.groupMin} <ChaosValue chaos={min} {divineRate} size={14} /></span>{/if}
  </div>
  {#if hint}<p class="gdesc">{hint}</p>{/if}
  {#if entries.length === 0}
    <div class="empty">{pm.groupEmpty}</div>
  {:else}
    {#each shown as entry, i (entry.index)}
      {@const listing = listings[entry.index]}
      {#if listing}
        <ListingRow
          {listing}
          index={start + i}
          best={false}
          {compact}
          peeking={peekIndex === entry.index}
          quality={entry.detail?.quality ?? null}
          onpeek={(row) => onpeek(entry, row)}
          {onunpeek}
          {onwhisper}
        />
      {/if}
    {/each}
    {#if hidden > 0}
      <button class="more" onclick={() => (expanded = true)}>{pm.showMore(hidden)}</button>
    {:else if expanded && entries.length > GROUP_PREVIEW}
      <button class="more" onclick={() => (expanded = false)}>{pm.showLess}</button>
    {/if}
  {/if}
</div>

<style>
  .group + :global(.group) { margin-top: 6px; }
  .gh {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 30px;
    padding: 0 12px;
    background: var(--ms-surface-0);
    border-top: 1px solid var(--ms-border);
    border-bottom: 1px solid var(--ms-border);
  }
  .gmin { display: inline-flex; align-items: center; gap: 5px; margin-left: auto; font-size: 11px; color: var(--ms-text-3); white-space: nowrap; }
  .gmin :global(b) { color: var(--ms-price); font-weight: 700; }
  .gdesc { padding: 4px 12px 2px; font-size: 11px; color: var(--ms-text-3); }
  .empty { display: flex; align-items: center; height: 34px; padding: 0 12px; font-size: 12px; font-style: italic; color: var(--ms-text-3); }
  .more {
    display: flex;
    align-items: center;
    width: 100%;
    height: 30px;
    padding: 0 12px;
    font-size: 11.5px;
    font-weight: 650;
    color: var(--ms-accent);
    transition: background var(--ms-t-fast);
  }
  .more:hover { background: var(--ms-surface-2); }
  /* Janela principal: mesmo recuo das linhas largas. */
  .wide .gh, .wide .gdesc, .wide .empty, .wide .more { padding-left: 14px; padding-right: 14px; }
</style>
