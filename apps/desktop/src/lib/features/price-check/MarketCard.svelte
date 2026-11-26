<script lang="ts">
  import { untrack } from 'svelte';
  import type { GroupEntry, Listing, ListingGroup as Group } from '$lib/types';
  import { describeError } from '$lib/backend';
  import { formatAmount } from '$lib/format';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import { ninjaDivineRate, suggestedFor } from '$lib/pricing';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { PEEK_EXTRA, SETTLE_MS, overlayPeek } from '$lib/stores/overlayPeek.svelte';
  import ChaosValue from '$lib/features/shared/ChaosValue.svelte';
  import ProblemState from '$lib/features/shared/ProblemState.svelte';
  import StateMessage from '$lib/features/shared/StateMessage.svelte';
  import Tabs from '$lib/ui/Tabs.svelte';
  import ListingRow from './ListingRow.svelte';
  import ListingGroup from './ListingGroup.svelte';
  import ListedItemCard from './ListedItemCard.svelte';
  import { singleListText } from './groups';

  /**
   * Cartão de mercado: abas Listagens / Distribuição.
   * `anchor`: elemento sobre/ao lado do qual o card do olho é posicionado (padrão: o próprio cartão).
   * `compact` (overlay): só listagens, linhas de 34px.
   */
  let { anchor, compact = false }: { anchor?: HTMLElement | undefined; compact?: boolean } = $props();

  type Tab = 'listings' | 'dist';
  let tab = $state<Tab>('listings');
  let self: HTMLElement | undefined = $state();

  const phase = $derived(priceCheck.phase);
  const trade = $derived(priceCheck.output?.trade ?? null);
  /** Na ordem da trade API: do mais barato ao mais caro (base das estatísticas). */
  const listings = $derived(trade?.listings ?? []);
  /** Exibição: do mais barato ao mais caro (ordem da trade API), no overlay e na janela principal. */
  const shown = $derived(listings);
  const summary = $derived(phase.kind === 'loading' ? null : (priceCheck.output?.summary ?? null));
  const scope = $derived(
    priceCheck.filters ? `${priceCheck.filters.online_only ? pm.scopeOnline : pm.scopeAll} · ${settings.current?.league ?? '—'}` : '',
  );

  /** trade_ids dos filtros ativos na última busca: marcam mods "que batem" no card. */
  const matchedIds = $derived(
    new Set((priceCheck.output?.filters.stats ?? []).filter((s) => s.enabled).map((s) => s.trade_id)),
  );

  // ---------- Agrupamento por status (PoE1) ----------
  const cmp = $derived(phase.kind === 'loading' ? null : (priceCheck.output?.comparison ?? null));
  const grouped = $derived(cmp?.mode === 'grouped' ? cmp : null);
  const singleNotice = $derived(cmp?.mode === 'single' ? singleListText(cmp.reason) : null);
  /** Pior/Igual/Melhor sempre; Não comparável e Sem dados só com anúncios. `start` = posição contínua. */
  const visibleGroups = $derived.by(() => {
    const out: { group: Group; start: number }[] = [];
    let start = 0;
    for (const group of grouped?.groups ?? []) {
      const always = group.kind === 'worse' || group.kind === 'equal' || group.kind === 'better';
      if (!always && group.entries.length === 0) continue;
      out.push({ group, start });
      start += group.entries.length;
    }
    return out;
  });
  const you = $derived(grouped?.you ?? null);
  /** Cotação do divine para o "mín." dos grupos: resumo, com a reserva do poe.ninja (como o PriceCheckPanel). */
  const divineRate = $derived(summary?.divine_chaos ?? ninjaDivineRate(priceCheck.output?.indexed));

  const CARD_WIDTH = 330;
  /** `index`: posição em `listings` (a mesma nos dois modos). `entry`: anúncio no modo agrupado. */
  let peek = $state<{ index: number; left: number; top: number; entry: GroupEntry | null } | null>(null);
  /** Olho sob o mouse agora — descarta respostas atrasadas da expansão da janela. */
  let hovering: number | null = null;

  /** Linha do olho em hover (para conferir o `:hover` depois que a janela se acomoda). */
  let peekRow: HTMLElement | null = null;
  /** Até quando a janela pode estar crescendo/encolhendo sob o mouse. */
  let settleUntil = 0;
  let opening = false;
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  const eyeHovered = (row: HTMLElement | null) => !!row?.querySelector('button:hover');

  async function showPeek(index: number, row: HTMLElement, entry: GroupEntry | null = null) {
    hovering = index;
    peekRow = row;
    clearTimeout(hideTimer);
    overlayPeek.keep();
    // No overlay, a janela cresce para o card ficar fora do painel.
    opening = true;
    const side = await overlayPeek.open().finally(() => {
      opening = false;
      settleUntil = Date.now() + SETTLE_MS;
    });
    // A janela mexeu e o olho "saiu" de baixo do mouse por um instante: se ele ainda está ali, segue.
    if (hovering === null && peekRow === row && eyeHovered(row)) hovering = index;
    if (hovering !== index) return;

    const rowRect = row.getBoundingClientRect();
    const panelRect = (anchor ?? self)?.getBoundingClientRect();
    let left: number;
    if (side === 'right') {
      left = (panelRect?.right ?? overlayPeek.panelWidth) + 12;
    } else if (side === 'left') {
      left = Math.max(8, (panelRect?.left ?? PEEK_EXTRA) - CARD_WIDTH - 12);
    } else {
      // Janela principal ou monitor sem espaço: ao lado do painel se couber, senão por cima.
      const roomLeft = panelRect ? panelRect.left - CARD_WIDTH - 12 : -1;
      left = roomLeft >= 8 ? roomLeft : (panelRect?.left ?? 0) + 14;
    }
    // Altura estimada; o clamp mantém o card dentro da janela.
    // + bloco "Comparação" (cabeçalho e 3 linhas) quando o anúncio é comparável.
    const estimated = 120 + (listings[index]?.item?.modifiers.length ?? 0) * 24 + (entry?.detail ? 90 : 0);
    const top = Math.max(8, Math.min(rowRect.top + rowRect.height / 2 - estimated / 2, window.innerHeight - estimated - 8));
    peek = { index, left, top, entry };
  }

  /**
   * Mouse saiu do olho. Logo depois de a janela crescer, isso costuma ser o painel andando
   * sob o mouse (não o usuário): espera assentar e confere o `:hover` antes de fechar,
   * senão fechar → encolher → hover de novo → crescer vira um loop que pisca.
   */
  function hidePeek() {
    hovering = null;
    clearTimeout(hideTimer);
    const wait = opening ? SETTLE_MS : settleUntil - Date.now();
    if (wait > 0) {
      hideTimer = setTimeout(confirmHide, wait);
      return;
    }
    closePeek();
  }

  function confirmHide() {
    if (hovering !== null) return;
    if (opening) {
      hideTimer = setTimeout(confirmHide, SETTLE_MS);
      return;
    }
    if (peek && eyeHovered(peekRow)) {
      hovering = peek.index;
      return;
    }
    closePeek();
  }

  function closePeek() {
    clearTimeout(hideTimer);
    hovering = null;
    peek = null;
    peekRow = null;
    overlayPeek.close();
  }

  async function copyWhisper(listing: Listing) {
    if (!listing.whisper) return;
    try {
      await navigator.clipboard.writeText(listing.whisper);
      toasts.push('success', pm.whisperCopied(listing.account));
    } catch {
      toasts.push('error', pm.whisperFailed);
    }
  }

  const peekListing = $derived(peek ? listings[peek.index] : undefined);
  /** Grupo do anúncio do olho (selo no card). */
  const peekGroup = $derived(
    peek && grouped ? grouped.groups.find((g) => g.entries.some((e) => e.index === peek?.index))?.kind : undefined,
  );

  // Nova busca: as linhas somem sem `mouseleave` e o índice passaria a apontar para outro anúncio.
  $effect(() => {
    void shown;
    void cmp;
    void phase.kind;
    if (untrack(() => peek) !== null) untrack(closePeek);
  });

  // ---------- Distribuição ----------
  const peak = $derived(Math.max(1, ...(summary?.histogram ?? [])));
  const span = $derived(summary ? summary.max_chaos - summary.min_chaos : 0);
  const at = (value: number) => (summary && span > 0 ? ((value - summary.min_chaos) / span) * 100 : 0);
  const markers = $derived(
    summary?.suggested
      ? ([
          { level: 'buy', tag: 'p25', value: summary.suggested.buy_chaos },
          { level: 'fair', tag: 'p50', value: summary.suggested.fair_chaos },
          { level: 'sell', tag: 'p75', value: summary.suggested.sell_chaos },
        ] as const)
      : [],
  );
  /** Faixa (bucket) i toca o intervalo p25–p75? */
  function inBand(i: number): boolean {
    const s = summary?.suggested;
    if (!summary || !s) return false;
    const n = summary.histogram.length;
    const lo = summary.min_chaos + (span * i) / n;
    const hi = summary.min_chaos + (span * (i + 1)) / n;
    return hi >= s.buy_chaos && lo <= s.sell_chaos;
  }
  const selectedChaos = $derived(suggestedFor(summary, priceCheck.level));
