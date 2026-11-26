<script lang="ts">
  import type { CompareDetail, GroupKind, Item, Listing, NeedChange, StatDelta } from '$lib/types';
  import { INFLUENCE_LABEL, KIND_LABEL, RARITY_COLOR, formatAmount, relativeAge } from '$lib/format';
  import Sockets from '$lib/ui/Sockets.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import ModText from '$lib/features/shared/ModText.svelte';
  import { portal } from '$lib/ui/portal';
  import { common } from '$lib/i18n/messages/common';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import { evalMsgs as em } from '$lib/i18n/messages/evaluation';
  import GroupTag from './GroupTag.svelte';
  import QualityChip from './QualityChip.svelte';
  import ModEvaluationList from './ModEvaluationList.svelte';
  import { GROUP_COLOR } from './groups';
  import { alignTo } from './modOrder';

  /**
   * Detalhe do item de uma listagem, exibido ao passar o mouse no olho.
   * Na listagem agrupada (PoE1) recebe o grupo e a avaliação do anúncio: o bloco Upgrade
   * (para a build) e o bloco Qualidade (independente da build) — um anúncio pode ter rolls
   * excelentes e ainda assim ser PIOR para você.
   */
  let {
    listing,
    item,
    matchedIds,
    group,
    detail,
    yourIds,
    yourOrder = [],
    buildName,
    left,
    top,
  }: {
    listing: Listing;
    item: Item;
    matchedIds: Set<string>;
    group?: GroupKind;
    detail?: CompareDetail;
    /** trade_ids dos mods do seu item (destacam os mods em comum). */
    yourIds?: Set<string>;
    /** trade_ids do seu item na ordem do editor: os mods do anúncio seguem essa ordem. */
    yourOrder?: string[];
    /** Build usado na relevância; `null` = pesos genéricos. */
    buildName?: string | null;
    left: number;
    top: number;
  } = $props();

  const grouped = $derived(group !== undefined);
  const hitIds = $derived(grouped && yourIds ? yourIds : matchedIds);
  const hasMatches = $derived(item.modifiers.some((m) => hitIds.has(m.trade_id)));
  const listedLine = $derived.by(() => {
    const age = relativeAge(listing.indexed_at);
    return age === common.now ? pm.listedNow : pm.listedAgo(age);
  });

  /** Fração → "+18%" / "−24%" / "0%". */
  const percent = (v: number) => {
    const n = Math.round(Math.abs(v) * 100);
    return n === 0 ? '0%' : `${v > 0 ? '+' : '−'}${n}%`;
  };
  /** Texto do stat sem os números ("+#% to Fire Resistance" → "Fire Resistance"). */
  const statLabel = (d: StatDelta) =>
    d.text
      .replace(/[+-]?(#|\d+(?:\.\d+)?)%?/g, '')
      .replace(/\s+/g, ' ')
      .trim()
      .replace(/:$/, '')
      .replace(/^(to|increased|reduced) /i, '') || d.key;
  const statValue = (v: number, d: StatDelta) => (v === 0 ? '0' : `${formatAmount(v)}${/#%|\d%/.test(d.text) ? '%' : ''}`);

  function needText(n: NeedChange): string {
    const element = (em.elements as Record<string, string>)[n.stat];
    const attr = (em.attributes as Record<string, string>)[n.stat];
    if (element) return n.short_after > 0 ? em.resShort(element, Math.round(n.short_after)) : em.resFixed(element);
    const name = attr ?? n.stat;
    return n.short_after > 0 ? em.attrShort(name, Math.round(n.short_after)) : em.attrFixed(name);
  }

  /** Linhas do "O que muda": ganhos e perdas juntos, do maior impacto ao menor (até 5). */
  const changes = $derived.by(() => {
    if (!detail) return [];
    const up = detail.upgrade;
    return [...up.gains, ...up.losses]
      .sort((a, b) => Math.abs(b.contribution) - Math.abs(a.contribution))
      .slice(0, 5)
      .map((d) => ({ d, share: d.contribution / up.reference }));
  });
  /** Maior |%| das linhas: a barra de cada linha é proporcional a ele. */
  const maxShare = $derived(Math.max(1e-9, ...changes.map((c) => Math.abs(c.share))));
  /** Régua: ±50% enche cada metade. */
  const scaleFill = $derived(detail ? Math.min(Math.abs(detail.upgrade.relative), 0.5) * 100 : 0);
  const verdictText = $derived.by(() => {
    const up = detail?.upgrade;
    if (!up) return '';
    if (up.verdict === 'better') return em.betterThan;
    if (up.verdict === 'worse') return em.worseThan;
    return changes.length === 0 ? em.sameRolls : em.equivalentText;
  });

  const quality = $derived(detail?.quality ?? null);
  /** Mods do anúncio na ordem dos seus (os que você não tem vão para o fim). */
  const listedMods = $derived(alignTo(item.modifiers, yourOrder));
  const evaluatedMods = $derived(quality ? alignTo(quality.mods, yourOrder) : []);
  const crafting = $derived(quality?.crafting ?? null);
  const unique = $derived(quality?.unique ?? null);
</script>

<div class="card" use:portal style:left="{left}px" style:top="{top}px" style:--rar={RARITY_COLOR[item.rarity]} role="tooltip">
  <header>
    <div class="nline">
      <div class="name">{item.name ?? item.base_type}</div>
      {#if group}<GroupTag kind={group} />{/if}
    </div>
    <div class="base">
      {item.name ? `${item.base_type} · ` : ''}{item.item_class}{item.item_level ? ` · iLvl ${item.item_level}` : ''}
    </div>
    <div class="chips">
      {#each item.influences as influence (influence)}<span class="chip inf">{INFLUENCE_LABEL[influence]}</span>{/each}
      {#if item.modifiers.some((m) => m.kind === 'fractured')}<span class="chip frac">Fractured</span>{/if}
      {#if item.corrupted}<span class="chip corr">Corrupted</span>{/if}
      {#if item.sockets}<Sockets sockets={item.sockets} />{/if}
      {#if item.quality}<span class="chip mono">Q {item.quality}%</span>{/if}
      {#if listing.at_max_quality}<span class="chip maxq" title={pm.atMaxQualityTitle}>{pm.atMaxQuality}</span>{/if}
    </div>
  </header>

  {#if detail && group}
    {@const up = detail.upgrade}
    <section class="up {up.verdict}" style:--grp={GROUP_COLOR[group]}>
      <div class="sh">
        <span class="for">{em.buildFor(buildName ?? null)}</span>
        <span class="r conf {up.confidence}"><i></i><i></i><i></i>{em.confidenceShort[up.confidence]}</span>
      </div>
      <div class="verdict"><span class="big mono">{percent(up.relative)}</span><span class="vtxt">{verdictText}</span></div>
      <div class="scale" aria-hidden="true">
        {#if up.verdict !== 'equivalent'}<span class="fill" style:width="{scaleFill}%"></span>{/if}
        <span class="mid"></span>
        <span class="dot" style:left="{up.verdict === 'better' ? 50 + scaleFill : up.verdict === 'worse' ? 50 - scaleFill : 50}%"></span>
      </div>
      <div class="ends"><span>{em.scale.worse}</span><span>{em.scale.yours}</span><span>{em.scale.better}</span></div>

      {#if changes.length > 0}
        <div class="wh"><span></span><span>{em.colChange}</span><span>{em.colYoursTheirs}</span><span>{em.colValue}</span></div>
        {#each changes as { d, share } (d.key)}
          <div class="ch" class:upc={share > 0} class:downc={share < 0}>
            <span class="ic">{share > 0 ? '↑' : '↓'}</span>
            <span class="st">{statLabel(d)}</span>
            <span class="ft mono">{statValue(d.yours, d)} → <b>{statValue(d.theirs, d)}</b></span>
            <span class="pc mono">{percent(share)}</span>
            <span class="w"><i style:width="{(Math.abs(share) / maxShare) * 100}%"></i></span>
          </div>
        {/each}
      {:else if up.verdict === 'equivalent'}
        <p class="muted">{em.sameNote}</p>
      {:else}
        <p class="muted">{em.noDifferences}</p>
      {/if}
      {#each up.needs as n (n.stat)}
        <div class="need" class:fix={n.contribution > 0}>
          <span aria-hidden="true">{n.contribution > 0 ? '✓' : '⚠'}</span>
          <span class="nt">{needText(n)}</span>
          <b class="mono">{percent(n.contribution / up.reference)}</b>
        </div>
      {/each}
      {#if changes.length > 0 || up.needs.length > 0}<p class="how">{em.how}</p>{/if}
    </section>

    {#if quality}
      <section>
        <div class="sh">
          <span>{em.qualityTitle}</span>
          <span class="r conf {quality.confidence}"><i></i><i></i><i></i>{em.confidence[quality.confidence]}</span>
        </div>
        <div class="qhead">
          <span class="score mono">{quality.score === null ? '—' : Math.round(quality.score)}<small>/100</small></span>
          <QualityChip {quality} />
        </div>
        {#if unique}
          <p class="uinfo">
            {#if unique.found}
              {unique.version ? em.uniqueVersion(unique.version, unique.legacy) : em.uniqueCurrent}
              {#if unique.variant} · {em.uniqueVariant(unique.variant)}{/if}
              · {em.uniqueRolls(unique.variable_mods, unique.fixed_mods)}
            {:else}
              {em.reason.unique_not_in_database}
            {/if}
            {#if quality.sockets && quality.sockets.weight > 0} · {em.links(quality.sockets.links, quality.sockets.max)}{/if}
          </p>
        {/if}
        <ModEvaluationList mods={evaluatedMods} {hitIds} />
        {#if crafting}
          <div class="craft">
            {#each crafting.blocked as b (b)}<span class="chip no">{em.blocked[b]}</span>{/each}
            {#if crafting.blocked.length === 0}
              {#if crafting.open_suffixes !== null}<span class="chip" class:ok={crafting.valued === 'suffix' && crafting.open_suffixes > 0}>{em.openSuffixes(crafting.open_suffixes)}</span>{/if}
              {#if crafting.open_prefixes !== null}<span class="chip" class:ok={crafting.valued === 'prefix' && crafting.open_prefixes > 0}>{em.openPrefixes(crafting.open_prefixes)}</span>{/if}
              {#if crafting.crafted > 0}<span class="chip ok">{em.craftedReplaceable(crafting.crafted)}</span>{/if}
              {#if crafting.fractured_high_tier > 0}<span class="chip ok">{em.fracturedHigh}</span>{/if}
              {#if crafting.ilvl_allows_top_tier !== null}
                <span class="chip" class:ok={crafting.ilvl_allows_top_tier}>{crafting.ilvl_allows_top_tier ? em.ilvlTop : em.ilvlLow}</span>
              {/if}
            {/if}
          </div>
        {/if}
      </section>
    {/if}
  {:else if item.modifiers.length > 0}
    <section>
      {#each listedMods as mod, i (i)}
        <div class="mod" class:match={hitIds.has(mod.trade_id)}>
          <span class="tag t-{mod.kind}">{KIND_LABEL[mod.kind].slice(0, 4)}</span>
          {#if mod.affix}
            <span class="affix mono" title={pm.affixTitle[mod.affix]}>{mod.affix === 'prefix' ? 'P' : 'S'}</span>
          {:else}
            <span class="affix none"></span>
          {/if}
          <span class="txt">{#if hitIds.has(mod.trade_id)}<span class="hit"></span>{/if}<ModText text={mod.text} /></span>
          {#if mod.tier !== null}<span class="tier" class:t1={mod.tier === 1} class:t2={mod.tier === 2}>T{mod.tier}</span>{:else}<span></span>{/if}
        </div>
      {/each}
      {#if hasMatches}<p class="legend"><span class="hit"></span>{grouped ? pm.inCommon : pm.matchesFilters}</p>{/if}
    </section>
  {/if}

  <footer>
    <span class="p mono"><CurrencyIcon currency={listing.price.currency} size={18} />{formatAmount(listing.price.amount)}</span>
    <span class="s">
      <span class="dot" class:on={listing.online} class:afk={!listing.online}></span>
      {listing.account}<br />{listedLine}
    </span>
  </footer>
</div>

<style>
  .card {
    position: fixed;
    z-index: 60;
    width: 320px;
    border-radius: 12px;
    overflow: hidden;
    pointer-events: none;
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border-strong);
    box-shadow: 0 30px 60px -20px #000, 0 0 0 1px #000;
    transform-origin: right center;
    animation: peek-in 350ms var(--ms-spring) both;
  }
  .card::before { content: ''; position: absolute; inset: 0 0 auto; height: 3px; background: var(--rar); }
  @keyframes peek-in { from { transform: scale(0.92) translateX(10px); } }
  header { padding: 10px 12px 8px; border-bottom: 1px solid var(--ms-border); }
  .nline { display: flex; align-items: center; gap: 8px; }
  .name { flex: 1; min-width: 0; font-size: 14px; font-weight: 700; color: var(--rar); }
  .base { font-size: 11.5px; color: var(--ms-text-2); margin: 0 0 5px; }
  .chips { display: flex; flex-wrap: wrap; gap: 4px; }
  section { padding: 6px 12px; border-bottom: 1px solid var(--ms-border); }

  .sh {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 3px;
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ms-text-3);
  }
  .sh .r { margin-left: auto; text-transform: none; letter-spacing: 0; font-weight: 600; }
  .conf { display: inline-flex; align-items: center; gap: 3px; }
  .conf i { display: inline-block; width: 6px; height: 6px; border-radius: 2px; background: var(--ms-surface-4); }
  .conf i:last-of-type { margin-right: 3px; }
  .conf.high i { background: var(--ms-success); }
  .conf.medium i:nth-child(-n + 2) { background: var(--ms-warning); }
  .conf.low i:first-child { background: var(--ms-danger); }

  .sh .for { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--ms-suggest); text-transform: none; letter-spacing: 0; font-weight: 700; }
  .verdict { display: flex; align-items: baseline; gap: 6px; margin: 4px 0 2px; }
  .big { font-size: 19px; font-weight: 800; line-height: 1; color: var(--grp); }
  .vtxt { font-size: 11.5px; color: var(--ms-text-2); }
  /* Régua divergente: o centro é o seu item. */
  .scale { position: relative; height: 7px; margin: 7px 0 3px; border-radius: 7px; background: var(--ms-surface-4); }
  .scale .mid { position: absolute; left: 50%; top: -3px; bottom: -3px; width: 2px; margin-left: -1px; border-radius: 2px; background: var(--ms-text-2); }
  .scale .fill { position: absolute; top: 0; bottom: 0; border-radius: 7px; background: var(--grp); animation: ms-grow-x 500ms var(--ms-spring) both; }
  .worse .scale .fill { right: 50%; transform-origin: right; }
  .better .scale .fill { left: 50%; transform-origin: left; }
  .scale .dot { position: absolute; top: 50%; width: 11px; height: 11px; margin: -5.5px 0 0 -5.5px; border-radius: 50%; background: var(--ms-text); box-shadow: 0 0 0 3px var(--ms-surface-2); }
  .ends { display: flex; justify-content: space-between; margin-bottom: 5px; font-size: 9.5px; color: var(--ms-text-3); }
  .ends span:nth-child(2) { color: var(--ms-text-2); font-weight: 700; }
  .wh, .ch { display: grid; grid-template-columns: 12px minmax(0, 1fr) auto 36px; gap: 6px; align-items: center; }
  .wh { margin: 2px 0 1px; font-size: 9.5px; font-weight: 700; letter-spacing: 0.05em; text-transform: uppercase; color: var(--ms-text-3); }
  .wh span:nth-child(4) { text-align: right; }
  .ch { min-height: 17px; font-size: 11.5px; }
  .ch .ic { font-size: 11px; font-weight: 900; text-align: center; }
  .ch.upc .ic, .ch.upc .pc { color: var(--ms-info); }
  .ch.downc .ic, .ch.downc .pc { color: var(--ms-warning); }
  .ch .st { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--ms-text-2); }
  .ch .ft { white-space: nowrap; font-size: 11px; color: var(--ms-text-3); }
  .ch .ft b { color: var(--ms-text); font-weight: 700; }
  .ch .pc { text-align: right; font-weight: 800; }
  .ch .w { grid-column: 2 / 5; height: 3px; margin: -1px 0 2px; border-radius: 3px; background: var(--ms-surface-3); overflow: hidden; }
  .ch .w i { display: block; height: 100%; border-radius: 3px; }
  .ch.upc .w i { background: var(--ms-info); }
  .ch.downc .w i { background: var(--ms-warning); }
  .muted { font-size: 11.5px; color: var(--ms-text-2); }
  .how { margin-top: 5px; font-size: 9.5px; line-height: 1.35; color: var(--ms-text-3); }
  .need {
    display: flex;
    gap: 7px;
    align-items: flex-start;
    margin-top: 7px;
    padding: 6px 9px;
    border-radius: 7px;
    font-size: 11.5px;
    line-height: 1.35;
    color: var(--ms-danger);
    background: var(--ms-danger-soft);
    border: 1px solid var(--ms-danger-line);
  }
  .need .nt { flex: 1; }
  .need b { font-weight: 800; }
  .need.fix { color: var(--ms-success); background: var(--ms-success-soft); border-color: var(--ms-success-soft); }

  .qhead { display: flex; align-items: center; gap: 8px; margin-bottom: 3px; }
  .score { font-size: 17px; font-weight: 800; line-height: 1; color: var(--ms-text); }
  .score small { font-size: 11px; font-weight: 600; color: var(--ms-text-3); }
  .uinfo { margin-bottom: 2px; font-size: 10.5px; color: var(--ms-text-3); }
  .craft { display: flex; flex-wrap: wrap; gap: 3px; margin-top: 5px; }
  .chip.ok { color: var(--ms-success); }
  .chip.maxq { color: var(--ms-info); }
  .chip.no { color: var(--ms-danger); }

  .mod {
    display: grid;
    grid-template-columns: auto auto 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 3px 0;
    font-size: 12.5px;
    line-height: 1.3;
    color: var(--ms-text-2);
  }
  .mod.match .txt { color: var(--ms-text); }
  .tier { height: 18px; min-width: 24px; font-size: 9.5px; }
  .hit {
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-right: 6px;
    vertical-align: 1px;
    border-radius: 2px;
    background: var(--ms-accent);
  }
  .legend { font-size: 10.5px; color: var(--ms-text-3); margin-top: 6px; }
  .affix {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 4px;
    border: 1px solid var(--ms-border-strong);
    font-size: 9px;
    font-weight: 800;
    color: var(--ms-text-2);
  }
  .affix.none { border-color: transparent; }
  footer { display: flex; align-items: center; gap: 10px; padding: 7px 12px; background: var(--ms-surface-1); }
  .mod .txt { --num-color: var(--ms-text-2); }
  .mod.match .txt { --num-color: var(--ms-accent-2); }
  .p { display: flex; align-items: center; gap: 6px; font-size: 15px; font-weight: 700; color: var(--ms-price); }
  .s { flex: 1; text-align: right; font-size: 11px; color: var(--ms-text-2); line-height: 1.4; }
  .s .dot { margin-right: 5px; }
</style>
