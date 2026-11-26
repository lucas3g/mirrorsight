<script lang="ts">
  import type { StatFilter } from '$lib/types';
  import { KIND_COLOR, rollPercent, rollRange } from '$lib/format';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import ModText from '$lib/features/shared/ModText.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Toggle from '$lib/ui/Toggle.svelte';
  import { modsMsgs as mm } from '$lib/i18n/messages/mods';

  /**
   * Linha de mod: alça de arraste, switch, texto com barra de roll, tier e mín/máx.
   * `compact` (overlay): sem barra de roll e sem coluna de tier; mín e máx menores.
   */
  let {
    stat,
    index,
    order,
    compact = false,
    dragging = false,
    dragOffset = 0,
    ongrip,
    onmove,
  }: {
    stat: StatFilter;
    index: number;
    /** Posição para a animação de entrada. */
    order: number;
    compact?: boolean;
    dragging?: boolean;
    /** Deslocamento vertical (px) da linha arrastada. */
    dragOffset?: number;
    /** Início do arraste pela alça. */
    ongrip: (e: PointerEvent, index: number) => void;
    /** Alt+↑/↓ na alça: −1 sobe, +1 desce. */
    onmove: (index: number, delta: -1 | 1) => void;
  } = $props();

  const percent = $derived(rollPercent(stat.roll));
  const range = $derived(rollRange(stat.roll));
  const added = $derived(priceCheck.isAdded(index));
  const tinted = $derived(stat.kind === 'crafted' || stat.kind === 'pseudo');

  /**
   * Grava mín/máx. Dar um valor liga o mod: só mods ligados entram na busca, e um
   * mín/máx num mod desligado seria ignorado sem aviso. Apagar o valor não desliga.
   */
  function setBound(field: 'min' | 'max', value: number | null) {
    priceCheck.setStat(index, value === null ? { [field]: null } : { [field]: value, enabled: true });
  }

  function parse(raw: string): number | null {
    const n = Number.parseFloat(raw.replace(',', '.'));
    return Number.isFinite(n) ? n : null;
  }

  /**
   * Roda do mouse no input focado: para cima aumenta, para baixo diminui.
   * Passo 1 (0.1 se o valor for decimal); Shift = 10; Ctrl = 0.1. Sem foco, a roda rola a lista normalmente.
   */
  function wheelStep(node: HTMLInputElement, field: 'min' | 'max') {
    const onWheel = (e: WheelEvent) => {
      if (document.activeElement !== node || e.deltaY === 0) return;
      e.preventDefault();
      const base = parse(node.value) ?? stat.roll?.value ?? 0;
      const step = e.shiftKey ? 10 : e.ctrlKey || !Number.isInteger(base) ? 0.1 : 1;
      const next = Math.round((base + (e.deltaY < 0 ? step : -step)) * 100) / 100;
      node.value = String(next);
      setBound(field, next);
    };
    node.addEventListener('wheel', onWheel, { passive: false });
    return { destroy: () => node.removeEventListener('wheel', onWheel) };
  }

  /** Digitação: grava a cada tecla (a busca automática espera a pausa). Texto incompleto ("-", "1,") é ignorado. */
  function onType(e: Event & { currentTarget: HTMLInputElement }, field: 'min' | 'max') {
    const raw = e.currentTarget.value.trim();
    const n = parse(raw);
    if (raw !== '' && n === null) return;
    setBound(field, n);
  }

  /** Teto do tier ("+138(130-144)" → 144); sem a faixa (cópia sem Ctrl+Alt+C, atributos), o valor rolado. */
  const tierMax = $derived(stat.roll ? (stat.roll.max ?? stat.roll.value) : null);

  /**
   * Como no Awakened PoE Trade: o máx vem vazio e, ao clicar nele, recebe o teto do tier.
   * O texto fica selecionado, então digitar substitui o valor e a roda ajusta a partir dele.
   */
  function fillMax(e: MouseEvent & { currentTarget: HTMLInputElement }) {
    const input = e.currentTarget;
    if (stat.max !== null || tierMax === null) return;
    input.value = String(tierMax);
    setBound('max', tierMax);
    // Depois do mouseup do clique, que senão desfaria a seleção.
    setTimeout(() => input.select());
  }

  function onGripKey(e: KeyboardEvent) {
    if (!e.altKey || (e.key !== 'ArrowUp' && e.key !== 'ArrowDown')) return;
    e.preventDefault();
    onmove(index, e.key === 'ArrowUp' ? -1 : 1);
  }
</script>

<div
  class="mod"
  class:off={!stat.enabled}
  class:compact
  class:dragging
  class:tinted
  data-stat={index}
  data-kind={stat.kind}
  style:--i={order}
  style:--kind={KIND_COLOR[stat.kind]}
  style:transform={dragging ? `translateY(${dragOffset}px) rotate(-1deg)` : undefined}
