<script lang="ts">
  import { backend } from '$lib/backend';
  import type { ModifierKind, Settings, SuggestedLevel } from '$lib/types';
  import { LEVEL_META } from '$lib/pricing';
  import { HISTORY_LIMIT, history } from '$lib/stores/history.svelte';
  import { nav } from '$lib/stores/nav.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Popover from '$lib/ui/Popover.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';
  import Toggle from '$lib/ui/Toggle.svelte';
  import SettingRow from './SettingRow.svelte';

  let { current }: { current: Settings } = $props();

  /** Valor exibido durante o arraste; salva só ao soltar (evita gravar a cada pixel). */
  let range = $state(0);
  $effect(() => {
    range = current.mod_range_percent;
  });

  const KINDS: { kind: ModifierKind; label: string }[] = [
    { kind: 'explicit', label: 'Explicit' },
    { kind: 'fractured', label: 'Fractured' },
    { kind: 'implicit', label: 'Implicit' },
    { kind: 'crafted', label: 'Crafted' },
    { kind: 'enchant', label: 'Enchant' },
    { kind: 'property', label: 'Atributos (defesas/DPS)' },
  ];

  const LEVELS = (['buy', 'fair', 'sell'] as const).map((value) => ({
    value,
    label: `${LEVEL_META[value].name} · ${LEVEL_META[value].tag}`,
  }));

  /** Contagem de presets dos dois jogos (o store só guarda o jogo atual). */
  let presetCount = $state<{ poe1: number; poe2: number } | null>(null);
  $effect(() => {
    void history.load();
    void Promise.all([backend.listPresets('poe1'), backend.listPresets('poe2')])
      .then(([a, b]) => (presetCount = { poe1: a.length, poe2: b.length }))
      .catch(() => (presetCount = null));
  });

  let clearBtn: HTMLButtonElement | undefined = $state();
  let confirming = $state(false);

  function toggleKind(kind: ModifierKind) {
    const kinds = current.default_filter_kinds.includes(kind)
      ? current.default_filter_kinds.filter((k) => k !== kind)
      : [...current.default_filter_kinds, kind];
    void settings.update({ default_filter_kinds: kinds });
  }

  async function clearHistory() {
    confirming = false;
    await history.clear();
  }
</script>

<div class="label sub">Busca</div>
<div class="group">
  <SettingRow label="Range dos mods" description="Mínimo pré-preenchido = valor rolado − %" keywords="porcentagem filtro mínimo margem">
    <input class="slider" type="range" min="0" max="50" style="--p:{(range / 50) * 100}%" bind:value={range}
      onchange={() => settings.update({ mod_range_percent: range })} />
    <span class="mono val">{range}%</span>
  </SettingRow>
  <SettingRow label="Mostrar vendedores offline" description="Por padrão só online" keywords="offline afk">
    <Toggle label="Mostrar vendedores offline" checked={current.show_offline} onchange={(show_offline) => settings.update({ show_offline })} />
  </SettingRow>
  <SettingRow label="Filtros marcados por padrão" description="Tipos de mod que já vêm selecionados" keywords="explicit fractured implicit crafted enchant" wrap>
    {#each KINDS as k (k.kind)}
      <button class="chip" class:on={current.default_filter_kinds.includes(k.kind)} aria-pressed={current.default_filter_kinds.includes(k.kind)}
        onclick={() => toggleKind(k.kind)}>
        {k.label}
      </button>
    {/each}
  </SettingRow>
</div>

<div class="label sub">Preço sugerido <span class="new">NOVO</span></div>
<div class="group">
  <SettingRow label="Nível padrão" description="Qual dos três níveis vem selecionado e vai para a nota ~b/o" keywords="compra justo venda percentil p25 p50 p75 mediana sugerido estratégia">
    <Segmented
      tone="suggest"
      ariaLabel="Nível padrão do preço sugerido"
      options={LEVELS}
      value={current.suggested_level}
      onchange={(suggested_level: SuggestedLevel) => settings.update({ suggested_level })}
    />
  </SettingRow>
  <div class="info">
    <Icon name="info" size={15} />
    <p>
      Os níveis são percentis dos anúncios carregados, em chaos: Compra = p25, Justo = mediana, Venda = p75.
      Com menos de 5 anúncios os três mostram “—”.
    </p>
  </div>
</div>

<div class="label sub">Dados locais</div>
<div class="group">
  <SettingRow label="Histórico" description="{history.entries.length} de {HISTORY_LIMIT} price checks salvos neste computador" keywords="histórico apagar limpar recentes">
    <button class="btn btn-ghost btn-sm" onclick={() => nav.go('history')}>Ver histórico</button>
    <button class="btn btn-ghost btn-sm danger" bind:this={clearBtn} disabled={history.entries.length === 0}
      onclick={() => (confirming = !confirming)}>Limpar</button>
  </SettingRow>
  <SettingRow
    label="Presets de mods"
    description={presetCount ? `${presetCount.poe1} no PoE 1 · ${presetCount.poe2} no PoE 2` : 'Não foi possível contar os presets'}
    keywords="preset salvos mods seleção"
  >
    <span class="note">Gerencie pelo menu Preset no Price Check</span>
  </SettingRow>
</div>

{#if confirming}
  <Popover anchor={clearBtn} align="end" width={280} label="Confirmar limpeza" onclose={() => (confirming = false)}>
    <div class="confirm">
      <b>Apagar os {history.entries.length} price checks?</b>
      <p>Não dá para desfazer. Presets e configurações não são afetados.</p>
      <div class="actions">
        <button class="btn btn-ghost btn-sm" onclick={() => (confirming = false)}>Cancelar</button>
        <button class="btn btn-danger btn-sm" onclick={clearHistory}>Limpar</button>
      </div>
    </div>
  </Popover>
{/if}

<style>
  .val { width: 36px; text-align: right; }
  .sub { display: flex; align-items: center; gap: 8px; margin: 4px 0 8px; }
  .new {
    font-size: 9.5px;
    font-weight: 800;
    letter-spacing: 0.06em;
    padding: 2px 6px;
    border-radius: var(--ms-r-sm);
    color: var(--ms-suggest);
    background: var(--ms-suggest-soft);
    border: 1px solid #3d3166;
  }
  .info {
    display: flex;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--ms-r-md);
    background: var(--ms-suggest-soft);
    border: 1px solid #3d3166;
    color: var(--ms-suggest);
  }
  .info :global(svg) { flex: none; margin-top: 1px; }
  .info p { font-size: 12px; color: var(--ms-text-2); line-height: 1.5; }
  .note { font-size: 11.5px; color: var(--ms-text-3); }
  .confirm { padding: 14px; display: flex; flex-direction: column; gap: 6px; }
  .confirm p { font-size: 12px; color: var(--ms-text-2); }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 6px; }
</style>
