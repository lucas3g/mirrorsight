<script lang="ts">
  /** Switch 32×19: ligado em menta com bolinha branca; desligado em surface-4. */
  let {
    checked,
    label,
    onchange,
  }: { checked: boolean; label: string; onchange: (value: boolean) => void } = $props();
</script>

<button
  type="button"
  class="toggle"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  title={label}
  onclick={() => onchange(!checked)}
></button>

<style>
  .toggle {
    position: relative;
    width: 32px;
    height: 19px;
    flex: none;
    border-radius: 99px;
    background: var(--ms-surface-4);
    transition: background 200ms;
  }
  .toggle::after {
    content: '';
    position: absolute;
    top: 2.5px;
    left: 2.5px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #e8e8f0;
    transition: transform 350ms var(--ms-spring);
  }
  .toggle[aria-checked='true'] { background: var(--ms-accent); }
  .toggle[aria-checked='true']::after { transform: translateX(13px); background: #fff; }
  .toggle:focus-visible { box-shadow: 0 0 0 3px var(--ms-accent-soft), 0 0 0 1px var(--ms-accent); border-radius: 99px; }
</style>
