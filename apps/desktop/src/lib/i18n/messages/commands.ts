import { defineMessages } from '../core.svelte';

interface CommandsMessages {
  /** Paleta Ctrl+K. */
  paletteAria: string;
  placeholder: string;
  searchAria: string;
  notFound: (query: string) => string;
  noneAvailable: string;
  hintNav: string;
  hintToggle: string;
  /** Toasts de nota sugerida e presets. */
  fewListings: string;
  noteCopied: (note: string) => string;
  noteCopyFailed: string;
  presetMissing: (n: number) => string;
  presetApplied: (name: string) => string;
  /** Comandos e grupos. */
  searchAgain: string;
  copyNote: string;
  relax: string;
  openTrade: string;
  currentItem: (name: string) => string;
  applyPreset: (name: string) => string;
  modCount: (n: number) => string;
  goTo: string;
  historyDetail: (n: number) => string;
  switchGame: (version: string) => string;
  switchLeague: string;
  clearHistory: string;
  reopen: (name: string) => string;
}

export const commandsMsgs = defineMessages<CommandsMessages>({
  'pt-BR': {
    paletteAria: 'Paleta de comandos',
    placeholder: 'O que você quer fazer?',
    searchAria: 'Buscar comando',
    notFound: (q) => `Nada encontrado para “${q}”`,
    noneAvailable: 'Nenhum comando disponível agora',
    hintNav: '↑ ↓ navegar · ↵ executar',
    hintToggle: 'abrir/fechar',
    fewListings: 'Poucos anúncios para sugerir preço',
    noteCopied: (n) => `Nota copiada: ${n}`,
    noteCopyFailed: 'Não foi possível copiar a nota',
    presetMissing: (n) => (n === 1 ? '1 mod do preset não está neste item' : `${n} mods do preset não estão neste item`),
    presetApplied: (name) => `Preset “${name}” aplicado — buscando…`,
    searchAgain: 'Refazer busca',
    copyNote: 'Copiar preço sugerido',
    relax: 'Relaxar filtros −10%',
    openTrade: 'Abrir no site de trade',
    currentItem: (name) => `Item atual · ${name}`,
    applyPreset: (name) => `Aplicar preset “${name}”`,
    modCount: (n) => `${n} ${n === 1 ? 'mod' : 'mods'}`,
    goTo: 'Ir para',
    historyDetail: (n) => `${n} price checks`,
    switchGame: (v) => `Trocar jogo para PoE ${v}`,
    switchLeague: 'Trocar liga…',
    clearHistory: 'Limpar histórico',
    reopen: (name) => `Reabrir ${name}`,
  },
  en: {
    paletteAria: 'Command palette',
    placeholder: 'What do you want to do?',
    searchAria: 'Search commands',
    notFound: (q) => `Nothing found for “${q}”`,
    noneAvailable: 'No commands available right now',
    hintNav: '↑ ↓ navigate · ↵ run',
    hintToggle: 'open/close',
    fewListings: 'Too few listings to suggest a price',
    noteCopied: (n) => `Note copied: ${n}`,
    noteCopyFailed: "Couldn't copy the note",
    presetMissing: (n) => (n === 1 ? "1 preset mod isn't on this item" : `${n} preset mods aren't on this item`),
    presetApplied: (name) => `Preset “${name}” applied — searching…`,
    searchAgain: 'Search again',
    copyNote: 'Copy suggested price',
    relax: 'Relax filters −10%',
    openTrade: 'Open on trade site',
    currentItem: (name) => `Current item · ${name}`,
    applyPreset: (name) => `Apply preset “${name}”`,
    modCount: (n) => `${n} ${n === 1 ? 'mod' : 'mods'}`,
    goTo: 'Go to',
    historyDetail: (n) => `${n} price ${n === 1 ? 'check' : 'checks'}`,
    switchGame: (v) => `Switch game to PoE ${v}`,
    switchLeague: 'Switch league…',
    clearHistory: 'Clear history',
    reopen: (name) => `Reopen ${name}`,
  },
  es: {
    paletteAria: 'Paleta de comandos',
    placeholder: '¿Qué quieres hacer?',
    searchAria: 'Buscar comando',
    notFound: (q) => `Nada encontrado para “${q}”`,
    noneAvailable: 'No hay comandos disponibles ahora',
    hintNav: '↑ ↓ navegar · ↵ ejecutar',
    hintToggle: 'abrir/cerrar',
    fewListings: 'Pocos anuncios para sugerir un precio',
    noteCopied: (n) => `Nota copiada: ${n}`,
    noteCopyFailed: 'No se pudo copiar la nota',
    presetMissing: (n) => (n === 1 ? '1 mod del preset no está en este objeto' : `${n} mods del preset no están en este objeto`),
    presetApplied: (name) => `Preset “${name}” aplicado — buscando…`,
    searchAgain: 'Repetir búsqueda',
    copyNote: 'Copiar precio sugerido',
    relax: 'Relajar filtros −10%',
    openTrade: 'Abrir en el sitio de trade',
    currentItem: (name) => `Objeto actual · ${name}`,
    applyPreset: (name) => `Aplicar preset “${name}”`,
    modCount: (n) => `${n} ${n === 1 ? 'mod' : 'mods'}`,
    goTo: 'Ir a',
    historyDetail: (n) => `${n} price checks`,
    switchGame: (v) => `Cambiar juego a PoE ${v}`,
    switchLeague: 'Cambiar liga…',
    clearHistory: 'Borrar historial',
    reopen: (name) => `Reabrir ${name}`,
  },
  de: {
    paletteAria: 'Befehlspalette',
    placeholder: 'Was möchtest du tun?',
    searchAria: 'Befehl suchen',
    notFound: (q) => `Nichts gefunden für „${q}“`,
    noneAvailable: 'Gerade keine Befehle verfügbar',
    hintNav: '↑ ↓ navigieren · ↵ ausführen',
    hintToggle: 'öffnen/schließen',
    fewListings: 'Zu wenige Angebote für einen Preisvorschlag',
    noteCopied: (n) => `Notiz kopiert: ${n}`,
    noteCopyFailed: 'Notiz konnte nicht kopiert werden',
    presetMissing: (n) => (n === 1 ? '1 Preset-Mod fehlt auf diesem Item' : `${n} Preset-Mods fehlen auf diesem Item`),
    presetApplied: (name) => `Preset „${name}“ angewendet — suche…`,
    searchAgain: 'Erneut suchen',
    copyNote: 'Preisvorschlag kopieren',
    relax: 'Filter lockern −10%',
    openTrade: 'Auf Trade-Seite öffnen',
    currentItem: (name) => `Aktuelles Item · ${name}`,
    applyPreset: (name) => `Preset „${name}“ anwenden`,
    modCount: (n) => `${n} ${n === 1 ? 'Mod' : 'Mods'}`,
    goTo: 'Gehe zu',
    historyDetail: (n) => `${n} Price Checks`,
    switchGame: (v) => `Zu PoE ${v} wechseln`,
    switchLeague: 'Liga wechseln…',
    clearHistory: 'Verlauf löschen',
    reopen: (name) => `${name} erneut öffnen`,
  },
  'zh-CN': {
    paletteAria: '命令面板',
    placeholder: '你想做什么？',
    searchAria: '搜索命令',
    notFound: (q) => `未找到与“${q}”相关的结果`,
    noneAvailable: '当前没有可用命令',
    hintNav: '↑ ↓ 选择 · ↵ 执行',
    hintToggle: '打开/关闭',
    fewListings: '挂单太少，无法建议价格',
    noteCopied: (n) => `已复制备注：${n}`,
    noteCopyFailed: '无法复制备注',
    presetMissing: (n) => `预设中有 ${n} 个词缀不在此物品上`,
    presetApplied: (name) => `已应用预设“${name}”，正在搜索…`,
    searchAgain: '重新搜索',
    copyNote: '复制建议价格',
    relax: '放宽筛选 −10%',
    openTrade: '在交易网站打开',
    currentItem: (name) => `当前物品 · ${name}`,
    applyPreset: (name) => `应用预设“${name}”`,
    modCount: (n) => `${n} 个词缀`,
    goTo: '前往',
    historyDetail: (n) => `${n} 次查价`,
    switchGame: (v) => `切换到 PoE ${v}`,
    switchLeague: '切换赛季…',
    clearHistory: '清空历史记录',
    reopen: (name) => `重新打开 ${name}`,
  },
  ja: {
    paletteAria: 'コマンドパレット',
    placeholder: '何をしますか？',
    searchAria: 'コマンドを検索',
    notFound: (q) => `「${q}」に一致するものはありません`,
    noneAvailable: '現在使用できるコマンドはありません',
    hintNav: '↑ ↓ 移動 · ↵ 実行',
    hintToggle: '開く/閉じる',
    fewListings: '出品が少なすぎて価格を提案できません',
    noteCopied: (n) => `メモをコピーしました: ${n}`,
    noteCopyFailed: 'メモをコピーできませんでした',
    presetMissing: (n) => `プリセットの Mod ${n} 個がこのアイテムにありません`,
    presetApplied: (name) => `プリセット「${name}」を適用 — 検索中…`,
    searchAgain: '再検索',
    copyNote: '推奨価格をコピー',
    relax: 'フィルターを緩める −10%',
    openTrade: 'トレードサイトで開く',
    currentItem: (name) => `現在のアイテム · ${name}`,
    applyPreset: (name) => `プリセット「${name}」を適用`,
    modCount: (n) => `${n} Mod`,
    goTo: '移動',
    historyDetail: (n) => `${n} 件の価格チェック`,
    switchGame: (v) => `PoE ${v} に切り替え`,
    switchLeague: 'リーグを切り替え…',
    clearHistory: '履歴を消去',
    reopen: (name) => `${name} を再表示`,
  },
  ko: {
    paletteAria: '명령 팔레트',
    placeholder: '무엇을 할까요?',
    searchAria: '명령 검색',
    notFound: (q) => `“${q}”에 대한 결과 없음`,
    noneAvailable: '지금 사용할 수 있는 명령이 없습니다',
    hintNav: '↑ ↓ 이동 · ↵ 실행',
    hintToggle: '열기/닫기',
    fewListings: '매물이 적어 가격을 추천할 수 없습니다',
    noteCopied: (n) => `메모 복사됨: ${n}`,
    noteCopyFailed: '메모를 복사할 수 없습니다',
    presetMissing: (n) => `프리셋 옵션 ${n}개가 이 아이템에 없습니다`,
    presetApplied: (name) => `프리셋 “${name}” 적용됨 — 검색 중…`,
    searchAgain: '다시 검색',
    copyNote: '추천 가격 복사',
    relax: '필터 완화 −10%',
    openTrade: '거래 사이트에서 열기',
    currentItem: (name) => `현재 아이템 · ${name}`,
    applyPreset: (name) => `프리셋 “${name}” 적용`,
    modCount: (n) => `옵션 ${n}개`,
    goTo: '이동',
    historyDetail: (n) => `시세 확인 ${n}회`,
    switchGame: (v) => `PoE ${v}로 전환`,
    switchLeague: '리그 변경…',
    clearHistory: '기록 삭제',
    reopen: (name) => `${name} 다시 열기`,
  },
});

