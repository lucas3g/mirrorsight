<script lang="ts">
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

  /**
   * `search`: `[rótulo, descrição + palavras-chave]` de cada `SettingRow` da seção, na forma estática
   * (sem as contagens que algumas descrições mostram). A busca consulta esta lista sem montar as
   * seções e acha a linha na tela pelo rótulo exato; ao criar ou renomear um ajuste, atualize aqui.
   */
  const PANES: { id: SettingsPane; icon: IconName; description: string; search: [label: string, text: string][] }[] = [
    {
      id: 'general',
      icon: 'gear',
      description: 'Jogo, liga e comportamento básico do overlay.',
      search: [
        ['Jogo', 'Define parser, base de mods e API usada poe path of exile 2 versão'],
        ['Liga', 'A primeira é a liga de desafio atual Não foi possível carregar as ligas league temporada'],
        ['Idioma do cliente', 'Idioma em que o jogo copia os itens language lingua'],
        ['Moeda de exibição', 'Valores acima de 1 divine sempre aparecem em divine chaos divine conversão'],
        ['Restaurar clipboard', 'Devolve o conteúdo anterior após copiar o item clipboard área de transferência'],
        ['Iniciar com o Windows', 'Fica na bandeja do sistema startup inicialização bandeja'],
      ],
    },
    {
      id: 'hotkeys',
      icon: 'key',
      description: 'Clique numa tecla e pressione a nova combinação. Esc cancela.',
      search: [
        ['Price check', 'Painel completo do item sob o cursor hotkey atalho tecla'],
        ['Price check fixado', 'Abre o painel já fixado na tela hotkey atalho tecla'],
        ['Mostrar/ocultar overlay', 'Alterna a visibilidade dos painéis hotkey atalho tecla'],
        ['Abrir configurações', 'Esta janela hotkey atalho tecla'],
        ['Paleta de comandos', 'Só dentro do Mirrorsight · não editável hotkey atalho ctrl k comandos busca'],
      ],
    },
    {
      id: 'price-check',
      icon: 'tag',
      description: 'Filtros padrão, ranges da busca, preço sugerido e dados locais.',
      search: [
        ['Range dos mods', 'Mínimo pré-preenchido = valor rolado − % porcentagem filtro mínimo margem'],
        ['Mostrar vendedores offline', 'Por padrão só online offline afk'],
        ['Filtros marcados por padrão', 'Tipos de mod que já vêm selecionados explicit fractured implicit crafted enchant'],
        ['Nível padrão', 'Qual dos três níveis vem selecionado e vai para a nota ~b/o compra justo venda percentil p25 p50 p75 mediana sugerido estratégia'],
        ['Histórico', 'price checks salvos neste computador histórico apagar limpar recentes'],
        ['Presets de mods', 'no PoE 1 no PoE 2 preset salvos mods seleção'],
      ],
    },
    {
      id: 'appearance',
      icon: 'brush',
      description: 'Escala, densidade e movimento.',
      search: [
        ['Escala da interface', 'Para monitores 1440p/4K zoom tamanho'],
        ['Densidade', 'Altura das linhas de listagem compacto'],
        ['Reduzir animações', 'Troca instantânea, sem movimento motion acessibilidade'],
      ],
    },
    { id: 'about', icon: 'info', description: 'Versão e créditos.', search: [] },
  ];

  let query = $state('');
  let content: HTMLElement | undefined = $state();
  let hlTimer: ReturnType<typeof setTimeout> | undefined;
  /** Cada tecla invalida a busca anterior (ela pode ainda estar esperando o `tick`). */
  let searchSeq = 0;

  const pane = $derived(PANES.find((p) => p.id === nav.settingsPane) ?? PANES[0]!);

  type Pane = (typeof PANES)[number];

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

<article class="settings" aria-label="Configurações">
  <nav class="side" aria-label="Seções das configurações">
    <label class="search">
      <Icon name="search" size={14} />
      <input class="field" placeholder="Buscar ajuste…" aria-label="Buscar ajuste" bind:value={query} oninput={search} />
    </label>
    {#each PANES as p (p.id)}
      <button class="nav-item" aria-current={nav.settingsPane === p.id ? 'page' : undefined} onclick={() => (nav.settingsPane = p.id)}>
        <Icon name={p.icon} size={16} />{PANE_LABEL[p.id]}
      </button>
    {/each}
    <div class="ver mono">v0.1.0 · {settings.current?.game === 'poe2' ? 'PoE2' : 'PoE1'}</div>
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
      <p class="error">{settings.loadError ?? 'Carregando…'}</p>
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
