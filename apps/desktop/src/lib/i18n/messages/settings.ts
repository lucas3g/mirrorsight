import type { HotkeyAction } from '$lib/types';
import type { Screen, SettingsPane } from '$lib/stores/nav.svelte';
import { defineMessages } from '../core.svelte';

type Row = { label: string; description: string };

interface SettingsMessages {
  /** Nomes das telas (barra de título/menu lateral). */
  screens: Record<Screen, string>;
  /** Nomes das seções das configurações. */
  panes: Record<SettingsPane, string>;
  /** Subtítulo de cada seção (também entra na busca). */
  paneDescriptions: Record<SettingsPane, string>;
  view: { aria: string; sectionsAria: string; searchPlaceholder: string; searchAria: string; loading: string };
  general: {
    game: Row;
    league: Row;
    leagueError: string;
    currency: Row;
    restoreClipboard: Row;
    interceptCopy: Row;
    startup: Row;
  };
  hotkeys: {
    actions: Record<HotkeyAction, Row>;
    palette: Row;
    saved: string;
    defaultRestored: string;
    pressCombo: string;
    restoreDefault: string;
    fixedShortcut: string;
  };
  priceCheck: {
    searchSection: string;
    modRange: Row;
    showOffline: Row;
    defaultFilters: Row;
    /** Chip do tipo `property` (os demais são termos do jogo). */
    propertyKind: string;
    suggestedSection: string;
    newBadge: string;
    defaultLevel: Row;
    defaultLevelAria: string;
    /** Nomes dos níveis vêm de `LEVEL_META`. */
    levelsInfo: (buy: string, fair: string, sell: string) => string;
    localSection: string;
    history: { label: string; description: (n: number, max: number) => string; search: string };
    viewHistory: string;
    clear: string;
    presets: { label: string; description: (poe1: number, poe2: number) => string; search: string; error: string };
    presetsNote: string;
    confirmLabel: string;
    confirmTitle: (n: number) => string;
    confirmBody: string;
    cancel: string;
  };
  appearance: {
    scale: Row;
    scaleAria: string;
    density: Row;
    compact: string;
    normal: string;
    reduceMotion: Row;
  };
  about: { version: (v: string) => string; demo: string; credits: string };
}

const STACK = 'Rust + Tauri 2 + Svelte 5';

