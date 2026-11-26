<script lang="ts">
  import type { DisplayCurrency, GameVersion, Settings } from '$lib/types';
  import { leagues } from '$lib/stores/leagues.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';
  import Toggle from '$lib/ui/Toggle.svelte';
  import SettingRow from './SettingRow.svelte';

  let { current }: { current: Settings } = $props();

  const LANGUAGES = [
    { value: 'en', label: 'English' },
    { value: 'pt', label: 'Português (BR)' },
    { value: 'ru', label: 'Русский' },
    { value: 'de', label: 'Deutsch' },
    { value: 'fr', label: 'Français' },
    { value: 'es', label: 'Español' },
  ];

  // Ligas e troca de jogo vêm do store compartilhado com a barra de título.
  $effect(() => {
    void leagues.load(current.game);
  });
</script>

<div class="group">
  <SettingRow label="Jogo" description="Define parser, base de mods e API usada" keywords="poe path of exile 2 versão">
    <Segmented
      size="lg"
      ariaLabel="Jogo"
      options={[{ value: 'poe1', label: 'Path of Exile' }, { value: 'poe2', label: 'Path of Exile 2' }]}
      value={current.game}
      onchange={(game: GameVersion) => leagues.changeGame(game)}
    />
  </SettingRow>
  <SettingRow
    label="Liga"
    description={leagues.error ? 'Não foi possível carregar as ligas — usando a salva' : 'A primeira é a liga de desafio atual'}
    keywords="league temporada"
  >
    <select class="field league" value={current.league} disabled={leagues.loading}
      onchange={(e) => leagues.changeLeague(e.currentTarget.value)}>
      {#each leagues.list as league (league)}<option value={league}>{league}</option>{/each}
    </select>
  </SettingRow>
  <SettingRow label="Idioma do cliente" description="Idioma em que o jogo copia os itens" keywords="language lingua">
    <select class="field" style="width:200px" value={current.language} onchange={(e) => settings.update({ language: e.currentTarget.value })}>
      {#each LANGUAGES as lang (lang.value)}<option value={lang.value}>{lang.label}</option>{/each}
    </select>
  </SettingRow>
  <SettingRow label="Moeda de exibição" description="Valores acima de 1 divine sempre aparecem em divine" keywords="chaos divine conversão">
    <Segmented
      ariaLabel="Moeda de exibição"
      options={[{ value: 'chaos', label: 'Chaos' }, { value: 'divine', label: 'Divine' }]}
      value={current.display_currency}
      onchange={(display_currency: DisplayCurrency) => settings.update({ display_currency })}
    >
      {#snippet item(opt)}<CurrencyIcon currency={opt.value} size={18} />{opt.label}{/snippet}
    </Segmented>
  </SettingRow>
</div>

<div class="group">
  <SettingRow label="Restaurar clipboard" description="Devolve o conteúdo anterior após copiar o item" keywords="clipboard área de transferência">
    <Toggle label="Restaurar clipboard" checked={current.restore_clipboard}
      onchange={(restore_clipboard) => settings.update({ restore_clipboard })} />
  </SettingRow>
  <SettingRow label="Iniciar com o Windows" description="Fica na bandeja do sistema" keywords="startup inicialização bandeja">
    <Toggle label="Iniciar com o Windows" checked={current.launch_on_startup}
      onchange={(launch_on_startup) => settings.update({ launch_on_startup })} />
  </SettingRow>
</div>

<style>
  .league { width: 220px; }
</style>
