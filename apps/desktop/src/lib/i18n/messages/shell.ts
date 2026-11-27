import { defineMessages } from '../core.svelte';

interface ShellMessages {
  /** Menu lateral (os nomes das telas vêm de `SCREEN_LABEL`). */
  nav: { aria: string; market: string; system: string; expand: string; collapse: string };
  /** Barra de status. */
  status: {
    apiRateLimit: (seconds: number) => string;
    apiError: string;
    apiLoading: string;
    apiOk: string;
    keyPriceCheck: string;
    keyLocked: string;
    keyCommands: string;
    keyClose: string;
    /** Chip "vX pronta · Reiniciar" (a versão vem antes, separada). */
    updateChip: string;
    updateChipTitle: string;
    updateToast: (version: string) => string;
  };
  /** Barra de título e menu de ligas. */
  title: {
    tagline: string;
    crumbAria: string;
    paletteTitle: string;
    searchPlaceholder: string;
    game: string;
    leagueTitle: string;
    leagues: string;
    league: string;
    leaguesHead: (game: string) => string;
    loadingLeagues: string;
    current: string;
    leaguesError: string;
  };
  /** Botões de janela (WindowControls). */
  window: { minimize: string; maximize: string; restore: string; close: string; closeTitle: string };
}

// "price check", "Trade API" e "Path of Exile" ficam como os jogadores usam.
export const shellMsgs = defineMessages<ShellMessages>({
  'pt-BR': {
    nav: { aria: 'Navegação', market: 'Mercado', system: 'Sistema', expand: 'Expandir menu', collapse: 'Recolher menu' },
    status: {
      apiRateLimit: (s) => `Rate limit · ${s}s`,
      apiError: 'Trade API com erro',
      apiLoading: 'Trade API · buscando…',
      apiOk: 'Trade API ok',
      keyPriceCheck: 'price check',
      keyLocked: 'fixado',
      keyCommands: 'comandos',
      keyClose: 'fechar',
      updateChip: 'pronta · Reiniciar',
      updateChipTitle: 'Instalar a atualização e reiniciar o Mirrorsight',
      updateToast: (v) => `Mirrorsight v${v} baixado — reinicie quando quiser para atualizar.`,
    },
    title: {
      tagline: 'price check · Path of Exile',
      crumbAria: 'Você está em',
      paletteTitle: 'Paleta de comandos (Ctrl+K)',
      searchPlaceholder: 'Buscar ação, item, configuração…',
      game: 'Jogo',
      leagueTitle: 'Liga de trade',
      leagues: 'Ligas',
      league: 'Liga',
      leaguesHead: (g) => `Ligas · ${g}`,
      loadingLeagues: 'Carregando ligas…',
      current: 'atual',
      leaguesError: 'Não foi possível carregar as ligas.',
    },
    window: { minimize: 'Minimizar', maximize: 'Maximizar', restore: 'Restaurar', close: 'Fechar', closeTitle: 'Fechar (continua na bandeja)' },
  },
  en: {
    nav: { aria: 'Navigation', market: 'Market', system: 'System', expand: 'Expand menu', collapse: 'Collapse menu' },
    status: {
      apiRateLimit: (s) => `Rate limit · ${s}s`,
      apiError: 'Trade API error',
      apiLoading: 'Trade API · searching…',
      apiOk: 'Trade API ok',
      keyPriceCheck: 'price check',
      keyLocked: 'pinned',
      keyCommands: 'commands',
      keyClose: 'close',
      updateChip: 'ready · Restart',
      updateChipTitle: 'Install the update and restart Mirrorsight',
      updateToast: (v) => `Mirrorsight v${v} downloaded — restart whenever you like to update.`,
    },
    title: {
      tagline: 'price check · Path of Exile',
      crumbAria: 'You are here',
      paletteTitle: 'Command palette (Ctrl+K)',
      searchPlaceholder: 'Search actions, items, settings…',
      game: 'Game',
      leagueTitle: 'Trade league',
      leagues: 'Leagues',
      league: 'League',
      leaguesHead: (g) => `Leagues · ${g}`,
      loadingLeagues: 'Loading leagues…',
      current: 'current',
      leaguesError: "Couldn't load leagues.",
    },
    window: { minimize: 'Minimize', maximize: 'Maximize', restore: 'Restore', close: 'Close', closeTitle: 'Close (keeps running in the tray)' },
  },
  es: {
    nav: { aria: 'Navegación', market: 'Mercado', system: 'Sistema', expand: 'Expandir menú', collapse: 'Contraer menú' },
    status: {
      apiRateLimit: (s) => `Rate limit · ${s}s`,
      apiError: 'Error en la Trade API',
      apiLoading: 'Trade API · buscando…',
      apiOk: 'Trade API ok',
      keyPriceCheck: 'price check',
      keyLocked: 'fijado',
      keyCommands: 'comandos',
      keyClose: 'cerrar',
      updateChip: 'lista · Reiniciar',
      updateChipTitle: 'Instalar la actualización y reiniciar Mirrorsight',
      updateToast: (v) => `Mirrorsight v${v} descargado — reinicia cuando quieras para actualizar.`,
    },
    title: {
      tagline: 'price check · Path of Exile',
      crumbAria: 'Estás en',
      paletteTitle: 'Paleta de comandos (Ctrl+K)',
      searchPlaceholder: 'Buscar acción, objeto, ajuste…',
      game: 'Juego',
      leagueTitle: 'Liga de comercio',
      leagues: 'Ligas',
      league: 'Liga',
      leaguesHead: (g) => `Ligas · ${g}`,
      loadingLeagues: 'Cargando ligas…',
      current: 'actual',
      leaguesError: 'No se pudieron cargar las ligas.',
    },
    window: { minimize: 'Minimizar', maximize: 'Maximizar', restore: 'Restaurar', close: 'Cerrar', closeTitle: 'Cerrar (sigue en la bandeja)' },
  },
  de: {
    nav: { aria: 'Navigation', market: 'Markt', system: 'System', expand: 'Menü ausklappen', collapse: 'Menü einklappen' },
    status: {
      apiRateLimit: (s) => `Rate-Limit · ${s}s`,
      apiError: 'Trade-API-Fehler',
      apiLoading: 'Trade-API · sucht…',
      apiOk: 'Trade-API ok',
      keyPriceCheck: 'Price Check',
      keyLocked: 'fixiert',
      keyCommands: 'Befehle',
      keyClose: 'schließen',
      updateChip: 'bereit · Neu starten',
      updateChipTitle: 'Update installieren und Mirrorsight neu starten',
      updateToast: (v) => `Mirrorsight v${v} heruntergeladen — starte neu, wann du willst, um zu aktualisieren.`,
    },
    title: {
      tagline: 'price check · Path of Exile',
      crumbAria: 'Sie sind hier',
      paletteTitle: 'Befehlspalette (Ctrl+K)',
      searchPlaceholder: 'Aktion, Item, Einstellung suchen…',
      game: 'Spiel',
      leagueTitle: 'Trade-Liga',
      leagues: 'Ligen',
      league: 'Liga',
      leaguesHead: (g) => `Ligen · ${g}`,
      loadingLeagues: 'Ligen werden geladen…',
      current: 'aktuell',
      leaguesError: 'Ligen konnten nicht geladen werden.',
    },
    window: { minimize: 'Minimieren', maximize: 'Maximieren', restore: 'Wiederherstellen', close: 'Schließen', closeTitle: 'Schließen (läuft im Tray weiter)' },
  },
  'zh-CN': {
    nav: { aria: '导航', market: '市场', system: '系统', expand: '展开菜单', collapse: '收起菜单' },
    status: {
      apiRateLimit: (s) => `速率限制 · ${s} 秒`,
      apiError: '交易 API 出错',
      apiLoading: '交易 API · 搜索中…',
      apiOk: '交易 API 正常',
      keyPriceCheck: '查价',
      keyLocked: '固定',
      keyCommands: '命令',
      keyClose: '关闭',
      updateChip: '已就绪 · 重启',
      updateChipTitle: '安装更新并重启 Mirrorsight',
      updateToast: (v) => `Mirrorsight v${v} 已下载 — 随时重启即可更新。`,
    },
    title: {
      tagline: 'price check · Path of Exile',
      crumbAria: '当前位置',
      paletteTitle: '命令面板 (Ctrl+K)',
      searchPlaceholder: '搜索操作、物品、设置…',
      game: '游戏',
      leagueTitle: '交易赛季',
      leagues: '赛季',
      league: '赛季',
      leaguesHead: (g) => `赛季 · ${g}`,
      loadingLeagues: '正在加载赛季…',
      current: '当前',
      leaguesError: '无法加载赛季。',
    },
    window: { minimize: '最小化', maximize: '最大化', restore: '还原', close: '关闭', closeTitle: '关闭（继续在托盘运行）' },
  },
  ja: {
    nav: { aria: 'ナビゲーション', market: 'マーケット', system: 'システム', expand: 'メニューを展開', collapse: 'メニューを折りたたむ' },
    status: {
      apiRateLimit: (s) => `レート制限 · ${s}秒`,
      apiError: 'トレード API エラー',
      apiLoading: 'トレード API · 検索中…',
      apiOk: 'トレード API 正常',
      keyPriceCheck: '価格チェック',
      keyLocked: '固定',
      keyCommands: 'コマンド',
      keyClose: '閉じる',
      updateChip: '準備完了 · 再起動',
      updateChipTitle: 'アップデートをインストールして Mirrorsight を再起動',
      updateToast: (v) => `Mirrorsight v${v} をダウンロードしました — 好きなときに再起動して更新できます。`,
    },
    title: {
      tagline: 'price check · Path of Exile',
      crumbAria: '現在地',
      paletteTitle: 'コマンドパレット (Ctrl+K)',
      searchPlaceholder: 'アクション・アイテム・設定を検索…',
      game: 'ゲーム',
      leagueTitle: 'トレードリーグ',
      leagues: 'リーグ',
      league: 'リーグ',
      leaguesHead: (g) => `リーグ · ${g}`,
      loadingLeagues: 'リーグを読み込み中…',
      current: '現行',
      leaguesError: 'リーグを読み込めませんでした。',
    },
    window: { minimize: '最小化', maximize: '最大化', restore: '元に戻す', close: '閉じる', closeTitle: '閉じる（トレイで実行を継続）' },
  },
  ko: {
    nav: { aria: '탐색', market: '마켓', system: '시스템', expand: '메뉴 펼치기', collapse: '메뉴 접기' },
    status: {
      apiRateLimit: (s) => `요청 한도 · ${s}초`,
      apiError: '거래 API 오류',
      apiLoading: '거래 API · 검색 중…',
      apiOk: '거래 API 정상',
      keyPriceCheck: '시세 확인',
      keyLocked: '고정',
      keyCommands: '명령',
      keyClose: '닫기',
      updateChip: '준비됨 · 재시작',
      updateChipTitle: '업데이트를 설치하고 Mirrorsight 재시작',
      updateToast: (v) => `Mirrorsight v${v} 다운로드 완료 — 원할 때 재시작하면 업데이트됩니다.`,
    },
    title: {
      tagline: 'price check · Path of Exile',
      crumbAria: '현재 위치',
      paletteTitle: '명령 팔레트 (Ctrl+K)',
      searchPlaceholder: '작업, 아이템, 설정 검색…',
      game: '게임',
      leagueTitle: '거래 리그',
      leagues: '리그',
      league: '리그',
      leaguesHead: (g) => `리그 · ${g}`,
      loadingLeagues: '리그 불러오는 중…',
      current: '현재',
      leaguesError: '리그를 불러올 수 없습니다.',
    },
    window: { minimize: '최소화', maximize: '최대화', restore: '이전 크기로', close: '닫기', closeTitle: '닫기 (트레이에서 계속 실행)' },
  },
});
