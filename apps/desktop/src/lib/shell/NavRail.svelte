<script lang="ts">
  import { history } from '$lib/stores/history.svelte';
  import { shellMsgs as m } from '$lib/i18n/messages/shell';
  import { SCREEN_LABEL, nav, type Screen } from '$lib/stores/nav.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon, { type IconName } from '$lib/ui/Icon.svelte';

  /** Menu lateral: 212px; recolhido (preferência ou janela < 1180px) = 62px, só ícones. */
  let { collapsed }: { collapsed: boolean } = $props();

  const priceKey = $derived(
    (settings.current?.hotkeys.find((h) => h.action === 'price-check')?.hotkey ?? 'Ctrl + D').replace(/\s*\+\s*/g, ' '),
  );

  interface Item {
    screen: Screen;
    icon: IconName;
  }
  const MARKET: Item[] = [
    { screen: 'price', icon: 'tag' },
    { screen: 'history', icon: 'clock' },
  ];
</script>

<nav class="rail" class:mini={collapsed} aria-label={m.nav.aria}>
  <div class="sec">{m.nav.market}</div>
  {#each MARKET as item (item.screen)}
    <button
      class="nav"
      class:on={nav.screen === item.screen}
      aria-current={nav.screen === item.screen ? 'page' : undefined}
      title={collapsed ? SCREEN_LABEL[item.screen] : undefined}
      onclick={() => nav.go(item.screen)}
    >
      <Icon name={item.icon} size={18} />
      <span class="txt">{SCREEN_LABEL[item.screen]}</span>
      {#if item.screen === 'price'}
        <kbd class="kbd end">{priceKey}</kbd>
      {:else}
        <span class="cnt end">{history.entries.length}</span>
      {/if}
    </button>
  {/each}

  <div class="sec">{m.nav.system}</div>
  <button
    class="nav"
    class:on={nav.screen === 'settings'}
    aria-current={nav.screen === 'settings' ? 'page' : undefined}
    title={collapsed ? SCREEN_LABEL.settings : undefined}
    onclick={() => nav.openSettings()}
  >
    <Icon name="gear" size={18} />
    <span class="txt">{SCREEN_LABEL.settings}</span>
  </button>

  <div class="grow"></div>
  <button class="nav tog" onclick={() => nav.toggleRail()} title={collapsed ? m.nav.expand : m.nav.collapse}>
    <Icon name="chevron-left" size={18} />
    <span class="txt">{m.nav.collapse}</span>
  </button>
</nav>

<style>
  .rail {
    width: 212px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 10px;
    overflow: hidden;
    border-right: 1px solid var(--ms-border);
    background: var(--ms-surface-0);
    transition: width 350ms var(--ms-spring);
  }
  .rail.mini { width: 62px; }
  .sec {
    padding: 10px 10px 6px;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--ms-text-3);
    white-space: nowrap;
  }
  .mini .sec { visibility: hidden; height: 12px; padding: 0; }
  .nav {
    position: relative;
    display: flex;
    align-items: center;
    gap: 11px;
    height: 38px;
    padding: 0 11px;
    border-radius: var(--ms-r-md);
    font-weight: 600;
    color: var(--ms-text-2);
    white-space: nowrap;
    transition: background var(--ms-t-fast), color var(--ms-t-fast);
  }
  .nav :global(svg) { flex: none; }
  .nav:hover { background: var(--ms-surface-2); color: var(--ms-text); }
  .nav.on { background: var(--ms-accent-soft); color: var(--ms-accent); }
  .nav.on::before {
    content: '';
    position: absolute;
    left: -10px;
    top: 9px;
    bottom: 9px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--ms-accent);
    animation: ms-grow-y 350ms var(--ms-spring);
  }
  .end { margin-left: auto; }
  .mini .txt,
  .mini .end { display: none; }
  .grow { flex: 1; }
  .tog { color: var(--ms-text-3); }
  .tog :global(svg) { transition: transform 350ms var(--ms-spring); }
  .mini .tog :global(svg) { transform: rotate(180deg); }
</style>
