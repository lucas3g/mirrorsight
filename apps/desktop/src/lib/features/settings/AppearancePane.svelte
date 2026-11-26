<script lang="ts">
  import type { Density, Settings } from '$lib/types';
  import { settings } from '$lib/stores/settings.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';
  import Toggle from '$lib/ui/Toggle.svelte';
  import SettingRow from './SettingRow.svelte';

  let { current }: { current: Settings } = $props();

  const SCALES = ['0.9', '1', '1.15', '1.3'] as const;
  type Scale = (typeof SCALES)[number];
  const scale = $derived<Scale>(SCALES.find((s) => Number(s) === current.ui_scale) ?? '1');
</script>

<div class="group">
  <SettingRow label="Escala da interface" description="Para monitores 1440p/4K" keywords="zoom tamanho">
    <Segmented
      ariaLabel="Escala"
      options={SCALES.map((s) => ({ value: s, label: `${Math.round(Number(s) * 100)}%` }))}
      value={scale}
      onchange={(v: Scale) => settings.update({ ui_scale: Number(v) })}
    />
  </SettingRow>
  <SettingRow label="Densidade" description="Altura das linhas de listagem" keywords="compacto">
    <Segmented
      ariaLabel="Densidade"
      options={[{ value: 'compact', label: 'Compacta' }, { value: 'normal', label: 'Normal' }]}
      value={current.density}
      onchange={(density: Density) => settings.update({ density })}
    />
  </SettingRow>
  <SettingRow label="Reduzir animações" description="Troca instantânea, sem movimento" keywords="motion acessibilidade">
    <Toggle label="Reduzir animações" checked={current.reduce_motion} onchange={(reduce_motion) => settings.update({ reduce_motion })} />
  </SettingRow>
</div>
