<script lang="ts">
  import type { GameVersion, HistoryEntry } from '$lib/types';
  import { RARITY_COLOR, normalize, relativeWhen } from '$lib/format';
  import { HISTORY_LIMIT, history } from '$lib/stores/history.svelte';
  import { nav } from '$lib/stores/nav.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import ChaosValue from '$lib/features/shared/ChaosValue.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import ItemArt from '$lib/ui/ItemArt.svelte';
  import Popover from '$lib/ui/Popover.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';

  /** Últimos 50 price checks, salvos neste computador. */
  type GameFilter = 'all' | GameVersion;
  let query = $state('');
  let game = $state<GameFilter>('all');
  let clearBtn: HTMLButtonElement | undefined = $state();
  /** Relógio para "há 4 min" se atualizar sozinho. */
  let now = $state(Date.now());

  $effect(() => {
    void history.load();
    const timer = setInterval(() => (now = Date.now()), 30_000);
    return () => {
      clearInterval(timer);
      // Saiu da tela (ex.: Ctrl+K → Enter) com a confirmação aberta: não pode reaparecer na volta.
      nav.confirmClearHistory = false;
    };
  });

  // Histórico vazio: um pedido de confirmação pendente não pode reaparecer depois.
  $effect(() => {
    if (nav.confirmClearHistory && history.entries.length === 0) nav.confirmClearHistory = false;
  });

  const currentGame = $derived(settings.current?.game ?? 'poe1');
  const rows = $derived.by(() => {
    const q = normalize(query.trim());
    return history.entries.filter(
      (e) =>
        (game === 'all' || e.game === game) &&
        (!q || normalize(`${e.item.name ?? ''} ${e.item.base_type}`).includes(q)),
    );
  });

  function reopen(entry: HistoryEntry) {
    nav.go('price');
    void priceCheck.reopen(entry);
  }

  async function confirmClear() {
    nav.confirmClearHistory = false;
    await history.clear();
  }
</script>

