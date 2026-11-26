/**
 * Dados fictícios para o adapter de demonstração. Nada aqui é usado no app real.
 */
import type {
  Affix,
  BuildDetails,
  BuildFile,
  BuildSummary,
  CatalogStat,
  CompareDetail,
  CraftingPotential,
  GroupEntry,
  GroupKind,
  Item,
  ItemQuality,
  Listing,
  ListingComparison,
  ModEvaluation,
  Modifier,
  ModifierKind,
  NeedChange,
  PriceCheckOutput,
  SearchFilters,
  StatDelta,
  SuggestedPrice,
  UpgradeVerdict,
} from '$lib/types';

export const SAMPLE_RARE_TEXT = `Item Class: Gloves
Rarity: Rare
Doom Grip
Sorcerer Gloves
--------
Quality: +20%
Energy Shield: 112
--------
Item Level: 86
--------
+78 to maximum Life (fractured)
+45% to Fire Resistance
+41% to Cold Resistance
Adds 12 to 21 Physical Damage to Attacks
+38 to Dexterity
+16% to Chaos Resistance (crafted)`;

export const SAMPLE_CURRENCY_TEXT = `Item Class: Stackable Currency
Rarity: Currency
Divine Orb
--------
Stack Size: 7/10
--------
Randomises the numeric values of the random modifiers on an item`;

/** Marcadores aceitos pelo adapter demo para simular falhas. */
export const DEMO_RATE_LIMIT = '#mirrorsight:rate-limit';
export const DEMO_NETWORK = '#mirrorsight:network';
/** Poucas listagens (3): exercita o estado "Poucos anúncios" do preço sugerido. */
export const DEMO_FEW = '#mirrorsight:few';

function mod(
  kind: ModifierKind,
  id: string,
  text: string,
  value: number,
  min: number,
  max: number,
  tier: number | null = null,
  affix: Affix | null = null,
): Modifier {
  return { trade_id: `${kind}.${id}`, text, kind, roll: { value, min, max }, tier, affix };
}

const GLOVE_MODS: Modifier[] = [
  mod('fractured', 'stat_3299347043', '+78 to maximum Life', 78, 70, 79, 2, 'prefix'),
  mod('explicit', 'stat_3372524247', '+45% to Fire Resistance', 45, 42, 45, 1, 'suffix'),
  mod('explicit', 'stat_4220027924', '+41% to Cold Resistance', 41, 36, 41, 2, 'suffix'),
  mod('explicit', 'stat_3032590688', 'Adds 12 to 21 Physical Damage to Attacks', 16.5, 13.5, 19.5, 4, 'prefix'),
  mod('explicit', 'stat_3261801346', '+38 to Dexterity', 38, 33, 42, 3, 'suffix'),
  mod('crafted', 'stat_2923486259', '+16% to Chaos Resistance', 16, 12, 16, null, 'suffix'),
];

const GLOVES: Item = {
  item_class: 'Gloves',
  rarity: 'rare',
  name: 'Doom Grip',
  base_type: 'Sorcerer Gloves',
  item_level: 86,
  quality: 20,
  corrupted: false,
  identified: true,
  sockets: 'B-B-R G',
  influences: ['hunter'],
  modifiers: GLOVE_MODS,
  unknown_lines: [],
  affix_slots: { prefixes: 2, suffixes: 4 },
};

const RARE_NAMES = ['Corpse Fist', 'Rapture Hold', 'Hate Mitts', 'Storm Talons', 'Gloom Palm', 'Blight Clutch', 'Vortex Hand', 'Dusk Grasp'];
const SELLERS = ['VaalWhisperer', 'Kitava_Slayer', 'HideoutArchitect', 'ExileTrader99', 'AtlasCartographer', 'MapDevice_Enjoyer', 'Oriath_Merchant', 'DelveDeeper'];
const PRICES: [number, string][] = [[42, 'chaos'], [45, 'chaos'], [3, 'exalted'], [50, 'chaos'], [50, 'chaos'], [55, 'chaos'], [0.3, 'divine'], [68, 'chaos']];
const AGES_MIN = [2, 11, 34, 60, 130, 190, 300, 1500];

