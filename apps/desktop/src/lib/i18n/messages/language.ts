import { defineMessages } from '../core.svelte';

interface LanguageMessages {
  label: string;
  description: string;
  /** Opção `null`: segue o idioma do Windows. */
  automatic: (detected: string) => string;
  gameLabel: string;
  gameDescription: string;
}

/** Linha "Idioma da interface" em Configurações › Geral. */
export const languageMsgs = defineMessages<LanguageMessages>({
  'pt-BR': {
    label: 'Idioma da interface',
    description: 'Idioma dos menus, botões e mensagens do Mirrorsight',
    automatic: (d) => `Automático (${d})`,
    gameLabel: 'Idioma do cliente do jogo',
    gameDescription: 'Idioma em que o jogo copia os itens',
  },
  en: {
    label: 'Interface language',
    description: 'Language of Mirrorsight menus, buttons and messages',
    automatic: (d) => `Automatic (${d})`,
    gameLabel: 'Game client language',
    gameDescription: 'Language the game uses when copying items',
  },
  es: {
    label: 'Idioma de la interfaz',
    description: 'Idioma de los menús, botones y mensajes de Mirrorsight',
    automatic: (d) => `Automático (${d})`,
    gameLabel: 'Idioma del cliente del juego',
    gameDescription: 'Idioma en el que el juego copia los objetos',
  },
  de: {
    label: 'Sprache der Oberfläche',
    description: 'Sprache der Menüs, Schaltflächen und Meldungen von Mirrorsight',
    automatic: (d) => `Automatisch (${d})`,
    gameLabel: 'Sprache des Spielclients',
    gameDescription: 'Sprache, in der das Spiel Gegenstände kopiert',
  },
  'zh-CN': {
    label: '界面语言',
    description: 'Mirrorsight 菜单、按钮和提示所使用的语言',
    automatic: (d) => `自动（${d}）`,
    gameLabel: '游戏客户端语言',
    gameDescription: '游戏复制物品时使用的语言',
  },
  ja: {
    label: '表示言語',
    description: 'Mirrorsight のメニュー、ボタン、メッセージの言語',
    automatic: (d) => `自動（${d}）`,
    gameLabel: 'ゲームクライアントの言語',
    gameDescription: 'ゲームがアイテムをコピーするときの言語',
  },
  ko: {
    label: '인터페이스 언어',
    description: 'Mirrorsight 메뉴, 버튼, 메시지에 사용할 언어',
    automatic: (d) => `자동 (${d})`,
    gameLabel: '게임 클라이언트 언어',
    gameDescription: '게임이 아이템을 복사할 때 사용하는 언어',
  },
});

/** Palavras para a busca das configurações achar a linha em qualquer idioma. */
export const LANGUAGE_KEYWORDS = 'language idioma lingua sprache 语言 言語 언어 interface ui';
