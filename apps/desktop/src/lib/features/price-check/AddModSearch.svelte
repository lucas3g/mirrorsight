<script lang="ts">
  import { highlight, KIND_NAME } from '$lib/format';
  import type { CatalogStat, GameVersion, ModifierKind } from '$lib/types';
  import { stats } from '$lib/stores/stats.svelte';
  import Icon from '$lib/ui/Icon.svelte';

  /** Busca no catálogo de stats do jogo para adicionar um mod ao filtro. */
  let {
    game,
    onpick,
    onclose,
  }: { game: GameVersion; onpick: (stat: CatalogStat) => void; onclose: () => void } = $props();

  type KindFilter = ModifierKind | 'all';

  let query = $state('');
  let kind = $state<KindFilter>('all');
  let active = $state(0);
  let input: HTMLInputElement | undefined = $state();
  let list: HTMLElement | undefined = $state();

  $effect(() => {
    void stats.ensure(game);
    input?.focus();
  });

  const available = $derived(stats.kinds(game));
  /** O chip "Pseudo" só aparece se o catálogo tiver pseudo (o TSV real ainda pode não ter). */
  const chips = $derived(
    (
      [
        { value: 'all', label: 'Todos' },
        { value: 'pseudo', label: 'Pseudo' },
        { value: 'explicit', label: 'Explícito' },
        { value: 'implicit', label: 'Implícito' },
      ] as { value: KindFilter; label: string }[]
    ).filter((c) => c.value === 'all' || available.has(c.value)),
  );
  const groups = $derived(stats.search(game, query, kind));
  const flat = $derived(groups.flatMap((g) => g.stats));

  $effect(() => {
    void query;
    void kind;
    active = 0;
  });

  function pick(stat: CatalogStat | undefined) {
    if (stat) onpick(stat);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (flat.length === 0) return;
      active = (active + (e.key === 'ArrowDown' ? 1 : -1) + flat.length) % flat.length;
      list?.querySelector<HTMLElement>(`[data-row="${active}"]`)?.scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'Enter') {
      e.preventDefault();
      pick(flat[active]);
    } else if (e.key === 'Escape') {
      // preventDefault: o handler global não fecha o price check.
      e.preventDefault();
      e.stopPropagation();
      onclose();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="card add" onkeydown={onKeydown}>
  <div class="top">
    <label class="search">
      <Icon name="search" size={15} />
      <input bind:this={input} bind:value={query} placeholder="Buscar mod…" aria-label="Buscar mod" />
    </label>
    <button class="ib sm" title="Fechar (Esc)" aria-label="Fechar busca" onclick={onclose}><Icon name="x" size={14} /></button>
  </div>
  <div class="chips">
    {#each chips as c (c.value)}
      <button class="chip" class:on={kind === c.value} onclick={() => (kind = c.value)}>{c.label}</button>
    {/each}
  </div>

  <div class="results scroll" bind:this={list} role="listbox" aria-label="Mods encontrados">
    {#if stats.loading}
      <div class="empty"><span class="spinner"></span>Carregando catálogo de mods…</div>
    {:else if stats.error}
      <div class="empty">Não foi possível carregar o catálogo de mods.</div>
    {:else if !query.trim()}
      <div class="empty">Digite parte do texto do mod, ex.: “life”, “resist”, “attack speed”.</div>
    {:else if flat.length === 0}
      <div class="empty">Nenhum mod encontrado para “{query.trim()}”</div>
    {:else}
      {#each groups as group (group.kind)}
        <div class="label grp">{KIND_NAME[group.kind]}</div>
        {#each group.stats as stat (stat.trade_id)}
          {@const n = flat.indexOf(stat)}
          <button
            class="row"
            class:on={n === active}
            role="option"
            aria-selected={n === active}
            data-row={n}
            onmouseenter={() => (active = n)}
            onclick={() => pick(stat)}
          >
            <span class="tag t-{stat.kind}">{KIND_NAME[stat.kind]}</span>
            <span class="txt">
              {#each highlight(stat.text, query) as part, i (i)}{#if part.hit}<b>{part.text}</b>{:else}{part.text}{/if}{/each}
            </span>
            {#if n === active}<kbd class="kbd">↵</kbd>{/if}
          </button>
        {/each}
      {/each}
    {/if}
  </div>
  <div class="foot">↑ ↓ navegar · ↵ adicionar · Esc fechar</div>
</div>

<style>
  .add { padding: 12px; display: flex; flex-direction: column; gap: 10px; animation: ms-in-up 400ms var(--ms-spring) both; }
  .top { display: flex; gap: 8px; align-items: center; }
  .search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 10px;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-0);
    border: 1px solid var(--ms-border-strong);
    color: var(--ms-text-3);
  }
  .search:focus-within { border-color: var(--ms-accent); box-shadow: 0 0 0 3px var(--ms-accent-soft); }
  .search input { flex: 1; min-width: 0; background: none; border: 0; outline: none; color: var(--ms-text); }
  .chips { display: flex; gap: 6px; flex-wrap: wrap; }
  .results { max-height: 260px; display: flex; flex-direction: column; gap: 2px; }
  .grp { padding: 8px 6px 4px; }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 36px;
    padding: 4px 10px;
    border-radius: var(--ms-r-md);
    text-align: left;
    color: var(--ms-text-2);
  }
  .row.on { background: var(--ms-accent-soft); color: var(--ms-text); }
  .row .tag { flex: none; min-width: 64px; text-align: center; }
  .txt { flex: 1; min-width: 0; }
  .txt b { color: var(--ms-accent); font-weight: 700; }
  .empty { display: flex; align-items: center; gap: 8px; padding: 14px 6px; font-size: 12px; color: var(--ms-text-3); }
  .foot { font-size: 11px; color: var(--ms-text-3); }
</style>