/** Cotações fictícias (chaos por unidade), como as do poe.ninja no backend real. */
const DEMO_RATES: Record<string, number> = { chaos: 1, divine: 214, exalted: 16 };
const DIVINE_CHAOS = 214;

/** Mesmo cálculo do Rust (`crates/domain/src/price.rs`): interpolação linear, rank = q·(n−1). */
function percentile(sorted: number[], q: number): number {
  const rank = q * (sorted.length - 1);
  const lo = Math.floor(rank);
  const hi = Math.min(lo + 1, sorted.length - 1);
  return sorted[lo]! + (sorted[hi]! - sorted[lo]!) * (rank - lo);
}

function suggestedOf(sorted: number[]): SuggestedPrice | null {
  if (sorted.length < 5) return null;
  return { buy_chaos: percentile(sorted, 0.25), fair_chaos: percentile(sorted, 0.5), sell_chaos: percentile(sorted, 0.75) };
}

function minutesAgo(min: number): string {
  return new Date(Date.now() - min * 60_000).toISOString();
}

function listedVariant(i: number): Item {
  const shift = (v: number) => Math.max(1, v - (i % 4));
  return {
    ...GLOVES,
    name: RARE_NAMES[i] ?? 'Doom Grip',
    item_level: [86, 84, 85, 86, 83, 86, 84, 85][i] ?? 84,
    corrupted: i === 5,
    sockets: ['B-B-R G', 'B-B-B R', 'G-B-B B', 'B R-R B'][i % 4] ?? null,
    influences: i % 3 !== 2 ? ['hunter'] : [],
    modifiers: ([
      { ...GLOVE_MODS[0]!, kind: i % 2 === 0 ? 'fractured' : 'explicit', text: `+${78 - i} to maximum Life` },
      { ...GLOVE_MODS[1]!, text: `+${shift(45)}% to Fire Resistance` },
      { ...GLOVE_MODS[2]!, text: `+${41 - (i % 3) * 2}% to Cold Resistance` },
      i % 3 === 0
        ? mod('explicit', 'stat_4015621042', `${30 + i * 2}% increased Energy Shield`, 30 + i * 2, 27, 42, 5, 'prefix')
        : { ...GLOVE_MODS[4]!, text: `+${36 + i} to Dexterity` },
      i % 2
        ? { ...GLOVE_MODS[5]!, text: `+${14 + (i % 3)}% to Chaos Resistance` }
        : mod('explicit', 'stat_681332047', `${9 + (i % 4)}% increased Attack Speed`, 9 + (i % 4), 8, 12, 3, 'suffix'),
    ] satisfies Modifier[]).map((m): Modifier => (i === NOT_COMPARABLE ? { ...m, tier: null } : m)),
  };
}

// ---------- Avaliação (espelho simplificado de crates/domain/src/evaluation) ----------

/** Anúncio sem mods comparáveis (vai para "Não comparável"). */
const NOT_COMPARABLE = 4;
/** Anúncio com rolls excelentes que mesmo assim é PIOR (perde as resistências). */
const EXCELLENT_BUT_WORSE = 6;
/** Anúncio com um mod sem dados (inconclusivo). */
const WITH_INCONCLUSIVE = 3;
/** Anúncio melhor que o seu item (mais vida e resistências). */
const BETTER = 1;
/** Anúncio igual ao seu item. */
const EQUIVALENT = 2;

const EXCELLENT_ITEM: Item = {
  ...GLOVES,
  name: 'Vortex Hand',
  influences: [],
  sockets: 'B-B-B B',
  modifiers: [
    mod('fractured', 'stat_3299347043', '+129 to maximum Life', 129, 115, 129, 1, 'prefix'),
    mod('explicit', 'stat_681332047', '16% increased Attack Speed', 16, 14, 16, 1, 'suffix'),
  ],
  affix_slots: { prefixes: 1, suffixes: 1 },
};

