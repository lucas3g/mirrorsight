/** Navegação da janela principal: tela atual, seção das configurações e menu lateral. */
export type Screen = 'price' | 'history' | 'settings';
export type SettingsPane = 'general' | 'hotkeys' | 'price-check' | 'appearance' | 'about';

const RAIL_KEY = 'mirrorsight.rail.collapsed';

export const SCREEN_LABEL: Record<Screen, string> = {
  price: 'Price Check',
  history: 'Histórico',
  settings: 'Configurações',
};

export const PANE_LABEL: Record<SettingsPane, string> = {
  general: 'Geral',
  hotkeys: 'Hotkeys',
  'price-check': 'Price check',
  appearance: 'Aparência',
  about: 'Sobre',
};

class NavStore {
  screen = $state<Screen>('price');
  settingsPane = $state<SettingsPane>('general');
  /** Preferência do usuário (abaixo de 1180px o menu recolhe de qualquer jeito). */
  railCollapsed = $state(localStorage.getItem(RAIL_KEY) === '1');
  /** Pedido para abrir a confirmação "Limpar histórico" (vem da paleta/configurações). */
  confirmClearHistory = $state(false);
  /** Pedido para abrir o dropdown de liga da barra de título (vem da paleta). */
  leagueMenuOpen = $state(false);

  go(screen: Screen): void {
    this.screen = screen;
  }

  openSettings(pane?: SettingsPane): void {
    if (pane) this.settingsPane = pane;
    this.screen = 'settings';
  }

  toggleRail(): void {
    this.railCollapsed = !this.railCollapsed;
    localStorage.setItem(RAIL_KEY, this.railCollapsed ? '1' : '0');
  }
}

export const nav = new NavStore();
