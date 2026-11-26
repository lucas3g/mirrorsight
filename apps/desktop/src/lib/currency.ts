/**
 * Resolução de ícones de currency.
 *
 * Os PNGs vêm da instalação do jogo (Bundles2) e ficam em `public/assets/currency/`,
 * com `manifest.json` mapeando nome do item → arquivo. A trade API usa tags curtas
 * ("chaos", "divine"…), traduzidas aqui para o nome do item.
 */

const TAG_TO_NAME: Record<string, string> = {
  chaos: 'Chaos Orb',
  divine: 'Divine Orb',
  exalted: 'Exalted Orb',
  mirror: 'Mirror of Kalandra',
  alch: 'Orb of Alchemy',
  vaal: 'Vaal Orb',
  alt: 'Orb of Alteration',
  chrome: 'Chromatic Orb',
  fusing: 'Orb of Fusing',
  jewellers: "Jeweller's Orb",
  regal: 'Regal Orb',
  chance: 'Orb of Chance',
  scour: 'Orb of Scouring',
  regret: 'Orb of Regret',
  annul: 'Orb of Annulment',
  blessed: 'Blessed Orb',
  gcp: "Gemcutter's Prism",
  chisel: "Cartographer's Chisel",
  aug: 'Orb of Augmentation',
  transmute: 'Orb of Transmutation',
  wisdom: 'Scroll of Wisdom',
  portal: 'Portal Scroll',
  ancient: 'Ancient Orb',
};

interface ManifestEntry {
  id: string;
  icon: string;
}

let manifest: Record<string, ManifestEntry> = {};

/** Carrega o manifest uma vez, antes de montar a UI (arquivo local, poucos KB). */
export async function preloadCurrencyManifest(): Promise<void> {
  try {
    const res = await fetch('/assets/currency/manifest.json');
    if (res.ok) manifest = (await res.json()) as Record<string, ManifestEntry>;
  } catch {
    manifest = {};
  }
}

export function currencyName(tagOrName: string): string {
  return TAG_TO_NAME[tagOrName] ?? tagOrName;
}

/** URL do ícone para uma tag da trade API ou nome do item; `null` se desconhecido. */
export function currencyIconUrl(tagOrName: string): string | null {
  const entry = manifest[currencyName(tagOrName)];
  return entry ? `/assets/${entry.icon}` : null;
}