const BETTER_ITEM: Item = {
  ...GLOVES,
  name: 'Rapture Hold',
  influences: [],
  modifiers: [
    mod('explicit', 'stat_3299347043', '+127 to maximum Life', 127, 115, 129, 1, 'prefix'),
    mod('explicit', 'stat_3372524247', '+46% to Fire Resistance', 46, 46, 48, 1, 'suffix'),
    mod('explicit', 'stat_4220027924', '+44% to Cold Resistance', 44, 42, 45, 1, 'suffix'),
    mod('explicit', 'stat_3032590688', 'Adds 12 to 21 Physical Damage to Attacks', 16.5, 13.5, 19.5, 4, 'prefix'),
    mod('explicit', 'stat_3261801346', '+38 to Dexterity', 38, 33, 42, 3, 'suffix'),
    mod('crafted', 'stat_2923486259', '+16% to Chaos Resistance', 16, 12, 16, null, 'suffix'),
  ],
  affix_slots: { prefixes: 2, suffixes: 4 },
};

/** Por stat: peso genérico, escala (T1 máx.), nº de tiers na base e tags. */
const STAT_INFO: Record<string, { weight: number; scale: number; tiers: number; tags: string[] }> = {
  stat_3299347043: { weight: 1.5, scale: 129, tiers: 8, tags: ['resource', 'life'] },
  stat_3372524247: { weight: 1.2, scale: 48, tiers: 8, tags: ['elemental', 'fire', 'resistance'] },
  stat_4220027924: { weight: 1.2, scale: 48, tiers: 8, tags: ['elemental', 'cold', 'resistance'] },
  stat_2923486259: { weight: 1.2, scale: 35, tiers: 5, tags: ['chaos', 'resistance'] },
  stat_3032590688: { weight: 1, scale: 20, tiers: 9, tags: ['damage', 'physical_damage', 'attack'] },
  stat_3261801346: { weight: 0.6, scale: 55, tiers: 9, tags: ['attribute'] },
  stat_4015621042: { weight: 1.2, scale: 42, tiers: 8, tags: ['defences', 'energy_shield'] },
  stat_681332047: { weight: 1, scale: 16, tiers: 5, tags: ['attack', 'speed'] },
};
const statId = (m: Modifier) => m.trade_id.split('.')[1] ?? '';

/** Valor do mod pelo texto (média dos números, como o `StatRoll`). */
function valueOf(m: Modifier): number {
  const nums = (m.text.match(/\d+(?:\.\d+)?/g) ?? []).map(Number);
  return nums.length ? nums.reduce((a, b) => a + b, 0) / nums.length : 1;
}

function evaluateMod(m: Modifier, inconclusive: boolean): ModEvaluation {
  const info = STAT_INFO[statId(m)];
  const value = valueOf(m);
  const base: ModEvaluation = {
    trade_id: m.trade_id,
    text: m.text,
    kind: m.kind,
    origin: null,
    value,
    min: m.roll?.min ?? null,
    max: m.roll?.max ?? null,
    tier: m.tier,
    tier_count: info?.tiers ?? null,
    roll: null,
    global_roll: null,
    quality: null,
    importance: (info?.weight ?? 0.5) * (m.kind === 'crafted' ? 0.5 : 1),
    status: { kind: 'exact' },
    variant: null,
    range_mismatch: false,
    tags: info?.tags ?? [],
  };
  if (inconclusive || base.min === null || base.max === null) {
    return { ...base, min: null, max: null, tier_count: null, status: { kind: 'inconclusive', reason: 'no_range' } };
  }
  const roll = base.max > base.min ? Math.min(1, Math.max(0, (value - base.min) / (base.max - base.min))) : 1;
  const count = base.tier_count ?? 0;
  const tierScore = m.tier && count > 1 ? 1 - (m.tier - 1) / (count - 1) : 1;
  const quality = m.kind === 'crafted' ? roll : 0.7 * tierScore + 0.3 * roll;
  return { ...base, roll, quality };
}

