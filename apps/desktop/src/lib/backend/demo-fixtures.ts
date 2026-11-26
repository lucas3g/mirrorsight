/**
 * Dados fictícios para o adapter de demonstração. Nada aqui é usado no app real.
 */
import type {
  Affix,
  CatalogStat,
  CompareDetail,
  GroupEntry,
  GroupKind,
  Item,
  Listing,
  ListingComparison,
  Modifier,
  ModifierKind,
  PriceCheckOutput,
  SearchFilters,
  SuggestedPrice,
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

// ---------- Agrupamento por status (espelho simplificado de crates/domain/src/compare.rs) ----------

/** Anúncio sem tiers (vai para "Não comparável"). */
const NOT_COMPARABLE = 4;
/** Tier e percentil dos mods do seu item que entram na qualidade (explícitos/fractured). */
const YOUR_STATS: Record<string, [number, number]> = {
  stat_3299347043: [2, 0.89],
  stat_3372524247: [1, 1],
  stat_4220027924: [2, 1],
  stat_3032590688: [4, 0.5],
  stat_3261801346: [3, 0.56],
};
/** Seu item: 4 suffixes ocupados (crafted incluso) → 0 livre. */
const YOUR_FREE = 0;
/** Valores fictícios de cada anúncio nos mods em comum: [tier médio, percentil médio, suffixes livres]. */
const LISTING_SCORES: [number, number, number][] = [
  [2.33, 0.7, 0],
  [2, 0.71, 1],
  [2, 0.86, 1],
  [2, 0.9, 0],
  [0, 0, 0],
  [1.5, 0.6, 0],
  [1.67, 0.96, 1],
  [1.75, 0.8, 0],
];

const avg = (v: number[]) => v.reduce((a, b) => a + b, 0) / Math.max(1, v.length);
const r2 = (v: number) => Math.round(v * 100);

function demoDetail(listing: Listing, i: number): { kind: GroupKind; detail: CompareDetail | null } {
  const item = listing.item;
  if (!item) return { kind: 'no_item', detail: null };
  if (i === NOT_COMPARABLE) return { kind: 'not_comparable', detail: null };
  const common = item.modifiers.filter((m) => m.kind !== 'crafted' && YOUR_STATS[m.trade_id.split('.')[1] ?? '']);
  const mine = common.map((m) => YOUR_STATS[m.trade_id.split('.')[1] ?? '']!);
  const [tier, pct, free] = LISTING_SCORES[i] ?? [2, 0.5, 0];
  const yours = { avg_tier: avg(mine.map((s) => s[0])), avg_percentile: avg(mine.map((s) => s[1])), free_valued: YOUR_FREE };
  const steps: [CompareDetail['decided'], number][] = [
    ['tier', r2(yours.avg_tier) - r2(tier)],
    ['percentile', r2(pct) - r2(yours.avg_percentile)],
    ['slots', free - YOUR_FREE],
  ];
  const step = steps.find(([, diff]) => diff !== 0);
  return {
    kind: !step ? 'equal' : step[1] > 0 ? 'better' : 'worse',
    detail: {
      common: common.length,
      common_ids: common.map((m) => m.trade_id),
      listing: { avg_tier: tier, avg_percentile: pct, free_valued: free },
      yours,
      decided: step?.[0] ?? null,
    },
  };
}

function demoComparison(listings: Listing[]): ListingComparison {
  const order: GroupKind[] = ['worse', 'equal', 'better', 'not_comparable', 'no_item'];
  const groups = order.map((kind) => ({ kind, entries: [] as GroupEntry[] }));
  listings.forEach((listing, index) => {
    const { kind, detail } = demoDetail(listing, index);
    const chaos = listing.price.amount * (DEMO_RATES[listing.price.currency] ?? 1);
    groups.find((g) => g.kind === kind)?.entries.push({ index, chaos, detail });
  });
  for (const g of groups) g.entries.sort((a, b) => (a.chaos ?? Infinity) - (b.chaos ?? Infinity));
  const stats = Object.values(YOUR_STATS);
  return {
    mode: 'grouped',
    you: {
      avg_tier: avg(stats.map((s) => s[0])),
      avg_percentile: avg(stats.map((s) => s[1])),
      free_valued: YOUR_FREE,
      valued: 'suffix',
      mods: [],
    },
    groups,
  };
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

export function rareOutput(filters: SearchFilters, few = false): PriceCheckOutput {
  const enabled = filters.stats.filter((s) => s.enabled).length;
  const looser = Math.max(0, 3 - enabled);
  const prices = few ? PRICES.slice(0, 3) : PRICES;
  const listings: Listing[] = prices.map(([amount, currency], i) => ({
    price: { amount: currency === 'chaos' ? Math.max(5, amount - looser * 6) : amount, currency },
    stack_size: null,
    account: SELLERS[i] ?? 'Exile',
    online: i !== 2 && i !== 5,
    indexed_at: minutesAgo(AGES_MIN[i] ?? 60),
    item: listedVariant(i),
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
    comparison: demoComparison(listings),
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
