import type { Rarity, SuggestedLevel } from '$lib/types';
import { defineMessages } from '../core.svelte';

interface LevelText {
  name: string;
  hint: string;
  /** Frase da linha de dica, com o valor já formatado. */
  sentence: (value: string) => string;
}

interface PriceCheckMessages {
  // ---------- Ações do painel ----------
  closeEsc: string;
  close: string;
  retry: string;
  pinPanel: string;
  pinnedToast: string;
  unpinnedToast: string;
  openTrade: string;
  dragToMove: string;
  itemAria: string;
  priceAria: string;
  /** Nome da raridade no cartão do item (currency/divination card ficam como no jogo). */
  rarity: Record<Rarity, string>;
  quality: string;
  corrupted: string;
  unidentified: string;
  advancedCopy: string;

  // ---------- Preço ----------
  estimatedPrice: string;
  /** "Confiança baixa" (cartão de preço). */
  confidence: (level: number) => string;
  /** Plural de "anúncios" depois do número em negrito. */
  listingsUnit: (n: number) => string;
  /** Linha sob o preço no overlay. */
  estimateLine: (chaos: string, confidence: number, n: number) => string;
  ninjaPriceNoListings: string;
  ninjaNoListings: string;
  noPricedListings: string;
  ninjaReference: string;
  min: string;
  median: string;
  max: string;
  /** Marcador curto da mediana no histograma. */
  medianShort: string;

  // ---------- Filtros / rodapé ----------
  sellers: string;
  online: string;
  any: string;
  leagueHint: string;
  filtersChanged: string;
  relax10: string;
  searching: string;

  // ---------- Mercado ----------
  market: string;
  listings: string;
  distribution: string;
  scopeOnline: string;
  scopeAll: string;
  colPrice: string;
  colSeller: string;
  colListed: string;
  colAge: string;
  noListings: string;
  tradeFailed: string;
  noneFound: string;
  noneFoundHint: string;
  relaxFilters10: string;
  staleFilters: string;
  histogramAria: string;
  listingsKpi: string;
  noDistData: string;
  whisperCopied: (account: string) => string;
  whisperFailed: string;

  // ---------- Linha / card da listagem ----------
  cheapest: string;
  viewListedItem: string;
  viewItem: string;
  copyWhisper: string;
  copyWhisperFor: (account: string) => string;
  matchesFilters: string;
  /** Recebe `relativeAge` ("5m", "2h"…). */
  listedAgo: (age: string) => string;
  listedNow: string;

  // ---------- Preço sugerido ----------
  suggestedPrice: string;
  percentilesOf: (n: number) => string;
  levelAria: string;
  copyNote: string;
  tooFew: (n: number) => string;
  levels: Record<SuggestedLevel, LevelText>;

  // ---------- Ocioso ----------
  readyTitle: string;
  /** Texto antes/depois da tecla de atalho. */
  leadBefore: string;
  leadAfter: string;
  pasteBefore: string;
  pasteAfter: string;
  recent: string;
  reopenSame: string;
  demoMode: string;
  demoRare: string;
  demoFew: string;
  demoNetwork: string;
  demoUnrecognized: string;

  // ---------- Quick price ----------
  ninja7d: string;
  seeListings: string;
  noReference: string;
  openFull: string;

  // ---------- Estados de falha ----------
  tooManyTitle: string;
  tooManyText: string;
  noConnectionTitle: string;
  notRecognizedTitle: string;
  couldNotFinish: string;
  seconds: (n: number) => string;
}

const PT_CONF = ['nenhuma', 'baixa', 'média', 'alta'];
const EN_CONF = ['No', 'Low', 'Medium', 'High'];
const ES_CONF = ['ninguna', 'baja', 'media', 'alta'];
const DE_CONF = ['keine', 'niedrig', 'mittel', 'hoch'];
const ZH_CONF = ['无', '低', '中', '高'];
const JA_CONF = ['なし', '低', '中', '高'];
const KO_CONF = ['없음', '낮음', '보통', '높음'];