function qualityOf(item: Item, inconclusiveExtra = false): ItemQuality {
  const mods = item.modifiers.map((m) => evaluateMod(m, false));
  if (inconclusiveExtra) {
    mods.push(evaluateMod(mod('explicit', 'stat_9999', '+1 to Level of Socketed Gems', 1, 0, 0), true));
  }
  const scored = mods.filter((m) => m.quality !== null);
  const weight = scored.reduce((a, m) => a + m.importance, 0);
  const modsScore = weight > 0 ? scored.reduce((a, m) => a + (m.quality ?? 0) * m.importance, 0) / weight : null;
  const slots = item.affix_slots ?? { prefixes: 3, suffixes: 3 };
  const openPrefixes = Math.max(0, 3 - slots.prefixes);
  const openSuffixes = Math.max(0, 3 - slots.suffixes);
  const crafted = mods.filter((m) => m.kind === 'crafted').length;
  const fracturedHigh = mods.filter((m) => m.kind === 'fractured' && (m.tier ?? 9) <= 2).length;
  const blocked: CraftingPotential['blocked'] = item.corrupted ? ['corrupted'] : [];
  const craftScore = blocked.length
    ? 0
    : Math.min(
        1,
        0.35 * Math.min(2, openSuffixes) +
          0.12 * Math.min(3, openPrefixes) +
          0.2 * Math.min(1, crafted) +
          0.2 * Math.min(1, fracturedHigh) +
          0.1,
      );
  const score = modsScore === null ? null : Math.round(1000 * (0.75 * modsScore + 0.25 * craftScore)) / 10;
  const total = mods.reduce((a, m) => a + m.importance, 0);
  const unknown = mods.filter((m) => m.status.kind === 'inconclusive').reduce((a, m) => a + m.importance, 0);
  const share = total > 0 ? unknown / total : 0;
  return {
    score,
    grade: score === null ? null : score >= 85 ? 'excellent' : score >= 65 ? 'good' : score >= 40 ? 'average' : 'weak',
    confidence: score === null ? 'low' : share <= 0.1 ? 'high' : share <= 0.35 ? 'medium' : 'low',
    mods_score: modsScore,
    mods,
    crafting: {
      open_prefixes: openPrefixes,
      open_suffixes: openSuffixes,
      valued: 'suffix',
      crafted,
      fractured_high_tier: fracturedHigh,
      ilvl_allows_top_tier: (item.item_level ?? 0) >= 82,
      blocked,
      score: craftScore,
    },
    unique: null,
    sockets: null,
  };
}

function vectorOf(item: Item): Map<string, { value: number; text: string }> {
  const out = new Map<string, { value: number; text: string }>();
  for (const m of item.modifiers) {
    const key = statId(m);
    const prev = out.get(key);
    out.set(key, { value: (prev?.value ?? 0) + valueOf(m), text: m.text.replace(/[+-]?\d+(?:\.\d+)?/g, '#') });
  }
  return out;
}

