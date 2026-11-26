<script lang="ts">
  import { backend, describeError, toAppError } from '$lib/backend';
  import type { HotkeyAction, HotkeyBinding, Settings } from '$lib/types';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Keycaps from '$lib/ui/Keycaps.svelte';
  import SettingRow from './SettingRow.svelte';

  let { current }: { current: Settings } = $props();

  const LABELS: Record<HotkeyAction, { name: string; description: string; fallback: string }> = {
    'price-check': { name: 'Price check', description: 'Painel completo do item sob o cursor', fallback: 'Ctrl + D' },
    'price-check-locked': { name: 'Price check fixado', description: 'Abre o painel já fixado na tela', fallback: 'Ctrl + Alt + D' },
    'toggle-overlay': { name: 'Mostrar/ocultar overlay', description: 'Alterna a visibilidade dos painéis', fallback: 'Shift + Space' },
    'open-settings': { name: 'Abrir configurações', description: 'Esta janela', fallback: 'Ctrl + Shift + O' },
  };

  let capturing = $state<HotkeyAction | null>(null);
  let feedback = $state<{ action: HotkeyAction; ok: boolean; message: string } | null>(null);
  let shaking = $state<HotkeyAction | null>(null);

  const MODIFIERS = new Set(['Control', 'Alt', 'Shift', 'Meta']);

  /** Converte o evento para o formato normalizado do domínio ("Ctrl + Shift + D"). */
  function comboOf(e: KeyboardEvent): string | null {
    if (MODIFIERS.has(e.key)) return null;
    const key = e.code === 'Space' ? 'Space' : e.key.length === 1 ? e.key.toUpperCase() : e.key;
    return [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', key].filter(Boolean).join(' + ');
  }

  function start(action: HotkeyAction) {
    capturing = action;
    feedback = null;
  }

  async function onKeydown(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape') {
      capturing = null;
      return;
    }
    const combo = comboOf(e);
    if (!combo) return;
    const action = capturing;
    const bindings: HotkeyBinding[] = current.hotkeys.map((b) => (b.action === action ? { action, hotkey: combo } : b));
    try {
      await backend.validateHotkeys(bindings);
      capturing = null;
      if (await settings.update({ hotkeys: bindings })) {
        feedback = { action, ok: true, message: 'Salvo' };
      }
    } catch (err) {
      feedback = { action, ok: false, message: describeError(toAppError(err)) };
      shaking = action;
      setTimeout(() => (shaking = null), 300);
    }
  }

  async function reset(action: HotkeyAction) {
    const hotkey = LABELS[action].fallback;
    const bindings = current.hotkeys.map((b) => (b.action === action ? { action, hotkey } : b));
    capturing = null;
    if (await settings.update({ hotkeys: bindings })) feedback = { action, ok: true, message: 'Padrão restaurado' };
  }
</script>

<svelte:window onkeydowncapture={onKeydown} />

<div class="group">
  {#each current.hotkeys as binding (binding.action)}
    {@const meta = LABELS[binding.action]}
    <SettingRow label={meta.name} description={meta.description} keywords="hotkey atalho tecla">
      {#snippet extra()}
        {#if feedback?.action === binding.action}
          <span class="msg" class:ok={feedback.ok}>{feedback.ok ? '✓ ' : ''}{feedback.message}</span>
        {/if}
      {/snippet}
      <button
        class="cap"
        class:capturing={capturing === binding.action}
        class:invalid={shaking === binding.action}
        onclick={() => start(binding.action)}
        onblur={() => capturing === binding.action && (capturing = null)}
      >
        {#if capturing === binding.action}Pressione a combinação…{:else}<Keycaps combo={binding.hotkey} />{/if}
      </button>
      <button class="icon-btn" title="Restaurar padrão" onclick={() => reset(binding.action)}><Icon name="reset" size={14} /></button>
    </SettingRow>
  {/each}
  <SettingRow label="Paleta de comandos" description="Só dentro do Mirrorsight · não editável" keywords="hotkey atalho ctrl k comandos busca">
    <span class="cap fixed" title="Atalho fixo"><Keycaps combo="Ctrl + K" /></span>
    <span class="ib-spacer" aria-hidden="true"></span>
  </SettingRow>
</div>

<style>
  .cap {
    min-width: 150px;
    height: 30px;
    padding: 0 10px;
    border-radius: 7px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--ms-border);
    background: var(--ms-bg);
    font-size: 11.5px;
    color: var(--ms-text-2);
    transition: border-color var(--ms-t-fast);
  }
  .cap:hover { border-color: var(--ms-border-strong); }
  .cap.fixed { cursor: default; background: var(--ms-surface-0); }
  .cap.fixed:hover { border-color: var(--ms-border); }
  .ib-spacer { width: 30px; flex: none; }
  .capturing { border-color: var(--ms-accent); animation: cap 1.1s ease-in-out infinite; }
  @keyframes cap { 50% { box-shadow: 0 0 0 4px var(--ms-accent-soft); } }
  .invalid { border-color: var(--ms-danger); animation: shake 280ms; }
  @keyframes shake { 25% { transform: translateX(-3px); } 75% { transform: translateX(3px); } }
  .msg { font-size: 11px; color: var(--ms-danger); margin-top: 2px; }
  .msg.ok { color: var(--ms-success); }
</style>
