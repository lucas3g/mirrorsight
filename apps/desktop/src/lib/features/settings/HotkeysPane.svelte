<script lang="ts">
  import { backend, describeError, toAppError } from '$lib/backend';
  import type { HotkeyAction, HotkeyBinding, Settings } from '$lib/types';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Keycaps from '$lib/ui/Keycaps.svelte';
  import SettingRow from './SettingRow.svelte';
  import { SETTINGS_KEYWORDS as KW, settingsMsgs } from '$lib/i18n/messages/settings';

  let { current }: { current: Settings } = $props();

  const m = $derived(settingsMsgs.hotkeys);

  /** Combinação padrão de cada ação (nome e descrição vêm de `m.actions`). */
  const FALLBACK: Record<HotkeyAction, string> = {
    'price-check': 'Ctrl + D',
    'price-check-locked': 'Ctrl + Alt + D',
    'toggle-overlay': 'Shift + Space',
    'open-settings': 'Ctrl + Shift + O',
  };

  let capturing = $state<HotkeyAction | null>(null);
  /** `message` é função para acompanhar o idioma (erros do backend já vêm prontos). */
  let feedback = $state<{ action: HotkeyAction; ok: boolean; message: () => string } | null>(null);
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
        feedback = { action, ok: true, message: () => m.saved };
      }
    } catch (err) {
      const message = describeError(toAppError(err));
      feedback = { action, ok: false, message: () => message };
      shaking = action;
      setTimeout(() => (shaking = null), 300);
    }
  }

  async function reset(action: HotkeyAction) {
    const hotkey = FALLBACK[action];
    const bindings = current.hotkeys.map((b) => (b.action === action ? { action, hotkey } : b));
    capturing = null;
    if (await settings.update({ hotkeys: bindings })) feedback = { action, ok: true, message: () => m.defaultRestored };
  }
</script>

<svelte:window onkeydowncapture={onKeydown} />

<div class="group">
  {#each current.hotkeys as binding (binding.action)}
    {@const meta = m.actions[binding.action]}
    <SettingRow label={meta.label} description={meta.description} keywords={KW.hotkey}>
      {#snippet extra()}
        {#if feedback?.action === binding.action}
          <span class="msg" class:ok={feedback.ok}>{feedback.ok ? '✓ ' : ''}{feedback.message()}</span>
        {/if}
      {/snippet}
      <button
        class="cap"
        class:capturing={capturing === binding.action}
        class:invalid={shaking === binding.action}
        onclick={() => start(binding.action)}
        onblur={() => capturing === binding.action && (capturing = null)}
      >
        {#if capturing === binding.action}{m.pressCombo}{:else}<Keycaps combo={binding.hotkey} />{/if}
      </button>
      <button class="icon-btn" title={m.restoreDefault} onclick={() => reset(binding.action)}><Icon name="reset" size={14} /></button>
    </SettingRow>
  {/each}
  <SettingRow label={m.palette.label} description={m.palette.description} keywords={KW.palette}>
    <span class="cap fixed" title={m.fixedShortcut}><Keycaps combo="Ctrl + K" /></span>
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