function upgradeOf(mine: Item, theirs: Item, build: DemoBuild | null): UpgradeVerdict {
  const a = vectorOf(mine);
  const b = vectorOf(theirs);
  const deltas: StatDelta[] = [];
  let magnitude = 0;
  for (const key of new Set([...a.keys(), ...b.keys()])) {
    const info = STAT_INFO[key] ?? { weight: 0.5, scale: Math.max(a.get(key)?.value ?? 1, b.get(key)?.value ?? 1), tiers: 0, tags: [] };
    const boosted = build !== null && info.tags.includes('damage') && info.tags.some((t) => build.summary.focus.includes(t));
    const weight = boosted ? info.weight * 1.8 : info.weight;
    const yours = a.get(key)?.value ?? 0;
    const theirsValue = b.get(key)?.value ?? 0;
    magnitude += (weight * yours) / info.scale;
    const contribution = (weight * (theirsValue - yours)) / info.scale;
    const text = b.get(key)?.text ?? a.get(key)?.text ?? key;
    if (Math.abs(contribution) > 1e-9) {
      deltas.push({ key, text, yours, theirs: theirsValue, weight, contribution: Math.round(contribution * 100) / 100 });
    }
  }
  const needs: NeedChange[] = [];
  if (build) {
    const total = 75 + fireOvercapOf(build);
    const after = total - (a.get('stat_3372524247')?.value ?? 0) + (b.get('stat_3372524247')?.value ?? 0);
    const shortAfter = Math.max(0, 75 - after);
    if (shortAfter > 0) {
      needs.push({ stat: 'fire', short_before: 0, short_after: shortAfter, contribution: -Math.round(shortAfter * 6) / 100 });
    }
  }
  const delta = deltas.reduce((s, d) => s + d.contribution, 0) + needs.reduce((s, n) => s + n.contribution, 0);
  const relative = delta / Math.max(1, magnitude);
  // Igual só com exatamente os mesmos mods e rolls (espelho de `same_rolls`).
  const texts = (item: Item) => item.modifiers.map((m) => m.text.toLowerCase()).sort().join('\n');
  const verdict = texts(mine) === texts(theirs) ? 'equivalent' : delta > 0 ? 'better' : 'worse';
  deltas.sort((x, y) => y.contribution - x.contribution);
  return {
    verdict,
    delta: Math.round(delta * 100) / 100,
    relative: Math.round(relative * 100) / 100,
    reference: Math.round(Math.max(1, magnitude) * 100) / 100,
    gains: deltas.filter((d) => d.contribution > 0).slice(0, 3),
    losses: deltas.filter((d) => d.contribution < 0).reverse().slice(0, 3),
    synergy: 0,
    needs,
    relevant_loss: deltas.some((d) => d.contribution <= -0.5) || needs.length > 0,
    confidence: !build ? 'medium' : build.details?.summary.needs_estimated ? 'medium' : 'high',
  };
}

function demoDetail(listing: Listing, i: number, build: DemoBuild | null): { kind: GroupKind; detail: CompareDetail | null } {
  const item = listing.item;
  if (!item) return { kind: 'no_item', detail: null };
  if (i === NOT_COMPARABLE) return { kind: 'not_comparable', detail: null };
  const upgrade = upgradeOf(GLOVES, item, build);
  const kind: GroupKind = upgrade.verdict === 'better' ? 'better' : upgrade.verdict === 'worse' ? 'worse' : 'equal';
  return { kind, detail: { upgrade, quality: qualityOf(item, i === WITH_INCONCLUSIVE) } };
}

function demoComparison(listings: Listing[], build: DemoBuild | null): ListingComparison {
  const order: GroupKind[] = ['worse', 'equal', 'better', 'not_comparable', 'no_item'];
  const groups = order.map((kind) => ({ kind, entries: [] as GroupEntry[] }));
  listings.forEach((listing, index) => {
    const { kind, detail } = demoDetail(listing, index, build);
    const chaos = listing.price.amount * (DEMO_RATES[listing.price.currency] ?? 1);
    groups.find((g) => g.kind === kind)?.entries.push({ index, chaos, detail });
  });
  for (const g of groups) g.entries.sort((a, b) => (a.chaos ?? Infinity) - (b.chaos ?? Infinity));
  return {
    mode: 'grouped',
    you: {
      quality: qualityOf(GLOVES),
      build: build?.details?.summary ?? {
        name: null,
        generic: true,
        class: null,
        main_skill: null,
        focus: [],
        item_set: null,
        skill_set: null,
        needs_estimated: false,
      },
      data_source: 'PoB v2.69.0 · demo',
    },
    groups,
  };
}

/** Build fictício do PoB (a demo não lê arquivos). */
export interface DemoBuild {
  file: BuildFile;
  summary: BuildSummary;
  itemSets: { id: string; title: string; fireOvercap: number }[];
  skillSets: { id: string; title: string; mainSkill: string | null }[];
  activeItemSet: string;
  activeSkillSet: string;
  /** Preenchido com a escolha de sets das configurações. */
  details?: BuildDetails;
}

const summaryOf = (name: string, cls: string, focus: string[]): BuildSummary => ({
  name,
  generic: false,
  class: cls,
  main_skill: null,
  focus,
  item_set: null,
  skill_set: null,
  needs_estimated: false,
});

