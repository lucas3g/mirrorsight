<script lang="ts" module>
  export type IconName =
    | 'pin' | 'x' | 'ext' | 'eye' | 'search' | 'gear' | 'key' | 'tag' | 'brush'
    | 'info' | 'wifi' | 'question' | 'empty' | 'item' | 'clipboard' | 'reset' | 'bolt'
    | 'clock' | 'copy' | 'trash' | 'pencil' | 'grip' | 'chevron-down' | 'chevron-left' | 'chevron-right'
    | 'check' | 'plus' | 'chat' | 'warning' | 'command' | 'layers'
    | 'win-minimize' | 'win-maximize' | 'win-restore' | 'win-close';
</script>

<script lang="ts">
  let { name, size = 15 }: { name: IconName; size?: number } = $props();
</script>

<svg viewBox="0 0 24 24" width={size} height={size} aria-hidden="true" fill="none" stroke="currentColor"
  stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
  {#if name === 'pin'}
    <path d="M15 3l6 6-4 1-4 4 1 5-2 2-4-4-5 5M8 13l-4-4 2-2 5 1 4-4z" />
  {:else if name === 'x'}
    <path d="M6 6l12 12M18 6L6 18" />
  {:else if name === 'ext'}
    <path d="M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5" />
  {:else if name === 'eye'}
    <path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12z" /><circle cx="12" cy="12" r="3.2" />
  {:else if name === 'search'}
    <circle cx="11" cy="11" r="6.5" /><path d="M16 16l4.5 4.5" />
  {:else if name === 'gear'}
    <circle cx="12" cy="12" r="3" /><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1" />
  {:else if name === 'key'}
    <rect x="2" y="6" width="20" height="12" rx="2" /><path d="M6 10h1M10 10h1M14 10h1M7 14h10" />
  {:else if name === 'tag'}
    <path d="M3 12V4h8l10 10-8 8z" /><circle cx="7.5" cy="8.5" r="1.2" />
  {:else if name === 'brush'}
    <path d="M14 4l6 6-8 8-6-6zM6 12l-3 6 3 3 6-3" />
  {:else if name === 'info'}
    <circle cx="12" cy="12" r="9" /><path d="M12 11v6M12 7.5v.5" />
  {:else if name === 'wifi'}
    <path d="M2 8.5a15 15 0 0 1 20 0M5 12a10 10 0 0 1 14 0M8.5 15.5a5 5 0 0 1 7 0M3 3l18 18" />
  {:else if name === 'question'}
    <circle cx="12" cy="12" r="9" /><path d="M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.7.3-1 .9-1 1.7M12 17v.5" />
  {:else if name === 'empty'}
    <rect x="3" y="5" width="18" height="14" rx="2" /><path d="M3 9h18M8 14h8" />
  {:else if name === 'item'}
    <path d="M12 2l8 5v10l-8 5-8-5V7z" /><path d="M12 7l4 2.5v5L12 17l-4-2.5v-5z" />
  {:else if name === 'clipboard'}
    <rect x="5" y="4" width="14" height="17" rx="2" /><path d="M9 4V3h6v1M9 10h6M9 14h4" />
  {:else if name === 'reset'}
    <path d="M4 12a8 8 0 1 0 2.4-5.7M4 4v4h4" />
  {:else if name === 'bolt'}
    <path d="M13 2L4 14h7l-1 8 9-12h-7z" />
  {:else if name === 'clock'}
    <circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" />
  {:else if name === 'copy'}
    <rect x="8" y="8" width="12" height="12" rx="2" /><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
  {:else if name === 'trash'}
    <path d="M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3" />
  {:else if name === 'pencil'}
    <path d="M4 20h4L19 9l-4-4L4 16v4zM13.5 6.5l4 4" />
  {:else if name === 'grip'}
    <g fill="currentColor" stroke="none"><circle cx="9" cy="6" r="1.6" /><circle cx="15" cy="6" r="1.6" /><circle cx="9" cy="12" r="1.6" /><circle cx="15" cy="12" r="1.6" /><circle cx="9" cy="18" r="1.6" /><circle cx="15" cy="18" r="1.6" /></g>
  {:else if name === 'chevron-down'}
    <path d="M6 9l6 6 6-6" />
  {:else if name === 'chevron-left'}
    <path d="M15 6l-6 6 6 6" />
  {:else if name === 'chevron-right'}
    <path d="M9 6l6 6-6 6" />
  {:else if name === 'check'}
    <path d="M5 12.5l4.5 4.5L19 7" />
  {:else if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'chat'}
    <path d="M4 5h16v11H9l-5 4z" /><path d="M8 9.5h8M8 12.5h5" />
  {:else if name === 'warning'}
    <path d="M12 3l10 18H2z" /><path d="M12 10v5M12 18v.5" />
  {:else if name === 'command'}
    <path d="M9 6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3z" />
  {:else if name === 'layers'}
    <path d="M12 3l9 5-9 5-9-5zM3 13l9 5 9-5" />
  {:else if name === 'win-minimize'}
    <path d="M6 12h12" stroke-width="1.6" />
  {:else if name === 'win-maximize'}
    <rect x="6.5" y="6.5" width="11" height="11" rx="1.5" stroke-width="1.6" />
  {:else if name === 'win-restore'}
    <rect x="5.5" y="8.5" width="10" height="10" rx="1.5" stroke-width="1.6" /><path d="M8.5 8.5V7a1.5 1.5 0 0 1 1.5-1.5h7A1.5 1.5 0 0 1 18.5 7v7a1.5 1.5 0 0 1-1.5 1.5h-1.5" stroke-width="1.6" />
  {:else if name === 'win-close'}
    <path d="M7 7l10 10M17 7L7 17" stroke-width="1.6" />
  {/if}
</svg>
