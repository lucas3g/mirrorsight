<script lang="ts">
  import { APP_VERSION } from '$lib/version';
  import { backend } from '$lib/backend';
  import Icon from '$lib/ui/Icon.svelte';
  import Logo from '$lib/ui/Logo.svelte';
  import { settingsMsgs } from '$lib/i18n/messages/settings';
  import { settings } from '$lib/stores/settings.svelte';
  import { releaseUrl, updater } from '$lib/stores/updater.svelte';

  const m = $derived(settingsMsgs.about);
  const u = $derived(settingsMsgs.about.update);
  const phase = $derived(updater.phase);
  const autoOn = $derived(settings.current?.auto_update ?? true);
</script>

<div class="about">
  <Logo size={64} />
  <h4>Mirrorsight</h4>
  <p>{m.version(APP_VERSION)}</p>
  {#if backend.kind === 'demo'}<span class="chip">{m.demo}</span>{/if}

  <div class="upbox" aria-live="polite">
    {#if phase.kind === 'idle'}
      <button class="btn" onclick={() => void updater.check()}><Icon name="reset" size={14} />{u.check}</button>
      <span class="fine">{autoOn ? u.autoOn : u.autoOff}</span>
    {:else if phase.kind === 'checking'}
      <span class="msg"><span class="spin"><Icon name="reset" size={14} /></span>{u.checking}</span>
      <button class="btn" disabled>{u.checkingButton}</button>
    {:else if phase.kind === 'up-to-date'}
      <span class="msg ok"><Icon name="check" size={14} />{u.upToDate}</span>
      <button class="btn" onclick={() => void updater.check()}><Icon name="reset" size={14} />{u.checkAgain}</button>
    {:else if phase.kind === 'error'}
      <span class="msg err"><Icon name="warning" size={14} />{u.error}</span>
      <button class="btn" onclick={() => void updater.check()}><Icon name="reset" size={14} />{u.retry}</button>
    {:else}
      <span class="msg ready"><Icon name="check" size={14} />{u.ready(phase.info.version)}</span>
      <button class="btn primary" disabled={phase.kind === 'installing'} onclick={() => void updater.install()}>
        <span class:spin={phase.kind === 'installing'}><Icon name="reset" size={14} /></span>
        {phase.kind === 'installing' ? u.installing : u.install}
      </button>
      <button class="link" onclick={() => void backend.openExternal(releaseUrl(phase.info.version))}>{u.notes}</button>
    {/if}
  </div>

  <p class="fine credits">{m.credits}</p>
</div>

<style>
  .about { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 10px; padding: 30px; }
  h4 { font-size: 22px; }
  p { color: var(--ms-text-2); }
  .fine { font-size: 11px; color: var(--ms-text-3); }
  .credits { max-width: 380px; }

  .upbox {
    display: grid;
    justify-items: center;
    gap: 9px;
    width: 100%;
    max-width: 340px;
    margin: 6px 0;
    padding: 12px;
    border: 1px solid var(--ms-border);
    border-radius: var(--ms-r-lg);
    background: var(--ms-surface-2);
  }
  .msg { display: flex; align-items: center; gap: 7px; font-size: 12.5px; color: var(--ms-text-2); }
  .msg.ok { color: var(--ms-success); }
  .msg.ready { color: var(--ms-accent); }
  .msg.err { color: var(--ms-danger); }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 14px;
    border: 1px solid var(--ms-border-strong);
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-3);
    color: var(--ms-text);
    font: inherit;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
  }
  .btn:hover:not(:disabled) { background: var(--ms-surface-4); }
  .btn.primary { border-color: var(--ms-accent); background: var(--ms-accent); color: var(--ms-on-accent); }
  .btn.primary:hover:not(:disabled) { background: var(--ms-accent-2); }
  .btn:disabled { cursor: default; color: var(--ms-text-3); }
  .btn.primary:disabled { color: var(--ms-on-accent); cursor: progress; }
  .btn:focus-visible, .link:focus-visible { outline: none; box-shadow: var(--ms-focus); }

  .link { border: 0; background: none; padding: 0; font: inherit; font-size: 11px; color: var(--ms-text-2); text-decoration: underline; cursor: pointer; }
  .link:hover { color: var(--ms-text); }

  .spin { display: inline-flex; animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