export const DEMO_BUILDS: DemoBuild[] = [
  {
    file: { name: 'RF 3.27', path: 'demo://RF 3.27.xml', folder: null },
    summary: summaryOf('RF 3.27', 'Chieftain', ['caster', 'fire', 'life', 'gem', 'resistance']),
    itemSets: [
      { id: '1', title: 'Level 1-12', fireOvercap: -20 },
      { id: '2', title: 'Maps', fireOvercap: 0 },
      { id: '3', title: 'Melding', fireOvercap: 12 },
    ],
    skillSets: [
      { id: '1', title: 'Leveling', mainSkill: 'Rolling Magma' },
      { id: '2', title: 'Endgame', mainSkill: 'Righteous Fire' },
    ],
    activeItemSet: '3',
    activeSkillSet: '2',
  },
  {
    file: { name: 'KB 3.27', path: 'demo://Liga/KB 3.27.xml', folder: 'Liga' },
    summary: summaryOf('KB 3.27', 'Necromancer', ['attack', 'physical', 'life', 'critical', 'gem']),
    itemSets: [{ id: '1', title: 'Default', fireOvercap: 0 }],
    skillSets: [{ id: '1', title: 'Default', mainSkill: 'Kinetic Blast of Clustering' }],
    activeItemSet: '1',
    activeSkillSet: '1',
  },
  {
    file: { name: 'POISON SPECTRE 3.27', path: 'demo://POISON SPECTRE 3.27.xml', folder: null },
    summary: summaryOf('POISON SPECTRE 3.27', 'Necromancer', ['minion', 'life', 'gem', 'resistance']),
    itemSets: [
      { id: '1', title: 'League start', fireOvercap: 5 },
      { id: '2', title: 'Endgame', fireOvercap: 30 },
    ],
    skillSets: [{ id: '1', title: 'Default', mainSkill: 'Raise Spectre' }],
    activeItemSet: '2',
    activeSkillSet: '1',
  },
];

/** Espelho de `BuildInput::details`: resumo com os sets escolhidos e os sets disponíveis. */
export function demoBuildDetails(build: DemoBuild, itemSet: string | null, skillSet: string | null): BuildDetails {
  const items = build.itemSets.find((s) => s.id === itemSet) ?? build.itemSets.find((s) => s.id === build.activeItemSet)!;
  const skills = build.skillSets.find((s) => s.id === skillSet) ?? build.skillSets.find((s) => s.id === build.activeSkillSet)!;
  return {
    summary: {
      ...build.summary,
      main_skill: skills.mainSkill,
      item_set: items.title,
      skill_set: skills.title,
      needs_estimated: items.id !== build.activeItemSet,
    },
    item_sets: build.itemSets.map(({ id, title }) => ({ id, title })),
    skill_sets: build.skillSets.map(({ id, title }) => ({ id, title })),
    active_item_set: build.activeItemSet,
    active_skill_set: build.activeSkillSet,
  };
}

/** Fogo acima do cap no item set escolhido (para o aviso de "perda relevante"). */
function fireOvercapOf(build: DemoBuild): number {
  const id = build.details?.summary.item_set;
  return build.itemSets.find((s) => s.title === id)?.fireOvercap ?? 0;
}

export function defaultFilters(item: Item, rangePercent: number, kinds: ModifierKind[], showOffline: boolean): SearchFilters {
  const range = rangePercent / 100;
  return {
    online_only: !showOffline,
    stats: item.modifiers.map((m) => ({
      trade_id: m.trade_id,
      text: m.text,
      kind: m.kind,
      roll: m.roll,
      tier: m.tier,
      enabled: kinds.includes(m.kind),
      min: m.roll ? Math.floor(m.roll.value - Math.abs(m.roll.value) * range) : null,
      max: null,
    })),
    item_level_min: null,
    quality_min: null,
    links_min: null,
  };
}

