import type {
  AppError,
  BuildDetails,
  BuildFile,
  CatalogStat,
  GameVersion,
  HistoryEntry,
  HotkeyBinding,
  ModPreset,
  PriceCheckOutput,
  SearchFilters,
  Settings,
} from '$lib/types';
import { common } from '$lib/i18n/messages/common';
import { describeReason } from '$lib/i18n/messages/reasons';

/**
 * Porta única entre a UI e o mundo externo (Dependency Inversion).
 * Componentes e stores dependem só desta interface; o adapter concreto é
 * escolhido em `backend/index.ts`.
 */
export interface Backend {
  readonly kind: 'tauri' | 'demo';
  getSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<void>;
  /** Valida com as regras do domínio sem persistir. Rejeita com `AppError`. */
  validateHotkeys(bindings: HotkeyBinding[]): Promise<void>;
  /** Ligas de trade do jogo; a primeira é a liga de desafio atual. */
  listLeagues(game: GameVersion): Promise<string[]>;
  /** `filters` ausente → o backend usa os filtros padrão. */
  priceCheckText(text: string, filters?: SearchFilters): Promise<PriceCheckOutput>;
  openExternal(url: string): Promise<void>;
  /** Zoom nativo do webview (mantém coordenadas de layout consistentes). */
  setUiScale(scale: number): Promise<void>;
  /** Traduz menu e tooltip da bandeja (no navegador não faz nada). */
  setTrayLabels(labels: TrayLabels): Promise<void>;

  /** Catálogo de stats do jogo (busca "Adicionar mod"). */
  listStats(game: GameVersion): Promise<CatalogStat[]>;
  /** Histórico local, mais recentes primeiro (máx. 50). */
  getHistory(): Promise<HistoryEntry[]>;
  /** Registra um price check; devolve o histórico atualizado. */
  recordHistory(entry: HistoryEntry): Promise<HistoryEntry[]>;
  removeHistory(id: string): Promise<HistoryEntry[]>;
  clearHistory(): Promise<void>;
  /** Presets de mods salvos para o jogo. */
  listPresets(game: GameVersion): Promise<ModPreset[]>;
  /** Cria/atualiza pelo id; devolve a lista do jogo do preset. Rejeita com `invalid-settings`. */
  upsertPreset(preset: ModPreset): Promise<ModPreset[]>;
  deletePreset(id: string): Promise<void>;
  /** Builds salvos do Path of Building (pasta padrão em Documentos). */
  listPobBuilds(): Promise<BuildFile[]>;
  /** O que a avaliação entende de um build com os sets escolhidos (`null` = ativo no PoB), e os sets. */
  pobBuildDetails(path: string, itemSet: string | null, skillSet: string | null): Promise<BuildDetails>;

  /** Qual janela está rodando este frontend. */
  windowRole(): WindowRole;
  /** Esconde o overlay e devolve o foco ao jogo. */
  hideOverlay(): Promise<void>;
  /** Começa a arrastar a janela atual (chamar no `mousedown`). */
  startWindowDrag(): Promise<void>;
  /**
   * Alarga o overlay em `extra` px lógicos para mostrar o card do olho fora do painel
   * (`open = false` restaura). Retorna para onde a janela cresceu.
   */
  setOverlayPeek(open: boolean, extra: number): Promise<PeekSide>;
  /** Item copiado do jogo pela hotkey. Retorna a função para cancelar a inscrição. */
  onPriceCheckRequest(handler: (request: PriceCheckRequest) => void): Promise<() => void>;
  /** Falha ao copiar o item (jogo sem foco, elevação…). */
  onPriceCheckError(handler: (error: AppError) => void): Promise<() => void>;
  /** Hotkey "mostrar overlay" com o painel fechado: reabrir o último price check. */
  onOverlayReopen(handler: () => void): Promise<() => void>;
  /** Pedido (bandeja/hotkey) para abrir a aba de configurações. */
  onOpenSettings(handler: () => void): Promise<() => void>;

  /** Atualização já baixada e pronta para instalar (`null` = nenhuma). */
  updateStatus(): Promise<UpdateInfo | null>;
  /** Checagem manual no GitHub; se houver versão nova, baixa (sem instalar). Rejeita com `AppError`. */
  checkForUpdate(): Promise<UpdateCheck>;
  /** Instala a atualização baixada e reinicia o app. */
  installUpdate(): Promise<void>;
  /** Atualização baixada em segundo plano durante a sessão. */
  onUpdateReady(handler: (info: UpdateInfo) => void): Promise<() => void>;

  /** Controles da janela (barra de título própria). `false` = navegador, sem controles. */
  readonly hasWindowControls: boolean;
  minimizeWindow(): Promise<void>;
  toggleMaximizeWindow(): Promise<void>;
  /** Na janela principal, "fechar" esconde na bandeja (ver `src-tauri/src/lib.rs`). */
  closeWindow(): Promise<void>;
  isWindowMaximized(): Promise<boolean>;
  onWindowResized(handler: () => void): Promise<() => void>;
}

export interface TrayLabels {
  open: string;
  settings: string;
  quit: string;
  tooltip: string;
  /** "Reiniciar para atualizar" (o Rust acrescenta a versão). */
  update: string;
}

/** Espelho de `UpdateInfo` (src-tauri/src/updater.rs). */
export interface UpdateInfo {
  version: string;
  notes: string | null;
}

/** Espelho de `UpdateCheck` (src-tauri/src/updater.rs). */
export type UpdateCheck = { status: 'up-to-date' } | ({ status: 'ready' } & UpdateInfo);

export type WindowRole = 'main' | 'overlay';

/** `inside` = sem espaço no monitor: o card fica sobre o próprio painel. */
export type PeekSide = 'right' | 'left' | 'inside';

export interface PriceCheckRequest {
  text: string;
  /** Abrir já fixado. */
  locked: boolean;
}

const KNOWN_CODES = new Set<AppError['code']>([
  'nothing-copied',
  'item-not-recognized',
  'rate-limited',
  'network',
  'invalid-settings',
  'internal',
]);

/** Normaliza qualquer rejeição (erro do Rust, exceção JS, string) em `AppError`. */
export function toAppError(err: unknown): AppError {
  if (typeof err === 'object' && err !== null && 'code' in err) {
    const code = (err as { code: unknown }).code;
    if (typeof code === 'string' && KNOWN_CODES.has(code as AppError['code'])) {
      return err as AppError;
    }
  }
  const message = err instanceof Error ? err.message : String(err);
  return { code: 'internal', data: { message } };
}

/** Mensagem curta, no idioma da interface, para exibir ao usuário. */
export function describeError(err: AppError): string {
  switch (err.code) {
    case 'nothing-copied':
      return common.errors.nothingCopied;
    case 'item-not-recognized':
      return common.errors.itemNotRecognized;
    case 'rate-limited':
      return common.errors.rateLimited;
    case 'network':
      return err.data.reason ? describeReason(err.data.reason) : common.errors.network(err.data.message);
    case 'invalid-settings':
    case 'internal':
      // Sem motivo conhecido, a mensagem é técnica (ex.: erro da trade API) e vai como veio.
      return err.data.reason ? describeReason(err.data.reason) : err.data.message;
  }
}
