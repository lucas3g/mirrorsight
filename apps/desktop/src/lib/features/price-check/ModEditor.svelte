<script lang="ts">
  import { tick } from 'svelte';
  import type { CatalogStat, Item, ModifierKind } from '$lib/types';
  import { KIND_COLOR, KIND_LABEL_PLURAL, KIND_ORDER, maxLinks } from '$lib/format';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import { modsMsgs as mm } from '$lib/i18n/messages/mods';
  import Segmented from '$lib/ui/Segmented.svelte';
  import Tabs from '$lib/ui/Tabs.svelte';
  import AddModSearch from './AddModSearch.svelte';
  import ModRow from './ModRow.svelte';
  import PresetMenu from './PresetMenu.svelte';

  /** Cartão de mods: switch, mín/máx, arrastar para reordenar, presets, margem e busca de mods. */
  let { item, compact = false }: { item: Item | null; compact?: boolean } = $props();

  type Tab = 'mods' | 'props';
  let tab = $state<Tab>('mods');
  let adding = $state(false);
  let listEl: HTMLElement | undefined = $state();

  const filters = $derived(priceCheck.filters);
  const stats = $derived(filters?.stats ?? []);
  const loading = $derived(priceCheck.phase.kind === 'loading');
  const enabledCount = $derived(stats.filter((s) => s.enabled).length);
  const hasTiers = $derived(stats.some((s) => s.tier !== null));
  const game = $derived(settings.current?.game ?? 'poe1');

  /** Linhas agrupadas por tipo, na ordem de exibição do store. */
  const groups = $derived.by(() => {
    const order = priceCheck.order.length === stats.length ? priceCheck.order : stats.map((_, i) => i);
    return KIND_ORDER.map((kind) => ({ kind, indices: order.filter((i) => stats[i]?.kind === kind) })).filter(
      (g) => g.indices.length > 0,
    );
  });

  type OnlineMode = 'online' | 'any';
  const onlineMode = $derived<OnlineMode>(filters?.online_only === false ? 'any' : 'online');

  // ---------- Arrastar para reordenar (pointer events; o DnD do HTML5 gera imagem fantasma) ----------
  let drag = $state<{ index: number; kind: ModifierKind; startY: number; offset: number; pos: number | null; line: number | null } | null>(null);

  function startDrag(e: PointerEvent, index: number) {
    const kind = stats[index]?.kind;
    if (e.button !== 0 || !kind) return;
    e.preventDefault();
    const grip = e.currentTarget as HTMLElement;
    grip.setPointerCapture(e.pointerId);
    drag = { index, kind, startY: e.clientY, offset: 0, pos: null, line: null };
    const move = (ev: PointerEvent) => onDragMove(ev);
    const end = () => {
      grip.removeEventListener('pointermove', move);
      grip.removeEventListener('pointerup', end);
      grip.removeEventListener('pointercancel', end);
      finishDrag();
    };
    grip.addEventListener('pointermove', move);
    grip.addEventListener('pointerup', end);
    grip.addEventListener('pointercancel', end);
  }

  function onDragMove(e: PointerEvent) {
    if (!drag || !listEl) return;
    drag.offset = e.clientY - drag.startY;
    // Destino = quantas linhas do mesmo grupo (sem a arrastada) têm o centro acima do ponteiro.
    const others = [...listEl.querySelectorAll<HTMLElement>(`[data-kind="${drag.kind}"][data-stat]`)].filter(
      (el) => Number(el.dataset.stat) !== drag?.index,
    );
    if (others.length === 0) return;
    const rects = others.map((el) => el.getBoundingClientRect());
    const pos = rects.filter((r) => r.top + r.height / 2 < e.clientY).length;
    const listTop = listEl.getBoundingClientRect().top - listEl.scrollTop;
    const boundary = pos < rects.length ? rects[pos]!.top - 3 : rects[rects.length - 1]!.bottom + 1;
    drag.pos = pos;
    drag.line = boundary - listTop;
  }

  function finishDrag() {
    const d = drag;
    drag = null;
    if (!d || d.pos === null) return;
    const group = groups.find((g) => g.kind === d.kind);
    if (group && group.indices.indexOf(d.index) !== d.pos) priceCheck.moveStat(d.index, d.pos);
  }

  async function moveByKey(index: number, delta: -1 | 1) {
    const group = groups.find((g) => g.indices.includes(index));
    if (!group) return;
    const pos = group.indices.indexOf(index) + delta;
    if (pos < 0 || pos >= group.indices.length) return;
    priceCheck.moveStat(index, pos);
    await tick();
    listEl?.querySelector<HTMLElement>(`[data-stat="${index}"] .grip`)?.focus();
  }

  // ---------- Adicionar mod ----------
  async function pick(stat: CatalogStat) {
    const index = priceCheck.addStat(stat);
    adding = false;
    if (index === null) return;
    await tick();
    const row = listEl?.querySelector<HTMLElement>(`[data-stat="${index}"]`);
    row?.scrollIntoView({ block: 'nearest' });
    row?.querySelector<HTMLInputElement>('input.min')?.focus();
  }

  function toInt(raw: string): number | null {
    const n = Number.parseInt(raw, 10);
    return Number.isFinite(n) ? n : null;
  }

  function groupEnabled(indices: number[]): number {
    return indices.filter((i) => stats[i]?.enabled).length;
  }
