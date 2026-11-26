<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon, { type IconName } from '$lib/ui/Icon.svelte';

  let {
    icon,
    title,
    text,
    tone = 'neutral',
    visual,
    children,
  }: {
    icon?: IconName;
    title: string;
    text?: string;
    tone?: 'neutral' | 'danger' | 'warning';
    /** Substitui o ícone (ex.: anel de contagem regressiva). */
    visual?: Snippet;
    children?: Snippet;
  } = $props();
</script>

<div class="state {tone}">
  {#if visual}
    {@render visual()}
  {:else if icon}
    <div class="ill"><Icon name={icon} size={26} /></div>
  {/if}
  <h5>{title}</h5>
  {#if text}<p>{text}</p>{/if}
  {#if children}<div class="actions">{@render children()}</div>{/if}
</div>

<style>
  .state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 10px;
    padding: 30px 18px;
    animation: ms-in-up 450ms var(--ms-spring) both;
  }
  h5 { font-size: 15px; font-weight: 700; }
  p { font-size: 12.5px; color: var(--ms-text-2); max-width: 320px; }
  .ill {
    width: 56px;
    height: 56px;
    display: grid;
    place-items: center;
    border-radius: 16px;
    background: var(--ms-surface-2);
    color: var(--ms-text-2);
  }
  .danger .ill { color: var(--ms-danger); background: var(--ms-danger-soft); }
  .warning .ill { color: var(--ms-warning); background: var(--ms-warning-soft); }
  .actions { display: flex; gap: 8px; margin-top: 4px; }
</style>