// Termos do jogo (Price check, chaos, divine, ~b/o, PoE, Preset…) ficam como estão.
export const settingsMsgs = defineMessages<SettingsMessages>({
  'pt-BR': {
    screens: { price: 'Price Check', history: 'Histórico', settings: 'Configurações' },
    panes: { general: 'Geral', hotkeys: 'Hotkeys', 'price-check': 'Price check', appearance: 'Aparência', about: 'Sobre' },
    paneDescriptions: {
      general: 'Jogo, liga e comportamento básico do overlay.',
      hotkeys: 'Clique numa tecla e pressione a nova combinação. Esc cancela.',
      'price-check': 'Filtros padrão, ranges da busca, preço sugerido e dados locais.',
      appearance: 'Escala, densidade e movimento.',
      about: 'Versão e créditos.',
    },
    view: { aria: 'Configurações', sectionsAria: 'Seções das configurações', searchPlaceholder: 'Buscar ajuste…', searchAria: 'Buscar ajuste', loading: 'Carregando…' },
    general: {
      game: { label: 'Jogo', description: 'Define parser, base de mods e API usada' },
      league: { label: 'Liga', description: 'A primeira é a liga de desafio atual' },
      leagueError: 'Não foi possível carregar as ligas — usando a salva',
      currency: { label: 'Moeda de exibição', description: 'Valores acima de 1 divine sempre aparecem em divine' },
      restoreClipboard: { label: 'Restaurar clipboard', description: 'Devolve o conteúdo anterior após copiar o item' },
      interceptCopy: { label: 'Ctrl+C abre o price check', description: 'Ctrl+C sobre um item no jogo lê os tiers e abre o overlay; o texto continua sendo copiado, com até ~0,2 s de atraso (também no chat)' },
      startup: { label: 'Iniciar com o Windows', description: 'Fica na bandeja do sistema' },
    },
    hotkeys: {
      actions: {
        'price-check': { label: 'Price check', description: 'Painel completo do item sob o cursor' },
        'price-check-locked': { label: 'Price check fixado', description: 'Abre o painel já fixado na tela' },
        'toggle-overlay': { label: 'Mostrar/ocultar overlay', description: 'Alterna a visibilidade dos painéis' },
        'open-settings': { label: 'Abrir configurações', description: 'Esta janela' },
      },
      palette: { label: 'Paleta de comandos', description: 'Só dentro do Mirrorsight · não editável' },
      saved: 'Salvo',
      defaultRestored: 'Padrão restaurado',
      pressCombo: 'Pressione a combinação…',
      restoreDefault: 'Restaurar padrão',
      fixedShortcut: 'Atalho fixo',
    },
    priceCheck: {
      searchSection: 'Busca',
      modRange: { label: 'Range dos mods', description: 'Mínimo pré-preenchido = valor rolado − %' },
      showOffline: { label: 'Mostrar vendedores offline', description: 'Por padrão só online' },
      defaultFilters: { label: 'Filtros marcados por padrão', description: 'Tipos de mod que já vêm selecionados' },
      propertyKind: 'Atributos (defesas/DPS)',
      suggestedSection: 'Preço sugerido',
      newBadge: 'NOVO',
      defaultLevel: { label: 'Nível padrão', description: 'Qual dos três níveis vem selecionado e vai para a nota ~b/o' },
      defaultLevelAria: 'Nível padrão do preço sugerido',
      levelsInfo: (b, f, s) =>
        `Os níveis são percentis dos anúncios carregados, em chaos: ${b} = p25, ${f} = mediana, ${s} = p75. Com menos de 5 anúncios os três mostram “—”.`,
      localSection: 'Dados locais',
      history: {
        label: 'Histórico',
        description: (n, max) => `${n} de ${max} price checks salvos neste computador`,
        search: 'price checks salvos neste computador',
      },
      viewHistory: 'Ver histórico',
      clear: 'Limpar',
      presets: {
        label: 'Presets de mods',
        description: (a, b) => `${a} no PoE 1 · ${b} no PoE 2`,
        search: 'no PoE 1 no PoE 2',
        error: 'Não foi possível contar os presets',
      },
      presetsNote: 'Gerencie pelo menu Preset no Price Check',
      confirmLabel: 'Confirmar limpeza',
      confirmTitle: (n) => `Apagar os ${n} price checks?`,
      confirmBody: 'Não dá para desfazer. Presets e configurações não são afetados.',
      cancel: 'Cancelar',
    },
    appearance: {
      scale: { label: 'Escala da interface', description: 'Para monitores 1440p/4K' },
      scaleAria: 'Escala',
      density: { label: 'Densidade', description: 'Altura das linhas de listagem' },
      compact: 'Compacta',
      normal: 'Normal',
      reduceMotion: { label: 'Reduzir animações', description: 'Troca instantânea, sem movimento' },
    },
    about: {
      version: (v) => `Versão ${v} · ${STACK}`,
      demo: 'Modo demonstração (fora do Tauri)',
      credits:
        'Projeto independente, não afiliado à Grinding Gear Games. Ícones de itens © Grinding Gear Games, extraídos da instalação local do jogo.',
    },
  },
  en: {
    screens: { price: 'Price Check', history: 'History', settings: 'Settings' },
    panes: { general: 'General', hotkeys: 'Hotkeys', 'price-check': 'Price check', appearance: 'Appearance', about: 'About' },
    paneDescriptions: {
      general: 'Game, league and basic overlay behavior.',
      hotkeys: 'Click a key and press the new combination. Esc cancels.',
      'price-check': 'Default filters, search ranges, suggested price and local data.',
      appearance: 'Scale, density and motion.',
      about: 'Version and credits.',
    },
    view: { aria: 'Settings', sectionsAria: 'Settings sections', searchPlaceholder: 'Search settings…', searchAria: 'Search settings', loading: 'Loading…' },
    general: {
      game: { label: 'Game', description: 'Sets the parser, mod database and API used' },
      league: { label: 'League', description: 'The first one is the current challenge league' },
      leagueError: "Couldn't load the leagues — using the saved one",
      currency: { label: 'Display currency', description: 'Values above 1 divine are always shown in divine' },
      restoreClipboard: { label: 'Restore clipboard', description: 'Puts back the previous content after copying the item' },
      interceptCopy: { label: 'Ctrl+C opens the price check', description: 'Ctrl+C over an item in game reads its tiers and opens the overlay; the text is still copied, up to ~0.2 s later (chat included)' },
      startup: { label: 'Start with Windows', description: 'Stays in the system tray' },
    },
    hotkeys: {
      actions: {
        'price-check': { label: 'Price check', description: 'Full panel for the item under the cursor' },
        'price-check-locked': { label: 'Pinned price check', description: 'Opens the panel already pinned on screen' },
        'toggle-overlay': { label: 'Show/hide overlay', description: 'Toggles panel visibility' },
        'open-settings': { label: 'Open settings', description: 'This window' },
      },
      palette: { label: 'Command palette', description: 'Only inside Mirrorsight · not editable' },
      saved: 'Saved',
      defaultRestored: 'Default restored',
      pressCombo: 'Press the combination…',
      restoreDefault: 'Restore default',
      fixedShortcut: 'Fixed shortcut',
    },
    priceCheck: {
      searchSection: 'Search',
      modRange: { label: 'Mod range', description: 'Pre-filled minimum = rolled value − %' },
      showOffline: { label: 'Show offline sellers', description: 'Online only by default' },
      defaultFilters: { label: 'Filters checked by default', description: 'Mod types that come pre-selected' },
      propertyKind: 'Properties (defenses/DPS)',
      suggestedSection: 'Suggested price',
      newBadge: 'NEW',
      defaultLevel: { label: 'Default level', description: 'Which of the three levels is pre-selected and goes into the ~b/o note' },
      defaultLevelAria: 'Default suggested price level',
      levelsInfo: (b, f, s) =>
        `Levels are percentiles of the loaded listings, in chaos: ${b} = p25, ${f} = median, ${s} = p75. With fewer than 5 listings all three show “—”.`,
      localSection: 'Local data',
      history: {
        label: 'History',
        description: (n, max) => `${n} of ${max} price checks saved on this computer`,
        search: 'price checks saved on this computer',
      },
      viewHistory: 'View history',
      clear: 'Clear',
      presets: {
        label: 'Mod presets',
        description: (a, b) => `${a} in PoE 1 · ${b} in PoE 2`,
        search: 'in PoE 1 in PoE 2',
        error: "Couldn't count the presets",
      },
      presetsNote: 'Manage them from the Preset menu in Price Check',
      confirmLabel: 'Confirm clearing',
      confirmTitle: (n) => `Delete the ${n} price checks?`,
      confirmBody: "This can't be undone. Presets and settings are not affected.",
      cancel: 'Cancel',
    },
    appearance: {
      scale: { label: 'Interface scale', description: 'For 1440p/4K monitors' },
      scaleAria: 'Scale',
      density: { label: 'Density', description: 'Height of listing rows' },
      compact: 'Compact',
      normal: 'Normal',
      reduceMotion: { label: 'Reduce motion', description: 'Instant transitions, no movement' },
    },
    about: {
      version: (v) => `Version ${v} · ${STACK}`,
      demo: 'Demo mode (outside Tauri)',
      credits:
        'Independent project, not affiliated with Grinding Gear Games. Item icons © Grinding Gear Games, extracted from the local game installation.',
    },
  },
  es: {
    screens: { price: 'Price Check', history: 'Historial', settings: 'Ajustes' },
    panes: { general: 'General', hotkeys: 'Hotkeys', 'price-check': 'Price check', appearance: 'Apariencia', about: 'Acerca de' },
    paneDescriptions: {
      general: 'Juego, liga y comportamiento básico del overlay.',
      hotkeys: 'Haz clic en una tecla y pulsa la nueva combinación. Esc cancela.',
      'price-check': 'Filtros por defecto, rangos de búsqueda, precio sugerido y datos locales.',
      appearance: 'Escala, densidad y movimiento.',
      about: 'Versión y créditos.',
    },
    view: { aria: 'Ajustes', sectionsAria: 'Secciones de los ajustes', searchPlaceholder: 'Buscar ajuste…', searchAria: 'Buscar ajuste', loading: 'Cargando…' },
    general: {
      game: { label: 'Juego', description: 'Define el parser, la base de mods y la API usada' },
      league: { label: 'Liga', description: 'La primera es la liga de desafío actual' },
      leagueError: 'No se pudieron cargar las ligas — usando la guardada',
      currency: { label: 'Moneda de visualización', description: 'Los valores por encima de 1 divine siempre se muestran en divine' },
      restoreClipboard: { label: 'Restaurar portapapeles', description: 'Devuelve el contenido anterior tras copiar el objeto' },
      interceptCopy: { label: 'Ctrl+C abre el price check', description: 'Ctrl+C sobre un objeto en el juego lee sus tiers y abre el overlay; el texto se sigue copiando, con hasta ~0,2 s de retraso (también en el chat)' },
      startup: { label: 'Iniciar con Windows', description: 'Se queda en la bandeja del sistema' },
    },
    hotkeys: {
      actions: {
        'price-check': { label: 'Price check', description: 'Panel completo del objeto bajo el cursor' },
        'price-check-locked': { label: 'Price check fijado', description: 'Abre el panel ya fijado en la pantalla' },
        'toggle-overlay': { label: 'Mostrar/ocultar overlay', description: 'Alterna la visibilidad de los paneles' },
        'open-settings': { label: 'Abrir ajustes', description: 'Esta ventana' },
      },
      palette: { label: 'Paleta de comandos', description: 'Solo dentro de Mirrorsight · no editable' },
      saved: 'Guardado',
      defaultRestored: 'Valor por defecto restaurado',
      pressCombo: 'Pulsa la combinación…',
      restoreDefault: 'Restaurar valor por defecto',
      fixedShortcut: 'Atajo fijo',
    },
    priceCheck: {
      searchSection: 'Búsqueda',
      modRange: { label: 'Rango de mods', description: 'Mínimo prerrellenado = valor obtenido − %' },
      showOffline: { label: 'Mostrar vendedores desconectados', description: 'Por defecto, solo conectados' },
      defaultFilters: { label: 'Filtros marcados por defecto', description: 'Tipos de mod que ya vienen seleccionados' },
      propertyKind: 'Atributos (defensas/DPS)',
      suggestedSection: 'Precio sugerido',
      newBadge: 'NUEVO',
      defaultLevel: { label: 'Nivel por defecto', description: 'Cuál de los tres niveles viene seleccionado y va a la nota ~b/o' },
      defaultLevelAria: 'Nivel por defecto del precio sugerido',
      levelsInfo: (b, f, s) =>
        `Los niveles son percentiles de los anuncios cargados, en chaos: ${b} = p25, ${f} = mediana, ${s} = p75. Con menos de 5 anuncios los tres muestran “—”.`,
      localSection: 'Datos locales',
      history: {
        label: 'Historial',
        description: (n, max) => `${n} de ${max} price checks guardados en este equipo`,
        search: 'price checks guardados en este equipo',
      },
      viewHistory: 'Ver historial',
      clear: 'Borrar',
      presets: {
        label: 'Presets de mods',
        description: (a, b) => `${a} en PoE 1 · ${b} en PoE 2`,
        search: 'en PoE 1 en PoE 2',
        error: 'No se pudieron contar los presets',
      },
      presetsNote: 'Gestiónalos desde el menú Preset en Price Check',
      confirmLabel: 'Confirmar borrado',
      confirmTitle: (n) => `¿Borrar los ${n} price checks?`,
      confirmBody: 'No se puede deshacer. Los presets y los ajustes no se ven afectados.',
      cancel: 'Cancelar',
    },
    appearance: {
      scale: { label: 'Escala de la interfaz', description: 'Para monitores 1440p/4K' },
      scaleAria: 'Escala',
      density: { label: 'Densidad', description: 'Altura de las filas de los listados' },
      compact: 'Compacta',
      normal: 'Normal',
      reduceMotion: { label: 'Reducir animaciones', description: 'Cambio instantáneo, sin movimiento' },
    },
    about: {
      version: (v) => `Versión ${v} · ${STACK}`,
      demo: 'Modo demostración (fuera de Tauri)',
      credits:
        'Proyecto independiente, no afiliado a Grinding Gear Games. Iconos de objetos © Grinding Gear Games, extraídos de la instalación local del juego.',
    },
  },
  de: {
    screens: { price: 'Price Check', history: 'Verlauf', settings: 'Einstellungen' },
    panes: { general: 'Allgemein', hotkeys: 'Hotkeys', 'price-check': 'Price check', appearance: 'Darstellung', about: 'Über' },
    paneDescriptions: {
      general: 'Spiel, Liga und grundlegendes Verhalten des Overlays.',
      hotkeys: 'Klicke auf eine Taste und drücke die neue Kombination. Esc bricht ab.',
      'price-check': 'Standardfilter, Suchbereiche, Preisvorschlag und lokale Daten.',
      appearance: 'Skalierung, Dichte und Bewegung.',
      about: 'Version und Credits.',
    },
    view: { aria: 'Einstellungen', sectionsAria: 'Bereiche der Einstellungen', searchPlaceholder: 'Einstellung suchen…', searchAria: 'Einstellung suchen', loading: 'Wird geladen…' },
    general: {
      game: { label: 'Spiel', description: 'Legt Parser, Mod-Datenbank und verwendete API fest' },
      league: { label: 'Liga', description: 'Die erste ist die aktuelle Challenge-Liga' },
      leagueError: 'Ligen konnten nicht geladen werden — gespeicherte wird verwendet',
      currency: { label: 'Anzeigewährung', description: 'Werte über 1 Divine werden immer in Divine angezeigt' },
      restoreClipboard: { label: 'Zwischenablage wiederherstellen', description: 'Stellt nach dem Kopieren des Items den vorherigen Inhalt wieder her' },
      interceptCopy: { label: 'Strg+C öffnet den Price Check', description: 'Strg+C über einem Item im Spiel liest die Tiers und öffnet das Overlay; der Text wird weiterhin kopiert, bis zu ~0,2 s verzögert (auch im Chat)' },
      startup: { label: 'Mit Windows starten', description: 'Bleibt im Infobereich der Taskleiste' },
    },
    hotkeys: {
      actions: {
        'price-check': { label: 'Price check', description: 'Vollständiges Panel für das Item unter dem Mauszeiger' },
        'price-check-locked': { label: 'Fixierter Price check', description: 'Öffnet das Panel direkt fixiert' },
        'toggle-overlay': { label: 'Overlay ein-/ausblenden', description: 'Schaltet die Sichtbarkeit der Panels um' },
        'open-settings': { label: 'Einstellungen öffnen', description: 'Dieses Fenster' },
      },
      palette: { label: 'Befehlspalette', description: 'Nur innerhalb von Mirrorsight · nicht änderbar' },
      saved: 'Gespeichert',
      defaultRestored: 'Standard wiederhergestellt',
      pressCombo: 'Kombination drücken…',
      restoreDefault: 'Standard wiederherstellen',
      fixedShortcut: 'Fester Hotkey',
    },
    priceCheck: {
      searchSection: 'Suche',
      modRange: { label: 'Mod-Wertebereich', description: 'Vorausgefülltes Minimum = gerollter Wert − %' },
      showOffline: { label: 'Offline-Verkäufer anzeigen', description: 'Standardmäßig nur online' },
      defaultFilters: { label: 'Standardmäßig aktive Filter', description: 'Mod-Typen, die bereits ausgewählt sind' },
      propertyKind: 'Eigenschaften (Verteidigung/DPS)',
      suggestedSection: 'Preisvorschlag',
      newBadge: 'NEU',
      defaultLevel: { label: 'Standardstufe', description: 'Welche der drei Stufen vorausgewählt ist und in die ~b/o-Notiz kommt' },
      defaultLevelAria: 'Standardstufe des Preisvorschlags',
      levelsInfo: (b, f, s) =>
        `Die Stufen sind Perzentile der geladenen Angebote, in Chaos: ${b} = p25, ${f} = Median, ${s} = p75. Bei weniger als 5 Angeboten zeigen alle drei „—“.`,
      localSection: 'Lokale Daten',
      history: {
        label: 'Verlauf',
        description: (n, max) => `${n} von ${max} Price checks auf diesem Computer gespeichert`,
        search: 'Price checks auf diesem Computer gespeichert',
      },
      viewHistory: 'Verlauf ansehen',
      clear: 'Leeren',
      presets: {
        label: 'Mod-Presets',
        description: (a, b) => `${a} in PoE 1 · ${b} in PoE 2`,
        search: 'in PoE 1 in PoE 2',
        error: 'Presets konnten nicht gezählt werden',
      },
      presetsNote: 'Verwaltung über das Preset-Menü im Price Check',
      confirmLabel: 'Leeren bestätigen',
      confirmTitle: (n) => `Alle ${n} Price checks löschen?`,
      confirmBody: 'Kann nicht rückgängig gemacht werden. Presets und Einstellungen bleiben erhalten.',
      cancel: 'Abbrechen',
    },
    appearance: {
      scale: { label: 'Skalierung der Oberfläche', description: 'Für 1440p-/4K-Monitore' },
      scaleAria: 'Skalierung',
      density: { label: 'Dichte', description: 'Zeilenhöhe der Angebotslisten' },
      compact: 'Kompakt',
      normal: 'Normal',
      reduceMotion: { label: 'Animationen reduzieren', description: 'Sofortiger Wechsel, keine Bewegung' },
    },
    about: {
      version: (v) => `Version ${v} · ${STACK}`,
      demo: 'Demomodus (außerhalb von Tauri)',
      credits:
        'Unabhängiges Projekt, nicht mit Grinding Gear Games verbunden. Item-Icons © Grinding Gear Games, aus der lokalen Spielinstallation extrahiert.',
    },
  },
  'zh-CN': {
    screens: { price: 'Price Check', history: '历史记录', settings: '设置' },
    panes: { general: '常规', hotkeys: '快捷键', 'price-check': 'Price check', appearance: '外观', about: '关于' },
    paneDescriptions: {
      general: '游戏、赛季和悬浮窗的基本行为。',
      hotkeys: '点击按键后按下新的组合键。Esc 取消。',
      'price-check': '默认筛选、搜索范围、建议价格和本地数据。',
      appearance: '缩放、密度和动画。',
      about: '版本与致谢。',
    },
    view: { aria: '设置', sectionsAria: '设置分区', searchPlaceholder: '搜索设置…', searchAria: '搜索设置', loading: '加载中…' },
    general: {
      game: { label: '游戏', description: '决定使用的解析器、词缀库和 API' },
      league: { label: '赛季', description: '第一个是当前的挑战赛季' },
      leagueError: '无法加载赛季列表 — 使用已保存的赛季',
      currency: { label: '显示货币', description: '超过 1 divine 的价格始终以 divine 显示' },
      restoreClipboard: { label: '恢复剪贴板', description: '复制物品后恢复之前的剪贴板内容' },
      interceptCopy: { label: 'Ctrl+C 打开查价', description: '在游戏中对物品按 Ctrl+C 会读取阶级并打开浮窗；文本仍会被复制，但最多延迟约 0.2 秒（聊天中也是）' },
      startup: { label: '开机自启', description: '驻留在系统托盘' },
    },
    hotkeys: {
      actions: {
        'price-check': { label: 'Price check', description: '光标下物品的完整面板' },
        'price-check-locked': { label: '固定的 Price check', description: '打开时面板已固定在屏幕上' },
        'toggle-overlay': { label: '显示/隐藏悬浮窗', description: '切换面板的可见性' },
        'open-settings': { label: '打开设置', description: '本窗口' },
      },
      palette: { label: '命令面板', description: '仅在 Mirrorsight 内 · 不可修改' },
      saved: '已保存',
      defaultRestored: '已恢复默认',
      pressCombo: '请按下组合键…',
      restoreDefault: '恢复默认',
      fixedShortcut: '固定快捷键',
    },
    priceCheck: {
      searchSection: '搜索',
      modRange: { label: '词缀数值范围', description: '预填最小值 = 实际数值 − %' },
      showOffline: { label: '显示离线卖家', description: '默认仅显示在线' },
      defaultFilters: { label: '默认勾选的筛选', description: '默认选中的词缀类型' },
      propertyKind: '属性（防御/DPS）',
      suggestedSection: '建议价格',
      newBadge: '新',
      defaultLevel: { label: '默认档位', description: '三个档位中默认选中并写入 ~b/o 备注的那个' },
      defaultLevelAria: '建议价格的默认档位',
      levelsInfo: (b, f, s) =>
        `档位是已加载挂单的百分位数（以 chaos 计）：${b} = p25，${f} = 中位数，${s} = p75。挂单少于 5 个时三者均显示“—”。`,
      localSection: '本地数据',
      history: {
        label: '历史记录',
        description: (n, max) => `本机已保存 ${n} / ${max} 条 price check`,
        search: '本机已保存 price check',
      },
      viewHistory: '查看历史记录',
      clear: '清空',
      presets: {
        label: '词缀预设',
        description: (a, b) => `PoE 1：${a} 个 · PoE 2：${b} 个`,
        search: 'PoE 1 PoE 2',
        error: '无法统计预设数量',
      },
      presetsNote: '请在 Price Check 的 Preset 菜单中管理',
      confirmLabel: '确认清空',
      confirmTitle: (n) => `删除全部 ${n} 条 price check？`,
      confirmBody: '此操作无法撤销。预设和设置不受影响。',
      cancel: '取消',
    },
    appearance: {
      scale: { label: '界面缩放', description: '适用于 1440p/4K 显示器' },
      scaleAria: '缩放',
      density: { label: '密度', description: '列表的行高' },
      compact: '紧凑',
      normal: '标准',
      reduceMotion: { label: '减少动画', description: '即时切换，无动画' },
    },
    about: {
      version: (v) => `版本 ${v} · ${STACK}`,
      demo: '演示模式（未在 Tauri 中运行）',
      credits: '独立项目，与 Grinding Gear Games 无关。物品图标 © Grinding Gear Games，提取自本地游戏安装目录。',
    },
  },
  ja: {
    screens: { price: 'Price Check', history: '履歴', settings: '設定' },
    panes: { general: '一般', hotkeys: 'ホットキー', 'price-check': 'Price check', appearance: '外観', about: '情報' },
    paneDescriptions: {
      general: 'ゲーム、リーグ、オーバーレイの基本動作。',
      hotkeys: 'キーをクリックして新しい組み合わせを押してください。Esc でキャンセル。',
      'price-check': 'デフォルトのフィルター、検索範囲、推奨価格、ローカルデータ。',
      appearance: 'スケール、表示密度、アニメーション。',
      about: 'バージョンとクレジット。',
    },
    view: { aria: '設定', sectionsAria: '設定のセクション', searchPlaceholder: '設定を検索…', searchAria: '設定を検索', loading: '読み込み中…' },
    general: {
      game: { label: 'ゲーム', description: '使用するパーサー、Mod データベース、API を決めます' },
      league: { label: 'リーグ', description: '先頭が現在のチャレンジリーグです' },
      leagueError: 'リーグを読み込めませんでした — 保存済みのリーグを使用中',
      currency: { label: '表示通貨', description: '1 divine を超える値は常に divine で表示されます' },
      restoreClipboard: { label: 'クリップボードの復元', description: 'アイテムのコピー後に以前の内容を戻します' },
      interceptCopy: { label: 'Ctrl+C で価格チェック', description: 'ゲーム内でアイテムに Ctrl+C するとTierを読み取りオーバーレイを開きます。テキストは通常どおりコピーされますが、最大約0.2秒遅れます（チャットも同様）' },
      startup: { label: 'Windows 起動時に開始', description: 'システムトレイに常駐します' },
    },
    hotkeys: {
      actions: {
        'price-check': { label: 'Price check', description: 'カーソル下のアイテムの詳細パネル' },
        'price-check-locked': { label: '固定 Price check', description: 'パネルを画面に固定した状態で開きます' },
        'toggle-overlay': { label: 'オーバーレイの表示/非表示', description: 'パネルの表示を切り替えます' },
        'open-settings': { label: '設定を開く', description: 'このウィンドウ' },
      },
      palette: { label: 'コマンドパレット', description: 'Mirrorsight 内のみ · 変更不可' },
      saved: '保存しました',
      defaultRestored: 'デフォルトに戻しました',
      pressCombo: '組み合わせを押してください…',
      restoreDefault: 'デフォルトに戻す',
      fixedShortcut: '固定ショートカット',
    },
    priceCheck: {
      searchSection: '検索',
      modRange: { label: 'Mod の範囲', description: '最小値の初期値 = ロール値 − %' },
      showOffline: { label: 'オフライン出品者を表示', description: 'デフォルトではオンラインのみ' },
      defaultFilters: { label: 'デフォルトで有効なフィルター', description: '最初から選択される Mod の種類' },
      propertyKind: 'プロパティ（防御/DPS）',
      suggestedSection: '推奨価格',
      newBadge: 'NEW',
      defaultLevel: { label: 'デフォルトのレベル', description: '3 つのレベルのうち最初に選択され、~b/o メモに入るもの' },
      defaultLevelAria: '推奨価格のデフォルトレベル',
      levelsInfo: (b, f, s) =>
        `レベルは読み込んだ出品のパーセンタイル（chaos 換算）です：${b} = p25、${f} = 中央値、${s} = p75。出品が 5 件未満の場合は 3 つとも「—」になります。`,
      localSection: 'ローカルデータ',
      history: {
        label: '履歴',
        description: (n, max) => `このコンピューターに保存された price check：${n} / ${max} 件`,
        search: 'このコンピューターに保存された price check',
      },
      viewHistory: '履歴を見る',
      clear: '消去',
      presets: {
        label: 'Mod プリセット',
        description: (a, b) => `PoE 1：${a} 件 · PoE 2：${b} 件`,
        search: 'PoE 1 PoE 2',
        error: 'プリセット数を取得できませんでした',
      },
      presetsNote: 'Price Check の Preset メニューで管理します',
      confirmLabel: '消去の確認',
      confirmTitle: (n) => `${n} 件の price check を削除しますか？`,
      confirmBody: '元に戻せません。プリセットと設定には影響しません。',
      cancel: 'キャンセル',
    },
    appearance: {
      scale: { label: 'UI スケール', description: '1440p/4K モニター向け' },
      scaleAria: 'スケール',
      density: { label: '表示密度', description: '出品リストの行の高さ' },
      compact: 'コンパクト',
      normal: '標準',
      reduceMotion: { label: 'アニメーションを減らす', description: '即座に切り替え、動きなし' },
    },
    about: {
      version: (v) => `バージョン ${v} · ${STACK}`,
      demo: 'デモモード（Tauri 外）',
      credits:
        '独立したプロジェクトであり、Grinding Gear Games とは提携していません。アイテムアイコン © Grinding Gear Games（ローカルのゲームインストールから抽出）。',
    },
  },
  ko: {
    screens: { price: 'Price Check', history: '기록', settings: '설정' },
    panes: { general: '일반', hotkeys: '단축키', 'price-check': 'Price check', appearance: '모양', about: '정보' },
    paneDescriptions: {
      general: '게임, 리그 및 오버레이 기본 동작.',
      hotkeys: '키를 클릭한 뒤 새 조합을 누르세요. Esc로 취소합니다.',
      'price-check': '기본 필터, 검색 범위, 추천 가격, 로컬 데이터.',
      appearance: '배율, 밀도, 애니메이션.',
      about: '버전 및 크레딧.',
    },
    view: { aria: '설정', sectionsAria: '설정 섹션', searchPlaceholder: '설정 검색…', searchAria: '설정 검색', loading: '불러오는 중…' },
    general: {
      game: { label: '게임', description: '사용할 파서, 옵션 데이터베이스, API를 정합니다' },
      league: { label: '리그', description: '첫 번째가 현재 챌린지 리그입니다' },
      leagueError: '리그를 불러올 수 없음 — 저장된 리그 사용 중',
      currency: { label: '표시 화폐', description: '1 divine을 넘는 값은 항상 divine으로 표시됩니다' },
      restoreClipboard: { label: '클립보드 복원', description: '아이템 복사 후 이전 내용을 되돌립니다' },
      interceptCopy: { label: 'Ctrl+C로 시세 확인', description: '게임에서 아이템 위에서 Ctrl+C를 누르면 티어를 읽고 오버레이를 엽니다. 텍스트는 그대로 복사되지만 최대 약 0.2초 늦어집니다(채팅 포함)' },
      startup: { label: 'Windows 시작 시 실행', description: '시스템 트레이에 머무릅니다' },
    },
    hotkeys: {
      actions: {
        'price-check': { label: 'Price check', description: '커서 아래 아이템의 전체 패널' },
        'price-check-locked': { label: '고정 Price check', description: '패널을 화면에 고정된 상태로 엽니다' },
        'toggle-overlay': { label: '오버레이 표시/숨기기', description: '패널 표시 여부를 전환합니다' },
        'open-settings': { label: '설정 열기', description: '이 창' },
      },
      palette: { label: '명령 팔레트', description: 'Mirrorsight 안에서만 · 변경 불가' },
      saved: '저장됨',
      defaultRestored: '기본값으로 복원됨',
      pressCombo: '조합을 누르세요…',
      restoreDefault: '기본값 복원',
      fixedShortcut: '고정 단축키',
    },
    priceCheck: {
      searchSection: '검색',
      modRange: { label: '옵션 수치 범위', description: '미리 채워지는 최소값 = 굴린 값 − %' },
      showOffline: { label: '오프라인 판매자 표시', description: '기본은 온라인만' },
      defaultFilters: { label: '기본 선택 필터', description: '미리 선택되는 옵션 종류' },
      propertyKind: '속성 (방어/DPS)',
      suggestedSection: '추천 가격',
      newBadge: 'NEW',
      defaultLevel: { label: '기본 단계', description: '세 단계 중 미리 선택되어 ~b/o 메모에 들어가는 단계' },
      defaultLevelAria: '추천 가격 기본 단계',
      levelsInfo: (b, f, s) =>
        `단계는 불러온 매물의 백분위수(chaos 기준)입니다: ${b} = p25, ${f} = 중앙값, ${s} = p75. 매물이 5개 미만이면 세 단계 모두 “—”로 표시됩니다.`,
      localSection: '로컬 데이터',
      history: {
        label: '기록',
        description: (n, max) => `이 컴퓨터에 저장된 price check ${n} / ${max}개`,
        search: '이 컴퓨터에 저장된 price check',
      },
      viewHistory: '기록 보기',
      clear: '지우기',
      presets: {
        label: '옵션 프리셋',
        description: (a, b) => `PoE 1: ${a}개 · PoE 2: ${b}개`,
        search: 'PoE 1 PoE 2',
        error: '프리셋 수를 셀 수 없음',
      },
      presetsNote: 'Price Check의 Preset 메뉴에서 관리하세요',
      confirmLabel: '지우기 확인',
      confirmTitle: (n) => `price check ${n}개를 모두 삭제할까요?`,
      confirmBody: '되돌릴 수 없습니다. 프리셋과 설정은 영향을 받지 않습니다.',
      cancel: '취소',
    },
    appearance: {
      scale: { label: 'UI 배율', description: '1440p/4K 모니터용' },
      scaleAria: '배율',
      density: { label: '밀도', description: '매물 목록의 행 높이' },
      compact: '좁게',
      normal: '보통',
      reduceMotion: { label: '애니메이션 줄이기', description: '즉시 전환, 움직임 없음' },
    },
    about: {
      version: (v) => `버전 ${v} · ${STACK}`,
      demo: '데모 모드 (Tauri 외부)',
      credits:
        '독립 프로젝트이며 Grinding Gear Games와 관련이 없습니다. 아이템 아이콘 © Grinding Gear Games, 로컬 게임 설치본에서 추출.',
    },
  },
});

