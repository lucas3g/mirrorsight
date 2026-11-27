<script lang="ts">
  /**
   * Ícone oficial do Mirrorsight: "M" escuro num hexágono laranja (o mesmo de
   * `src-tauri/icons`, redesenhado em vetor para ficar nítido em qualquer tamanho).
   */
  let { size = 28 }: { size?: number } = $props();
  const gradientId = $props.id();
</script>

<svg viewBox="48 48 928 928" width={size} height={size} aria-hidden="true">
  <defs>
    <linearGradient id={gradientId} x1="0" y1="64" x2="0" y2="960" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#fdac62" />
      <stop offset="1" stop-color="#e96013" />
    </linearGradient>
  </defs>
  <path d="M512 64 896 288v448L512 960 128 736V288z" fill="url(#{gradientId})" />
  <path d="M321 687V352l191 210 191-210v335" fill="none" stroke="#1b0d03" stroke-width="82"
    stroke-linecap="round" stroke-linejoin="round" />
</svg>
