<script lang="ts">
  /** Contagem regressiva circular (84px, âmbar) até `until` (epoch ms). */
  let { until, totalMs }: { until: number; totalMs: number } = $props();

  const RADIUS = 38;
  const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

  let now = $state(Date.now());

  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(timer);
  });

  const remaining = $derived(Math.max(0, until - now));
  const seconds = $derived(Math.ceil(remaining / 1000));
  const offset = $derived(CIRCUMFERENCE * (1 - remaining / Math.max(1, totalMs)));
</script>

<div class="ring" role="timer" aria-label="{seconds} segundos">
  <svg width="84" height="84" viewBox="0 0 84 84" aria-hidden="true">
    <circle class="track" cx="42" cy="42" r={RADIUS} />
    <circle class="bar" cx="42" cy="42" r={RADIUS} stroke-dasharray={CIRCUMFERENCE} stroke-dashoffset={offset} />
  </svg>
  <b class="mono">{seconds}s</b>
</div>

<style>
  .ring { position: relative; width: 84px; height: 84px; }
  svg { transform: rotate(-90deg); }
  circle { fill: none; stroke-width: 6; }
  .track { stroke: var(--ms-surface-3); }
  .bar { stroke: var(--ms-warning); stroke-linecap: round; transition: stroke-dashoffset 250ms linear; }
  b { position: absolute; inset: 0; display: grid; place-items: center; font-size: 20px; font-weight: 800; color: var(--ms-warning); }
</style>