</script>

<section class="card mods" class:compact aria-label={mm.modsAria}>
  {#if compact}
    <header class="chead">
      <span class="label">{mm.modsShort} <span class="cnt">{enabledCount}/{stats.length}</span></span>
      <span class="grow"></span>
      {#if filters}<PresetMenu />{/if}
      <button class="btn-text" disabled={!filters} onclick={() => priceCheck.setAllStats(false)}>{mm.none}</button>
    </header>
  {:else}
    <Tabs
      ariaLabel={mm.editorAria}
      tabs={[
        { value: 'mods', label: mm.tabMods, count: filters ? `${enabledCount}/${stats.length}` : null },
        { value: 'props', label: mm.tabProps },
      ]}
      value={tab}
      onchange={(t: Tab) => (tab = t)}
    />
  {/if}

  {#if tab === 'mods' || compact}
    {#if !compact}
      <div class="toolbar">
        <button class="chip" disabled={!filters} onclick={() => priceCheck.setAllStats(true)}>{mm.all}</button>
        <button class="chip" disabled={!filters} onclick={() => priceCheck.setAllStats(false)}>{mm.none}</button>
        <button
          class="chip"
          disabled={!hasTiers}
          title={hasTiers ? mm.onlyT12Title : mm.needTiersTitle}
          onclick={() => priceCheck.enableWhere((s) => s.tier !== null && s.tier <= 2)}
        >
          {mm.onlyT12}
        </button>
        <span class="divider"></span>
        {#if filters}<PresetMenu />{/if}
        <label class="margin" title={mm.marginTitle}>
          {mm.margin}
          <input
            class="slider"
            type="range"
            min="0"
            max="50"
            value={priceCheck.margin}
            style:--p="{(priceCheck.margin / 50) * 100}%"
            disabled={!filters}
            oninput={(e) => priceCheck.setMargin(Number(e.currentTarget.value))}
          />
          <b class="mono">−{priceCheck.margin}%</b>
        </label>
      </div>
      <div class="colhead label"><span></span><span></span><span>{mm.colMod}</span><span>{mm.colTier}</span><span>{mm.colMin}</span><span>{mm.colMax}</span></div>
    {/if}

    <div class="list scroll" bind:this={listEl}>
      {#if !filters}
        {#each [0, 1, 2, 3, 4] as i (i)}
          <div class="sk-row"><div class="sk" style="height:19px"></div><div class="sk" style="height:13px;width:{50 + ((i * 29) % 40)}%"></div><div class="sk" style="height:28px"></div></div>
        {/each}
      {:else}
        {#if stats.length === 0 && !adding}
          <p class="hint">{mm.noMods}</p>
        {/if}
        {#each groups as group (group.kind)}
          {@const on = groupEnabled(group.indices)}
          <div class="grp" style:--kind={KIND_COLOR[group.kind]}>
            <span class="gt">{KIND_LABEL_PLURAL[group.kind]}</span>
            <span class="gc mono">{on}/{group.indices.length}</span>
            <span class="ln"></span>
            <button onclick={() => priceCheck.setGroup(group.kind, on === 0)}>{on === 0 ? mm.checkGroup : mm.uncheckGroup}</button>
          </div>
          {#each group.indices as index, pos (index)}
            {@const stat = stats[index]}
            {#if stat}
              <ModRow
                {stat}
                {index}
                order={pos}
                {compact}
                dragging={drag?.index === index}
                dragOffset={drag?.index === index ? drag.offset : 0}
                ongrip={startDrag}
                onmove={moveByKey}
              />
            {/if}
          {/each}
        {/each}
        {#if drag?.line !== null && drag?.line !== undefined}
          <div class="dropline" style:transform="translateY({drag.line}px)"></div>
        {/if}

        {#if adding}
          <AddModSearch {game} onpick={pick} onclose={() => (adding = false)} />
        {:else}
          <button class="addbtn" onclick={() => (adding = true)}>
            <Icon name="plus" size={14} />{mm.addMod}
            {#if !compact}<small>{mm.addModSub}</small>{/if}
          </button>
          {#if !compact && stats.length > 1}
            <p class="kbhint">{mm.kbHint} <kbd class="kbd">Alt ↑</kbd> <kbd class="kbd">Alt ↓</kbd></p>
          {/if}
        {/if}
      {/if}
    </div>
  {:else if filters}
    <div class="propgrid">
      <label class="pfield">
        <span>{mm.itemLevelMin}<small>{mm.itemValue(String(item?.item_level ?? '—'))}</small></span>
        <input class="num" inputmode="numeric" placeholder={item?.item_level?.toString() ?? '—'} value={filters.item_level_min ?? ''}
          oninput={(e) => priceCheck.setFilter('item_level_min', toInt(e.currentTarget.value))} />
      </label>
      <label class="pfield">
        <span>{mm.qualityMin}<small>{mm.itemValue(item?.quality ? `${item.quality}%` : '—')}</small></span>
        <input class="num" inputmode="numeric" placeholder={item?.quality?.toString() ?? '—'} value={filters.quality_min ?? ''}
          oninput={(e) => priceCheck.setFilter('quality_min', toInt(e.currentTarget.value))} />
      </label>
      <label class="pfield">
        <span>{mm.linksMin}<small>{mm.itemValue(item?.sockets ? `${maxLinks(item.sockets)}L` : '—')}</small></span>
        <input class="num" inputmode="numeric" placeholder={item?.sockets ? String(maxLinks(item.sockets)) : '—'} value={filters.links_min ?? ''}
          oninput={(e) => priceCheck.setFilter('links_min', toInt(e.currentTarget.value))} />
      </label>
    </div>
  {:else}
    <div class="propgrid"><div class="sk" style="height:48px"></div><div class="sk" style="height:48px"></div></div>
  {/if}

  {#if !compact}
    <footer class="foot">
      <Segmented
        ariaLabel={pm.sellers}
        options={[{ value: 'online', label: pm.online }, { value: 'any', label: pm.any }]}
        value={onlineMode}
        onchange={(v: OnlineMode) => priceCheck.setFilter('online_only', v === 'online')}
      />
      {#if priceCheck.dirty}<span class="dirty"><Icon name="info" size={13} />{pm.filtersChanged}</span>{/if}
      <span class="grow"></span>
      <button class="btn btn-ghost btn-sm" disabled={!filters} onclick={() => priceCheck.relax(10)}>{pm.relax10}</button>
      {#if loading}<span class="busy"><span class="spinner"></span>{pm.searching}</span>{/if}
    </footer>
  {/if}
</section>

<style>
  .mods { display: flex; flex-direction: column; min-height: 0; overflow: hidden; }
  .chead { display: flex; align-items: center; gap: 8px; height: 38px; padding: 0 12px; border-bottom: 1px solid var(--ms-border); flex: none; }
  .chead .label { display: inline-flex; align-items: center; gap: 6px; }
  .chead :global(.chip) { height: 24px; font-size: 11.5px; }
  .grow { flex: 1; }
  .toolbar { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; padding: 10px 14px; border-bottom: 1px solid var(--ms-border); flex: none; }
  .toolbar .chip:disabled { color: var(--ms-text-3); cursor: default; }
  .divider { width: 1px; height: 18px; background: var(--ms-border-strong); }
  .margin { margin-left: auto; display: flex; align-items: center; gap: 9px; font-size: 12px; color: var(--ms-text-2); }
  .margin b { min-width: 38px; text-align: right; color: var(--ms-text); }
  .colhead {
    display: grid;
    grid-template-columns: 14px 32px minmax(0, 1fr) 28px 58px 58px;
    gap: 10px;
    padding: 8px 17px 0;
    flex: none;
  }
  .colhead span:nth-child(n + 4) { text-align: center; }
  .list { position: relative; flex: 1; min-height: 0; padding: 4px 8px 10px; }
  .compact .list { padding: 2px 6px 8px; }
  .grp { display: flex; align-items: center; gap: 8px; padding: 12px 8px 6px; }
  .compact .grp { padding: 8px 6px 4px; }
  .gt { font-size: 10.5px; font-weight: 800; letter-spacing: 0.09em; text-transform: uppercase; color: var(--kind); }
  .gc { font-size: 10.5px; font-weight: 700; color: var(--ms-text-3); }
  .ln { flex: 1; height: 1px; background: var(--ms-border); }
  .grp button { font-size: 11.5px; font-weight: 650; color: var(--ms-text-3); }
  .grp button:hover { color: var(--ms-accent); }
  .dropline {
    position: absolute;
    left: 8px;
    right: 8px;
    top: 0;
    height: 2px;
    border-radius: 2px;
    background: var(--ms-accent);
    box-shadow: 0 0 0 3px var(--ms-accent-soft);
    pointer-events: none;
    z-index: 4;
  }
  .addbtn {
    width: 100%;
    height: 34px;
    margin-top: 10px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    border-radius: var(--ms-r-md);
    border: 1.5px dashed var(--ms-border-strong);
    font-weight: 650;
    color: var(--ms-text-2);
    transition: border-color var(--ms-t-fast), color var(--ms-t-fast);
  }
  .addbtn:hover { border-color: var(--ms-accent-line); color: var(--ms-accent); }
  .addbtn small { margin-left: auto; font-size: 11.5px; font-weight: 500; color: var(--ms-text-3); }
  .list :global(.add) { margin-top: 10px; }
  .kbhint { margin-top: 8px; font-size: 11px; color: var(--ms-text-3); display: flex; align-items: center; gap: 4px; }
  .hint { padding: 14px 8px; font-size: 12px; color: var(--ms-text-3); }
  .sk-row { display: grid; grid-template-columns: 32px 1fr 58px; gap: 10px; align-items: center; padding: 12px 8px; }
  .propgrid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; padding: 14px; align-content: start; flex: 1; }
  .foot {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    padding: 10px 14px;
    border-top: 1px solid var(--ms-border);
    background: var(--ms-surface-0);
    flex: none;
  }
  .dirty { display: flex; align-items: center; gap: 6px; font-size: 11.5px; font-weight: 650; color: var(--ms-warning); animation: ms-in-right 350ms var(--ms-spring); }
  .busy { display: flex; align-items: center; gap: 7px; font-size: 12px; font-weight: 650; color: var(--ms-text-2); }
  @media (max-width: 560px) { .propgrid { grid-template-columns: 1fr; } }
</style>
