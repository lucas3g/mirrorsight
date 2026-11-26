#!/usr/bin/env node
/**
 * Gera a base de conhecimento do PoE1 usada na avaliação de itens, a partir dos dados do
 * Path of Building Community (MIT; dados do jogo (c) Grinding Gear Games):
 *
 *   crates/gamedata/data/uniques-poe1.json   únicos: bases, versões, variantes e mods com faixas
 *   crates/gamedata/data/mods-poe1.json      mods por stat da trade (tiers, nível, faixa, pesos, tags)
 *   crates/gamedata/data/bases-poe1.json     bases: tipo, tags de spawn e tags de influência
 *   crates/gamedata/data/gems-poe1.json      gemas: tags (para o perfil da build)
 *
 * Uso:
 *   node tools/gamedata/update-gamedata.mjs              # última release do PoB no GitHub
 *   node tools/gamedata/update-gamedata.mjs --ref dev    # outro branch/tag
 *   node tools/gamedata/update-gamedata.mjs --local "C:\...\Path of Building Community"   # instalação local
 *
 * Rode a cada liga/patch (junto com tools/stats/update-stats.mjs) e depois `cargo test`.
 * As regras de peso/sinergia ficam à parte, em crates/gamedata/data/rules-poe1.json (editadas à mão).
 */
import { readFile, readdir, writeFile, mkdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseReturn, parseAssignments, listOf, mapOf } from './lua.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const OUT_DIR = join(ROOT, 'crates', 'gamedata', 'data');
const REPO = 'PathOfBuildingCommunity/PathOfBuilding';
const USER_AGENT = 'Mirrorsight/0.2 (open-source PoE price checker; gamedata snapshot)';

const UNIQUE_FILES = [
  'amulet', 'axe', 'belt', 'body', 'boots', 'bow', 'claw', 'dagger', 'fishing', 'flask', 'gloves', 'graft',
  'helmet', 'jewel', 'mace', 'quiver', 'ring', 'shield', 'staff', 'sword', 'tincture', 'wand',
];
const BASE_FILES = [...UNIQUE_FILES, 'wickerman'];

// ---------------------------------------------------------------- fonte (GitHub ou local)

function parseArgs() {
  const args = process.argv.slice(2);
  const get = (flag) => {
    const i = args.indexOf(flag);
    return i >= 0 ? args[i + 1] : undefined;
  };
  return { local: get('--local'), ref: get('--ref') };
}

async function makeSource({ local, ref }) {
  if (local) {
    const dataDir = join(local, 'Data');
    let version = 'local';
    try {
      const manifest = await readFile(join(local, 'manifest.xml'), 'utf8');
      version = `local ${/<Version number="([^"]+)"/.exec(manifest)?.[1] ?? '?'}`;
    } catch {
      /* sem manifest */
    }
    return { version, read: (path) => readFile(join(dataDir, path), 'utf8'), list: async (dir) => readdir(join(dataDir, dir)) };
  }
  let tag = ref;
  if (!tag) {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { headers: { 'User-Agent': USER_AGENT } });
    if (!res.ok) throw new Error(`GitHub releases: HTTP ${res.status}`);
    tag = (await res.json()).tag_name;
  }
  const read = async (path) => {
    const url = `https://raw.githubusercontent.com/${REPO}/${tag}/src/Data/${path}`;
    const res = await fetch(url, { headers: { 'User-Agent': USER_AGENT } });
    if (!res.ok) throw new Error(`${path}: HTTP ${res.status}`);
    return res.text();
  };
  return { version: `PoB ${tag}`, read };
}

// ---------------------------------------------------------------- texto de mod → template + faixas

const RANGE = /\((-?\d+(?:\.\d+)?)-(-?\d+(?:\.\d+)?)\)/g;

/**
 * "+(40-55) to Strength" → { text, ranges: [[40,55]] }. Números fixos entram como [n,n], na ordem
 * em que aparecem (o app tira a média, como a trade API faz em "Adds # to #").
 */
