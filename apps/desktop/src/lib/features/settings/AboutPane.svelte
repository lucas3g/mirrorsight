<script lang="ts">
  import { APP_VERSION } from '$lib/version';
  import { backend } from '$lib/backend';
  import Logo from '$lib/ui/Logo.svelte';
  import { settingsMsgs } from '$lib/i18n/messages/settings';

  const m = $derived(settingsMsgs.about);
</script>

<div class="about">
  <Logo size={64} />
  <h4>Mirrorsight</h4>
  <p>{m.version(APP_VERSION)}</p>
  {#if backend.kind === 'demo'}<span class="chip">{m.demo}</span>{/if}
  <p class="fine">{m.credits}</p>
</div>

<style>
  .about { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 10px; padding: 30px; }
  h4 { font-size: 22px; }
  p { color: var(--ms-text-2); }
  .fine { font-size: 11px; color: var(--ms-text-3); max-width: 380px; }
</style>
