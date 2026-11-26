<script lang="ts">
  import { APP_VERSION } from '$lib/version';
  import { tick } from 'svelte';
  import { normalize } from '$lib/format';
  import { PANE_LABEL, nav, type SettingsPane } from '$lib/stores/nav.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon, { type IconName } from '$lib/ui/Icon.svelte';
  import GeneralPane from './GeneralPane.svelte';
  import HotkeysPane from './HotkeysPane.svelte';
  import PriceCheckPane from './PriceCheckPane.svelte';
  import AppearancePane from './AppearancePane.svelte';
  import AboutPane from './AboutPane.svelte';
  import { LANGUAGE_KEYWORDS, languageMsgs } from '$lib/i18n/messages/language';
  import { SETTINGS_KEYWORDS as KW, settingsMsgs as m } from '$lib/i18n/messages/settings';

  /**
   * `search`: `[rótulo, descrição + palavras-chave]` de cada `SettingRow` da seção, na forma estática
   * (sem as contagens que algumas descrições mostram). A busca consulta esta lista sem montar as
   * seções e acha a linha na tela pelo rótulo exato; ao criar ou renomear um ajuste, atualize aqui.
   * Usa as mesmas mensagens/palavras-chave das seções, então acompanha o idioma.
   */
  type Pane = { id: SettingsPane; icon: IconName; description: string; search: [label: string, text: string][] };
  const row = (r: { label: string; description: string }, keywords: string, extra = ''): [string, string] => [
    r.label,
    `${r.description} ${extra} ${keywords}`,
  ];
  const PANES: Pane[] = $derived.by(() => {
    const g = m.general;
    const h = m.hotkeys;
    const p = m.priceCheck;
    const a = m.appearance;
    const d = m.paneDescriptions;
    return [
      {
        id: 'general',
        icon: 'gear',
        description: d.general,
        search: [
          row(g.game, KW.game),
          row(g.league, KW.league, g.leagueError),
          [languageMsgs.label, `${languageMsgs.description} ${LANGUAGE_KEYWORDS}`],
          [languageMsgs.gameLabel, `${languageMsgs.gameDescription} ${KW.gameLanguage}`],
          row(g.currency, KW.currency),
          row(g.restoreClipboard, KW.restoreClipboard),
          row(g.startup, KW.startup),
        ],
      },
      {
        id: 'hotkeys',
        icon: 'key',
        description: d.hotkeys,
        search: [
          row(h.actions['price-check'], KW.hotkey),
          row(h.actions['price-check-locked'], KW.hotkey),
          row(h.actions['toggle-overlay'], KW.hotkey),
          row(h.actions['open-settings'], KW.hotkey),
          row(h.palette, KW.palette),
        ],
      },
      {
        id: 'price-check',
        icon: 'tag',
        description: d['price-check'],
        search: [
          row(p.modRange, KW.modRange),
          row(p.showOffline, KW.showOffline),
          row(p.defaultFilters, KW.defaultFilters),
          row(p.defaultLevel, KW.defaultLevel),
          [p.history.label, `${p.history.search} ${KW.history}`],
          [p.presets.label, `${p.presets.search} ${KW.presets}`],
        ],
      },
      {
        id: 'appearance',
        icon: 'brush',
        description: d.appearance,
        search: [row(a.scale, KW.scale), row(a.density, KW.density), row(a.reduceMotion, KW.reduceMotion)],
      },
      { id: 'about', icon: 'info', description: d.about, search: [] },
    ];
  });

  let query = $state('');
  let content: HTMLElement | undefined = $state();
  let hlTimer: ReturnType<typeof setTimeout> | undefined;
  /** Cada tecla invalida a busca anterior (ela pode ainda estar esperando o `tick`). */
  let searchSeq = 0;

  const pane = $derived(PANES.find((p) => p.id === nav.settingsPane) ?? PANES[0]!);

  /**
   * Busca nos textos das seções (sem montar nenhuma), por termos: "historico price" acha
   * "Histórico … price checks". Ordem: ajuste da seção atual, ajuste das outras seções (ordem
   * do menu), e por fim o nome/descrição da seção ("aparencia" abre Aparência sem destacar linha).
   * Só troca de seção se precisar; sem resultado, fica onde está.
   */
  async function search() {
    const seq = ++searchSeq;
    const q = normalize(query.trim());
    if (q.length < 2) return;
    const terms = q.split(/\s+/);
    const hit = (text: string) => {
      const hay = normalize(text);
      return terms.every((t) => hay.includes(t));
    };
    const rowIn = (p: Pane) => p.search.find(([label, text]) => hit(`${label} ${text}`))?.[0];

    let target: Pane | undefined;
    let label: string | undefined;
    for (const p of [pane, ...PANES.filter((x) => x !== pane)]) {
      label = rowIn(p);
      if (label) {
        target = p;
        break;
      }
    }
    target ??= PANES.find((p) => hit(`${PANE_LABEL[p.id]} ${p.description}`));
    if (!target) return;

    if (nav.settingsPane !== target.id) {
      nav.settingsPane = target.id;
      await tick();
      if (seq !== searchSeq) return;
    }
    content?.querySelectorAll('.hl').forEach((el) => el.classList.remove('hl'));
    if (!label) return;
    const row = [...(content?.querySelectorAll<HTMLElement>('[data-search]') ?? [])].find(
      (el) => el.querySelector('.lbl > b')?.textContent === label,
    );
    if (!row) return;
    row.classList.add('hl');
    row.scrollIntoView({ block: 'nearest' });
    clearTimeout(hlTimer);
    hlTimer = setTimeout(() => row.classList.remove('hl'), 1600);
  }
