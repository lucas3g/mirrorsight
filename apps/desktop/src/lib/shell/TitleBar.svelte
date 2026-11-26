<script lang="ts">
  import { backend } from '$lib/backend';
  import type { GameVersion } from '$lib/types';
  import { commands } from '$lib/stores/commands.svelte';
  import { leagues } from '$lib/stores/leagues.svelte';
  import { PANE_LABEL, SCREEN_LABEL, nav } from '$lib/stores/nav.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Logo from '$lib/ui/Logo.svelte';
  import Popover from '$lib/ui/Popover.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';
  import WindowControls from '$lib/ui/WindowControls.svelte';

  /** Barra de título própria (46px). Só os elementos decorativos deixam arrastar a janela. */
  let leagueChip: HTMLButtonElement | undefined = $state();

  const game = $derived<GameVersion>(settings.current?.game ?? 'poe1');

  $effect(() => {
    if (nav.leagueMenuOpen) void leagues.load(game);
  });

  async function pickLeague(league: string) {
    nav.leagueMenuOpen = false;
    if (league !== settings.current?.league) await leagues.changeLeague(league);
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <Logo size={28} />
    <div data-tauri-drag-region><b>Mirrorsight</b><small>price check · Path of Exile</small></div>
  </div>

  <nav class="crumb" aria-label="Você está em" data-tauri-drag-region>
    <span>Mirrorsight</span>
    <Icon name="chevron-right" size={12} />
    {#if nav.screen === 'settings'}
      <span>Configurações</span>
      <Icon name="chevron-right" size={12} />
      <b>{PANE_LABEL[nav.settingsPane]}</b>
    {:else}
      <b>{SCREEN_LABEL[nav.screen]}</b>
    {/if}
  </nav>

  <button class="cmdk" onclick={() => commands.toggle()} title="Paleta de comandos (Ctrl+K)">
    <Icon name="search" size={15} />
    <span>Buscar ação, item, configuração…</span>
    <kbd class="kbd">Ctrl K</kbd>
  </button>

  <Segmented
    pill
    ariaLabel="Jogo"
    options={[{ value: 'poe1', label: 'PoE 1' }, { value: 'poe2', label: 'PoE 2' }]}
    value={game}
    onchange={(g: GameVersion) => leagues.changeGame(g)}
  />

  <button
    class="chip league"
    bind:this={leagueChip}
    aria-haspopup="listbox"
    aria-expanded={nav.leagueMenuOpen}
    onclick={() => (nav.leagueMenuOpen = !nav.leagueMenuOpen)}
    title="Liga de trade"
  >
    <span class="led"></span>
    <span class="name">{settings.current?.league ?? '—'}</span>
    <Icon name="chevron-down" size={13} />
  </button>

  {#if backend.kind === 'demo'}<span class="demo" data-tauri-drag-region>DEMO</span>{/if}

  <WindowControls />
</header>

{#if nav.leagueMenuOpen}
  <Popover anchor={leagueChip} align="end" width={240} label="Ligas" onclose={() => (nav.leagueMenuOpen = false)}>
    <div class="menu" role="listbox" aria-label="Liga">
      <div class="label head">Ligas · {game === 'poe2' ? 'PoE 2' : 'PoE 1'}</div>
      {#if leagues.loading && leagues.fetched.length === 0}
        <div class="muted"><span class="spinner"></span>Carregando ligas…</div>
      {:else}
        {#each leagues.list as league, i (league)}
          <button
            role="option"
            aria-selected={league === settings.current?.league}
            class:on={league === settings.current?.league}
            onclick={() => pickLeague(league)}
          >
            {#if league === settings.current?.league}<Icon name="check" size={14} />{:else}<span class="ph"></span>{/if}
            {league}
            {#if i === 0 && !leagues.error}<small>atual</small>{/if}
          </button>
        {/each}
        {#if leagues.error}<div class="muted">Não foi possível carregar as ligas.</div>{/if}
      {/if}
    </div>
  </Popover>
{/if}

<style>
  .titlebar {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
    padding-left: 14px;
    border-bottom: 1px solid var(--ms-border);
    background: var(--ms-surface-0);
    user-select: none;
  }
  .brand { display: flex; align-items: center; gap: 10px; min-width: 188px; }
  .brand b { font-size: 14px; font-weight: 700; letter-spacing: -0.01em; }
  .brand small { display: block; margin-top: -2px; font-size: 10.5px; color: var(--ms-text-3); }
  .crumb { display: flex; align-items: center; gap: 6px; min-width: 0; font-size: 12px; color: var(--ms-text-3); white-space: nowrap; }
  .crumb b { color: var(--ms-text); font-weight: 600; }
  .cmdk {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    width: min(340px, 30vw);
    height: 30px;
    padding: 0 8px 0 10px;
    border-radius: var(--ms-r-md);
    background: var(--ms-surface-2);
    border: 1px solid var(--ms-border);
    color: var(--ms-text-3);
    transition: border-color var(--ms-t-fast), color var(--ms-t-fast), transform 200ms var(--ms-spring);
  }
  .cmdk:hover { border-color: var(--ms-border-strong); color: var(--ms-text-2); }
  .cmdk:active { transform: scale(0.98); }
  .cmdk span { flex: 1; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .league { max-width: 220px; }
  .league .name { overflow: hidden; text-overflow: ellipsis; }
  .led { width: 7px; height: 7px; flex: none; border-radius: 50%; background: var(--ms-success); box-shadow: 0 0 0 3px var(--ms-success-soft); }
  .demo {
    height: 22px;
    padding: 0 8px;
    display: inline-flex;
    align-items: center;
    border-radius: 99px;
    border: 1px solid var(--ms-accent-line);
    color: var(--ms-accent);
    font-size: 10.5px;
    font-weight: 800;
    letter-spacing: 0.08em;
  }
  .menu { padding: 6px; display: flex; flex-direction: column; gap: 2px; }
  .head { padding: 6px 8px 4px; }
  .menu button {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 10px;
    border-radius: var(--ms-r-md);
    text-align: left;
    font-weight: 600;
    color: var(--ms-text-2);
  }
  .menu button:hover { background: var(--ms-surface-3); color: var(--ms-text); }
  .menu button.on { color: var(--ms-accent); }
  .menu small { margin-left: auto; font-size: 10.5px; color: var(--ms-text-3); }
  .ph { width: 14px; flex: none; }
  .muted { display: flex; align-items: center; gap: 8px; padding: 8px 10px; font-size: 12px; color: var(--ms-text-3); }
  @media (max-width: 1000px) {
    .crumb { display: none; }
    .brand { min-width: 0; }
  }
  @media (max-width: 820px) {
    .brand div, .cmdk span { display: none; }
    .cmdk { width: auto; }
  }
</style>
