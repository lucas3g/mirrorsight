/** Helpers de apresentação. Sem regra de negócio — só formatação para a UI. */
import type { Influence, ModifierKind, Rarity, StatRoll } from '$lib/types';
import { i18n } from '$lib/i18n/core.svelte';
import { common } from '$lib/i18n/messages/common';

const numberFormats = new Map<string, { small: Intl.NumberFormat; large: Intl.NumberFormat }>();

function formatsFor(locale: string) {
  let f = numberFormats.get(locale);
  if (!f) {
    f = {
      small: new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }),
      large: new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }),
    };
    numberFormats.set(locale, f);
  }
  return f;
}

/** Número no formato do idioma da interface (reativo ao idioma). */
export function formatAmount(n: number): string {
  const f = formatsFor(i18n.locale);
  return Math.abs(n) >= 1000 ? f.large.format(n) : f.small.format(n);
}

/** Mapa por tipo de mod que sempre lê do idioma atual. */
function kindMap(pick: () => Record<ModifierKind, string>): Record<ModifierKind, string> {
  return new Proxy({} as Record<ModifierKind, string>, { get: (_t, k) => pick()[k as ModifierKind] });
}

/** "2m", "3h", "1d" a partir de um ISO-8601. */
export function relativeAge(iso: string, now = Date.now()): string {
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return '—';
  const minutes = Math.max(0, (now - t) / 60_000);
  if (minutes < 1) return common.now;
  if (minutes < 60) return `${Math.floor(minutes)}m`;
  const hours = minutes / 60;
  if (hours < 24) return `${Math.floor(hours)}h`;
  return `${Math.floor(hours / 24)}d`;
}

export interface TextPart {
  text: string;
  numeric: boolean;
}

/** Separa números do texto de um mod para destacá-los sem usar {@html}. */
export function splitNumbers(text: string): TextPart[] {
  return text
    .split(/([+-]?\d+(?:[.,]\d+)?)/)
    .filter((p) => p.length > 0)
    .map((p) => ({ text: p, numeric: /^[+-]?\d/.test(p) }));
}

/** Posição do valor rolado dentro do tier (0–100), ou `null` se a faixa é desconhecida. */
export function rollPercent(roll: StatRoll | null): number | null {
  if (!roll || roll.min === null || roll.max === null) return null;
  if (roll.max === roll.min) return 100;
  return Math.min(100, Math.max(0, ((roll.value - roll.min) / (roll.max - roll.min)) * 100));
}

export function rollRange(roll: StatRoll | null): string | null {
  if (!roll || roll.min === null || roll.max === null) return null;
  return `${formatAmount(roll.min)}–${formatAmount(roll.max)}`;
}

export const RARITY_COLOR: Record<Rarity, string> = {
  normal: 'var(--ms-normal)',
  magic: 'var(--ms-magic)',
  rare: 'var(--ms-rare)',
  unique: 'var(--ms-unique)',
  gem: 'var(--ms-gem)',
  currency: 'var(--ms-currency)',
  divinationcard: 'var(--ms-divination)',
  other: 'var(--ms-text)',
};

export const KIND_LABEL = kindMap(() => common.kindShort);

export const INFLUENCE_LABEL: Record<Influence, string> = {
  shaper: 'Shaper',
  elder: 'Elder',
  crusader: 'Crusader',
  hunter: 'Hunter',
  redeemer: 'Redeemer',
  warlord: 'Warlord',
  searing: 'Searing Exarch',
  eater: 'Eater of Worlds',
};

/** Cores dos sockets no texto do jogo (R/G/B/W, A = abyss, S = PoE2). */
const SOCKET_COLOR: Record<string, string> = {
  R: '#e04a4a',
  G: '#4ac46a',
  B: '#5a8cff',
  W: '#e8e8e8',
  A: '#5c5c5c',
  D: '#c8a2ff',
  S: '#a3a6ae',
};

export interface SocketGroup {
  colors: string[];
}

/** "B-B-R G" → grupos ligados com as cores de cada socket. */
export function parseSockets(raw: string | null): SocketGroup[] {
  if (!raw) return [];
  return raw
    .trim()
    .split(/\s+/)
    .map((group) => ({ colors: group.split('-').map((s) => SOCKET_COLOR[s] ?? '#888') }));
}

export function maxLinks(raw: string | null): number {
  return Math.max(0, ...parseSockets(raw).map((g) => g.colors.length));
}

/** Rótulos no singular, para badges. */
export const KIND_NAME = kindMap(() => common.kindName);

/** Títulos dos grupos no editor de mods. */
export const KIND_LABEL_PLURAL = kindMap(() => common.kindPlural);

/** Ordem dos grupos no editor de mods. */
export const KIND_ORDER: ModifierKind[] = ['property', 'explicit', 'fractured', 'implicit', 'crafted', 'enchant', 'rune', 'desecrated', 'pseudo'];

export const KIND_COLOR: Record<ModifierKind, string> = {
  explicit: 'var(--ms-rare)',
  fractured: 'var(--ms-fractured)',
  implicit: 'var(--ms-implicit)',
  crafted: 'var(--ms-crafted)',
  enchant: 'var(--ms-enchant)',
  rune: 'var(--ms-text-2)',
  desecrated: '#e07a9a',
  pseudo: 'var(--ms-pseudo)',
  property: 'var(--ms-text)',
};

/** "há 4 min", "há 3 h", "ontem", "23/11" — no idioma da interface. */
export function relativeWhen(ms: number, now = Date.now()): string {
  const minutes = Math.max(0, (now - ms) / 60_000);
  if (minutes < 1) return common.now;
  if (minutes < 60) return common.minutesAgo(Math.floor(minutes));
  const then = new Date(ms);
  const today = new Date(now);
  const startOfToday = new Date(today.getFullYear(), today.getMonth(), today.getDate()).getTime();
  if (ms >= startOfToday) return common.hoursAgo(Math.floor(minutes / 60));
  if (ms >= startOfToday - 86_400_000) return common.yesterday;
  return then.toLocaleDateString(i18n.locale, { day: '2-digit', month: '2-digit' });
}

/** Minúsculas e sem acentos, para buscas ("historico" acha "Histórico"). */
export function normalize(s: string): string {
  return s.toLowerCase().normalize('NFD').replace(/[̀-ͯ]/g, '');
}

export interface Highlighted {
  text: string;
  hit: boolean;
}

/** Divide `text` destacando a primeira ocorrência de `query` (sem acento), sem {@html}. */
export function highlight(text: string, query: string): Highlighted[] {
  const q = normalize(query.trim());
  if (!q) return [{ text, hit: false }];
  // Normaliza caractere a caractere para mapear a posição achada de volta ao texto original.
  const folded = [...text].map((c) => normalize(c));
  const joined = folded.join('');
  const at = joined.indexOf(q);
  if (at < 0) return [{ text, hit: false }];
  // Converte o índice da string normalizada para o índice na original.
  let pos = 0;
  let start = -1;
  let end = -1;
  const chars = [...text];
  for (let i = 0; i < chars.length; i++) {
    if (pos === at && start < 0) start = i;
    pos += folded[i]!.length;
    if (start >= 0 && pos >= at + q.length) {
      end = i + 1;
      break;
    }
  }
  if (start < 0 || end < 0) return [{ text, hit: false }];
  return [
    { text: chars.slice(0, start).join(''), hit: false },
    { text: chars.slice(start, end).join(''), hit: true },
    { text: chars.slice(end).join(''), hit: false },
  ].filter((p) => p.text.length > 0);
}
