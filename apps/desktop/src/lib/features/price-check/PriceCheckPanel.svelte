<script lang="ts">
  import { backend } from '$lib/backend';
  import { RARITY_COLOR, formatAmount, maxLinks } from '$lib/format';
  import { displayAmount, itemIconOf } from '$lib/pricing';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import ChaosValue from '$lib/features/shared/ChaosValue.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import ItemArt from '$lib/ui/ItemArt.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';
  import ItemCard from './ItemCard.svelte';
  import MarketCard from './MarketCard.svelte';
  import ModEditor from './ModEditor.svelte';
  import PriceCard from './PriceCard.svelte';
  import SuggestedPrice from './SuggestedPrice.svelte';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';

  /**
   * Price check completo. Janela principal: duas colunas de cartões (item + mods | preço + mercado).
   * `compact` (overlay, 820px): painel único com cabeçalho de preço, faixa do preço sugerido,
   * colunas mods 440px | listagens e rodapé.
   */
  let { compact = false }: { compact?: boolean } = $props();

  let panel: HTMLElement | undefined = $state();

  const output = $derived(priceCheck.output);
  const item = $derived(output?.item ?? null);
  const loading = $derived(priceCheck.phase.kind === 'loading');
  /** Sem `output` ainda (primeira busca do item) também conta como carregando. */
  const waiting = $derived(loading || (!output && priceCheck.phase.kind !== 'error'));
  const summary = $derived(loading ? null : (output?.summary ?? null));
  const indexed = $derived(output?.indexed ?? null);
  const itemIcon = $derived(itemIconOf(output));
  /** Cotação do divine derivada do próprio poe.ninja, para quando não há listagens. */
  const ninjaDivine = $derived(indexed?.divine_value ? indexed.chaos_value / indexed.divine_value : null);

  const big = $derived(summary ? displayAmount(summary.estimate_chaos, summary.divine_chaos, settings.current?.display_currency === 'divine') : null);
  const propsLine = $derived(
    item
      ? [
          item.item_level ? `iLvl ${item.item_level}` : null,
          item.quality ? `Q ${item.quality}%` : null,
          item.sockets && maxLinks(item.sockets) >= 2 ? `${maxLinks(item.sockets)}L` : null,
          item.corrupted ? pm.corrupted : null,
        ]
          .filter(Boolean)
          .join(' · ')
      : '',
  );

  type OnlineMode = 'online' | 'any';
  const onlineMode = $derived<OnlineMode>(priceCheck.filters?.online_only === false ? 'any' : 'online');

  function togglePin() {
    priceCheck.pinned = !priceCheck.pinned;
    toasts.push('info', priceCheck.pinned ? pm.pinnedToast : pm.unpinnedToast);
  }

  /** Arrasta o overlay pelo cabeçalho (menos pelos botões). */
  function dragOverlay(e: MouseEvent) {
    if (e.button !== 0 || backend.windowRole() !== 'overlay') return;
    if ((e.target as Element | null)?.closest('button, a, input, select, textarea, [role="button"]')) return;
    e.preventDefault();
    void backend.startWindowDrag();
  }

  async function openTrade() {
    const url = output?.trade?.web_url;
    if (url) await backend.openExternal(url);
  }
</script>

