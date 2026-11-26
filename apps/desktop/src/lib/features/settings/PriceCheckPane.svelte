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
  import { SETTINGS_KEYWORDS as KW, settingsMsgs } from '$lib/i18n/messages/settings';

  let { current }: { current: Settings } = $props();

  /** Valor exibido durante o arraste; salva só ao soltar (evita gravar a cada pixel). */
  let range = $state(0);
  $effect(() => {
    range = current.mod_range_percent;
  });

  const m = $derived(settingsMsgs.priceCheck);

  // Tipos de mod são termos do jogo; só `property` é traduzido.
  const KINDS: { kind: ModifierKind; label: string }[] = $derived([
    { kind: 'explicit', label: 'Explicit' },
    { kind: 'fractured', label: 'Fractured' },
    { kind: 'implicit', label: 'Implicit' },
    { kind: 'crafted', label: 'Crafted' },
    { kind: 'enchant', label: 'Enchant' },
    { kind: 'property', label: m.propertyKind },
  ]);

  // `$derived`: os nomes dos níveis podem acompanhar o idioma.
  const LEVELS = $derived(
    (['buy', 'fair', 'sell'] as const).map((value) => ({
      value,
      label: `${LEVEL_META[value].name} · ${LEVEL_META[value].tag}`,
    })),
  );

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

<div class="label sub">{m.searchSection}</div>
<div class="group">
  <SettingRow label={m.modRange.label} description={m.modRange.description} keywords={KW.modRange}>
    <input class="slider" type="range" min="0" max="50" style="--p:{(range / 50) * 100}%" bind:value={range}
      onchange={() => settings.update({ mod_range_percent: range })} />
    <span class="mono val">{range}%</span>
  </SettingRow>
  <SettingRow label={m.showOffline.label} description={m.showOffline.description} keywords={KW.showOffline}>
    <Toggle label={m.showOffline.label} checked={current.show_offline} onchange={(show_offline) => settings.update({ show_offline })} />
  </SettingRow>
  <SettingRow label={m.defaultFilters.label} description={m.defaultFilters.description} keywords={KW.defaultFilters} wrap>
    {#each KINDS as k (k.kind)}
      <button class="chip" class:on={current.default_filter_kinds.includes(k.kind)} aria-pressed={current.default_filter_kinds.includes(k.kind)}
        onclick={() => toggleKind(k.kind)}>
        {k.label}
      </button>
    {/each}
  </SettingRow>
</div>

<div class="label sub">{m.suggestedSection} <span class="new">{m.newBadge}</span></div>
<div class="group">
  <SettingRow label={m.defaultLevel.label} description={m.defaultLevel.description} keywords={KW.defaultLevel}>
    <Segmented
      tone="suggest"
      ariaLabel={m.defaultLevelAria}
      options={LEVELS}
      value={current.suggested_level}
      onchange={(suggested_level: SuggestedLevel) => settings.update({ suggested_level })}
    />
  </SettingRow>
  <div class="info">
    <Icon name="info" size={15} />
    <p>{m.levelsInfo(LEVEL_META.buy.name, LEVEL_META.fair.name, LEVEL_META.sell.name)}</p>
  </div>
</div>

<div class="label sub">{m.localSection}</div>
<div class="group">
  <SettingRow label={m.history.label} description={m.history.description(history.entries.length, HISTORY_LIMIT)} keywords={KW.history}>
    <button class="btn btn-ghost btn-sm" onclick={() => nav.go('history')}>{m.viewHistory}</button>
    <button class="btn btn-ghost btn-sm danger" bind:this={clearBtn} disabled={history.entries.length === 0}
      onclick={() => (confirming = !confirming)}>{m.clear}</button>
  </SettingRow>
  <SettingRow
    label={m.presets.label}
    description={presetCount ? m.presets.description(presetCount.poe1, presetCount.poe2) : m.presets.error}
    keywords={KW.presets}
  >
    <span class="note">{m.presetsNote}</span>
  </SettingRow>
</div>

{#if confirming}
  <Popover anchor={clearBtn} align="end" width={280} label={m.confirmLabel} onclose={() => (confirming = false)}>
    <div class="confirm">
      <b>{m.confirmTitle(history.entries.length)}</b>
      <p>{m.confirmBody}</p>
      <div class="actions">
        <button class="btn btn-ghost btn-sm" onclick={() => (confirming = false)}>{m.cancel}</button>
        <button class="btn btn-danger btn-sm" onclick={clearHistory}>{m.clear}</button>
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
