import { tick } from 'svelte';

import { backend } from '$lib/backend';
import type { IconName } from '$lib/ui/Icon.svelte';
import { normalize } from '$lib/format';
import { buyoutNote, suggestedFor } from '$lib/pricing';
import { history } from './history.svelte';
import { leagues } from './leagues.svelte';
import { nav } from './nav.svelte';
import { presets } from './presets.svelte';
import { priceCheck } from './priceCheck.svelte';
import { settings } from './settings.svelte';
import { toasts } from './toasts.svelte';

export interface Command {
  id: string;
  icon: IconName;
  label: string;
  detail?: string;
  kbd?: string;
  run: () => void | Promise<void>;
}

export interface CommandGroup {
  title: string;
  commands: Command[];
}

/** Copia a nota `~b/o` do nível selecionado. Compartilhado com o cartão de preço. */
export async function copySuggestedNote(): Promise<void> {
  const summary = priceCheck.output?.summary;
  const chaos = suggestedFor(summary, priceCheck.level);
  if (chaos === null || !summary) {
    toasts.push('warning', 'Poucos anúncios para sugerir preço');
    return;
  }
  const note = buyoutNote(chaos, summary.divine_chaos, settings.current?.display_currency === 'divine');
  try {
    await navigator.clipboard.writeText(note);
    toasts.push('success', `Nota copiada: ${note}`);
  } catch {
    toasts.push('error', 'Não foi possível copiar a nota');
  }
}

/** Aplica um preset ao item atual e avisa se algum mod dele não está no item. */
export function applyPresetWithToast(id: string): void {
  const preset = presets.list.find((p) => p.id === id);
  if (!preset) return;
  const missing = priceCheck.applyPreset(preset);
  if (missing > 0) {
    toasts.push('warning', missing === 1 ? '1 mod do preset não está neste item' : `${missing} mods do preset não estão neste item`);
  } else {
    toasts.push('info', `Preset “${preset.name}” aplicado — buscando…`);
  }
}

/** Paleta Ctrl+K: aberta/fechada, foco anterior e montagem dos comandos. */
class CommandsStore {
  open = $state(false);
  private previousFocus: HTMLElement | null = null;

  toggle(): void {
    if (this.open) this.close();
    else this.show();
  }

  show(): void {
    this.previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    this.open = true;
  }

  close(): void {
    if (!this.open) return;
    this.open = false;
    const el = this.previousFocus;
    this.previousFocus = null;
    void tick().then(() => el?.isConnected && el.focus());
  }

  /** Grupos de comandos para a busca. No overlay só entram os de item e presets. */
  build(query: string, role: 'main' | 'overlay'): CommandGroup[] {
    const groups: CommandGroup[] = [];
    const output = priceCheck.output;
    const hasItem = !!output && !!priceCheck.filters && output.item.rarity !== 'currency';

    if (output && priceCheck.filters) {
      const summary = output.summary;
      const chaos = suggestedFor(summary, priceCheck.level);
      const note = chaos !== null && summary ? buyoutNote(chaos, summary.divine_chaos, settings.current?.display_currency === 'divine') : null;
      const item: Command[] = [
        { id: 'search', icon: 'search', label: 'Refazer busca', run: () => priceCheck.search() },
      ];
      if (note) item.push({ id: 'copy-note', icon: 'copy', label: 'Copiar preço sugerido', detail: note, run: copySuggestedNote });
      if (hasItem) {
        item.push({
          id: 'relax',
          icon: 'reset',
          label: 'Relaxar filtros −10%',
          run: () => {
            priceCheck.relax(10);
            void priceCheck.search();
          },
        });
      }
      const url = output.trade?.web_url;
      if (url) item.push({ id: 'trade', icon: 'ext', label: 'Abrir no site de trade', run: () => backend.openExternal(url) });
      groups.push({ title: `Item atual · ${output.item.name ?? output.item.base_type}`, commands: item });
    }

    if (hasItem && presets.list.length > 0) {
      groups.push({
        title: `Presets · ${presets.game === 'poe2' ? 'PoE 2' : 'PoE 1'}`,
        commands: presets.list.map((p) => ({
          id: `preset-${p.id}`,
          icon: 'layers',
          label: `Aplicar preset “${p.name}”`,
          detail: `${p.mods.length} ${p.mods.length === 1 ? 'mod' : 'mods'}`,
          run: () => applyPresetWithToast(p.id),
        })),
      });
    }

    if (role === 'main') {
      groups.push({
        title: 'Ir para',
        commands: [
          { id: 'go-price', icon: 'tag', label: 'Price Check', run: () => nav.go('price') },
          { id: 'go-history', icon: 'clock', label: 'Histórico', detail: `${history.entries.length} price checks`, run: () => nav.go('history') },
          { id: 'go-settings', icon: 'gear', label: 'Configurações', run: () => nav.openSettings() },
        ],
      });

      const game = settings.current?.game ?? 'poe1';
      const other = game === 'poe1' ? 'poe2' : 'poe1';
      const system: Command[] = [
        { id: 'game', icon: 'reset', label: `Trocar jogo para PoE ${other === 'poe2' ? '2' : '1'}`, run: () => leagues.changeGame(other) },
        { id: 'league', icon: 'layers', label: 'Trocar liga…', detail: settings.current?.league, run: () => void tick().then(() => (nav.leagueMenuOpen = true)) },
      ];
      // Com o histórico vazio não há o que confirmar (o pedido ficaria pendente para depois).
      if (history.entries.length > 0) {
        system.push({
          id: 'clear-history',
          icon: 'trash',
          label: 'Limpar histórico',
          run: () => {
            nav.go('history');
            nav.confirmClearHistory = true;
          },
        });
      }
      groups.push({ title: 'Sistema', commands: system });

      if (query.trim()) {
        groups.push({
          title: 'Histórico',
          // Parser, catálogo e liga dependem do jogo: como no HistoryView, só reabre itens do jogo atual.
          commands: history.entries.filter((e) => e.game === settings.current?.game).map((e) => ({
            id: `hist-${e.id}`,
            icon: 'clock',
            label: `Reabrir ${e.item.name ?? e.item.base_type}`,
            detail: e.item.name ? e.item.base_type : undefined,
            run: () => {
              nav.go('price');
              void priceCheck.reopen(e);
            },
          })),
        });
      }
    }

    const q = normalize(query.trim());
    if (!q) return groups;
    const terms = q.split(/\s+/);
    return groups
      .map((g) => ({
        title: g.title,
        commands: g.commands.filter((c) => {
          const hay = normalize(`${c.label} ${c.detail ?? ''} ${g.title}`);
          return terms.every((t) => hay.includes(t));
        }),
      }))
      .filter((g) => g.commands.length > 0);
  }
}

export const commands = new CommandsStore();
