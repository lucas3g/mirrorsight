<script lang="ts">
  import { toasts } from '$lib/stores/toasts.svelte';
  import Icon, { type IconName } from './Icon.svelte';

  const ICON: Record<string, IconName> = { info: 'info', success: 'check', warning: 'warning', error: 'warning' };
</script>

<div class="toasts" aria-live="polite">
  {#each toasts.items as toast (toast.id)}
    <button type="button" class="toast {toast.kind}" class:leaving={toast.leaving} onclick={() => toasts.dismiss(toast.id)}>
      <Icon name={ICON[toast.kind] ?? 'info'} size={16} />
      <span>{toast.message}</span>
    </button>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 18px;
    bottom: 44px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    pointer-events: none;
  }
  :global(html.overlay) .toasts { bottom: 18px; }
  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 10px;
    text-align: left;
    max-width: 380px;
    padding: 10px 14px;
    border-radius: var(--ms-r-md);
    font-size: 12.5px;
    font-weight: 600;
    background: var(--ms-surface-3);
    border: 1px solid var(--ms-border-strong);
    box-shadow: 0 20px 40px -10px #000;
    animation: ms-in-right 450ms var(--ms-spring) both;
  }
  .toast :global(svg) { color: var(--c); }
  .toast.leaving { animation: ms-out-right 300ms var(--ms-ease) forwards; }
  .info { --c: var(--ms-accent); }
  .success { --c: var(--ms-accent); }
  .warning { --c: var(--ms-warning); }
  .error { --c: var(--ms-danger); }
</style>
