<script lang="ts">
  import { nav } from '$lib/stores/nav.svelte';
  import PriceCheckView from '$lib/features/price-check/PriceCheckView.svelte';
  import HistoryView from '$lib/features/history/HistoryView.svelte';
  import SettingsView from '$lib/features/settings/SettingsView.svelte';
  import NavRail from './NavRail.svelte';
  import StatusBar from './StatusBar.svelte';
  import TitleBar from './TitleBar.svelte';

  /** Casca da janela principal: título 46px / conteúdo / status 30px × menu lateral / conteúdo. */
  let narrow = $state(false);

  $effect(() => {
    const mq = window.matchMedia('(max-width: 1180px)');
    const update = () => (narrow = mq.matches);
    update();
    mq.addEventListener('change', update);
    return () => mq.removeEventListener('change', update);
  });
</script>

<div class="shell app-shell">
  <TitleBar />
  <NavRail collapsed={narrow || nav.railCollapsed} />
  <main>
    {#key nav.screen}
      <div class="screen">
        {#if nav.screen === 'price'}
          <PriceCheckView />
        {:else if nav.screen === 'history'}
          <HistoryView />
        {:else}
          <SettingsView />
        {/if}
      </div>
    {/key}
  </main>
  <StatusBar />
</div>

<style>
  .shell {
    height: 100%;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    grid-template-rows: 46px minmax(0, 1fr) 30px;
  }
  main { position: relative; min-height: 0; min-width: 0; overflow: hidden; }
  .screen { height: 100%; display: flex; animation: ms-in-up 500ms var(--ms-spring) both; }
  .screen > :global(*) { flex: 1; min-width: 0; }
</style>