/**
 * Palavras-chave da busca de ajustes (fixas, em português e inglês). Usadas pelo `SettingRow`
 * e pela lista de busca de `SettingsView`, então as duas ficam sempre iguais.
 */
export const SETTINGS_KEYWORDS = {
  game: 'poe path of exile 2 versão version',
  league: 'league liga temporada season',
  gameLanguage: 'language lingua client',
  currency: 'chaos divine conversão conversion currency',
  restoreClipboard: 'clipboard área de transferência',
  interceptCopy: 'ctrl c copiar copy tier prefix suffix price check overlay',
  startup: 'startup inicialização bandeja tray boot',
  hotkey: 'hotkey atalho tecla shortcut key',
  palette: 'hotkey atalho ctrl k comandos busca shortcut command palette search',
  modRange: 'porcentagem filtro mínimo margem percent filter minimum margin',
  showOffline: 'offline afk sellers',
  defaultFilters: 'explicit fractured implicit crafted enchant',
  defaultLevel:
    'compra justo venda percentil p25 p50 p75 mediana sugerido estratégia buy fair sell percentile median suggested strategy',
  history: 'histórico apagar limpar recentes history delete clear recent',
  presets: 'preset salvos mods seleção saved selection',
  scale: 'zoom tamanho size',
  density: 'compacto compact',
  reduceMotion: 'motion acessibilidade accessibility animation',
} as const;
