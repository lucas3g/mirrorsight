<script lang="ts">
  import { backend } from '$lib/backend';
  import { formatAmount, relativeAge } from '$lib/format';
  import { bulkMsgs as m } from '$lib/i18n/messages/bulk';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Sparkline from '$lib/ui/Sparkline.svelte';
  import ChaosValue from '$lib/features/shared/ChaosValue.svelte';
  import ProblemState from '$lib/features/shared/ProblemState.svelte';
  import StateMessage from '$lib/features/shared/StateMessage.svelte';

  /** Variante para currency/fragmentos: taxa de referência + ofertas por unidade. */
  const output = $derived(priceCheck.output);
  const phase = $derived(priceCheck.phase);
  const indexed = $derived(output?.indexed ?? null);
  const listings = $derived(output?.trade?.listings ?? []);
  const divineRate = $derived(output?.summary?.divine_chaos ?? null);
  const change = $derived(indexed?.change_24h_percent ?? null);
</script>

{#if output}
  <article class="card rarity bulk enter-right" style:--rar="var(--ms-currency)" aria-label="Bulk">
    <header class="head">
      <div class="ident">
        <div class="icon"><CurrencyIcon currency={output.item.base_type} size={42} /></div>
        <div>
          <h2>{output.item.base_type}</h2>
          <p>{output.item.item_class}</p>
        </div>
      </div>
      <div class="rate">
        <span class="label">{m.refRate}</span>
        {#if indexed}
          <div class="big"><ChaosValue chaos={indexed.chaos_value} {divineRate} size={30} itemName={output.item.base_type} /></div>
          <div class="sub">
            <Sparkline values={indexed.sparkline} width={70} height={18}
              color={change !== null && change < 0 ? 'var(--ms-danger)' : 'var(--ms-success)'} />
            {#if change !== null}
              <span class="chg" class:up={change >= 0} class:down={change < 0}>{change >= 0 ? '+' : ''}{formatAmount(change)}%</span>
            {/if}
          </div>
        {:else}
          <div class="big"><b class="mono none">—</b></div>
        {/if}
      </div>
      <button class="ib close" onclick={() => priceCheck.close()} title={m.closeTitle}><Icon name="x" /></button>
    </header>

    <div class="col-head">
      <span class="label">{m.offers} {#if output.trade}<span class="cnt">{output.trade.total}</span>{/if}</span>
      {#if output.trade?.web_url}
        <button class="btn-text" onclick={() => output.trade?.web_url && backend.openExternal(output.trade.web_url)}>
          {m.openSite} <Icon name="ext" size={12} />
        </button>
      {/if}
    </div>

    <div class="body scroll">
      {#if phase.kind === 'error' || phase.kind === 'rate-limited'}
        <ProblemState {phase} />
      {:else if listings.length === 0}
        <StateMessage icon="empty" title={m.noOffers} />
      {:else}
        <table>
          <thead><tr><th>{m.cols.price}</th><th>{m.cols.stock}</th><th>{m.cols.seller}</th><th class="r">{m.cols.age}</th></tr></thead>
          <tbody>
            {#each listings as l, i (i)}
              <tr style:--i={i}>
                <td><span class="amt mono"><CurrencyIcon currency={l.price.currency} size={20} />{formatAmount(l.price.amount)}</span></td>
                <td class="mono muted">{l.stack_size ?? '—'}</td>
                <td><span class="dot" class:on={l.online} class:afk={!l.online}></span> {l.account}</td>
                <td class="mono muted r">{relativeAge(l.indexed_at)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </article>
{/if}

<style>
  .bulk { width: min(600px, 100%); height: 100%; max-height: 100%; display: flex; flex-direction: column; overflow: hidden; }
  .head { position: relative; display: grid; grid-template-columns: 1fr auto; gap: 16px; padding: 18px 54px 16px 18px; }
  .close { position: absolute; right: 12px; top: 12px; }
  .ident { display: flex; gap: 12px; align-items: center; min-width: 0; }
  .icon {
    width: 58px;
    height: 58px;
    display: grid;
    place-items: center;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border-strong);
  }
  h2 { font-size: 19px; font-weight: 750; letter-spacing: -0.01em; color: var(--ms-currency); }
  p { font-size: 12.5px; color: var(--ms-text-2); }
  .rate { text-align: right; }
  .big { display: flex; justify-content: flex-end; margin-top: 4px; }
  .big :global(b) { font: 800 30px/1 var(--ms-mono); letter-spacing: -0.02em; color: var(--ms-price); }
  .big .none { color: var(--ms-text-3); }
  .sub { display: flex; justify-content: flex-end; align-items: center; gap: 6px; margin-top: 8px; }
  .col-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 42px;
    padding: 0 14px;
    border-top: 1px solid var(--ms-border);
    border-bottom: 1px solid var(--ms-border);
    flex: none;
  }
  .col-head .label { display: inline-flex; align-items: center; gap: 6px; }
  .col-head .btn-text { display: inline-flex; align-items: center; gap: 4px; }
  .body { flex: 1; min-height: 0; }
  table { width: 100%; border-collapse: collapse; font-size: 12.5px; }
  th {
    text-align: left;
    height: 34px;
    padding: 0 14px;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ms-text-3);
    border-bottom: 1px solid var(--ms-border);
  }
  td { height: 40px; padding: 0 14px; border-bottom: 1px solid var(--ms-border); }
  tbody tr { animation: ms-row-up 450ms var(--ms-spring) both; animation-delay: calc(var(--i) * 30ms); }
  tbody tr:hover td { background: var(--ms-surface-2); }
  .amt { display: inline-flex; align-items: center; gap: 6px; font-weight: 700; color: var(--ms-price); }
  .muted { color: var(--ms-text-3); }
  .r { text-align: right; }
</style>
