<script lang="ts">
  import { onMount } from 'svelte';
  import { backend } from '$lib/backend';
  import Icon from './Icon.svelte';

  /** Botões de janela da barra de título própria (a moldura nativa do Windows foi removida). */
  let maximized = $state(false);

  async function refresh() {
    maximized = await backend.isWindowMaximized();
  }

  onMount(() => {
    void refresh();
    const subscription = backend.onWindowResized(() => void refresh());
    return () => void subscription.then((unsubscribe) => unsubscribe());
  });
</script>

{#if backend.hasWindowControls}
  <div class="controls">
    <button type="button" title="Minimizar" aria-label="Minimizar" onclick={() => backend.minimizeWindow()}>
      <Icon name="win-minimize" size={16} />
    </button>
    <button
      type="button"
      title={maximized ? 'Restaurar' : 'Maximizar'}
      aria-label={maximized ? 'Restaurar' : 'Maximizar'}
      onclick={() => backend.toggleMaximizeWindow()}
    >
      <Icon name={maximized ? 'win-restore' : 'win-maximize'} size={16} />
    </button>
    <button type="button" class="close" title="Fechar (continua na bandeja)" aria-label="Fechar" onclick={() => backend.closeWindow()}>
      <Icon name="win-close" size={16} />
    </button>
  </div>
{/if}

<style>
  .controls { display: flex; align-self: stretch; height: 100%; }
  button {
    width: 46px;
    display: grid;
    place-items: center;
    color: var(--ms-text-2);
    transition: background var(--ms-t-fast), color var(--ms-t-fast);
  }
  button:hover { background: var(--ms-surface-3); color: var(--ms-text); }
  button:active { background: var(--ms-surface-4); }
  .close:hover { background: #c42b3f; color: #fff; }
  .close:active { background: #a3203a; }
  button:focus-visible { box-shadow: inset 0 0 0 2px var(--ms-accent-line); border-radius: 0; }
</style>
