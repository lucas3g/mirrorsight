<script lang="ts">
  import { RARITY_COLOR, formatAmount } from '$lib/format';
  import { divineRateOf, itemIconOf } from '$lib/pricing';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import ItemArt from '$lib/ui/ItemArt.svelte';
  import Sparkline from '$lib/ui/Sparkline.svelte';
  import ChaosValue from '$lib/features/shared/ChaosValue.svelte';

  /** Card compacto com o preço de referência (poe.ninja). Some sozinho; pausa no hover. */
  const LIFETIME_MS = 6000;

  const output = $derived(priceCheck.output);
  const indexed = $derived(output?.indexed ?? null);
  const change = $derived(indexed?.change_24h_percent ?? null);
  const loading = $derived(priceCheck.phase.kind === 'loading');

  let paused = $state(false);
  /** Tempo restante: o hover pausa (não reinicia) a contagem. */
  let remaining = LIFETIME_MS;
  /** Resultado a que `remaining` se refere: um item novo recomeça a contagem. */
  let countingFor: typeof output = null;

  $effect(() => {
    // Roda depois do teardown da execução anterior, que desconta o tempo do item antigo.
    if (output !== countingFor) {
      countingFor = output;
      remaining = LIFETIME_MS;
    }
    if (paused || loading || !output) return;
    const started = Date.now();
    const timer = setTimeout(() => {
      if (!priceCheck.pinned) priceCheck.close();
    }, remaining);
    return () => {
      clearTimeout(timer);
      remaining = Math.max(0, remaining - (Date.now() - started));
    };
  });
</script>

<div
  class="quick enter-up"
  role="dialog"
  aria-label="Quick price"
  tabindex="-1"
  onmouseenter={() => (paused = true)}
  onmouseleave={() => (paused = false)}
>
  {#if loading || !output}
    <div class="top">
      <div class="sk" style="width:36px;height:36px;border-radius:9px"></div>
      <div style="flex:1"><div class="sk" style="height:12px;width:60%"></div><div class="sk" style="height:22px;width:45%;margin-top:6px"></div></div>
    </div>
  {:else if indexed}
    <div class="top">
      <div class="icon" class:tall={output.item.rarity !== 'currency'}>
        <ItemArt item={output.item} icon={itemIconOf(output)} size="sm" />
      </div>
      <div class="info">
        <div class="name" style:--name-color={RARITY_COLOR[output.item.rarity]}>{output.item.name ?? output.item.base_type}</div>
        <div class="price"><ChaosValue chaos={indexed.chaos_value} divineRate={divineRateOf(output)} size={22} itemName={output.item.base_type} /></div>
      </div>
      {#if change !== null}
        <span class="chg" class:up={change >= 0} class:down={change < 0}>{change >= 0 ? '+' : ''}{formatAmount(change)}%</span>
      {/if}
    </div>
    <div class="spark">
      <Sparkline values={indexed.sparkline} width={290} height={44} color={change !== null && change < 0 ? 'var(--ms-danger)' : 'var(--ms-success)'} />
    </div>
    <div class="bot"><span>poe.ninja · 7 dias</span><button class="more" onclick={() => (priceCheck.mode = 'full')}>Ver listagens →</button></div>
    <div class="timer"><i class:paused style:animation-duration="{LIFETIME_MS}ms"></i></div>
  {:else}
    <div class="empty">
      <p>Sem preço de referência no poe.ninja para este item.</p>
      <button class="btn btn-primary btn-sm" onclick={() => (priceCheck.mode = 'full')}>Abrir price check completo</button>
    </div>
  {/if}
</div>

<style>
  .quick {
    position: relative;
    width: 320px;
    padding: 14px;
    overflow: hidden;
    border-radius: var(--ms-r-lg);
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border-strong);
    box-shadow: 0 30px 60px -20px #000;
  }
  .top { display: flex; align-items: center; gap: 12px; }
  .icon {
    width: 36px;
    height: 36px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-3);
    overflow: hidden;
  }
  /* Itens que não são currency: proporção de inventário, espaço para imagem ou nome. */
  .icon.tall { width: 40px; height: 52px; }
  .info { flex: 1; min-width: 0; }
  .name { font-weight: 700; color: var(--name-color, var(--ms-currency)); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .price :global(b) { font: 800 24px/1.1 var(--ms-mono); color: var(--ms-price); }
  .spark { margin-top: 8px; }
  .spark :global(svg) { width: 100%; }
  .bot { display: flex; align-items: center; justify-content: space-between; margin-top: 4px; font-size: 11.5px; color: var(--ms-text-3); }
  .more { font-weight: 650; color: var(--ms-accent); }
  .more:hover { color: var(--ms-accent-2); }
  .timer { position: absolute; left: 0; right: 0; bottom: 0; height: 3px; }
  .timer i { display: block; height: 100%; background: var(--ms-accent); transform-origin: left; animation: shrink linear forwards; }
  .timer i.paused { animation-play-state: paused; }
  @keyframes shrink { to { transform: scaleX(0); } }
  .empty { display: flex; flex-direction: column; gap: 10px; align-items: flex-start; font-size: 12px; color: var(--ms-text-2); }
</style>
