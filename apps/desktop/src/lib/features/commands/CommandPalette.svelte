<script lang="ts">
  import { tick } from 'svelte';
  import { commandsMsgs as m } from '$lib/i18n/messages/commands';
  import { commands, type Command } from '$lib/stores/commands.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import { portal } from '$lib/ui/portal';

  /**
   * Paleta Ctrl+K. Na janela principal tem todos os grupos; no overlay só item atual e presets.
   * Esc ou Ctrl+K fecham; o foco volta ao elemento anterior (`commands.close`).
   */
  let { role }: { role: 'main' | 'overlay' } = $props();

  let query = $state('');
  let active = $state(0);
  let input: HTMLInputElement | undefined = $state();
  let list: HTMLElement | undefined = $state();

  const groups = $derived(commands.open ? commands.build(query, role) : []);
  const flat = $derived(groups.flatMap((g) => g.commands));

  // Reabre sempre limpa e com o campo focado.
  $effect(() => {
    if (!commands.open) return;
    query = '';
    void tick().then(() => input?.focus());
  });

  $effect(() => {
    void query;
    active = 0;
  });

  async function run(cmd: Command | undefined) {
    if (!cmd) return;
    commands.close();
    await tick();
    await cmd.run();
  }

  function onKeydown(e: KeyboardEvent) {
    // Nada da paleta vaza para os atalhos do price check.
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (flat.length === 0) return;
      active = (active + (e.key === 'ArrowDown' ? 1 : -1) + flat.length) % flat.length;
      list?.querySelector<HTMLElement>(`[data-row="${active}"]`)?.scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'Enter') {
      e.preventDefault();
      void run(flat[active]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      commands.close();
    }
  }
</script>

{#if commands.open}
  <div class="scrim" use:portal role="presentation" onpointerdown={(e) => e.target === e.currentTarget && commands.close()}>
    <div class="palette" role="dialog" tabindex="-1" aria-modal="true" aria-label={m.paletteAria} data-popover onkeydown={onKeydown}>
      <label class="field-row">
        <Icon name="command" size={17} />
        <input
          bind:this={input}
          bind:value={query}
          placeholder={m.placeholder}
          aria-label={m.searchAria}
          role="combobox"
          aria-expanded="true"
          aria-controls="ms-command-list"
          aria-activedescendant={flat.length ? `ms-cmd-${active}` : undefined}
        />
        <kbd class="kbd">Esc</kbd>
      </label>

      <div class="list scroll" id="ms-command-list" role="listbox" bind:this={list}>
        {#if flat.length === 0}
          <div class="empty">{query.trim() ? m.notFound(query.trim()) : m.noneAvailable}</div>
        {:else}
          {#each groups as group (group.title)}
            <div class="label grp">{group.title}</div>
            {#each group.commands as cmd (cmd.id)}
              {@const n = flat.indexOf(cmd)}
              <button
                id="ms-cmd-{n}"
                class="item"
                class:on={n === active}
                role="option"
                aria-selected={n === active}
                tabindex="-1"
                data-row={n}
                onmouseenter={() => (active = n)}
                onclick={() => run(cmd)}
              >
                <Icon name={cmd.icon} size={16} />
                <span class="txt">{cmd.label}</span>
                {#if cmd.detail}<span class="detail">{cmd.detail}</span>{/if}
                {#if cmd.kbd}<kbd class="kbd">{cmd.kbd}</kbd>{/if}
              </button>
            {/each}
          {/each}
        {/if}
      </div>

      <div class="foot">
        <span>{m.hintNav}</span>
        <span><kbd class="kbd">Ctrl</kbd> <kbd class="kbd">K</kbd> {m.hintToggle}</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: #05050acc;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 12vh 16px 16px;
  }
  .palette {
    width: min(580px, 100%);
    max-height: calc(88vh - 16px);
    display: flex;
    flex-direction: column;
    border-radius: var(--ms-r-xl);
    background: var(--ms-surface-1);
    border: 1px solid var(--ms-border-strong);
    box-shadow: 0 40px 80px -20px #000;
    overflow: hidden;
    animation: ms-in-down 360ms var(--ms-spring) both;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 54px;
    padding: 0 16px;
    border-bottom: 1px solid var(--ms-border);
    color: var(--ms-text-3);
  }
  .field-row input {
    flex: 1;
    min-width: 0;
    background: none;
    border: 0;
    outline: none;
    font-size: 15px;
    color: var(--ms-text);
  }
  .field-row input::placeholder { color: var(--ms-text-3); }
  .list { max-height: 560px; min-height: 0; padding: 6px 8px 8px; }
  .grp { padding: 10px 8px 4px; }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 11px;
    height: 40px;
    padding: 0 10px;
    border-radius: var(--ms-r-md);
    text-align: left;
    color: var(--ms-text-2);
  }
  .item.on { background: var(--ms-accent-soft); color: var(--ms-accent); }
  .txt { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .item.on .txt { color: var(--ms-accent); }
  .detail {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    color: var(--ms-text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .item .kbd { margin-left: auto; }
  .empty { padding: 18px 10px; font-size: 12.5px; color: var(--ms-text-3); }
  .foot {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 9px 16px;
    font-size: 11.5px;
    color: var(--ms-text-3);
    background: var(--ms-surface-0);
    border-top: 1px solid var(--ms-border);
  }
</style>
