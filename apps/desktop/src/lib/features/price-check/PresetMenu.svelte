<script lang="ts">
  import { tick } from 'svelte';
  import type { ModPreset } from '$lib/types';
  import { applyPresetWithToast } from '$lib/stores/commands.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Popover from '$lib/ui/Popover.svelte';

  /** Chip "Presets ▾" / "Preset: {nome} ▾" com o menu de presets do jogo atual. */
  let open = $state(false);
  let chip: HTMLButtonElement | undefined = $state();
  let newName = $state('');
  let renaming = $state<string | null>(null);
  let renameValue = $state('');
  let confirming = $state<string | null>(null);

  const applied = $derived(presets.list.find((p) => p.id === priceCheck.presetId) ?? null);
  const enabledCount = $derived(priceCheck.filters?.stats.filter((s) => s.enabled).length ?? 0);
  const gameLabel = $derived(presets.game === 'poe2' ? 'PoE 2' : 'PoE 1');

  function missingIn(preset: ModPreset): number {
    const stats = priceCheck.filters?.stats ?? [];
    return preset.mods.filter((m) => !stats.some((s) => s.trade_id === m.trade_id)).length;
  }

  function detail(preset: ModPreset): string {
    const base = `${preset.mods.length} ${preset.mods.length === 1 ? 'mod' : 'mods'}`;
    if (preset.id === priceCheck.presetId) return `${base} · aplicado`;
    const missing = missingIn(preset);
    if (missing === 1) return `${base} · 1 não está neste item`;
    if (missing > 1) return `${base} · ${missing} não estão neste item`;
    return base;
  }

  function apply(preset: ModPreset) {
    applyPresetWithToast(preset.id);
    open = false;
  }

  async function startRename(preset: ModPreset) {
    confirming = null;
    renaming = preset.id;
    renameValue = preset.name;
    await tick();
    document.querySelector<HTMLInputElement>('[data-rename]')?.select();
  }

  async function commitRename(preset: ModPreset) {
    if (renameValue.trim() && renameValue.trim() !== preset.name) {
      if (!(await presets.rename(preset, renameValue))) return;
    }
    renaming = null;
  }

  async function saveCurrent() {
    if (!priceCheck.filters) return;
    if (await presets.save(newName, $state.snapshot(priceCheck.filters.stats))) {
      newName = '';
      const created = presets.list[presets.list.length - 1];
      if (created) priceCheck.presetId = created.id;
    }
  }

  function onRowKey(e: KeyboardEvent, preset: ModPreset) {
    if (e.key === 'Enter') {
      e.preventDefault();
      void commitRename(preset);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      renaming = null;
    }
  }
</script>

<button
  class="chip"
  class:on={!!applied}
  bind:this={chip}
  aria-haspopup="dialog"
  aria-expanded={open}
  onclick={() => (open = !open)}
>
  <Icon name="layers" size={13} />
  {applied ? `Preset: ${applied.name}` : 'Presets'}
  <Icon name="chevron-down" size={13} />
</button>

{#if open}
  <Popover anchor={chip} width={320} label="Presets de mods" onclose={() => { open = false; renaming = null; confirming = null; }}>
    <div class="menu">
      <header>
        <span class="label">Presets · {gameLabel}</span>
        <span class="count">{presets.list.length} {presets.list.length === 1 ? 'salvo' : 'salvos'}</span>
      </header>

      {#if presets.list.length === 0}
        <div class="empty">
          <b>Nenhum preset salvo para {gameLabel}</b>
          <span>Ligue os mods que quer e use “Salvar seleção atual”.</span>
        </div>
      {:else}
        <div class="list scroll">
          {#each presets.list as preset (preset.id)}
            {#if confirming === preset.id}
              <div class="row confirm">
                <span>Excluir “{preset.name}”?</span>
                <button class="btn btn-ghost btn-sm" onclick={() => (confirming = null)}>Cancelar</button>
                <button class="btn btn-danger btn-sm" onclick={() => { confirming = null; void presets.remove(preset); }}>Excluir</button>
              </div>
            {:else if renaming === preset.id}
              <div class="row">
                <input
                  class="field"
                  data-rename
                  maxlength="40"
                  aria-label="Novo nome"
                  bind:value={renameValue}
                  onkeydown={(e) => onRowKey(e, preset)}
                />
                <button class="ib sm" title="Salvar nome" aria-label="Salvar nome" onclick={() => commitRename(preset)}><Icon name="check" size={14} /></button>
              </div>
            {:else}
              <div class="row">
                <button class="apply" onclick={() => apply(preset)}>
                  <span class="mark">{#if preset.id === priceCheck.presetId}<Icon name="check" size={14} />{/if}</span>
                  <span class="txt"><b>{preset.name}</b><small>{detail(preset)}</small></span>
                </button>
                <button class="ib sm" title="Renomear" aria-label="Renomear {preset.name}" onclick={() => startRename(preset)}><Icon name="pencil" size={14} /></button>
                <button class="ib sm del" title="Excluir" aria-label="Excluir {preset.name}" onclick={() => { renaming = null; confirming = preset.id; }}><Icon name="trash" size={14} /></button>
              </div>
            {/if}
          {/each}
        </div>
      {/if}

      <div class="save">
        <span class="label">Salvar seleção atual ({enabledCount} {enabledCount === 1 ? 'mod' : 'mods'})</span>
        <form class="line" onsubmit={(e) => { e.preventDefault(); void saveCurrent(); }}>
          <input class="field" placeholder="Nome do preset" maxlength="40" aria-label="Nome do preset" bind:value={newName} />
          <button class="btn btn-primary btn-sm" type="submit" disabled={enabledCount === 0 || !newName.trim()}>Salvar</button>
        </form>
      </div>
    </div>
  </Popover>
{/if}

<style>
  .menu { display: flex; flex-direction: column; }
  header { display: flex; align-items: center; justify-content: space-between; padding: 12px 14px 8px; }
  .count { font-size: 11.5px; color: var(--ms-text-3); }
  .list { max-height: 260px; padding: 0 6px 6px; display: flex; flex-direction: column; gap: 2px; }
  .row { display: flex; align-items: center; gap: 4px; min-height: 44px; padding: 0 4px; border-radius: var(--ms-r-md); }
  .row:hover { background: var(--ms-surface-3); }
  .row .field { flex: 1; height: 30px; }
  .apply { flex: 1; min-width: 0; display: flex; align-items: center; gap: 8px; text-align: left; padding: 4px; }
  .mark { width: 16px; flex: none; color: var(--ms-accent); }
  .txt { min-width: 0; display: flex; flex-direction: column; }
  .txt b { font-weight: 650; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .txt small { font-size: 11px; color: var(--ms-text-3); }
  .del:hover { color: var(--ms-danger); background: var(--ms-danger-soft); }
  .confirm { gap: 6px; padding: 0 8px; }
  .confirm span { flex: 1; font-size: 12px; font-weight: 600; }
  .empty { display: flex; flex-direction: column; gap: 4px; padding: 6px 14px 12px; font-size: 12px; color: var(--ms-text-3); }
  .empty b { color: var(--ms-text-2); }
  .save { display: flex; flex-direction: column; gap: 8px; padding: 12px 14px 14px; border-top: 1px solid var(--ms-border); }
  .line { display: flex; gap: 6px; }
  .line .field { flex: 1; height: 30px; }
</style>
