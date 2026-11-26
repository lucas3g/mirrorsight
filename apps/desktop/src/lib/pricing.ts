/**
 * Exibição de preços e do preço sugerido. O cálculo dos percentis mora no Rust
 * (`crates/domain/src/price.rs`, que conhece as cotações de todas as moedas);
 * aqui só se escolhe a moeda de exibição e se monta a nota `~b/o`.
 */
import { currencyName } from '$lib/currency';
import { priceCheckMsgs } from '$lib/i18n/messages/priceCheck';
import type { IndexedPrice, PriceCheckOutput, PriceSummary, SuggestedLevel } from '$lib/types';

export interface DisplayAmount {
  amount: number;
  currency: 'chaos' | 'divine';
}

/**
 * Valor em chaos na moeda preferida. Acima de 1 divine mostra divine ("58 div" lê melhor
 * que "12.400c") — exceto quando o próprio item é a Divine Orb (`itemName`).
 */
export function displayAmount(
  chaos: number,
  divineRate: number | null,
  preferDivine: boolean,
  itemName?: string,
): DisplayAmount {
  const isDivineItem = itemName === currencyName('divine');
  if (divineRate && !isDivineItem && (preferDivine || chaos >= divineRate)) {
    return { amount: chaos / divineRate, currency: 'divine' };
  }
  return { amount: chaos, currency: 'chaos' };
}

/** Cotação do divine (chaos por divine) derivada do próprio preço do poe.ninja. */
export function ninjaDivineRate(indexed: IndexedPrice | null | undefined): number | null {
  return indexed?.divine_value ? indexed.chaos_value / indexed.divine_value : null;
}

/** Cotação do divine das listagens; sem elas (trade falhou), a do poe.ninja. */
export function divineRateOf(output: PriceCheckOutput | null | undefined): number | null {
  return output?.summary?.divine_chaos ?? ninjaDivineRate(output?.indexed);
}

/**
 * Imagem do item. O backend já escolhe (`output.icon`): poe.ninja → listagem da mesma base →
 * arte genérica da base. O resto é só reserva para respostas sem esse campo (modo demo).
 */
export function itemIconOf(output: PriceCheckOutput | null | undefined): string | null {
  if (!output) return null;
  if (output.icon) return output.icon;
  if (output.indexed?.icon) return output.indexed.icon;
  const base = output.item.base_type;
  return output.trade?.listings.find((l) => l.icon && l.item?.base_type === base)?.icon ?? null;
}

export const LEVELS: SuggestedLevel[] = ['buy', 'fair', 'sell'];

export interface LevelMeta {
  name: string;
  tag: string;
  hint: string;
  /** Frase da linha de dica, com o valor já formatado. */
  sentence: (value: string) => string;
}

const LEVEL_TAG: Record<SuggestedLevel, string> = { buy: 'p25', fair: 'p50', sell: 'p75' };

/** Textos no idioma atual (getters: reativo em templates/`$derived`). */
export const LEVEL_META: Record<SuggestedLevel, LevelMeta> = {
  get buy() {
    return { ...priceCheckMsgs.levels.buy, tag: LEVEL_TAG.buy };
  },
  get fair() {
    return { ...priceCheckMsgs.levels.fair, tag: LEVEL_TAG.fair };
  },
  get sell() {
    return { ...priceCheckMsgs.levels.sell, tag: LEVEL_TAG.sell };
  },
};

/** Valor em chaos do nível escolhido; `null` sem amostra suficiente. */
export function suggestedFor(summary: PriceSummary | null | undefined, level: SuggestedLevel): number | null {
  const s = summary?.suggested;
  if (!s) return null;
  return level === 'buy' ? s.buy_chaos : level === 'sell' ? s.sell_chaos : s.fair_chaos;
}

/**
 * Nota de preço no formato que o jogo entende: "~b/o 4.2 divine" / "~b/o 48 chaos".
 * Ponto decimal (não vírgula); divine com 1 casa, chaos inteiro (mínimo 1).
 */
export function buyoutNote(chaos: number, divineRate: number | null, preferDivine: boolean): string {
  const shown = displayAmount(chaos, divineRate, preferDivine);
  if (shown.currency === 'divine') {
    const amount = Math.round(shown.amount * 10) / 10;
    // Abaixo de 0.05 div o arredondamento daria 0 (ou forçaria 0.1, várias vezes o preço): usa chaos.
    if (amount > 0) return `~b/o ${String(amount)} divine`;
  }
  return `~b/o ${String(Math.max(1, Math.round(chaos)))} chaos`;
}
