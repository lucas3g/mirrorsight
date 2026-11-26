<script lang="ts">
  /** Losango menta → lavanda com centro dourado (marca Aurora). */
  let { size = 28 }: { size?: number } = $props();
  const gradientId = $props.id();
</script>

<svg viewBox="0 0 32 32" width={size} height={size} aria-hidden="true">
  <defs>
    <linearGradient id={gradientId} x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#4ff0c4" />
      <stop offset="1" stop-color="#a98cff" />
    </linearGradient>
  </defs>
  <path d="M16 2 29 16 16 30 3 16z" fill="url(#{gradientId})" />
  <path d="M16 8 23 16 16 24 9 16z" fill="#09090f" />
  <path d="M16 12 19.5 16 16 20 12.5 16z" fill="#ffcf6e" />
</svg>