>
  <button
    class="grip"
    aria-label={mm.reorderAria}
    title={mm.reorderTitle}
    onpointerdown={(e) => ongrip(e, index)}
    onkeydown={onGripKey}
  >
    <Icon name="grip" size={compact ? 12 : 14} />
  </button>

  <Toggle label={stat.enabled ? mm.disableMod : mm.enableMod} checked={stat.enabled} onchange={(enabled) => priceCheck.setStat(index, { enabled })} />

  <div class="body">
    <div class="line">
      <span class="text"><ModText text={stat.text} /></span>
      {#if added}
        <span class="added">{mm.added}</span>
        <button class="rm" title={mm.removeAdded} aria-label={mm.removeAdded} onclick={() => priceCheck.removeStat(index)}>
          <Icon name="x" size={12} />
        </button>
      {/if}
    </div>
    {#if !compact && (percent !== null || range)}
      <div class="roll">
        <span class="track" title={percent !== null ? mm.rollPercentile(Math.round(percent)) : undefined}>
          <i style:transform="scaleX({(percent ?? 0) / 100})"></i>
        </span>
        <span class="range mono">{stat.tier !== null ? `T${stat.tier} · ` : ''}{range ?? ''}</span>
      </div>
    {/if}
  </div>

  {#if !compact}
    <span class="tier" class:t1={stat.tier === 1} class:t2={stat.tier === 2} class:none={stat.tier === null} title={mm.tierTitle}>
      {stat.tier !== null ? `T${stat.tier}` : '—'}
    </span>
  {/if}

  <input
    class="num min"
    inputmode="decimal"
    placeholder={mm.minPlaceholder}
    aria-label={mm.minAria}
    title={mm.wheelHint}
    use:wheelStep={'min'}
    value={stat.min ?? ''}
    oninput={(e) => onType(e, 'min')}
  />
  <input
    class="num max"
    inputmode="decimal"
    placeholder={mm.maxPlaceholder}
    aria-label={mm.maxAria}
    title={tierMax !== null
      ? mm.maxTierHint(tierMax)
      : mm.wheelHint}
    use:wheelStep={'max'}
    value={stat.max ?? ''}
    onclick={fillMax}
    oninput={(e) => onType(e, 'max')}
  />
</div>

<style>
  .mod {
    position: relative;
    display: grid;
    grid-template-columns: 14px 32px minmax(0, 1fr) auto 58px 58px;
    align-items: start;
    gap: 10px;
    padding: 8px;
    border-radius: var(--ms-r-md);
    border: 1px solid transparent;
    background: var(--ms-surface-1);
    transition: background var(--ms-t-fast), border-color var(--ms-t-fast);
    animation: ms-row-in 450ms var(--ms-spring) both;
    animation-delay: calc(var(--i, 0) * 30ms);
  }
  .mod.compact { grid-template-columns: 10px 32px minmax(0, 1fr) 46px 46px; gap: 6px; padding: 6px; }
  .mod:hover { background: var(--ms-surface-2); border-color: var(--ms-border); }
  .mod.dragging {
    z-index: 5;
    background: var(--ms-surface-3);
    border-color: var(--ms-accent);
    box-shadow: 0 18px 34px -14px #000;
    animation: none;
    transition: none;
  }
  .grip {
    height: 28px;
    display: grid;
    place-items: center;
    color: var(--ms-text-3);
    cursor: grab;
    touch-action: none;
    border-radius: 4px;
  }
  .grip:hover { color: var(--ms-text-2); }
  .dragging .grip { cursor: grabbing; }
  .mod > :global(.toggle) { margin-top: 4.5px; }
  .compact > :global(.toggle) { margin-top: 2px; }
  .body { min-width: 0; padding-top: 4px; }
  .compact .body { padding-top: 2px; }
  .line { display: flex; align-items: flex-start; gap: 6px; min-width: 0; }
  .text { flex: 1; min-width: 0; font-size: 13px; line-height: 1.35; color: var(--ms-text); }
  .compact .text { font-size: 12.5px; }
  .tinted .text { color: var(--kind); }
  .off .text { color: var(--ms-text-3); --num-color: var(--ms-text-2); }
  .added {
    flex: none;
    height: 18px;
    padding: 0 6px;
    display: inline-flex;
    align-items: center;
    border-radius: 5px;
    font-size: 9.5px;
    font-weight: 800;
    letter-spacing: 0.05em;
    color: var(--ms-suggest);
    background: var(--ms-suggest-soft);
  }
  .rm { flex: none; width: 18px; height: 18px; display: grid; place-items: center; border-radius: 5px; color: var(--ms-text-3); }
  .rm:hover { color: var(--ms-danger); background: var(--ms-danger-soft); }
  .roll { display: flex; align-items: center; gap: 8px; margin-top: 6px; }
  .track { width: 160px; max-width: 45%; height: 3px; border-radius: 3px; background: var(--ms-surface-3); overflow: hidden; flex: none; }
  .track i {
    display: block;
    height: 100%;
    background: var(--ms-accent);
    transform-origin: left;
    transition: transform 400ms var(--ms-spring);
    animation: ms-grow-x 600ms var(--ms-spring) both;
    animation-delay: calc(var(--i, 0) * 30ms + 120ms);
  }
  .off .track i { background: var(--ms-surface-4); }
  .range { font-size: 10.5px; color: var(--ms-text-3); white-space: nowrap; }
  .tier { margin-top: 4px; }
  .compact .num { width: 46px; }
  .num:focus { cursor: ns-resize; }
</style>
