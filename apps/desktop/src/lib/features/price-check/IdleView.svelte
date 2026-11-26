<script lang="ts">
  import { backend } from '$lib/backend';
  import {
    DEMO_FEW,
    DEMO_NETWORK,
    DEMO_RATE_LIMIT,
    SAMPLE_CURRENCY_TEXT,
    SAMPLE_RARE_TEXT,
  } from '$lib/backend/demo-fixtures';
  import { RARITY_COLOR } from '$lib/format';
  import { priceCheckMsgs as pm } from '$lib/i18n/messages/priceCheck';
  import { history } from '$lib/stores/history.svelte';
  import { priceCheck } from '$lib/stores/priceCheck.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Icon from '$lib/ui/Icon.svelte';
  import Keycaps from '$lib/ui/Keycaps.svelte';
  import Logo from '$lib/ui/Logo.svelte';

  const priceCheckKey = $derived(
    settings.current?.hotkeys.find((h) => h.action === 'price-check')?.hotkey ?? 'Ctrl + D',
  );
  /** Recentes do jogo atual (reabrir exige o mesmo parser/catálogo). */
  const recent = $derived(history.entries.filter((e) => e.game === (settings.current?.game ?? 'poe1')).slice(0, 3));
</script>

<div class="idle">
  <div class="in enter-up">
    <div class="orb"><Logo size={54} /></div>
    <h1>{pm.readyTitle}</h1>
    <p class="lead">{pm.leadBefore} <Keycaps combo={priceCheckKey} />{pm.leadAfter}</p>
    <p class="note">
      <Icon name="info" size={13} />
      {pm.pasteBefore} <Keycaps combo="Ctrl + V" />{pm.pasteAfter} · <Keycaps combo="Ctrl + Shift + V" /> = Quick Price
    </p>

    {#if recent.length > 0}
      <div class="recent">
        <span class="label">{pm.recent}</span>
        <div class="row">
          {#each recent as entry (entry.id)}
            <button class="chip" onclick={() => priceCheck.reopen(entry)} title={pm.reopenSame}>
              <Icon name="clock" size={13} />
              <span style:color={RARITY_COLOR[entry.item.rarity]}>{entry.item.name ?? entry.item.base_type}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if backend.kind === 'demo'}
      <div class="demo">
        <span class="label">{pm.demoMode}</span>
        <div class="row">
          <button class="btn btn-primary btn-sm" onclick={() => priceCheck.check(SAMPLE_RARE_TEXT)}>{pm.demoRare}</button>
          <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.check(SAMPLE_CURRENCY_TEXT)}>Currency (bulk)</button>
          <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.check(SAMPLE_CURRENCY_TEXT, 'quick')}>Quick price</button>
          <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.check(`${SAMPLE_RARE_TEXT}\n${DEMO_FEW}`)}>{pm.demoFew}</button>
          <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.check(`${SAMPLE_RARE_TEXT}\n${DEMO_RATE_LIMIT}`)}>Rate limit</button>
          <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.check(`${SAMPLE_RARE_TEXT}\n${DEMO_NETWORK}`)}>{pm.demoNetwork}</button>
          <button class="btn btn-ghost btn-sm" onclick={() => priceCheck.check('texto qualquer')}>{pm.demoUnrecognized}</button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .idle { flex: 1; height: 100%; display: grid; place-items: center; padding: 20px; overflow: auto; }
  .in { width: min(620px, 100%); display: flex; flex-direction: column; align-items: center; text-align: center; gap: 12px; }
  .orb { position: relative; width: 92px; height: 92px; display: grid; place-items: center; }
  .orb::before,
  .orb::after {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 28px;
    border: 1.5px solid var(--ms-accent);
    animation: orb 3s var(--ms-ease) infinite;
  }
  .orb::after { border-color: var(--ms-suggest); animation-delay: 1.5s; }
  /* O anel cresce e some pela cor da borda (sem animar opacidade). */
  @keyframes orb {
    0% { transform: scale(0.7) rotate(0); }
    100% { transform: scale(1.5) rotate(45deg); border-color: transparent; }
  }
  h1 { font-size: 24px; font-weight: 750; letter-spacing: -0.02em; }
  .lead { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; justify-content: center; font-size: 14px; color: var(--ms-text-2); }
  .note { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; justify-content: center; font-size: 12px; color: var(--ms-text-3); }
  .recent, .demo { width: 100%; display: flex; flex-direction: column; align-items: center; gap: 10px; margin-top: 8px; }
  .demo { padding-top: 16px; border-top: 1px dashed var(--ms-border-strong); }
  .row { display: flex; gap: 8px; flex-wrap: wrap; justify-content: center; }
</style>