export function rareOutput(filters: SearchFilters, few = false, build: DemoBuild | null = null): PriceCheckOutput {
  const enabled = filters.stats.filter((s) => s.enabled).length;
  const looser = Math.max(0, 3 - enabled);
  const prices = few ? PRICES.slice(0, 3) : PRICES;
  const listings: Listing[] = prices.map(([amount, currency], i) => ({
    price: { amount: currency === 'chaos' ? Math.max(5, amount - looser * 6) : amount, currency },
    stack_size: null,
    account: SELLERS[i] ?? 'Exile',
    online: i !== 2 && i !== 5,
    indexed_at: minutesAgo(AGES_MIN[i] ?? 60),
    item:
      i === EXCELLENT_BUT_WORSE
        ? EXCELLENT_ITEM
        : i === BETTER
          ? BETTER_ITEM
          : i === EQUIVALENT
            ? { ...GLOVES, name: 'Hate Mitts' }
            : listedVariant(i),
    whisper: `@${SELLERS[i] ?? 'Exile'} Hi, I would like to buy your ${RARE_NAMES[i] ?? 'item'} Sorcerer Gloves listed for ${amount} ${currency} in Standard`,
  }));
  const values = listings.map((l) => l.price.amount * (DEMO_RATES[l.price.currency] ?? 1)).sort((a, b) => a - b);
  const window = values.slice(0, 10);
  return {
    item: GLOVES,
    filters,
    indexed: null,
    trade_error: null,
    trade: {
      query_id: 'demo',
      total: few ? 3 : 37 + looser * 23,
      listings,
      web_url: 'https://www.pathofexile.com/trade/search/Standard',
    },
    summary: {
      estimate_chaos: percentile(window, 0.5),
      min_chaos: values[0]!,
      median_chaos: percentile(values, 0.5),
      max_chaos: values[values.length - 1]!,
      sample_size: values.length,
      histogram: few ? [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1] : [1, 2, 4, 7, 9, 6, 4, 3, 2, 2, 1, 1],
      confidence: few ? 1 : 2,
      divine_chaos: DIVINE_CHAOS,
      suggested: suggestedOf(values),
    },
    comparison: demoComparison(listings, build),
  };
}

export function currencyOutput(): PriceCheckOutput {
  const item: Item = {
    item_class: 'Stackable Currency',
    rarity: 'currency',
    name: null,
    base_type: 'Divine Orb',
    item_level: null,
    quality: null,
    corrupted: false,
    identified: true,
    sockets: null,
    influences: [],
    modifiers: [],
    unknown_lines: [],
  };
  const offers: [number, number][] = [[212, 48], [213, 120], [214, 22], [214, 9], [215, 61], [216, 300]];
  return {
    item,
    filters: { online_only: true, stats: [], item_level_min: null, quality_min: null, links_min: null },
    indexed: {
      name: 'Divine Orb',
      chaos_value: 214,
      divine_value: 1,
      change_24h_percent: 3.2,
      sparkline: [201, 204, 203, 207, 209, 208, 211, 214],
      icon: null,
    },
    trade_error: null,
    trade: {
      query_id: 'demo-bulk',
      total: 128,
      listings: offers.map(([amount, stock], i) => ({
        price: { amount, currency: 'chaos' },
        stack_size: stock,
        account: SELLERS[i] ?? 'Exile',
        online: true,
        indexed_at: minutesAgo(AGES_MIN[i] ?? 60),
        item: null,
        whisper: `@${SELLERS[i] ?? 'Exile'} Hi, I'd like to buy your 1 Divine Orb for my ${amount} Chaos Orb in Standard.`,
      })),
      web_url: 'https://www.pathofexile.com/trade/exchange/Standard',
    },
    summary: {
      estimate_chaos: 214,
      min_chaos: 212,
      median_chaos: 214,
      max_chaos: 216,
      sample_size: 6,
      histogram: [1, 0, 0, 1, 0, 0, 2, 0, 0, 1, 0, 1],
      confidence: 1,
      divine_chaos: DIVINE_CHAOS,
      // Como no Rust: sugestão a partir de 5 listagens. O histórico de currency grava o poe.ninja.
      suggested: suggestedOf(offers.map(([amount]) => amount)),
    },
    comparison: { mode: 'single', reason: { kind: 'rarity' } },
  };
}