{#if compact}
  <article
    class="panel rarity ov enter-right"
    style:--rar={item ? RARITY_COLOR[item.rarity] : 'var(--ms-border-strong)'}
    bind:this={panel}
    aria-label="Price check"
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header class="ovhead" onmousedown={dragOverlay} title={pm.dragToMove}>
      <div class="art">
        {#if item}<ItemArt {item} icon={itemIcon} size="sm" />{:else}<div class="sk" style="width:100%;height:100%"></div>{/if}
      </div>
      <div class="ident">
        {#if item}
          <span class="label" style:color="var(--rar)">{item.item_class}</span>
          <div class="nm"><h2>{item.name ?? item.base_type}</h2>{#if item.name}<span class="base">{item.base_type}</span>{/if}</div>
          {#if propsLine}<span class="pl">{propsLine}</span>{/if}
        {:else}
          <div class="sk" style="width:90px;height:10px"></div>
          <div class="sk" style="width:220px;height:18px;margin-top:6px"></div>
        {/if}
      </div>
      <div class="est">
        <span class="label">{pm.estimatedPrice}</span>
        {#if waiting}
          <div class="sk" style="width:120px;height:28px;margin:4px 0 0 auto"></div>
        {:else if summary && big}
          <div class="ev"><b class="mono">{formatAmount(big.amount)}</b><CurrencyIcon currency={big.currency} size={20} /></div>
          <span class="es">
            {pm.estimateLine(formatAmount(summary.estimate_chaos), summary.confidence, summary.sample_size)}
          </span>
        {:else if indexed}
          <div class="ev ninja"><ChaosValue chaos={indexed.chaos_value} divineRate={ninjaDivine} size={20} itemName={item?.base_type} /></div>
          <span class="es">{pm.ninjaNoListings}</span>
        {:else}
          <div class="ev"><b class="mono none">—</b></div>
        {/if}
      </div>
      <div class="acts">
        <button class="ib" class:on={priceCheck.pinned} onclick={togglePin} title={pm.pinPanel} aria-pressed={priceCheck.pinned}><Icon name="pin" /></button>
        <button class="ib" onclick={() => priceCheck.close()} title={pm.closeEsc}><Icon name="x" /></button>
      </div>
    </header>

    <SuggestedPrice {summary} loading={waiting} compact />

    <div class="ovmain">
      <ModEditor {item} compact />
      <MarketCard anchor={panel} compact />
    </div>

    <footer class="ovfoot">
      <Segmented
        ariaLabel={pm.sellers}
        options={[{ value: 'online', label: pm.online }, { value: 'any', label: pm.any }]}
        value={onlineMode}
        onchange={(v: OnlineMode) => priceCheck.setFilter('online_only', v === 'online')}
      />
      <span class="league" title={pm.leagueHint}>{settings.current?.league ?? '—'}</span>
      {#if priceCheck.dirty}<span class="dirty"><Icon name="info" size={13} />{pm.filtersChanged}</span>{/if}
      <span class="grow"></span>
      <button class="btn btn-ghost btn-sm" disabled={!priceCheck.filters} onclick={() => priceCheck.relax(10)}>{pm.relax10}</button>
      {#if output?.trade?.web_url}
        <button class="btn btn-ghost btn-sm" onclick={openTrade}>Trade <Icon name="ext" size={13} /></button>
      {/if}
      {#if loading}<span class="busy"><span class="spinner"></span>{pm.searching}</span>{/if}
    </footer>
  </article>
{:else}
  <div class="pc scroll">
    <div class="col">
      <ItemCard {item} webUrl={output?.trade?.web_url ?? null} icon={itemIcon} />
      <ModEditor {item} />
    </div>
    <div class="col">
      <PriceCard {summary} indexed={output?.indexed ?? null} loading={waiting} />
      <MarketCard />
    </div>
  </div>
{/if}

<style>
  .pc {
    height: 100%;
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    /* Linha nunca menor que o conteúdo: em janela baixa a página rola em vez de um cartão cobrir o outro. */
    grid-auto-rows: minmax(min-content, auto);
    gap: 14px;
    padding: 14px;
  }
  .col { display: flex; flex-direction: column; gap: 14px; min-height: 0; min-width: 0; }
  .col > :global(.mods),
  .col > :global(.market) { flex: 1; min-height: 380px; }
  /*
   * Uma coluna (área estreita): a página rola e os cartões têm altura própria.
   * Medido pela largura real da área (container `pcview`), não da janela: a barra lateral também ocupa espaço.
   */
  @container pcview (max-width: 1000px) {
    .pc { grid-template-columns: minmax(0, 1fr); align-content: start; }
    .col > :global(.mods),
    .col > :global(.market) { flex: none; height: 560px; min-height: 0; }
  }

  /* ---------- Overlay compacto ---------- */
  .ov {
    width: 100%;
    height: 100%;
    border-radius: var(--ms-r-xl);
    box-shadow: var(--ms-shadow-pop);
  }
  .ovhead { display: grid; grid-template-columns: auto minmax(0, 1fr) auto auto; gap: 16px; align-items: start; padding: 16px 14px 12px 16px; cursor: grab; }
  .ovhead button { cursor: pointer; }
  .art {
    width: 40px;
    height: 52px;
    margin-right: -4px;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border-strong);
    overflow: hidden;
  }
  .ident { min-width: 0; }
  .nm { display: flex; align-items: baseline; gap: 8px; min-width: 0; margin-top: 2px; }
  h2 { font-size: 17px; font-weight: 750; color: var(--rar); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .base { font-size: 13px; font-weight: 600; color: var(--ms-text); white-space: nowrap; }
  .pl { display: block; margin-top: 3px; font-size: 11.5px; color: var(--ms-text-2); }
  .est { text-align: right; }
  .ev { display: flex; align-items: center; justify-content: flex-end; gap: 6px; margin-top: 2px; }
  .ev b { font: 800 28px/1 var(--ms-mono); letter-spacing: -0.02em; color: var(--ms-price); }
  .ev .none { color: var(--ms-text-3); }
  .ev.ninja :global(b) { font: 800 28px/1 var(--ms-mono); letter-spacing: -0.02em; color: var(--ms-price); }
  .es { display: block; margin-top: 4px; font-size: 11px; color: var(--ms-text-2); white-space: nowrap; }
  .acts { display: flex; gap: 2px; margin: -4px -2px 0 0; }
  .ovmain { flex: 1; min-height: 0; display: grid; grid-template-columns: 440px minmax(0, 1fr); }
  .ovmain > :global(.card) { border-radius: 0; border-width: 0; background: transparent; }
  .ovmain > :global(.card + .card) { border-left: 1px solid var(--ms-border); }
  .ovfoot {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    padding: 10px 14px;
    border-top: 1px solid var(--ms-border);
    background: var(--ms-surface-0);
  }
  .league { font-size: 11.5px; font-weight: 600; color: var(--ms-text-2); padding: 0 4px; }
  .busy { display: flex; align-items: center; gap: 6px; font-size: 11.5px; font-weight: 650; color: var(--ms-text-2); }
  .dirty { display: flex; align-items: center; gap: 5px; font-size: 11.5px; font-weight: 650; color: var(--ms-warning); }
  .grow { flex: 1; }
</style>
