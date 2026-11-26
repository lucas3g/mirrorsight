<script lang="ts">
  import type { Density, Settings } from '$lib/types';
  import { settings } from '$lib/stores/settings.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';
  import Toggle from '$lib/ui/Toggle.svelte';
  import SettingRow from './SettingRow.svelte';
  import { SETTINGS_KEYWORDS as KW, settingsMsgs } from '$lib/i18n/messages/settings';

  let { current }: { current: Settings } = $props();

  const m = $derived(settingsMsgs.appearance);

  const SCALES = ['0.9', '1', '1.15', '1.3'] as const;
  type Scale = (typeof SCALES)[number];
  const scale = $derived<Scale>(SCALES.find((s) => Number(s) === current.ui_scale) ?? '1');
</script>

<div class="group">
  <SettingRow label={m.scale.label} description={m.scale.description} keywords={KW.scale}>
    <Segmented
      ariaLabel={m.scaleAria}
      options={SCALES.map((s) => ({ value: s, label: `${Math.round(Number(s) * 100)}%` }))}
      value={scale}
      onchange={(v: Scale) => settings.update({ ui_scale: Number(v) })}
    />
  </SettingRow>
  <SettingRow label={m.density.label} description={m.density.description} keywords={KW.density}>
    <Segmented
      ariaLabel={m.density.label}
      options={[{ value: 'compact', label: m.compact }, { value: 'normal', label: m.normal }]}
      value={current.density}
      onchange={(density: Density) => settings.update({ density })}
    />
  </SettingRow>
  <SettingRow label={m.reduceMotion.label} description={m.reduceMotion.description} keywords={KW.reduceMotion}>
    <Toggle label={m.reduceMotion.label} checked={current.reduce_motion} onchange={(reduce_motion) => settings.update({ reduce_motion })} />
  </SettingRow>
</div>