<div class="hist">
  <header class="top">
    <div>
      <h1>Histórico de price checks</h1>
      <p>Os últimos {HISTORY_LIMIT} price checks, salvos só neste computador. Reabra para buscar de novo com os mesmos filtros.</p>
    </div>
    <div class="controls">
      <label class="filter">
        <Icon name="search" size={14} />
        <input placeholder="Filtrar por nome…" aria-label="Filtrar por nome" bind:value={query} />
      </label>
      <Segmented
        ariaLabel="Jogo"
        options={[{ value: 'all', label: 'Todos' }, { value: 'poe1', label: 'PoE 1' }, { value: 'poe2', label: 'PoE 2' }]}
        bind:value={game}
      />
      <button
        class="btn btn-ghost danger"
        bind:this={clearBtn}
        disabled={history.entries.length === 0}
        onclick={() => (nav.confirmClearHistory = !nav.confirmClearHistory)}
      >
        <Icon name="trash" size={14} />Limpar histórico
      </button>
    </div>
  </header>

  {#if nav.confirmClearHistory && history.entries.length > 0}
    <Popover anchor={clearBtn} align="end" width={280} label="Confirmar limpeza" onclose={() => (nav.confirmClearHistory = false)}>
      <div class="confirm">
        <b>Apagar os {history.entries.length} price checks?</b>
        <p>Não dá para desfazer. Presets e configurações não são afetados.</p>
        <div class="btns">
          <button class="btn btn-ghost btn-sm" onclick={() => (nav.confirmClearHistory = false)}>Cancelar</button>
          <button class="btn btn-danger btn-sm" onclick={confirmClear}>Limpar</button>
        </div>
      </div>
    </Popover>
  {/if}

  <section class="card table">
    {#if history.entries.length === 0}
      <div class="empty">
        <div class="ill"><Icon name="clock" size={26} /></div>
        <b>Nenhum price check ainda</b>
        <span>Os itens que você precificar aparecem aqui.</span>
      </div>
    {:else}
      <div class="scroll wrap">
        <table>
          <thead>
            <tr>
              <th>Item</th><th>Jogo</th><th>Liga</th><th class="r">Compra</th><th class="r">Justo</th><th class="r">Venda</th>
              <th class="r">Anúncios</th><th>Quando</th><th></th>
            </tr>
          </thead>
          <tbody>
            {#each rows as entry, i (entry.id)}
              {@const otherGame = entry.game !== currentGame}
              <tr style:--i={Math.min(i, 20)}>
                <td class="item">
                  <div class="itemcell">
                    <div class="art" style:--rar={RARITY_COLOR[entry.item.rarity]}>
                      <ItemArt item={entry.item} icon={entry.item.icon ?? null} size="sm" />
                    </div>
                    <div class="txt">
                      <span class="nm" style:color={RARITY_COLOR[entry.item.rarity]}>{entry.item.name ?? entry.item.base_type}</span>
                      {#if entry.item.name}<span class="bt">{entry.item.base_type}</span>{/if}
                    </div>
                  </div>
                </td>
                <td><span class="game">{entry.game === 'poe2' ? 'PoE 2' : 'PoE 1'}</span></td>
                <td class="muted league">{entry.league}</td>
                {#each [entry.buy_chaos, entry.fair_chaos, entry.sell_chaos] as value, j (j)}
                  <td class="r price" class:fair={j === 1}>
                    {#if value !== null}<ChaosValue chaos={value} divineRate={entry.divine_chaos} size={14} itemName={entry.item.base_type} />{:else}<b class="mono none">—</b>{/if}
                  </td>
                {/each}
                <td class="r mono muted">{entry.source === 'ninja' ? 'poe.ninja' : entry.sample}</td>
                <td class="muted when">{relativeWhen(entry.at_ms, now)}</td>
                <td class="acts">
                  <button
                    class="btn btn-sm"
                    disabled={otherGame}
                    title={otherGame ? `Troque para PoE ${entry.game === 'poe2' ? '2' : '1'} para reabrir` : 'Refazer o price check com os mesmos filtros'}
                    onclick={() => reopen(entry)}
                  >
                    Reabrir
                  </button>
                  <button class="ib sm del" title="Remover do histórico" aria-label="Remover do histórico" onclick={() => history.remove(entry.id)}>
                    <Icon name="x" size={14} />
                  </button>
                </td>
              </tr>
            {:else}
              <tr><td colspan="9" class="none-row">Nada encontrado para esse filtro.</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
      <footer class="foot">
        <b>{history.entries.length} de {HISTORY_LIMIT}</b> · os mais antigos saem quando chega a {HISTORY_LIMIT}
      </footer>
    {/if}
  </section>
</div>

<style>
  .hist { height: 100%; display: flex; flex-direction: column; gap: 14px; padding: 18px 20px; min-width: 0; }
  .top { display: flex; align-items: flex-end; gap: 16px; flex-wrap: wrap; }
  .top > div:first-child { flex: 1; min-width: 280px; }
  h1 { font-size: 20px; font-weight: 750; letter-spacing: -0.01em; }
  .top p { margin-top: 4px; color: var(--ms-text-2); }
  .controls { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  .filter {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 220px;
    height: 32px;
    padding: 0 10px;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-0);
    border: 1px solid var(--ms-border-strong);
    color: var(--ms-text-3);
  }
  .filter:focus-within { border-color: var(--ms-accent); box-shadow: 0 0 0 3px var(--ms-accent-soft); }
  .filter input { flex: 1; min-width: 0; background: none; border: 0; outline: none; color: var(--ms-text); }
  .confirm { padding: 14px; display: flex; flex-direction: column; gap: 6px; }
  .confirm p { font-size: 12px; color: var(--ms-text-2); }
  .btns { display: flex; justify-content: flex-end; gap: 6px; margin-top: 6px; }
  .table { flex: 1; min-height: 0; display: flex; flex-direction: column; overflow: hidden; }
  .wrap { flex: 1; min-height: 0; }
  table { width: 100%; min-width: 900px; border-collapse: collapse; font-size: 12.5px; }
  th {
    position: sticky;
    top: 0;
    z-index: 1;
    height: 36px;
    padding: 0 12px;
    text-align: left;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ms-text-3);
    background: var(--ms-surface-1);
    border-bottom: 1px solid var(--ms-border);
  }
  td { height: 48px; padding: 0 12px; border-bottom: 1px solid var(--ms-border); }
  tbody tr { animation: ms-row-up 450ms var(--ms-spring) both; animation-delay: calc(var(--i) * 25ms); }
  tbody tr:hover td { background: var(--ms-surface-2); }
  .r { text-align: right; }
  .item { max-width: 300px; }
  .itemcell { display: flex; align-items: center; gap: 10px; min-width: 0; }
  .art {
    flex: none;
    width: 30px;
    height: 38px;
    border-radius: var(--ms-r-sm, 6px);
    background: var(--ms-surface-2);
    border: 1px solid color-mix(in srgb, var(--rar) 35%, var(--ms-border));
    overflow: hidden;
  }
  .txt { min-width: 0; }
  .nm { display: block; font-weight: 700; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bt { display: block; font-size: 11.5px; color: var(--ms-text-3); }
  .game { font: 700 10.5px var(--ms-mono); padding: 2px 7px; border-radius: 99px; background: var(--ms-surface-3); color: var(--ms-text-2); }
  .muted { color: var(--ms-text-3); }
  .league { max-width: 160px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .price :global(.value) { justify-content: flex-end; }
  .price :global(b) { font: 700 12.5px var(--ms-mono); color: var(--ms-text); }
  .price.fair :global(b) { color: var(--ms-suggest); }
  .price .none { color: var(--ms-text-3); }
  .when { white-space: nowrap; }
  .acts { white-space: nowrap; text-align: right; }
  .acts > * { display: inline-grid; vertical-align: middle; }
  .del:hover { color: var(--ms-danger); background: var(--ms-danger-soft); }
  .none-row { text-align: center; color: var(--ms-text-3); }
  .foot { padding: 10px 14px; border-top: 1px solid var(--ms-border); background: var(--ms-surface-0); font-size: 12px; color: var(--ms-text-3); }
  .foot b { color: var(--ms-text-2); }
  .empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; padding: 40px; color: var(--ms-text-3); }
  .empty b { font-size: 15px; color: var(--ms-text); }
  .ill { width: 56px; height: 56px; display: grid; place-items: center; border-radius: 16px; background: var(--ms-surface-2); color: var(--ms-text-2); }
</style>