</script>

<article class="settings" aria-label={m.view.aria}>
  <nav class="side" aria-label={m.view.sectionsAria}>
    <label class="search">
      <Icon name="search" size={14} />
      <input class="field" placeholder={m.view.searchPlaceholder} aria-label={m.view.searchAria} bind:value={query} oninput={search} />
    </label>
    {#each PANES as p (p.id)}
      <button class="nav-item" aria-current={nav.settingsPane === p.id ? 'page' : undefined} onclick={() => (nav.settingsPane = p.id)}>
        <Icon name={p.icon} size={16} />{PANE_LABEL[p.id]}
      </button>
    {/each}
    <div class="ver mono">v{APP_VERSION} · {settings.current?.game === 'poe2' ? 'PoE2' : 'PoE1'}</div>
  </nav>

  <div class="content scroll" bind:this={content}>
    <header>
      <h1>{PANE_LABEL[pane.id]}</h1>
      <p>{pane.description}</p>
    </header>
    {#if settings.current}
      {#key nav.settingsPane}
        <div class="pane">
          {#if pane.id === 'general'}<GeneralPane current={settings.current} />
          {:else if pane.id === 'hotkeys'}<HotkeysPane current={settings.current} />
          {:else if pane.id === 'price-check'}<PriceCheckPane current={settings.current} />
          {:else if pane.id === 'appearance'}<AppearancePane current={settings.current} />
          {:else}<AboutPane />{/if}
        </div>
      {/key}
    {:else}
      <p class="error">{settings.loadError ?? m.view.loading}</p>
    {/if}
  </div>
</article>

<style>
  .settings { height: 100%; display: flex; min-width: 0; }
  .side {
    width: 240px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 16px 12px;
    border-right: 1px solid var(--ms-border);
    background: var(--ms-surface-0);
  }
  .search { position: relative; margin-bottom: 12px; color: var(--ms-text-3); }
  .search :global(svg) { position: absolute; left: 10px; top: 9px; }
  .search input { width: 100%; padding-left: 31px; }
  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 38px;
    padding: 0 12px;
    border-radius: var(--ms-r-md);
    font-weight: 600;
    color: var(--ms-text-2);
    text-align: left;
    transition: background var(--ms-t-fast), color var(--ms-t-fast);
  }
  .nav-item:hover { background: var(--ms-surface-2); color: var(--ms-text); }
  .nav-item[aria-current='page'] { background: var(--ms-accent-soft); color: var(--ms-accent); }
  .nav-item[aria-current='page']::before {
    content: '';
    position: absolute;
    left: 0;
    top: 9px;
    bottom: 9px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--ms-accent);
  }
  .ver { margin-top: auto; padding: 8px 12px; font-size: 11px; color: var(--ms-text-3); }
  .content { flex: 1; min-width: 0; padding: 22px 28px 28px; }
  header { margin-bottom: 18px; }
  h1 { font-size: 20px; font-weight: 750; letter-spacing: -0.01em; }
  header p { color: var(--ms-text-2); font-size: 12.5px; margin-top: 3px; }
  .pane { max-width: 860px; animation: ms-row-up 360ms var(--ms-spring) both; }
  .pane :global(.group) { display: flex; flex-direction: column; gap: 8px; margin-bottom: 18px; }
  .pane :global(.sub) { display: flex; align-items: center; gap: 8px; margin: 4px 0 8px; }
  .error { color: var(--ms-text-2); }

  @media (max-width: 900px) {
    .side { width: 200px; }
    .content { padding: 18px; }
  }
</style>
