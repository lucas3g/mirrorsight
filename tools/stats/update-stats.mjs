#!/usr/bin/env node
/**
 * Baixa a lista oficial de stats da trade API e gera os snapshots embutidos no parser:
 *   crates/parser/data/stats-poe1.tsv
 *   crates/parser/data/stats-poe2.tsv
 *
 * Formato (UTF-8, uma linha por stat):  grupo \t id \t texto
 * Quebras de linha do texto viram a sequência literal "\n".
 *
 * Uso:  node tools/stats/update-stats.mjs
 * Rode a cada liga nova (ou quando o parser não reconhecer mods novos).
 */
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const OUT_DIR = join(ROOT, 'crates', 'parser', 'data');

// A GGG pede um User-Agent identificável para clientes automatizados.
const USER_AGENT = 'Mirrorsight/0.1 (open-source PoE price checker; stats snapshot)';

/** Grupos que o app entende (ver `ModifierKind`); `pseudo` alimenta a busca "Adicionar mod". Os demais são ligas antigas/monstros. */
const GROUPS = new Set(['explicit', 'implicit', 'fractured', 'crafted', 'enchant', 'rune', 'desecrated', 'pseudo']);

const GAMES = [
  { name: 'poe1', url: 'https://www.pathofexile.com/api/trade/data/stats' },
  { name: 'poe2', url: 'https://www.pathofexile.com/api/trade2/data/stats' },
];

async function build({ name, url }) {
  const res = await fetch(url, { headers: { 'User-Agent': USER_AGENT, Accept: 'application/json' } });
  if (!res.ok) throw new Error(`${name}: HTTP ${res.status}`);
  const { result } = await res.json();

  const seen = new Set();
  const lines = [];
  let duplicates = 0;
  let withOptions = 0;

  for (const group of result) {
    if (!GROUPS.has(group.id)) continue;
    for (const entry of group.entries) {
      // Stats com "option" (ex.: "Grants Level # Aspect") exigem um valor extra na busca: fora por ora.
      if (entry.option) {
        withOptions++;
        continue;
      }
      const text = entry.text.replaceAll('\r', '').replaceAll('\n', '\\n');
      const key = `${group.id}\t${text}`;
      if (seen.has(key)) {
        duplicates++;
        continue;
      }
      seen.add(key);
      lines.push(`${group.id}\t${entry.id}\t${text}`);
    }
  }

  const file = join(OUT_DIR, `stats-${name}.tsv`);
  await writeFile(file, lines.join('\n') + '\n', 'utf8');
  console.log(`${name}: ${lines.length} stats (${duplicates} duplicados, ${withOptions} com opção ignorados) → ${file}`);
}

/**
 * Bases com discriminador (gemas transfiguradas): o jogo mostra "Kinetic Blast of Clustering",
 * mas a trade API busca por type "Kinetic Blast" + discriminator "alt_x".
 * Formato: texto \t type \t disc   →  crates/parser/data/bases-<jogo>.tsv
 */
async function buildBases({ name, url }) {
  const itemsUrl = url.replace(/\/stats$/, '/items');
  const res = await fetch(itemsUrl, { headers: { 'User-Agent': USER_AGENT, Accept: 'application/json' } });
  if (!res.ok) throw new Error(`${name} items: HTTP ${res.status}`);
  const { result } = await res.json();
  const lines = new Set();
  for (const group of result) {
    for (const entry of group.entries) {
      if (!entry.disc || !entry.text || !entry.type) continue;
      // Gemas transfiguradas (alt_x/alt_y/…) e mapas blighted: "Blighted Map (Strand)" → type "10021".
      // O parser monta essa chave a partir de "Blighted Map (Tier 13)" + "Map Area: Strand".
      const blightedMap = group.id === 'map' && (entry.disc === 'blighted' || entry.disc === 'uberblighted');
      if (group.id === 'gem' || blightedMap) lines.add(`${entry.text}\t${entry.type}\t${entry.disc}`);
    }
  }
  const file = join(OUT_DIR, `bases-${name}.tsv`);
  await writeFile(file, [...lines].join('\n') + '\n', 'utf8');
  console.log(`${name}: ${lines.size} bases com discriminador → ${file}`);
}

/**
 * Viscera (Covens, partes do Wickerman): o jogo mostra "Greater Carved Haunches", a trade API
 * só conhece o type "Carved Haunches". Uma base por linha → crates/parser/data/viscera-<jogo>.txt
 */
async function buildViscera({ name, url }) {
  const itemsUrl = url.replace(/\/stats$/, '/items');
  const res = await fetch(itemsUrl, { headers: { 'User-Agent': USER_AGENT, Accept: 'application/json' } });
  if (!res.ok) throw new Error(`${name} items: HTTP ${res.status}`);
  const { result } = await res.json();
  const types = new Set();
  for (const group of result) {
    if (group.id !== 'viscera') continue;
    for (const entry of group.entries) if (entry.type && !entry.name) types.add(entry.type);
  }
  const file = join(OUT_DIR, `viscera-${name}.txt`);
  await writeFile(file, [...types].sort().join('\n') + '\n', 'utf8');
  console.log(`${name}: ${types.size} bases de viscera → ${file}`);
}

await mkdir(OUT_DIR, { recursive: true });
for (const game of GAMES) {
  await build(game);
  await buildBases(game);
  await buildViscera(game);
}