/** Painel de price check (janela principal e overlay), quick price e estados de falha. */
export const priceCheckMsgs = defineMessages<PriceCheckMessages>({
  'pt-BR': {
    closeEsc: 'Fechar (Esc)',
    close: 'Fechar',
    retry: 'Tentar novamente',
    pinPanel: 'Fixar painel',
    pinnedToast: 'Painel fixado — não fecha ao voltar ao jogo',
    unpinnedToast: 'Painel desafixado',
    openTrade: 'Abrir no site de trade',
    dragToMove: 'Arraste para mover',
    itemAria: 'Item',
    priceAria: 'Preço',
    rarity: { normal: 'Normal', magic: 'Mágico', rare: 'Raro', unique: 'Único', gem: 'Gema', currency: 'Currency', divinationcard: 'Divination Card', other: 'Item' },
    quality: 'Qualidade',
    corrupted: 'Corrompido',
    unidentified: 'Não identificado',
    advancedCopy: 'Item copiado no modo avançado',

    estimatedPrice: 'Preço estimado',
    confidence: (c) => `Confiança ${PT_CONF[c] ?? '—'}`,
    listingsUnit: () => 'anúncios',
    estimateLine: (chaos, c, n) => `≈ ${chaos} c · confiança ${PT_CONF[c] ?? '—'} · ${n} anúncios`,
    ninjaPriceNoListings: 'preço do poe.ninja · sem listagens',
    ninjaNoListings: 'poe.ninja · sem listagens',
    noPricedListings: 'sem listagens precificadas',
    ninjaReference: 'Preço de referência do poe.ninja',
    min: 'Mínimo',
    median: 'Mediana',
    max: 'Máximo',
    medianShort: 'med',

    sellers: 'Vendedores',
    online: 'Online',
    any: 'Qualquer',
    leagueHint: 'Liga (altere na janela principal)',
    filtersChanged: 'Filtros alterados',
    relax10: 'Relaxar −10%',
    searching: 'Buscando…',

    market: 'Mercado',
    listings: 'Listagens',
    distribution: 'Distribuição',
    scopeOnline: 'só online',
    scopeAll: 'todos',
    colPrice: 'Preço',
    colSeller: 'Vendedor',
    colListed: 'Listado',
    colAge: 'Idade',
    noListings: 'Sem listagens',
    tradeFailed: 'A trade falhou. Mostrando só o preço do poe.ninja.',
    noneFound: 'Nenhuma listagem encontrada',
    noneFoundHint: 'Desligue alguns mods ou reduza os mínimos para ampliar a busca.',
    relaxFilters10: 'Relaxar filtros 10%',
    staleFilters: '● Filtros alterados — buscando em instantes…',
    histogramAria: 'Distribuição de preços',
    listingsKpi: 'Anúncios',
    noDistData: 'Sem dados para a distribuição',
    whisperCopied: (a) => `Whisper para ${a} copiado`,
    whisperFailed: 'Não foi possível copiar o whisper',

    cheapest: 'MAIS BARATO',
    viewListedItem: 'Ver item listado',
    viewItem: 'Ver item',
    copyWhisper: 'Copiar whisper',
    copyWhisperFor: (a) => `Copiar whisper para ${a}`,
    matchesFilters: 'bate com seus filtros',
    listedAgo: (age) => `listado há ${age}`,
    listedNow: 'listado agora',

    suggestedPrice: 'Preço sugerido',
    percentilesOf: (n) => `· percentis dos ${n} anúncios`,
    levelAria: 'Nível do preço sugerido',
    copyNote: 'Copiar nota de preço',
    tooFew: (n) => `Só ${n} ${n === 1 ? 'anúncio' : 'anúncios'} — poucos para sugerir preço.`,
    levels: {
      buy: { name: 'Compra', hint: 'Pague até isso', sentence: (v) => `25% dos anúncios custam até ${v} — bom teto para comprar.` },
      fair: { name: 'Justo', hint: 'Mediana do mercado', sentence: (v) => `Metade dos anúncios custa até ${v} — vende em algumas horas.` },
      sell: { name: 'Venda', hint: 'Liste e espere', sentence: (v) => `75% dos anúncios custam até ${v} — vende mais devagar.` },
    },

    readyTitle: 'Pronto para precificar',
    leadBefore: 'No jogo, passe o mouse sobre um item e pressione',
    leadAfter: '',
    pasteBefore: 'Sem o jogo? Cole o item com',
    pasteAfter: '',
    recent: 'Recentes',
    reopenSame: 'Reabrir com os mesmos filtros',
    demoMode: 'Modo demonstração',
    demoRare: 'Item raro',
    demoFew: 'Poucos anúncios',
    demoNetwork: 'Erro de rede',
    demoUnrecognized: 'Não reconhecido',

    ninja7d: 'poe.ninja · 7 dias',
    seeListings: 'Ver listagens →',
    noReference: 'Sem preço de referência no poe.ninja para este item.',
    openFull: 'Abrir price check completo',

    tooManyTitle: 'Muitas buscas seguidas',
    tooManyText: 'A Trade API pediu uma pausa. A busca volta sozinha quando o contador zerar.',
    noConnectionTitle: 'Sem conexão com o trade',
    notRecognizedTitle: 'Item não reconhecido',
    couldNotFinish: 'Não foi possível concluir',
    seconds: (n) => `${n} segundos`,
  },
  en: {
    closeEsc: 'Close (Esc)',
    close: 'Close',
    retry: 'Try again',
    pinPanel: 'Pin panel',
    pinnedToast: "Panel pinned — it won't close when you return to the game",
    unpinnedToast: 'Panel unpinned',
    openTrade: 'Open on the trade site',
    dragToMove: 'Drag to move',
    itemAria: 'Item',
    priceAria: 'Price',
    rarity: { normal: 'Normal', magic: 'Magic', rare: 'Rare', unique: 'Unique', gem: 'Gem', currency: 'Currency', divinationcard: 'Divination Card', other: 'Item' },
    quality: 'Quality',
    corrupted: 'Corrupted',
    unidentified: 'Unidentified',
    advancedCopy: 'Item copied in advanced mode',

    estimatedPrice: 'Estimated price',
    confidence: (c) => `${EN_CONF[c] ?? '—'} confidence`,
    listingsUnit: (n) => (n === 1 ? 'listing' : 'listings'),
    estimateLine: (chaos, c, n) => `≈ ${chaos} c · ${(EN_CONF[c] ?? '—').toLowerCase()} confidence · ${n} ${n === 1 ? 'listing' : 'listings'}`,
    ninjaPriceNoListings: 'poe.ninja price · no listings',
    ninjaNoListings: 'poe.ninja · no listings',
    noPricedListings: 'no priced listings',
    ninjaReference: 'poe.ninja reference price',
    min: 'Minimum',
    median: 'Median',
    max: 'Maximum',
    medianShort: 'med',

    sellers: 'Sellers',
    online: 'Online',
    any: 'Any',
    leagueHint: 'League (change it in the main window)',
    filtersChanged: 'Filters changed',
    relax10: 'Relax −10%',
    searching: 'Searching…',

    market: 'Market',
    listings: 'Listings',
    distribution: 'Distribution',
    scopeOnline: 'online only',
    scopeAll: 'all',
    colPrice: 'Price',
    colSeller: 'Seller',
    colListed: 'Listed',
    colAge: 'Age',
    noListings: 'No listings',
    tradeFailed: 'The trade search failed. Showing the poe.ninja price only.',
    noneFound: 'No listings found',
    noneFoundHint: 'Turn off some mods or lower the minimums to widen the search.',
    relaxFilters10: 'Relax filters 10%',
    staleFilters: '● Filters changed — searching shortly…',
    histogramAria: 'Price distribution',
    listingsKpi: 'Listings',
    noDistData: 'No data for the distribution',
    whisperCopied: (a) => `Whisper to ${a} copied`,
    whisperFailed: "Couldn't copy the whisper",

    cheapest: 'CHEAPEST',
    viewListedItem: 'View listed item',
    viewItem: 'View item',
    copyWhisper: 'Copy whisper',
    copyWhisperFor: (a) => `Copy whisper to ${a}`,
    matchesFilters: 'matches your filters',
    listedAgo: (age) => `listed ${age} ago`,
    listedNow: 'listed just now',

    suggestedPrice: 'Suggested price',
    percentilesOf: (n) => `· percentiles of ${n} ${n === 1 ? 'listing' : 'listings'}`,
    levelAria: 'Suggested price level',
    copyNote: 'Copy price note',
    tooFew: (n) => `Only ${n} ${n === 1 ? 'listing' : 'listings'} — too few to suggest a price.`,
    levels: {
      buy: { name: 'Buy', hint: 'Pay up to this', sentence: (v) => `25% of listings cost up to ${v} — a good buying ceiling.` },
      fair: { name: 'Fair', hint: 'Market median', sentence: (v) => `Half of the listings cost up to ${v} — sells within a few hours.` },
      sell: { name: 'Sell', hint: 'List and wait', sentence: (v) => `75% of listings cost up to ${v} — sells more slowly.` },
    },

    readyTitle: 'Ready to price check',
    leadBefore: 'In game, hover an item and press',
    leadAfter: '',
    pasteBefore: 'No game? Paste the item with',
    pasteAfter: '',
    recent: 'Recent',
    reopenSame: 'Reopen with the same filters',
    demoMode: 'Demo mode',
    demoRare: 'Rare item',
    demoFew: 'Few listings',
    demoNetwork: 'Network error',
    demoUnrecognized: 'Not recognized',

    ninja7d: 'poe.ninja · 7 days',
    seeListings: 'See listings →',
    noReference: 'No poe.ninja reference price for this item.',
    openFull: 'Open full price check',

    tooManyTitle: 'Too many searches in a row',
    tooManyText: 'The Trade API asked for a break. The search resumes on its own when the timer runs out.',
    noConnectionTitle: 'No connection to trade',
    notRecognizedTitle: 'Item not recognized',
    couldNotFinish: "Couldn't finish",
    seconds: (n) => `${n} seconds`,
  },
  es: {
    closeEsc: 'Cerrar (Esc)',
    close: 'Cerrar',
    retry: 'Reintentar',
    pinPanel: 'Fijar panel',
    pinnedToast: 'Panel fijado — no se cierra al volver al juego',
    unpinnedToast: 'Panel desfijado',
    openTrade: 'Abrir en el sitio de trade',
    dragToMove: 'Arrastra para mover',
    itemAria: 'Objeto',
    priceAria: 'Precio',
    rarity: { normal: 'Normal', magic: 'Mágico', rare: 'Raro', unique: 'Único', gem: 'Gema', currency: 'Currency', divinationcard: 'Divination Card', other: 'Objeto' },
    quality: 'Calidad',
    corrupted: 'Corrupted',
    unidentified: 'Sin identificar',
    advancedCopy: 'Objeto copiado en modo avanzado',

    estimatedPrice: 'Precio estimado',
    confidence: (c) => `Confianza ${ES_CONF[c] ?? '—'}`,
    listingsUnit: (n) => (n === 1 ? 'anuncio' : 'anuncios'),
    estimateLine: (chaos, c, n) => `≈ ${chaos} c · confianza ${ES_CONF[c] ?? '—'} · ${n} ${n === 1 ? 'anuncio' : 'anuncios'}`,
    ninjaPriceNoListings: 'precio de poe.ninja · sin anuncios',
    ninjaNoListings: 'poe.ninja · sin anuncios',
    noPricedListings: 'sin anuncios con precio',
    ninjaReference: 'Precio de referencia de poe.ninja',
    min: 'Mínimo',
    median: 'Mediana',
    max: 'Máximo',
    medianShort: 'med',

    sellers: 'Vendedores',
    online: 'Online',
    any: 'Todos',
    leagueHint: 'Liga (cámbiala en la ventana principal)',
    filtersChanged: 'Filtros modificados',
    relax10: 'Relajar −10%',
    searching: 'Buscando…',

    market: 'Mercado',
    listings: 'Anuncios',
    distribution: 'Distribución',
    scopeOnline: 'solo online',
    scopeAll: 'todos',
    colPrice: 'Precio',
    colSeller: 'Vendedor',
    colListed: 'Publicado',
    colAge: 'Edad',
    noListings: 'Sin anuncios',
    tradeFailed: 'La búsqueda en trade falló. Solo se muestra el precio de poe.ninja.',
    noneFound: 'No se encontraron anuncios',
    noneFoundHint: 'Desactiva algunos mods o baja los mínimos para ampliar la búsqueda.',
    relaxFilters10: 'Relajar filtros 10%',
    staleFilters: '● Filtros modificados — buscando en un momento…',
    histogramAria: 'Distribución de precios',
    listingsKpi: 'Anuncios',
    noDistData: 'Sin datos para la distribución',
    whisperCopied: (a) => `Whisper para ${a} copiado`,
    whisperFailed: 'No se pudo copiar el whisper',

    cheapest: 'MÁS BARATO',
    viewListedItem: 'Ver objeto anunciado',
    viewItem: 'Ver objeto',
    copyWhisper: 'Copiar whisper',
    copyWhisperFor: (a) => `Copiar whisper para ${a}`,
    matchesFilters: 'coincide con tus filtros',
    listedAgo: (age) => `publicado hace ${age}`,
    listedNow: 'publicado ahora',

    suggestedPrice: 'Precio sugerido',
    percentilesOf: (n) => `· percentiles de ${n} ${n === 1 ? 'anuncio' : 'anuncios'}`,
    levelAria: 'Nivel del precio sugerido',
    copyNote: 'Copiar nota de precio',
    tooFew: (n) => `Solo ${n} ${n === 1 ? 'anuncio' : 'anuncios'}: muy pocos para sugerir un precio.`,
    levels: {
      buy: { name: 'Compra', hint: 'Paga hasta esto', sentence: (v) => `El 25% de los anuncios cuesta hasta ${v}: buen tope para comprar.` },
      fair: { name: 'Justo', hint: 'Mediana del mercado', sentence: (v) => `La mitad de los anuncios cuesta hasta ${v}: se vende en unas horas.` },
      sell: { name: 'Venta', hint: 'Publica y espera', sentence: (v) => `El 75% de los anuncios cuesta hasta ${v}: se vende más lento.` },
    },

    readyTitle: 'Listo para consultar precios',
    leadBefore: 'En el juego, pasa el ratón sobre un objeto y pulsa',
    leadAfter: '',
    pasteBefore: '¿Sin el juego? Pega el objeto con',
    pasteAfter: '',
    recent: 'Recientes',
    reopenSame: 'Reabrir con los mismos filtros',
    demoMode: 'Modo demo',
    demoRare: 'Objeto raro',
    demoFew: 'Pocos anuncios',
    demoNetwork: 'Error de red',
    demoUnrecognized: 'No reconocido',

    ninja7d: 'poe.ninja · 7 días',
    seeListings: 'Ver anuncios →',
    noReference: 'No hay precio de referencia en poe.ninja para este objeto.',
    openFull: 'Abrir consulta completa',

    tooManyTitle: 'Demasiadas búsquedas seguidas',
    tooManyText: 'La Trade API pidió una pausa. La búsqueda se reanuda sola cuando el contador llegue a cero.',
    noConnectionTitle: 'Sin conexión con trade',
    notRecognizedTitle: 'Objeto no reconocido',
    couldNotFinish: 'No se pudo completar',
    seconds: (n) => `${n} segundos`,
  },
  de: {
    closeEsc: 'Schließen (Esc)',
    close: 'Schließen',
    retry: 'Erneut versuchen',
    pinPanel: 'Panel anheften',
    pinnedToast: 'Panel angeheftet — bleibt offen, wenn du ins Spiel zurückkehrst',
    unpinnedToast: 'Panel gelöst',
    openTrade: 'Auf der Trade-Seite öffnen',
    dragToMove: 'Ziehen zum Verschieben',
    itemAria: 'Gegenstand',
    priceAria: 'Preis',
    rarity: { normal: 'Normal', magic: 'Magisch', rare: 'Selten', unique: 'Einzigartig', gem: 'Gemme', currency: 'Currency', divinationcard: 'Divination Card', other: 'Gegenstand' },
    quality: 'Qualität',
    corrupted: 'Corrupted',
    unidentified: 'Nicht identifiziert',
    advancedCopy: 'Im erweiterten Modus kopiert',

    estimatedPrice: 'Geschätzter Preis',
    confidence: (c) => `Vertrauen ${DE_CONF[c] ?? '—'}`,
    listingsUnit: (n) => (n === 1 ? 'Angebot' : 'Angebote'),
    estimateLine: (chaos, c, n) => `≈ ${chaos} c · Vertrauen ${DE_CONF[c] ?? '—'} · ${n} ${n === 1 ? 'Angebot' : 'Angebote'}`,
    ninjaPriceNoListings: 'poe.ninja-Preis · keine Angebote',
    ninjaNoListings: 'poe.ninja · keine Angebote',
    noPricedListings: 'keine Angebote mit Preis',
    ninjaReference: 'Referenzpreis von poe.ninja',
    min: 'Minimum',
    median: 'Median',
    max: 'Maximum',
    medianShort: 'Med',

    sellers: 'Verkäufer',
    online: 'Online',
    any: 'Alle',
    leagueHint: 'Liga (im Hauptfenster ändern)',
    filtersChanged: 'Filter geändert',
    relax10: 'Lockern −10%',
    searching: 'Suche…',

    market: 'Markt',
    listings: 'Angebote',
    distribution: 'Verteilung',
    scopeOnline: 'nur online',
    scopeAll: 'alle',
    colPrice: 'Preis',
    colSeller: 'Verkäufer',
    colListed: 'Gelistet',
    colAge: 'Alter',
    noListings: 'Keine Angebote',
    tradeFailed: 'Die Trade-Suche ist fehlgeschlagen. Nur der poe.ninja-Preis wird angezeigt.',
    noneFound: 'Keine Angebote gefunden',
    noneFoundHint: 'Schalte einige Mods ab oder senke die Minima, um die Suche zu erweitern.',
    relaxFilters10: 'Filter um 10% lockern',
    staleFilters: '● Filter geändert — Suche startet gleich…',
    histogramAria: 'Preisverteilung',
    listingsKpi: 'Angebote',
    noDistData: 'Keine Daten für die Verteilung',
    whisperCopied: (a) => `Whisper an ${a} kopiert`,
    whisperFailed: 'Whisper konnte nicht kopiert werden',

    cheapest: 'GÜNSTIGSTES',
    viewListedItem: 'Angebotenen Gegenstand ansehen',
    viewItem: 'Gegenstand ansehen',
    copyWhisper: 'Whisper kopieren',
    copyWhisperFor: (a) => `Whisper an ${a} kopieren`,
    matchesFilters: 'passt zu deinen Filtern',
    listedAgo: (age) => `gelistet vor ${age}`,
    listedNow: 'gerade gelistet',

    suggestedPrice: 'Preisvorschlag',
    percentilesOf: (n) => `· Perzentile aus ${n} ${n === 1 ? 'Angebot' : 'Angeboten'}`,
    levelAria: 'Stufe des Preisvorschlags',
    copyNote: 'Preisnotiz kopieren',
    tooFew: (n) => `Nur ${n} ${n === 1 ? 'Angebot' : 'Angebote'} — zu wenige für einen Vorschlag.`,
    levels: {
      buy: { name: 'Kaufen', hint: 'Bis hierhin zahlen', sentence: (v) => `25 % der Angebote kosten bis ${v} — gute Obergrenze zum Kaufen.` },
      fair: { name: 'Fair', hint: 'Marktmedian', sentence: (v) => `Die Hälfte der Angebote kostet bis ${v} — verkauft sich in wenigen Stunden.` },
      sell: { name: 'Verkaufen', hint: 'Einstellen und warten', sentence: (v) => `75 % der Angebote kosten bis ${v} — verkauft sich langsamer.` },
    },

    readyTitle: 'Bereit zum Preischeck',
    leadBefore: 'Im Spiel mit der Maus über einen Gegenstand fahren und',
    leadAfter: ' drücken',
    pasteBefore: 'Kein Spiel? Gegenstand einfügen mit',
    pasteAfter: '',
    recent: 'Zuletzt',
    reopenSame: 'Mit denselben Filtern erneut öffnen',
    demoMode: 'Demo-Modus',
    demoRare: 'Seltener Gegenstand',
    demoFew: 'Wenige Angebote',
    demoNetwork: 'Netzwerkfehler',
    demoUnrecognized: 'Nicht erkannt',

    ninja7d: 'poe.ninja · 7 Tage',
    seeListings: 'Angebote ansehen →',
    noReference: 'Kein poe.ninja-Referenzpreis für diesen Gegenstand.',
    openFull: 'Vollständigen Preischeck öffnen',

    tooManyTitle: 'Zu viele Suchen hintereinander',
    tooManyText: 'Die Trade-API verlangt eine Pause. Die Suche läuft automatisch weiter, wenn der Zähler abgelaufen ist.',
    noConnectionTitle: 'Keine Verbindung zum Trade',
    notRecognizedTitle: 'Gegenstand nicht erkannt',
    couldNotFinish: 'Konnte nicht abgeschlossen werden',
    seconds: (n) => `${n} Sekunden`,
  },
  'zh-CN': {
    closeEsc: '关闭 (Esc)',
    close: '关闭',
    retry: '重试',
    pinPanel: '固定面板',
    pinnedToast: '面板已固定 —— 返回游戏时不会关闭',
    unpinnedToast: '已取消固定面板',
    openTrade: '在交易网站中打开',
    dragToMove: '拖动以移动',
    itemAria: '物品',
    priceAria: '价格',
    rarity: { normal: '普通', magic: '魔法', rare: '稀有', unique: '传奇', gem: '宝石', currency: 'Currency', divinationcard: 'Divination Card', other: '物品' },
    quality: '品质',
    corrupted: 'Corrupted',
    unidentified: '未鉴定',
    advancedCopy: '以高级模式复制的物品',

    estimatedPrice: '估价',
    confidence: (c) => `可信度 ${ZH_CONF[c] ?? '—'}`,
    listingsUnit: () => '条挂单',
    estimateLine: (chaos, c, n) => `≈ ${chaos} c · 可信度 ${ZH_CONF[c] ?? '—'} · ${n} 条挂单`,
    ninjaPriceNoListings: 'poe.ninja 价格 · 无挂单',
    ninjaNoListings: 'poe.ninja · 无挂单',
    noPricedListings: '没有标价的挂单',
    ninjaReference: 'poe.ninja 参考价格',
    min: '最低',
    median: '中位数',
    max: '最高',
    medianShort: '中位',

    sellers: '卖家',
    online: '在线',
    any: '全部',
    leagueHint: '赛季（在主窗口中更改）',
    filtersChanged: '筛选已更改',
    relax10: '放宽 −10%',
    searching: '搜索中…',

    market: '市场',
    listings: '挂单',
    distribution: '分布',
    scopeOnline: '仅在线',
    scopeAll: '全部',
    colPrice: '价格',
    colSeller: '卖家',
    colListed: '上架',
    colAge: '时间',
    noListings: '无挂单',
    tradeFailed: '交易搜索失败，仅显示 poe.ninja 价格。',
    noneFound: '未找到挂单',
    noneFoundHint: '关闭部分词缀或降低最小值以扩大搜索范围。',
    relaxFilters10: '筛选放宽 10%',
    staleFilters: '● 筛选已更改 —— 即将搜索…',
    histogramAria: '价格分布',
    listingsKpi: '挂单',
    noDistData: '没有可用于分布的数据',
    whisperCopied: (a) => `已复制发给 ${a} 的密语`,
    whisperFailed: '无法复制密语',

    cheapest: '最便宜',
    viewListedItem: '查看挂单物品',
    viewItem: '查看物品',
    copyWhisper: '复制密语',
    copyWhisperFor: (a) => `复制发给 ${a} 的密语`,
    matchesFilters: '符合你的筛选',
    listedAgo: (age) => `${age} 前上架`,
    listedNow: '刚刚上架',

    suggestedPrice: '建议价格',
    percentilesOf: (n) => `· 基于 ${n} 条挂单的百分位`,
    levelAria: '建议价格档位',
    copyNote: '复制标价备注',
    tooFew: (n) => `仅 ${n} 条挂单 —— 太少，无法建议价格。`,
    levels: {
      buy: { name: '买入', hint: '最多付这么多', sentence: (v) => `25% 的挂单不高于 ${v} —— 适合作为买入上限。` },
      fair: { name: '公道', hint: '市场中位数', sentence: (v) => `一半的挂单不高于 ${v} —— 几小时内能卖出。` },
      sell: { name: '卖出', hint: '挂上慢慢等', sentence: (v) => `75% 的挂单不高于 ${v} —— 卖得较慢。` },
    },

    readyTitle: '准备查价',
    leadBefore: '在游戏中将鼠标悬停在物品上，按下',
    leadAfter: '',
    pasteBefore: '没开游戏？用',
    pasteAfter: ' 粘贴物品',
    recent: '最近',
    reopenSame: '用相同筛选重新打开',
    demoMode: '演示模式',
    demoRare: '稀有物品',
    demoFew: '挂单很少',
    demoNetwork: '网络错误',
    demoUnrecognized: '无法识别',

    ninja7d: 'poe.ninja · 7 日',
    seeListings: '查看挂单 →',
    noReference: 'poe.ninja 上没有此物品的参考价格。',
    openFull: '打开完整查价',

    tooManyTitle: '搜索过于频繁',
    tooManyText: 'Trade API 要求暂停。倒计时结束后搜索会自动继续。',
    noConnectionTitle: '无法连接交易服务',
    notRecognizedTitle: '无法识别物品',
    couldNotFinish: '无法完成',
    seconds: (n) => `${n} 秒`,
  },
  ja: {
    closeEsc: '閉じる (Esc)',
    close: '閉じる',
    retry: '再試行',
    pinPanel: 'パネルを固定',
    pinnedToast: 'パネルを固定しました — ゲームに戻っても閉じません',
    unpinnedToast: 'パネルの固定を解除しました',
    openTrade: 'トレードサイトで開く',
    dragToMove: 'ドラッグで移動',
    itemAria: 'アイテム',
    priceAria: '価格',
    rarity: { normal: 'ノーマル', magic: 'マジック', rare: 'レア', unique: 'ユニーク', gem: 'ジェム', currency: 'Currency', divinationcard: 'Divination Card', other: 'アイテム' },
    quality: '品質',
    corrupted: 'Corrupted',
    unidentified: '未鑑定',
    advancedCopy: '詳細モードでコピーされたアイテム',

    estimatedPrice: '推定価格',
    confidence: (c) => `信頼度 ${JA_CONF[c] ?? '—'}`,
    listingsUnit: () => '件の出品',
    estimateLine: (chaos, c, n) => `≈ ${chaos} c · 信頼度 ${JA_CONF[c] ?? '—'} · 出品 ${n} 件`,
    ninjaPriceNoListings: 'poe.ninja の価格 · 出品なし',
    ninjaNoListings: 'poe.ninja · 出品なし',
    noPricedListings: '価格付きの出品なし',
    ninjaReference: 'poe.ninja の参考価格',
    min: '最安',
    median: '中央値',
    max: '最高',
    medianShort: '中央',

    sellers: '出品者',
    online: 'オンライン',
    any: 'すべて',
    leagueHint: 'リーグ（メインウィンドウで変更）',
    filtersChanged: 'フィルター変更あり',
    relax10: '緩和 −10%',
    searching: '検索中…',

    market: 'マーケット',
    listings: '出品',
    distribution: '分布',
    scopeOnline: 'オンラインのみ',
    scopeAll: 'すべて',
    colPrice: '価格',
    colSeller: '出品者',
    colListed: '出品',
    colAge: '経過',
    noListings: '出品なし',
    tradeFailed: 'トレード検索に失敗しました。poe.ninja の価格のみ表示しています。',
    noneFound: '出品が見つかりません',
    noneFoundHint: 'Mod をいくつかオフにするか最小値を下げて、検索範囲を広げてください。',
    relaxFilters10: 'フィルターを 10% 緩和',
    staleFilters: '● フィルター変更あり — まもなく検索します…',
    histogramAria: '価格分布',
    listingsKpi: '出品数',
    noDistData: '分布のデータがありません',
    whisperCopied: (a) => `${a} への Whisper をコピーしました`,
    whisperFailed: 'Whisper をコピーできませんでした',

    cheapest: '最安',
    viewListedItem: '出品アイテムを表示',
    viewItem: 'アイテムを表示',
    copyWhisper: 'Whisper をコピー',
    copyWhisperFor: (a) => `${a} への Whisper をコピー`,
    matchesFilters: 'フィルターに一致',
    listedAgo: (age) => `${age} 前に出品`,
    listedNow: 'たった今出品',

    suggestedPrice: '推奨価格',
    percentilesOf: (n) => `· 出品 ${n} 件のパーセンタイル`,
    levelAria: '推奨価格のレベル',
    copyNote: '価格ノートをコピー',
    tooFew: (n) => `出品が ${n} 件のみ — 推奨価格を出すには少なすぎます。`,
    levels: {
      buy: { name: '買い', hint: 'ここまでなら買い', sentence: (v) => `出品の 25% が ${v} 以下 — 購入の上限の目安です。` },
      fair: { name: '適正', hint: '市場の中央値', sentence: (v) => `出品の半数が ${v} 以下 — 数時間で売れる目安です。` },
      sell: { name: '売り', hint: '出品して待つ', sentence: (v) => `出品の 75% が ${v} 以下 — 売れるのは遅めです。` },
    },

    readyTitle: '価格チェックの準備完了',
    leadBefore: 'ゲーム内でアイテムにカーソルを合わせて',
    leadAfter: ' を押します',
    pasteBefore: 'ゲームなし？',
    pasteAfter: ' でアイテムを貼り付け',
    recent: '最近',
    reopenSame: '同じフィルターで開き直す',
    demoMode: 'デモモード',
    demoRare: 'レアアイテム',
    demoFew: '出品が少ない',
    demoNetwork: 'ネットワークエラー',
    demoUnrecognized: '認識不可',

    ninja7d: 'poe.ninja · 7 日間',
    seeListings: '出品を見る →',
    noReference: 'このアイテムの poe.ninja 参考価格はありません。',
    openFull: '完全な価格チェックを開く',

    tooManyTitle: '検索が連続しすぎています',
    tooManyText: 'Trade API が休止を要求しました。カウンターが 0 になると自動で再開します。',
    noConnectionTitle: 'トレードに接続できません',
    notRecognizedTitle: 'アイテムを認識できません',
    couldNotFinish: '完了できませんでした',
    seconds: (n) => `${n} 秒`,
  },
  ko: {
    closeEsc: '닫기 (Esc)',
    close: '닫기',
    retry: '다시 시도',
    pinPanel: '패널 고정',
    pinnedToast: '패널 고정됨 — 게임으로 돌아가도 닫히지 않습니다',
    unpinnedToast: '패널 고정 해제됨',
    openTrade: '거래 사이트에서 열기',
    dragToMove: '드래그하여 이동',
    itemAria: '아이템',
    priceAria: '가격',
    rarity: { normal: '일반', magic: '마법', rare: '희귀', unique: '고유', gem: '젬', currency: 'Currency', divinationcard: 'Divination Card', other: '아이템' },
    quality: '퀄리티',
    corrupted: 'Corrupted',
    unidentified: '미확인',
    advancedCopy: '고급 모드로 복사된 아이템',

    estimatedPrice: '예상 가격',
    confidence: (c) => `신뢰도 ${KO_CONF[c] ?? '—'}`,
    listingsUnit: () => '건',
    estimateLine: (chaos, c, n) => `≈ ${chaos} c · 신뢰도 ${KO_CONF[c] ?? '—'} · 매물 ${n}건`,
    ninjaPriceNoListings: 'poe.ninja 가격 · 매물 없음',
    ninjaNoListings: 'poe.ninja · 매물 없음',
    noPricedListings: '가격이 있는 매물 없음',
    ninjaReference: 'poe.ninja 참고 가격',
    min: '최저',
    median: '중앙값',
    max: '최고',
    medianShort: '중앙',

    sellers: '판매자',
    online: '온라인',
    any: '전체',
    leagueHint: '리그 (메인 창에서 변경)',
    filtersChanged: '필터 변경됨',
    relax10: '완화 −10%',
    searching: '검색 중…',

    market: '시장',
    listings: '매물',
    distribution: '분포',
    scopeOnline: '온라인만',
    scopeAll: '전체',
    colPrice: '가격',
    colSeller: '판매자',
    colListed: '등록',
    colAge: '경과',
    noListings: '매물 없음',
    tradeFailed: '거래 검색에 실패했습니다. poe.ninja 가격만 표시합니다.',
    noneFound: '매물을 찾지 못했습니다',
    noneFoundHint: '일부 옵션을 끄거나 최솟값을 낮춰 검색 범위를 넓히세요.',
    relaxFilters10: '필터 10% 완화',
    staleFilters: '● 필터 변경됨 — 곧 검색합니다…',
    histogramAria: '가격 분포',
    listingsKpi: '매물',
    noDistData: '분포 데이터 없음',
    whisperCopied: (a) => `${a}에게 보낼 귓속말 복사됨`,
    whisperFailed: '귓속말을 복사할 수 없습니다',

    cheapest: '최저가',
    viewListedItem: '등록된 아이템 보기',
    viewItem: '아이템 보기',
    copyWhisper: '귓속말 복사',
    copyWhisperFor: (a) => `${a}에게 보낼 귓속말 복사`,
    matchesFilters: '필터와 일치',
    listedAgo: (age) => `${age} 전 등록`,
    listedNow: '방금 등록',

    suggestedPrice: '추천 가격',
    percentilesOf: (n) => `· 매물 ${n}건의 백분위`,
    levelAria: '추천 가격 단계',
    copyNote: '가격 메모 복사',
    tooFew: (n) => `매물이 ${n}건뿐 — 가격을 추천하기에 너무 적습니다.`,
    levels: {
      buy: { name: '구매', hint: '이 가격까지 지불', sentence: (v) => `매물의 25%가 ${v} 이하 — 구매 상한으로 적당합니다.` },
      fair: { name: '적정', hint: '시장 중앙값', sentence: (v) => `매물의 절반이 ${v} 이하 — 몇 시간 안에 팔립니다.` },
      sell: { name: '판매', hint: '등록하고 기다리기', sentence: (v) => `매물의 75%가 ${v} 이하 — 더 천천히 팔립니다.` },
    },

    readyTitle: '시세 확인 준비 완료',
    leadBefore: '게임에서 아이템 위에 마우스를 올리고',
    leadAfter: ' 키를 누르세요',
    pasteBefore: '게임이 없나요?',
    pasteAfter: '(으)로 아이템 붙여넣기',
    recent: '최근',
    reopenSame: '같은 필터로 다시 열기',
    demoMode: '데모 모드',
    demoRare: '희귀 아이템',
    demoFew: '매물 적음',
    demoNetwork: '네트워크 오류',
    demoUnrecognized: '인식 불가',

    ninja7d: 'poe.ninja · 7일',
    seeListings: '매물 보기 →',
    noReference: '이 아이템의 poe.ninja 참고 가격이 없습니다.',
    openFull: '전체 시세 확인 열기',

    tooManyTitle: '검색이 너무 잦습니다',
    tooManyText: 'Trade API가 잠시 멈추라고 요청했습니다. 카운터가 0이 되면 검색이 자동으로 재개됩니다.',
    noConnectionTitle: '거래 서버에 연결할 수 없음',
    notRecognizedTitle: '아이템을 인식할 수 없음',
    couldNotFinish: '완료할 수 없습니다',
    seconds: (n) => `${n}초`,
  },
});
