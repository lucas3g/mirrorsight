import type { ModifierKind } from '$lib/types';
import { defineMessages } from '../core.svelte';

type KindMap = Record<ModifierKind, string>;

interface CommonMessages {
  /** `relativeAge`/`relativeWhen`. */
  now: string;
  minutesAgo: (n: number) => string;
  hoursAgo: (n: number) => string;
  yesterday: string;
  /** Rótulo curto do tipo de mod (badges/filtros). */
  kindShort: KindMap;
  /** Singular, para badges. */
  kindName: KindMap;
  /** Títulos dos grupos no editor de mods. */
  kindPlural: KindMap;
  errors: {
    nothingCopied: string;
    itemNotRecognized: string;
    rateLimited: string;
    network: (detail: string) => string;
  };
  tray: { open: string; settings: string; quit: string; tooltip: (hotkey: string) => string; update: string };
}

// Termos do jogo (fractured, enchant, desecrated, pseudo…) ficam como o cliente em inglês os mostra.
export const common = defineMessages<CommonMessages>({
  'pt-BR': {
    now: 'agora',
    minutesAgo: (n) => `há ${n} min`,
    hoursAgo: (n) => `há ${n} h`,
    yesterday: 'ontem',
    kindShort: { implicit: 'implicit', explicit: 'explicit', crafted: 'crafted', fractured: 'fractured', enchant: 'enchant', rune: 'rune', desecrated: 'desecrated', pseudo: 'pseudo', property: 'atributo' },
    kindName: { implicit: 'Implícito', explicit: 'Explícito', crafted: 'Craftado', fractured: 'Fractured', enchant: 'Enchant', rune: 'Runa', desecrated: 'Desecrated', pseudo: 'Pseudo', property: 'Atributo' },
    kindPlural: { explicit: 'Explícitos', fractured: 'Fractured', implicit: 'Implícitos', crafted: 'Craftados', enchant: 'Enchants', rune: 'Runas', desecrated: 'Desecrated', pseudo: 'Pseudo (totais)', property: 'Atributos' },
    errors: {
      nothingCopied: 'Nenhum item foi copiado. O jogo está em foco?',
      itemNotRecognized: 'O texto copiado não parece um item do Path of Exile.',
      rateLimited: 'Limite da API atingido. Aguarde um instante.',
      network: (d) => `Falha de rede: ${d}`,
    },
    tray: { open: 'Abrir Mirrorsight', settings: 'Configurações', quit: 'Sair', tooltip: (k) => `Mirrorsight — ${k} sobre um item no jogo`, update: 'Reiniciar para atualizar' },
  },
  en: {
    now: 'now',
    minutesAgo: (n) => `${n} min ago`,
    hoursAgo: (n) => `${n} h ago`,
    yesterday: 'yesterday',
    kindShort: { implicit: 'implicit', explicit: 'explicit', crafted: 'crafted', fractured: 'fractured', enchant: 'enchant', rune: 'rune', desecrated: 'desecrated', pseudo: 'pseudo', property: 'property' },
    kindName: { implicit: 'Implicit', explicit: 'Explicit', crafted: 'Crafted', fractured: 'Fractured', enchant: 'Enchant', rune: 'Rune', desecrated: 'Desecrated', pseudo: 'Pseudo', property: 'Property' },
    kindPlural: { explicit: 'Explicit', fractured: 'Fractured', implicit: 'Implicit', crafted: 'Crafted', enchant: 'Enchants', rune: 'Runes', desecrated: 'Desecrated', pseudo: 'Pseudo (totals)', property: 'Properties' },
    errors: {
      nothingCopied: 'No item was copied. Is the game focused?',
      itemNotRecognized: "The copied text doesn't look like a Path of Exile item.",
      rateLimited: 'API rate limit reached. Wait a moment.',
      network: (d) => `Network error: ${d}`,
    },
    tray: { open: 'Open Mirrorsight', settings: 'Settings', quit: 'Quit', tooltip: (k) => `Mirrorsight — ${k} over an item in game`, update: 'Restart to update' },
  },
  es: {
    now: 'ahora',
    minutesAgo: (n) => `hace ${n} min`,
    hoursAgo: (n) => `hace ${n} h`,
    yesterday: 'ayer',
    kindShort: { implicit: 'implicit', explicit: 'explicit', crafted: 'crafted', fractured: 'fractured', enchant: 'enchant', rune: 'rune', desecrated: 'desecrated', pseudo: 'pseudo', property: 'atributo' },
    kindName: { implicit: 'Implícito', explicit: 'Explícito', crafted: 'Crafteado', fractured: 'Fractured', enchant: 'Enchant', rune: 'Runa', desecrated: 'Desecrated', pseudo: 'Pseudo', property: 'Atributo' },
    kindPlural: { explicit: 'Explícitos', fractured: 'Fractured', implicit: 'Implícitos', crafted: 'Crafteados', enchant: 'Enchants', rune: 'Runas', desecrated: 'Desecrated', pseudo: 'Pseudo (totales)', property: 'Atributos' },
    errors: {
      nothingCopied: 'No se copió ningún objeto. ¿El juego está en foco?',
      itemNotRecognized: 'El texto copiado no parece un objeto de Path of Exile.',
      rateLimited: 'Límite de la API alcanzado. Espera un momento.',
      network: (d) => `Error de red: ${d}`,
    },
    tray: { open: 'Abrir Mirrorsight', settings: 'Configuración', quit: 'Salir', tooltip: (k) => `Mirrorsight — ${k} sobre un objeto en el juego`, update: 'Reiniciar para actualizar' },
  },
  de: {
    now: 'jetzt',
    minutesAgo: (n) => `vor ${n} Min.`,
    hoursAgo: (n) => `vor ${n} Std.`,
    yesterday: 'gestern',
    kindShort: { implicit: 'implicit', explicit: 'explicit', crafted: 'crafted', fractured: 'fractured', enchant: 'enchant', rune: 'rune', desecrated: 'desecrated', pseudo: 'pseudo', property: 'Eigenschaft' },
    kindName: { implicit: 'Implizit', explicit: 'Explizit', crafted: 'Gecraftet', fractured: 'Fractured', enchant: 'Enchant', rune: 'Rune', desecrated: 'Desecrated', pseudo: 'Pseudo', property: 'Eigenschaft' },
    kindPlural: { explicit: 'Explizit', fractured: 'Fractured', implicit: 'Implizit', crafted: 'Gecraftet', enchant: 'Enchants', rune: 'Runen', desecrated: 'Desecrated', pseudo: 'Pseudo (Summen)', property: 'Eigenschaften' },
    errors: {
      nothingCopied: 'Es wurde kein Gegenstand kopiert. Ist das Spiel im Fokus?',
      itemNotRecognized: 'Der kopierte Text sieht nicht nach einem Path-of-Exile-Gegenstand aus.',
      rateLimited: 'API-Limit erreicht. Bitte kurz warten.',
      network: (d) => `Netzwerkfehler: ${d}`,
    },
    tray: { open: 'Mirrorsight öffnen', settings: 'Einstellungen', quit: 'Beenden', tooltip: (k) => `Mirrorsight — ${k} über einem Gegenstand im Spiel`, update: 'Neu starten zum Aktualisieren' },
  },
  'zh-CN': {
    now: '刚刚',
    minutesAgo: (n) => `${n} 分钟前`,
    hoursAgo: (n) => `${n} 小时前`,
    yesterday: '昨天',
    kindShort: { implicit: '基底', explicit: '词缀', crafted: '工艺', fractured: '破裂', enchant: '附魔', rune: '符文', desecrated: '亵渎', pseudo: '伪属性', property: '属性' },
    kindName: { implicit: '基底词缀', explicit: '显性词缀', crafted: '工艺词缀', fractured: '破裂词缀', enchant: '附魔', rune: '符文', desecrated: '亵渎词缀', pseudo: '伪属性', property: '属性' },
    kindPlural: { explicit: '显性词缀', fractured: '破裂词缀', implicit: '基底词缀', crafted: '工艺词缀', enchant: '附魔', rune: '符文', desecrated: '亵渎词缀', pseudo: '伪属性（合计）', property: '属性' },
    errors: {
      nothingCopied: '没有复制到物品。游戏窗口是否处于焦点？',
      itemNotRecognized: '复制的文本不像是流放之路的物品。',
      rateLimited: '已达到 API 速率限制，请稍候。',
      network: (d) => `网络错误：${d}`,
    },
    tray: { open: '打开 Mirrorsight', settings: '设置', quit: '退出', tooltip: (k) => `Mirrorsight — 在游戏中悬停物品按 ${k}`, update: '重启以更新' },
  },
  ja: {
    now: 'たった今',
    minutesAgo: (n) => `${n} 分前`,
    hoursAgo: (n) => `${n} 時間前`,
    yesterday: '昨日',
    kindShort: { implicit: 'implicit', explicit: 'explicit', crafted: 'crafted', fractured: 'fractured', enchant: 'enchant', rune: 'rune', desecrated: 'desecrated', pseudo: 'pseudo', property: '属性' },
    kindName: { implicit: 'インプリシット', explicit: 'エクスプリシット', crafted: 'クラフト', fractured: 'フラクチャード', enchant: 'エンチャント', rune: 'ルーン', desecrated: 'デセクレイテッド', pseudo: '疑似', property: '属性' },
    kindPlural: { explicit: 'エクスプリシット', fractured: 'フラクチャード', implicit: 'インプリシット', crafted: 'クラフト', enchant: 'エンチャント', rune: 'ルーン', desecrated: 'デセクレイテッド', pseudo: '疑似（合計）', property: '属性' },
    errors: {
      nothingCopied: 'アイテムがコピーされませんでした。ゲームがフォーカスされていますか？',
      itemNotRecognized: 'コピーされたテキストは Path of Exile のアイテムではないようです。',
      rateLimited: 'API のレート制限に達しました。少し待ってください。',
      network: (d) => `ネットワークエラー: ${d}`,
    },
    tray: { open: 'Mirrorsight を開く', settings: '設定', quit: '終了', tooltip: (k) => `Mirrorsight — ゲーム内のアイテムにカーソルを合わせて ${k}`, update: '再起動して更新' },
  },
  ko: {
    now: '방금',
    minutesAgo: (n) => `${n}분 전`,
    hoursAgo: (n) => `${n}시간 전`,
    yesterday: '어제',
    kindShort: { implicit: 'implicit', explicit: 'explicit', crafted: 'crafted', fractured: 'fractured', enchant: 'enchant', rune: 'rune', desecrated: 'desecrated', pseudo: 'pseudo', property: '속성' },
    kindName: { implicit: '고유 옵션', explicit: '부여 옵션', crafted: '제작 옵션', fractured: '분열 옵션', enchant: '인챈트', rune: '룬', desecrated: '모독 옵션', pseudo: '의사 옵션', property: '속성' },
    kindPlural: { explicit: '부여 옵션', fractured: '분열 옵션', implicit: '고유 옵션', crafted: '제작 옵션', enchant: '인챈트', rune: '룬', desecrated: '모독 옵션', pseudo: '의사 옵션 (합계)', property: '속성' },
    errors: {
      nothingCopied: '복사된 아이템이 없습니다. 게임 창이 활성화되어 있나요?',
      itemNotRecognized: '복사된 텍스트가 Path of Exile 아이템이 아닌 것 같습니다.',
      rateLimited: 'API 요청 한도에 도달했습니다. 잠시 기다려 주세요.',
      network: (d) => `네트워크 오류: ${d}`,
    },
    tray: { open: 'Mirrorsight 열기', settings: '설정', quit: '종료', tooltip: (k) => `Mirrorsight — 게임에서 아이템 위에 ${k}`, update: '재시작하여 업데이트' },
  },
});