/**
 * Palavras extras para a busca da paleta achar cada comando em qualquer idioma
 * (inclusive o termo em português quando a interface está em outro idioma).
 */
export const COMMAND_KEYWORDS: Record<string, string> = {
  search: 'refazer busca buscar search again repetir busqueda erneut suchen 重新搜索 再検索 다시 검색',
  'copy-note': 'copiar preço sugerido nota copy suggested price note precio sugerido preisvorschlag kopieren notiz 复制 建议价格 推奨価格 コピー 추천 가격 복사 ~b/o',
  relax: 'relaxar filtros relax filters relajar lockern 放宽 筛选 緩める フィルター 필터 완화',
  trade: 'abrir site trade open trade site sitio seite öffnen 交易网站 トレードサイト 거래 사이트',
  preset: 'aplicar preset apply anwenden 应用预设 プリセット 適用 프리셋 적용',
  'go-price': 'price check preço price precio preis 查价 価格チェック 시세 확인',
  'go-history': 'histórico history historial verlauf 历史记录 履歴 기록',
  'go-settings': 'configurações opções settings options ajustes configuración einstellungen 设置 設定 설정',
  game: 'trocar jogo switch game cambiar juego spiel wechseln 切换游戏 ゲーム 切り替え 게임 전환 poe1 poe2',
  league: 'trocar liga switch league cambiar liga wechseln 切换赛季 リーグ 리그 변경',
  'clear-history': 'limpar apagar histórico clear history borrar historial verlauf löschen 清空历史 履歴 消去 기록 삭제',
  hist: 'reabrir reopen erneut öffnen 重新打开 再表示 다시 열기',
};
