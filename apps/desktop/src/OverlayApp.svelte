<script lang="ts">
  import { onMount } from 'svelte';
  import { backend } from '$lib/backend';
  import CommandPalette from '$lib/features/commands/CommandPalette.svelte';
  import { handleCommandKey, handlePriceCheckKeys } from '$lib/features/price-check/keyboard';
  import PriceCheckView from '$lib/features/price-check/PriceCheckView.svelte';
  import { useAppearance } from '$lib/stores/appearance.svelte';
  import { commands } from '$lib/stores/commands.svelte';
  import { PEEK_EXTRA, overlayPeek } from '$lib/stores/overlayPeek.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Toasts from '$lib/ui/Toasts.svelte';

  /**
   * Janela overlay: só o painel, sobre o jogo. Abre via hotkey (evento do backend),
   * fecha com Esc/botão — ou ao perder o foco, se não estiver fixado.
   */
  let visible = $state(false);

  useAppearance();

  /** Sobe a cada hotkey (ou erro de cópia): descarta leituras atrasadas e esconderes pendentes. */
  let requestSeq = 0;
  /** Quando foi a última leitura do disco (o foco logo após a hotkey não relê de novo). */
  let lastSync = 0;

  /**
   * O overlay só se esconde, nunca fecha: jogo, liga, nível padrão, moeda e presets podem ter
   * mudado na janela principal. Relê configurações e presets do jogo atual (um `listPresets` só).
   */
  async function syncFromDisk(): Promise<void> {
    lastSync = Date.now();
    await settings.load();
    const game = settings.current?.game;
    if (game) await presets.load(game);
  }

  onMount(() => {
    void syncFromDisk();
    const subscriptions = [
      backend.onPriceCheckRequest(async ({ text, locked }) => {
        const seq = ++requestSeq;
        await syncFromDisk();
        // Outra hotkey chegou enquanto lia o disco: a mais nova vence.
        if (seq !== requestSeq) return;
        // `visible` só depois do `check` começar: com a fase ainda `idle`, o efeito abaixo esconderia a janela.
        void priceCheck.check(text);
        visible = true;
        // Só aqui: fixar depois do `await` refixaria um painel fechado durante o carregamento.
        if (locked) priceCheck.pinned = true;
      }),
      backend.onOverlayReopen(() => {
        requestSeq++;
        // Painel já aberto: nada a fazer. Sem price check anterior: não deixa a janela vazia na tela.
        if (priceCheck.phase.kind !== 'idle') return;
        if (priceCheck.restoreLast()) visible = true;
        else void backend.hideOverlay();
      }),
      backend.onPriceCheckError((error) => {
        requestSeq++;
        visible = true;
        priceCheck.fail(error);
      }),
    ];
    return () => subscriptions.forEach((s) => void s.then((unsubscribe) => unsubscribe()));
  });

  // Painel fechado (Esc, botão ✕, quick price expirado) → esconde a janela e volta ao jogo.
  $effect(() => {
    if (visible && priceCheck.phase.kind === 'idle') {
      visible = false;
      commands.close();
      const seq = requestSeq;
      // Uma hotkey nova no meio do caminho reabriu o painel: não esconde a janela dela.
      void overlayPeek.restore().then(() => {
        if (seq === requestSeq && !visible) void backend.hideOverlay();
      });
    }
  });

  /** Janela expandida para o card do olho: o painel mantém a largura e a posição na tela. */
  const stageStyle = $derived.by(() => {
    if (!overlayPeek.side || overlayPeek.side === 'inside') return '';
    const width = `width:${overlayPeek.panelWidth - 20}px;flex:none;`;
    return overlayPeek.side === 'left' ? `${width}margin-left:${PEEK_EXTRA}px;` : width;
  });

  /** Com a paleta aberta o painel não fecha por perda de foco. */
  function onBlur() {
    if (visible && !priceCheck.pinned && !commands.open) priceCheck.close();
  }

  /**
   * Painel fixado e janela principal editou presets: recarrega ao voltar ao overlay.
   * O foco que vem junto com a hotkey é ignorado (painel ainda oculto, ou leitura recente):
   * a própria hotkey lê o disco.
   */
  function onFocus() {
    if (!visible || Date.now() - lastSync < 1500) return;
    lastSync = Date.now();
    void presets.load(presets.game);
  }

  /** Ctrl+K só com o painel aberto (no overlay a paleta tem só item atual e presets). */
  function onKeydown(e: KeyboardEvent) {
    if (priceCheck.phase.kind !== 'idle' && handleCommandKey(e)) return;
    handlePriceCheckKeys(e);
  }
</script>

<svelte:window onkeydown={onKeydown} onblur={onBlur} onfocus={onFocus} />

<div class="overlay">
  {#if priceCheck.phase.kind !== 'idle'}
    <div class="stage" style={stageStyle}>
      <PriceCheckView />
    </div>
  {/if}
</div>

<CommandPalette role="overlay" />
<Toasts />

<style>
  :global(html.overlay),
  :global(html.overlay body) {
    background: transparent;
  }
  .overlay { height: 100%; padding: 10px; display: flex; }
  .stage { flex: 1; display: flex; min-width: 0; }
  .stage > :global(*) { flex: 1; }
</style>
