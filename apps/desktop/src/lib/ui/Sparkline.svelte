<script lang="ts">
  let {
    values,
    width = 110,
    height = 36,
    color = 'var(--ms-success)',
  }: { values: number[]; width?: number; height?: number; color?: string } = $props();

  const gradientId = $props.id();

  const path = $derived.by(() => {
    if (values.length < 2) return null;
    const min = Math.min(...values);
    const span = Math.max(...values) - min || 1;
    const step = width / (values.length - 1);
    return values
      .map((v, i) => `${i ? 'L' : 'M'}${(i * step).toFixed(1)} ${(height - 3 - ((v - min) / span) * (height - 6)).toFixed(1)}`)
      .join(' ');
  });
</script>

{#if path}
  <svg viewBox="0 0 {width} {height}" {width} {height} preserveAspectRatio="none" style:color aria-hidden="true">
    <defs>
      <linearGradient id={gradientId} x1="0" x2="0" y1="0" y2="1">
        <stop offset="0" stop-color="currentColor" stop-opacity=".22" />
        <stop offset="1" stop-color="currentColor" stop-opacity="0" />
      </linearGradient>
    </defs>
    <path d="{path} L{width} {height} L0 {height}Z" fill="url(#{gradientId})" />
    <path class="line" d={path} pathLength="1" fill="none" stroke="currentColor" stroke-width="1.6"
      stroke-linejoin="round" stroke-linecap="round" />
  </svg>
{/if}

<style>
  svg { display: block; }
  .line {
    stroke-dasharray: 1;
    stroke-dashoffset: 1;
    animation: draw 700ms var(--ms-ease) 60ms forwards;
  }
  @keyframes draw { to { stroke-dashoffset: 0; } }
</style>
