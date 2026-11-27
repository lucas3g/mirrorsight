import { backend, describeError, toAppError, type UpdateInfo } from '$lib/backend';
import { shellMsgs } from '$lib/i18n/messages/shell';
import { toasts } from './toasts.svelte';

export type UpdatePhase =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'up-to-date' }
  | { kind: 'ready'; info: UpdateInfo }
  | { kind: 'installing'; info: UpdateInfo }
  | { kind: 'error' };

/** Página da release no GitHub (link "Ver novidades"). */
export function releaseUrl(version: string): string {
  return `https://github.com/lucas3g/mirrorsight/releases/tag/v${version}`;
}

/**
 * Atualização automática (o fluxo mora em `src-tauri/src/updater.rs`): aqui só o estado
 * da janela principal — aviso de versão baixada, checagem manual e "Reiniciar".
 */
class UpdaterStore {
  phase = $state<UpdatePhase>({ kind: 'idle' });

  /** Versão baixada e pronta para instalar (alimenta o chip da barra de status). */
  get ready(): UpdateInfo | null {
    const p = this.phase;
    return p.kind === 'ready' || p.kind === 'installing' ? p.info : null;
  }

  /** Lê o estado atual e escuta as atualizações baixadas em segundo plano. Retorna o cancelamento. */
  start(): () => void {
    void backend
      .updateStatus()
      .then((info) => {
        if (info) this.phase = { kind: 'ready', info };
      })
      .catch(() => {});
    const subscription = backend.onUpdateReady((info) => {
      // Já instalando: o Rust instala a mais nova de qualquer jeito; não reabilita o botão.
      if (this.phase.kind === 'installing') return;
      // Checagem manual em andamento: a própria tela "Sobre" mostra o resultado, sem toast.
      const announce = this.phase.kind !== 'checking' && this.ready?.version !== info.version;
      this.phase = { kind: 'ready', info };
      if (announce) toasts.push('info', shellMsgs.status.updateToast(info.version));
    });
    return () => void subscription.then((unsubscribe) => unsubscribe());
  }

  async check(): Promise<void> {
    if (this.phase.kind === 'checking' || this.phase.kind === 'installing') return;
    this.phase = { kind: 'checking' };
    try {
      const result = await backend.checkForUpdate();
      this.phase = result.status === 'ready'
        ? { kind: 'ready', info: { version: result.version, notes: result.notes } }
        : { kind: 'up-to-date' };
    } catch {
      this.phase = { kind: 'error' };
    }
  }

  async install(): Promise<void> {
    const info = this.ready;
    if (!info || this.phase.kind === 'installing') return;
    this.phase = { kind: 'installing', info };
    try {
      // Se der certo o processo é encerrado pelo instalador e esta promessa nunca volta.
      await backend.installUpdate();
      // Voltou sem encerrar: não havia nada pendente no Rust.
      this.phase = { kind: 'idle' };
    } catch (err) {
      this.phase = { kind: 'ready', info };
      toasts.push('error', describeError(toAppError(err)));
    }
  }
}

export const updater = new UpdaterStore();
