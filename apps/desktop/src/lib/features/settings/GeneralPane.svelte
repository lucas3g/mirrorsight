<script lang="ts">
  import type { DisplayCurrency, GameVersion, Settings } from '$lib/types';
  import { leagues } from '$lib/stores/leagues.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import CurrencyIcon from '$lib/ui/CurrencyIcon.svelte';
  import Segmented from '$lib/ui/Segmented.svelte';
  import Toggle from '$lib/ui/Toggle.svelte';
  import SettingRow from './SettingRow.svelte';
  import { LOCALES, resolveLocale } from '$lib/i18n/core.svelte';
  import { LANGUAGE_KEYWORDS, languageMsgs } from '$lib/i18n/messages/language';
  import { SETTINGS_KEYWORDS as KW, settingsMsgs } from '$lib/i18n/messages/settings';

  const m = $derived(settingsMsgs.general);

  let { current }: { current: Settings } = $props();

  const LANGUAGES = [
    { value: 'en', label: 'English' },
    { value: 'pt', label: 'Português (BR)' },
    { value: 'ru', label: 'Русский' },
    { value: 'de', label: 'Deutsch' },
    { value: 'fr', label: 'Français' },
    { value: 'es', label: 'Español' },
  ];

  /** Valor do `<select>` para `ui_language = null` (segue o sistema). */
  const AUTO = 'auto';
  const systemLabel = LOCALES.find((l) => l.code === resolveLocale(null))?.label ?? 'English';

  // Ligas e troca de jogo vêm do store compartilhado com a barra de título.
  $effect(() => {
    void leagues.load(current.game);
  });
</script>

<div class="group">
  <SettingRow label={languageMsgs.label} description={languageMsgs.description} keywords={LANGUAGE_KEYWORDS}>
    <select class="field" style="width:220px" value={current.ui_language ?? AUTO}
      onchange={(e) => settings.update({ ui_language: e.currentTarget.value === AUTO ? null : e.currentTarget.value })}>
      <option value={AUTO}>{languageMsgs.automatic(systemLabel)}</option>
      {#each LOCALES as loc (loc.code)}<option value={loc.code}>{loc.label}</option>{/each}
    </select>
  </SettingRow>
  <SettingRow label={m.game.label} description={m.game.description} keywords={KW.game}>
    <Segmented
      size="lg"
      ariaLabel={m.game.label}
      options={[{ value: 'poe1', label: 'Path of Exile' }, { value: 'poe2', label: 'Path of Exile 2' }]}
      value={current.game}
      onchange={(game: GameVersion) => leagues.changeGame(game)}
    />
  </SettingRow>
  <SettingRow
    label={m.league.label}
    description={leagues.error ? m.leagueError : m.league.description}
    keywords={KW.league}
  >
    <select class="field league" value={current.league} disabled={leagues.loading}
      onchange={(e) => leagues.changeLeague(e.currentTarget.value)}>
      {#each leagues.list as league (league)}<option value={league}>{league}</option>{/each}
    </select>
  </SettingRow>
  <SettingRow label={languageMsgs.gameLabel} description={languageMsgs.gameDescription} keywords={KW.gameLanguage}>
    <select class="field" style="width:200px" value={current.language} onchange={(e) => settings.update({ language: e.currentTarget.value })}>
      {#each LANGUAGES as lang (lang.value)}<option value={lang.value}>{lang.label}</option>{/each}
    </select>
  </SettingRow>
  <SettingRow label={m.currency.label} description={m.currency.description} keywords={KW.currency}>
    <Segmented
      ariaLabel={m.currency.label}
      options={[{ value: 'chaos', label: 'Chaos' }, { value: 'divine', label: 'Divine' }]}
      value={current.display_currency}
      onchange={(display_currency: DisplayCurrency) => settings.update({ display_currency })}
    >
      {#snippet item(opt)}<CurrencyIcon currency={opt.value} size={18} />{opt.label}{/snippet}
    </Segmented>
  </SettingRow>
</div>

<div class="group">
  <SettingRow label={m.restoreClipboard.label} description={m.restoreClipboard.description} keywords={KW.restoreClipboard}>
    <Toggle label={m.restoreClipboard.label} checked={current.restore_clipboard}
      onchange={(restore_clipboard) => settings.update({ restore_clipboard })} />
  </SettingRow>
  <SettingRow label={m.startup.label} description={m.startup.description} keywords={KW.startup}>
    <Toggle label={m.startup.label} checked={current.launch_on_startup}
      onchange={(launch_on_startup) => settings.update({ launch_on_startup })} />
  </SettingRow>
</div>

<style>
  .league { width: 220px; }
</style>