</script>

<section class="card market" class:compact bind:this={self} aria-label={pm.market}>
  {#if phase.kind === 'loading'}<div class="loadbar"></div>{/if}
  {#if compact}
    <header class="chead">
      <span class="label">{pm.listings} {#if trade}<span class="cnt">{trade.total}</span>{/if}</span>
      <span class="scope">{priceCheck.filters?.online_only === false ? pm.scopeAll : pm.scopeOnline}</span>
    </header>
  {:else}
    <Tabs
      ariaLabel={pm.market}
      tabs={[
        { value: 'listings', label: pm.listings, count: trade ? trade.total : null },
        { value: 'dist', label: pm.distribution },
      ]}
      value={tab}
      onchange={(t: Tab) => (tab = t)}
    >
      {#snippet aside()}<span class="scope">{scope}</span>{/snippet}
    </Tabs>
  {/if}

  {#if tab === 'listings' || compact}
    {#if you && listings.length > 0}
      <div class="you">
        <span class="label">{pm.yourItem}</span>
        <span class="prop">{pm.avgTier} <b>{you.avg_tier.toFixed(2)}</b></span>
        <span class="prop">{pm.percentile} <b>{Math.round(you.avg_percentile * 100)}%</b></span>
        <span class="prop">{you.valued === 'prefix' ? pm.freePrefix : pm.freeSuffix} <b>{you.free_valued}</b></span>
      </div>
    {/if}
    {#if compact}
      <div class="lhead label clh"><span>{pm.colPrice}</span><span class="c">iLvl</span><span>{pm.colSeller}</span><span class="r">{pm.colListed}</span><span></span></div>
    {:else}
      <div class="lhead label"><span>{pm.colPrice}</span><span>{pm.colSeller}</span><span class="r">{pm.colAge}</span><span></span></div>
    {/if}
    <div class="body scroll" onscroll={closePeek}>
      {#if phase.kind === 'loading'}
        {#each Array.from({ length: 8 }, (_, i) => i) as i (i)}
          <div class="sk-row">
            <div class="sk" style="height:14px"></div>
            <div class="sk" style="height:12px;width:{55 + ((i * 37) % 40)}%"></div>
            <div class="sk" style="height:10px"></div>
          </div>
        {/each}
      {:else if phase.kind === 'error' || phase.kind === 'rate-limited'}
        <ProblemState {phase} />
      {:else if priceCheck.output && !trade && priceCheck.output.trade_error}
        <StateMessage icon="empty" title={pm.noListings} text={pm.tradeFailed}>
          <span class="why" title={describeError(priceCheck.output.trade_error)}>{describeError(priceCheck.output.trade_error)}</span>
          <button class="btn btn-ghost btn-sm" onclick={() => void priceCheck.search()}>{pm.retry}</button>
        </StateMessage>
      {:else if listings.length === 0}
        <StateMessage icon="empty" title={pm.noneFound} text={pm.noneFoundHint}>
          <button class="btn btn-primary btn-sm" onclick={() => { priceCheck.relax(10); void priceCheck.search(); }}>
            {pm.relaxFilters10}
          </button>
        </StateMessage>
      {:else}
        {#if priceCheck.dirty}
          <div class="stale">{pm.staleFilters}</div>
        {/if}
        {#if grouped}
          <div class:stale-list={priceCheck.dirty}>
            {#each visibleGroups as { group, start } (group.kind)}
              <ListingGroup
                kind={group.kind}
                entries={group.entries}
                {listings}
                {divineRate}
                {start}
                {compact}
                peekIndex={peek?.index ?? null}
                onpeek={(entry, row) => void showPeek(entry.index, row, entry)}
                onunpeek={hidePeek}
                onwhisper={copyWhisper}
              />
            {/each}
          </div>
        {:else}
          {#if singleNotice}
            <div class="notice" role="note">
              <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                <circle cx="8" cy="8" r="6.5" fill="none" stroke="currentColor" stroke-width="2" />
                <path d="M8 7.2v4M8 4.8v.01" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
              </svg>
              <span>{singleNotice}</span>
            </div>
          {/if}
          <div class:stale-list={priceCheck.dirty}>
            {#each shown as listing, i (i)}
              <ListingRow
                {listing}
                index={i}
                best={listing === listings[0]}
                {compact}
                peeking={peek?.index === i}
                onpeek={(row) => void showPeek(i, row)}
                onunpeek={hidePeek}
                onwhisper={copyWhisper}
              />
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  {:else}
    <div class="dist scroll">
      {#if summary}
        <div class="hist" aria-label={pm.histogramAria}>
          {#each summary.histogram as count, i (i)}
            <i class:in={inBand(i)} style:height="{Math.max(2, (count / peak) * 100)}%" style:--i={i}></i>
          {/each}
          {#each markers as m (m.level)}
            <span class="mark" class:sel={priceCheck.level === m.level} style:left="{at(m.value)}%">
              <span class="mono">{m.tag}</span>
            </span>
          {/each}
          {#if markers.length === 0}
            <span class="mark" style:left="{at(summary.median_chaos)}%"><span class="mono">{pm.medianShort}</span></span>
          {/if}
        </div>
        <div class="axis mono">
          <span>{formatAmount(summary.min_chaos)} c</span>
          {#if selectedChaos !== null}<span class="selv">{formatAmount(selectedChaos)} c</span>{/if}
          <span>{formatAmount(summary.max_chaos)} c</span>
        </div>
        <div class="kpis">
          <div><span class="label">{pm.min}</span><ChaosValue chaos={summary.min_chaos} divineRate={summary.divine_chaos} /></div>
          <div><span class="label">{pm.median}</span><ChaosValue chaos={summary.median_chaos} divineRate={summary.divine_chaos} /></div>
          <div><span class="label">{pm.max}</span><ChaosValue chaos={summary.max_chaos} divineRate={summary.divine_chaos} /></div>
          <div><span class="label">{pm.listingsKpi}</span><b class="mono">{summary.sample_size}</b></div>
        </div>
      {:else if phase.kind === 'loading'}
        <div class="sk" style="height:170px"></div>
      {:else}
        <StateMessage icon="empty" title={pm.noDistData} />
      {/if}
    </div>
  {/if}
</section>

{#if peek && peekListing?.item}
  <ListedItemCard
    listing={peekListing}
    item={peekListing.item}
    {matchedIds}
    group={peekGroup}
    detail={peek.entry?.detail ?? undefined}
    valued={you?.valued}
    left={peek.left}
    top={peek.top}
  />
{/if}

<style>
  .market { display: flex; flex-direction: column; min-height: 0; overflow: hidden; }
  .chead { display: flex; align-items: center; justify-content: space-between; gap: 8px; height: 38px; padding: 0 12px; border-bottom: 1px solid var(--ms-border); flex: none; }
  .chead .label { display: inline-flex; align-items: center; gap: 6px; }
  .scope { font-size: 11.5px; color: var(--ms-text-3); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .lhead {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr) 70px 64px;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 14px;
    border-bottom: 1px solid var(--ms-border);
    flex: none;
  }
  /* Overlay: mesmas colunas de `.listing.compact` em ListingRow. */
  .lhead.clh { grid-template-columns: 88px 34px minmax(0, 1fr) 52px 28px; height: 28px; padding: 0 12px; }
  .c { text-align: center; }
  .r { text-align: right; }
  .body { flex: 1; min-height: 0; padding: 4px 0; }
  .sk-row { display: grid; grid-template-columns: 120px 1fr 70px; gap: 8px; height: var(--ms-row-h); align-items: center; padding: 0 14px; }
  .stale {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 6px 14px 8px;
    padding: 8px 10px;
    border-radius: var(--ms-r-md);
    font-size: 11.5px;
    color: var(--ms-warning);
    background: var(--ms-warning-soft);
    border: 1px solid var(--ms-warning-line);
  }
  .stale-list :global(.price b) { color: var(--ms-text-2); }
  .you { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; padding: 8px 12px; border-bottom: 1px solid var(--ms-border); flex: none; }
  .you .label { margin-right: 2px; }
  /* Informativo e neutro: o amarelo fica para "Filtros alterados". */
  .notice {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 6px 12px 8px;
    padding: 8px 10px;
    border-radius: var(--ms-r-md);
    font-size: 11.5px;
    line-height: 1.4;
    color: var(--ms-text-2);
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border);
  }
  .notice svg { flex: none; margin-top: 1px; color: var(--ms-info); }
  .why { align-self: center; max-width: 220px; font: 600 11px var(--ms-mono); color: var(--ms-text-3); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .dist { flex: 1; min-height: 0; padding: 32px 16px 14px; display: flex; flex-direction: column; gap: 14px; }
  .hist {
    position: relative;
    height: 170px;
    flex: none;
    display: flex;
    align-items: flex-end;
    gap: 4px;
    border-bottom: 1px solid var(--ms-border);
  }
  .hist i {
    flex: 1;
    border-radius: 4px 4px 0 0;
    background: var(--ms-surface-4);
    transform-origin: bottom;
    animation: ms-grow-y 700ms var(--ms-spring) both;
    animation-delay: calc(var(--i) * 25ms);
  }
  .hist i.in { background: var(--ms-accent); }
  .mark { position: absolute; bottom: 0; top: -6px; width: 2px; margin-left: -1px; background: var(--ms-text-3); transition: left 500ms var(--ms-spring); }
  .mark span {
    position: absolute;
    top: -18px;
    left: 50%;
    transform: translateX(-50%);
    padding: 1px 6px;
    border-radius: 5px;
    font-size: 10.5px;
    font-weight: 700;
    white-space: nowrap;
    color: var(--ms-bg);
    background: var(--ms-text-3);
  }
  .mark.sel { background: var(--ms-suggest); z-index: 1; }
  .mark.sel span { background: var(--ms-suggest); color: #160d33; }
  .axis { display: flex; justify-content: space-between; font-size: 10.5px; font-weight: 600; color: var(--ms-text-3); margin-top: -6px; }
  .selv { color: var(--ms-suggest); }
  .kpis { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
  .kpis div { display: flex; flex-direction: column; gap: 4px; padding: 10px; border-radius: var(--ms-r-md); background: var(--ms-surface-2); border: 1px solid var(--ms-border); }
  .kpis :global(b) { font: 800 16px var(--ms-mono); color: var(--ms-text); }
  @media (max-width: 560px) { .kpis { grid-template-columns: repeat(2, 1fr); } }
</style>