export function modRanges(text) {
  const ranges = [];
  // Marca as faixas para não ler os números de dentro delas de novo.
  const marked = text.replace(RANGE, (_, a, b) => {
    const lo = Number(a);
    const hi = Number(b);
    ranges.push({ at: ranges.length, v: [Math.min(lo, hi), Math.max(lo, hi)] });
    return `\u0000${ranges.length - 1}\u0000`;
  });
  const ordered = [];
  const re = /\u0000(\d+)\u0000|(?<![A-Za-z0-9#])(-?\d+(?:\.\d+)?)/g;
  let m;
  while ((m = re.exec(marked))) {
    if (m[1] !== undefined) ordered.push(ranges[Number(m[1])].v);
    else ordered.push([Number(m[2]), Number(m[2])]);
  }
  return ordered;
}

/** Média das faixas (mesma regra do `StatRoll` do app). */
function averageRange(ranges) {
  if (ranges.length === 0) return null;
  const n = ranges.length;
  const lo = ranges.reduce((s, r) => s + r[0], 0) / n;
  const hi = ranges.reduce((s, r) => s + r[1], 0) / n;
  return [round(lo), round(hi)];
}

const round = (v) => Math.round(v * 1000) / 1000;

// ---------------------------------------------------------------- únicos

const HEADER_KEY =
  /^(Variant|Version|Source|League|Limited to|Radius|Upgrade|Has Alt Variant(?: Two| Three)?|Selected (?:Alt )?Variant(?: Two| Three| Group)?|Sockets|Implicits|Talisman Tier|Crafted|LevelReq|Item Level|Rarity|Unique ID|Catalyst|Quality|Item Class|Energy Shield|Evasion|Armour)\s*:/;
const FLAG_LINES = new Set([
  'Corrupted', 'Mirrored', 'Split', 'Shaper Item', 'Elder Item', 'Crusader Item', 'Hunter Item', 'Redeemer Item',
  'Warlord Item', 'Synthesised Item', 'Fractured Item', 'Searing Exarch Item', 'Eater of Worlds Item',
]);

/** "{variant:1,2}{tags:life}texto" → { markers: {variant:'1,2', tags:'life'}, text } */
function stripMarkers(line) {
  const markers = {};
  let text = line;
  for (;;) {
    const m = /^\{([A-Za-z]+)(?::([^}]*))?\}/.exec(text);
    if (!m) break;
    markers[m[1]] = m[2] ?? true;
    text = text.slice(m[0].length);
  }
  return { markers, text: text.trim() };
}

/** "1,3-5" → [1,3,4,5] */
function indexList(raw) {
  if (raw === undefined || raw === true) return null;
  const out = [];
  for (const part of String(raw).split(',')) {
    const [a, b] = part.split('-').map((x) => Number(x.trim()));
    if (Number.isNaN(a)) continue;
    if (b !== undefined && !Number.isNaN(b)) for (let i = a; i <= b; i++) out.push(i);
    else out.push(a);
  }
  return out.length ? out : null;
}

/** Nome de base sem acentos ("Maelström" ≡ "Maelstrom") → nome canônico. Preenchido antes dos únicos. */
let BASE_NAMES = new Map();
const plain = (s) => s.normalize('NFD').replace(/[̀-ͯ]/g, '');

function parseUnique(block, extraLines = []) {
  const lines = [...block.split(/\r?\n/), ...extraLines]
    .map((l) => l.trim())
    .filter(Boolean)
    .filter((l) => !/^(Item Class|Rarity):/.test(l));
  if (lines.length < 2) return null;
  const name = lines[0];
  const bases = [];
  let i = 1;
  const variants = [];
  const versions = [];
  let implicits = 0;
  let pool = false;
  const flags = [];
  const mods = [];
  for (; i < lines.length; i++) {
    const line = lines[i];
    const header = HEADER_KEY.exec(line);
    if (header) {
      const value = line.slice(line.indexOf(':') + 1).trim();
      if (header[1] === 'Variant') variants.push(value);
      else if (header[1] === 'Version') versions.push(value);
      else if (header[1] === 'Implicits') implicits = Number(value) || 0;
      else if (header[1].startsWith('Has Alt Variant')) pool = true;
      continue;
    }
    if (/^Requires /.test(line)) continue;
    if (FLAG_LINES.has(line)) {
      flags.push(line);
      continue;
    }
    const { markers, text } = stripMarkers(line);
    if (!text) continue;
    // Base: qualquer linha com nome de base antes do primeiro mod (pode vir depois de
    // "Source:" ou de "Shaper Item", e ter {variant:} quando a base muda entre versões).
    const base = BASE_NAMES.get(plain(text));
    if (mods.length === 0 && base) {
      bases.push({ base, variants: indexList(markers.variant) });
      continue;
    }
    const ranges = modRanges(text);
    mods.push({
      text,
      ranges,
      implicit: mods.length < implicits,
      variants: indexList(markers.variant),
      versions: indexList(markers.version),
      group: markers.group !== undefined && markers.group !== true ? Number(markers.group) : null,
      tags: typeof markers.tags === 'string' ? markers.tags.split(',').map((t) => t.trim()).filter(Boolean) : [],
    });
  }
  return { name, bases, versions, variants, pool, flags, mods };
}

/** Únicos montados por código em Special/Generated.lua: os de pool (Watcher's Eye…) e stubs. */
function generatedUniques(generatedSrc, watchersEye, boundByDestiny, baseNames) {
  const out = [];
  const headerOf = (varName) => {
    const m = new RegExp(`local ${varName} = \\{\\s*\\[\\[([\\s\\S]*?)\\]\\]`).exec(generatedSrc);
    return m ? m[1] : null;
  };
  const insertsOf = (varName) => {
    const re = new RegExp(`table\\.insert\\(${varName},\\s*\\[\\[([\\s\\S]*?)\\]\\]\\)`, 'g');
    return [...generatedSrc.matchAll(re)].map((m) => m[1]);
  };
  const poolMods = (entries) =>
    entries.map(([, mod], idx) =>
      listOf(mod).filter((l) => typeof l === 'string').map((text) => ({
        text,
        ranges: modRanges(text),
        implicit: false,
        variants: [idx + 1],
        versions: null,
        group: null,
        tags: listOf(mapOf(mod).modTags).filter((t) => typeof t === 'string'),
      })),
    ).flat();
  const sorted = (table) => Object.entries(mapOf(table)).sort(([a], [b]) => a.localeCompare(b));

  const pools = [
    { varName: 'watchersEye', entries: sorted(watchersEye).filter(([id]) => !/^(SublimeVision|SummonArbalist)/.test(id)) },
    { varName: 'sublimeVision', entries: sorted(watchersEye).filter(([id]) => /^SublimeVision/.test(id)) },
    { varName: 'voranasMarch', entries: sorted(watchersEye).filter(([id]) => /^SummonArbalist/.test(id)) },
    {
      varName: 'boundByDestiny',
      entries: Object.entries(mapOf(boundByDestiny)).sort(([a, ma], [b, mb]) => {
        const ta = mapOf(ma).type ?? '';
        const tb = mapOf(mb).type ?? '';
        return ta === tb ? a.localeCompare(b) : ta.localeCompare(tb);
      }),
    },
  ];
  const done = new Set();
  for (const { varName, entries } of pools) {
    const header = headerOf(varName);
    if (!header) continue;
    const unique = parseUnique(header, insertsOf(varName).flatMap((b) => b.split(/\r?\n/)));
    if (!unique) continue;
    unique.pool = true;
    unique.mods.push(...poolMods(entries));
    out.push(unique);
    done.add(unique.name);
  }
  // Demais únicos gerados (Precursor's Emblem, Forbidden Flame…): conhecidos, mas sem dados de mods.
  const stubRe = /local \w+ = \{\s*(?:\[\[\s*([^\n\]]+)\n\s*([^\n\]]+)|"([^"]+)",\s*"([^"]+)")/g;
  for (const m of generatedSrc.matchAll(stubRe)) {
    const name = (m[1] ?? m[3]).trim();
    const base = BASE_NAMES.get(plain(stripMarkers((m[2] ?? m[4]).trim()).text));
    // Só tabelas que descrevem um item de verdade (a segunda linha é uma base conhecida).
    if (done.has(name) || !base) continue;
    done.add(name);
    out.push({ name, bases: [{ base, variants: null }], versions: [], variants: [], pool: false, flags: [], mods: [], generated: true });
  }
  return out;
}

async function buildUniques(src, baseNames) {
  BASE_NAMES = new Map([...baseNames].map((b) => [plain(b), b]));
  const uniques = [];
  for (const file of UNIQUE_FILES) {
    const table = parseReturn(await src.read(`Uniques/${file}.lua`));
    for (const block of listOf(table)) {
      if (typeof block !== 'string') continue;
      const u = parseUnique(block);
      if (u) uniques.push(u);
    }
  }
  try {
    const generated = await src.read('Uniques/Special/Generated.lua');
    const watchersEye = parseReturn(await src.read('Uniques/Special/WatchersEye.lua'));
    const boundByDestiny = parseReturn(await src.read('Uniques/Special/BoundByDestiny.lua'));
    uniques.push(...generatedUniques(generated, watchersEye, boundByDestiny, baseNames));
  } catch (err) {
    console.warn(`únicos gerados ignorados: ${err.message}`);
  }
  // Mutações Foulborn: único → mods que podem substituir um dos originais (ModFoulbornMap.lua).
  let foulborn = {};
  try {
    foulborn = mapOf(parseReturn(await src.read('ModFoulbornMap.lua')));
  } catch (err) {
    console.warn(`mutações Foulborn ignoradas: ${err.message}`);
  }
  const plainName = (n) => n.normalize('NFD').replace(/[\u0300-\u036f]/g, '');
  const foulbornByName = new Map(Object.entries(foulborn).map(([name, list]) => [plainName(name), list]));
  let withFoulborn = 0;
  // Compacta: campos vazios saem do JSON.
  return uniques.map((u) => {
    const mutations = listOf(foulbornByName.get(plainName(u.name))).filter((t) => typeof t === 'string');
    if (mutations.length) withFoulborn++;
    return {
    ...(mutations.length ? { foulborn: mutations.map((text) => ({ text, ranges: modRanges(text) })) } : {}),
    name: u.name,
    bases: u.bases.map((b) => (b.variants ? b : { base: b.base })),
    ...(u.versions.length ? { versions: u.versions } : {}),
    ...(u.variants.length ? { variants: u.variants } : {}),
    ...(u.pool ? { pool: true } : {}),
    ...(u.generated ? { generated: true } : {}),
    ...(u.flags.length ? { flags: u.flags } : {}),
    mods: u.mods.map((m) => ({
      text: m.text,
      ranges: m.ranges,
      ...(m.implicit ? { implicit: true } : {}),
      ...(m.variants ? { variants: m.variants } : {}),
      ...(m.versions ? { versions: m.versions } : {}),
      ...(m.group !== null && !Number.isNaN(m.group) ? { group: m.group } : {}),
      ...(m.tags.length ? { tags: m.tags } : {}),
    })),
    };
  }).filter((u, i, all) => {
    // Só para registrar quantos únicos ganharam mutações.
    if (i === all.length - 1) console.log(`únicos com mutação Foulborn: ${withFoulborn}`);
    return true;
  });
}

// ---------------------------------------------------------------- mods com tier (por stat da trade)

/** Uma entrada por (mod, stat): grupo, nome do afixo, tipo, nível, faixa média, pesos de spawn. */
function collectMods(table, source, stats, tags) {
  const entries = Object.keys(mapOf(table)).length ? Object.values(mapOf(table)) : listOf(table);
  for (const mod of entries) {
    const m = mapOf(mod);
    const hashes = mapOf(m.tradeHashes);
    const modTags = listOf(m.modTags).filter((t) => typeof t === 'string');
    const lines = listOf(mod).filter((l) => typeof l === 'string');
    const hashList = Object.keys(hashes);
    // ModMaster não traz tradeHashes: casa pelo texto depois (via `byText`).
    const targets = hashList.length
      ? hashList.map((h) => ({ hash: h, text: listOf(hashes[h]).find((l) => typeof l === 'string') ?? lines[0] }))
      : lines.map((text) => ({ hash: null, text }));
    for (const { hash, text } of targets) {
      if (!text) continue;
      const range = averageRange(modRanges(text));
      if (!range) continue;
      const entry = {
        g: m.group ?? '',
        n: m.affix ?? '',
        t: typeof m.type === 'string' ? m.type : '',
        l: typeof m.level === 'number' ? m.level : 0,
        r: range,
        s: source,
      };
      if (source === 'explicit' || source === 'corrupted') {
        const keys = listOf(m.weightKey);
        const vals = listOf(m.weightVal);
        entry.w = keys.map((k, i) => [k, vals[i] ?? 0]);
      } else if (source === 'crafted') {
        entry.c = Object.keys(mapOf(m.types));
      }
      if (hash) hashByText[templateKey(text)] ??= hash;
      const key = hash ?? `text:${templateKey(text)}`;
      (stats[key] ??= []).push(entry);
      if (modTags.length) {
        // Tags de mods híbridos misturam os dois stats (vida + armadura): ficam só como reserva.
        const slot = targets.length === 1 ? 'pure' : 'hybrid';
        const bucket = (tags[key] ??= { pure: new Set(), hybrid: new Set() });
        modTags.forEach((t) => bucket[slot].add(t));
      }
    }
  }
}

/** Mesmo formato do `template_key` do Rust: números e faixas → "#", sem sinais, minúsculas. */
export function templateKey(text) {
  return text
    .replace(RANGE, '#')
    .replace(/(?<![A-Za-z0-9#])\d+(?:\.\d+)?/g, '#')
    .replace(/[+-]#/g, '#')
    .replace(/\s+/g, ' ')
    .trim()
    .toLowerCase();
}

/** template do texto de cada linha com hash → hash (para ligar os crafted, que vêm sem hash). */
const hashByText = {};

async function buildMods(src) {
  const stats = {};
  const tags = {};
  collectMods(parseReturn(await src.read('ModExplicit.lua')), 'explicit', stats, tags);
  collectMods(parseReturn(await src.read('ModCorrupted.lua')), 'corrupted', stats, tags);
  collectMods(parseReturn(await src.read('ModMaster.lua')), 'crafted', stats, tags);
  // Crafted sem hash: liga ao hash do explícito/corrompido de mesmo texto, quando houver.
  for (const key of Object.keys(stats)) {
    if (!key.startsWith('text:')) continue;
    const list = stats[key];
    const hash = hashByText[key.slice('text:'.length)];
    if (!hash) continue;
    stats[hash].push(...list);
    if (tags[key]) {
      const into = (tags[hash] ??= { pure: new Set(), hybrid: new Set() });
      tags[key].pure.forEach((t) => into.pure.add(t));
      tags[key].hybrid.forEach((t) => into.hybrid.add(t));
    }
    delete stats[key];
    delete tags[key];
  }
  const flatTags = {};
  for (const [key, { pure, hybrid }] of Object.entries(tags)) {
    flatTags[key] = [...(pure.size ? pure : hybrid)];
  }
  return { stats, tags: flatTags };
}

// ---------------------------------------------------------------- bases e gemas

async function buildBases(src) {
  const bases = {};
  for (const file of BASE_FILES) {
    let text;
    try {
      text = await src.read(`Bases/${file}.lua`);
    } catch {
      continue;
    }
    for (const [name, table] of Object.entries(parseAssignments(text, 'itemBases'))) {
      const b = mapOf(table);
      bases[name] = {
        type: b.type ?? '',
        ...(b.subType ? { sub: b.subType } : {}),
        tags: Object.entries(mapOf(b.tags)).filter(([, v]) => v === true).map(([k]) => k),
        ...(b.influenceTags ? { infl: mapOf(b.influenceTags) } : {}),
        ...(typeof b.socketLimit === 'number' ? { sockets: b.socketLimit } : {}),
      };
    }
  }
  return bases;
}

async function buildGems(src) {
  const gems = {};
  for (const gem of Object.values(mapOf(parseReturn(await src.read('Gems.lua'))))) {
    const g = mapOf(gem);
    if (typeof g.name !== 'string') continue;
    const tags = Object.entries(mapOf(g.tags)).filter(([, v]) => v === true).map(([k]) => k);
    gems[g.name] = tags;
  }
  return gems;
}

// ---------------------------------------------------------------- main

async function main() {
  const src = await makeSource(parseArgs());
  const meta = { source: src.version, generated: new Date().toISOString().slice(0, 10), license: 'Path of Building Community (MIT); item data (c) Grinding Gear Games' };
  await mkdir(OUT_DIR, { recursive: true });

  const write = async (name, data) => {
    const file = join(OUT_DIR, name);
    await writeFile(file, JSON.stringify({ meta, ...data }) + '\n', 'utf8');
    return file;
  };

  const bases = await buildBases(src);
  console.log(`bases: ${Object.keys(bases).length} → ${await write('bases-poe1.json', { bases })}`);

  const uniques = await buildUniques(src, new Set(Object.keys(bases)));
  console.log(`únicos: ${uniques.length} (${uniques.filter((u) => u.generated).length} sem dados) → ${await write('uniques-poe1.json', { uniques })}`);

  const { stats, tags } = await buildMods(src);
  console.log(`mods: ${Object.keys(stats).length} stats → ${await write('mods-poe1.json', { stats, tags })}`);

  const gems = await buildGems(src);
  console.log(`gemas: ${Object.keys(gems).length} → ${await write('gems-poe1.json', { gems })}`);
  console.log(`fonte: ${src.version}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
