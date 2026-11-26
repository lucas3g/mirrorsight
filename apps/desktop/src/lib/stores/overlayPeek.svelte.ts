import { backend, type PeekSide } from '$lib/backend';

/** Espaço reservado ao lado do painel para o card do item listado (px lógicos). */
export const PEEK_EXTRA = 350;
/** Atraso para encolher: passar de um olho ao outro não faz a janela "piscar". */
const CLOSE_DELAY_MS = 160;
/** Depois de a janela crescer/encolher, o painel pode "andar" sob o mouse por alguns quadros. */
export const SETTLE_MS = 220;

/**
 * Expansão do overlay para mostrar o card do olho **fora** do painel.
 * Na janela principal (ou no navegador) não há expansão: o card fica sobre o painel.
 */
class OverlayPeekStore {
  /** Lado para onde a janela cresceu; `null` = tamanho normal. */
  side = $state<PeekSide | null>(null);
  /** Largura do painel (janela antes de crescer), em px lógicos. */
  panelWidth = $state(0);

  private closeTimer: ReturnType<typeof setTimeout> | undefined;
  private pending: Promise<PeekSide> | null = null;

  async open(): Promise<PeekSide> {
    clearTimeout(this.closeTimer);
    if (backend.windowRole() !== 'overlay') return 'inside';
    if (this.side) return this.side;
    if (!this.pending) {
      this.panelWidth = window.innerWidth;
      // Px físicos: `devicePixelRatio` já inclui o DPI do monitor e a escala da interface (zoom).
      this.pending = backend
        .setOverlayPeek(true, Math.ceil(PEEK_EXTRA * window.devicePixelRatio))
        .catch((): PeekSide => 'inside')
        .then((side) => {
          this.side = side;
          this.pending = null;
          return side;
        });
    }
    return this.pending;
  }

  close(): void {
    clearTimeout(this.closeTimer);
    this.closeTimer = setTimeout(() => void this.restore(), CLOSE_DELAY_MS);
  }

  /** Cancela um fechamento agendado (o mouse voltou ao olho). */
  keep(): void {
    clearTimeout(this.closeTimer);
  }

  /** Restaura imediatamente (ex.: painel fechado). */
  async restore(): Promise<void> {
    clearTimeout(this.closeTimer);
    // Expansão ainda em andamento: espera, senão a janela ficaria expandida para sempre.
    if (this.pending) await this.pending;
    if (!this.side) return;
    this.side = null;
    await backend.setOverlayPeek(false, 0).catch(() => undefined);
  }
}

export const overlayPeek = new OverlayPeekStore();