function stat(kind: CatalogStat['kind'], id: string, text: string): CatalogStat {
  return { trade_id: `${kind}.${id}`, text, kind };
}

/** Recorte do catálogo de stats (o real vem do TSV embutido no parser). */
export const DEMO_STATS: CatalogStat[] = [
  stat('pseudo', 'pseudo_total_life', '+# total maximum Life'),
  stat('pseudo', 'pseudo_total_energy_shield', '+# total maximum Energy Shield'),
  stat('pseudo', 'pseudo_total_elemental_resistance', '+#% total Elemental Resistance'),
  stat('pseudo', 'pseudo_total_resistance', '+#% total Resistance'),
  stat('pseudo', 'pseudo_total_fire_resistance', '+#% total to Fire Resistance'),
  stat('pseudo', 'pseudo_total_cold_resistance', '+#% total to Cold Resistance'),
  stat('pseudo', 'pseudo_total_lightning_resistance', '+#% total to Lightning Resistance'),
  stat('pseudo', 'pseudo_total_chaos_resistance', '+#% total to Chaos Resistance'),
  stat('pseudo', 'pseudo_total_attributes', '+# total to all Attributes'),
  stat('pseudo', 'pseudo_total_dexterity', '+# total to Dexterity'),
  stat('pseudo', 'pseudo_total_strength', '+# total to Strength'),
  stat('pseudo', 'pseudo_total_intelligence', '+# total to Intelligence'),
  stat('pseudo', 'pseudo_increased_movement_speed', '#% increased Movement Speed'),
  stat('pseudo', 'pseudo_count_elemental_resistances', '# Elemental Resistances'),
  stat('explicit', 'stat_3299347043', '+# to maximum Life'),
  stat('explicit', 'stat_1050105434', '+# to maximum Mana'),
  stat('explicit', 'stat_3372524247', '+#% to Fire Resistance'),
  stat('explicit', 'stat_4220027924', '+#% to Cold Resistance'),
  stat('explicit', 'stat_1671376347', '+#% to Lightning Resistance'),
  stat('explicit', 'stat_2923486259', '+#% to Chaos Resistance'),
  stat('explicit', 'stat_2901986750', '+#% to all Elemental Resistances'),
  stat('explicit', 'stat_3261801346', '+# to Dexterity'),
  stat('explicit', 'stat_4080418644', '+# to Strength'),
  stat('explicit', 'stat_328541901', '+# to Intelligence'),
  stat('explicit', 'stat_1379411836', '+# to all Attributes'),
  stat('explicit', 'stat_3032590688', 'Adds # to # Physical Damage to Attacks'),
  stat('explicit', 'stat_681332047', '#% increased Attack Speed'),
  stat('explicit', 'stat_4015621042', '#% increased Energy Shield'),
  stat('explicit', 'stat_3489782002', '+# to maximum Energy Shield'),
  stat('explicit', 'stat_2250533757', '#% increased Movement Speed'),
  stat('explicit', 'stat_3917489142', '#% increased Rarity of Items found'),
  stat('explicit', 'stat_803737631', '+# to Accuracy Rating'),
  stat('explicit', 'stat_3325883026', 'Regenerate # Life per second'),
  stat('implicit', 'stat_3299347043', '+# to maximum Life'),
  stat('implicit', 'stat_2901986750', '+#% to all Elemental Resistances'),
  stat('implicit', 'stat_3261801346', '+# to Dexterity'),
  stat('implicit', 'stat_681332047', '#% increased Attack Speed'),
  stat('implicit', 'stat_1379411836', '+# to all Attributes'),
  stat('crafted', 'stat_3299347043', '+# to maximum Life'),
  stat('crafted', 'stat_2923486259', '+#% to Chaos Resistance'),
  stat('enchant', 'stat_2250533757', '#% increased Movement Speed'),
];

export { GLOVES as DEMO_GLOVES };
