import { backend } from '$lib/backend';
import type { GameVersion } from '$lib/types';
import { settings } from './settings.svelte';

/** Ligas de trade do jogo atual (compartilhado entre a barra de título e Configurações › Geral). */
class LeaguesStore {
  fetched = $state<string[]>([]);
  loading = $state(false);
  error = $state(false);
  private loadedFor: GameVersion | null = null;

  /** A liga salva sempre aparece, mesmo se a API não a listar (ex.: offline). */
  get list(): string[] {
    const current = settings.current?.league;
    return [...new Set([...this.fetched, ...(current ? [current] : [])])];
  }

  async load(game: GameVersion, force = false): Promise<string[]> {
    if (!force && this.loadedFor === game && this.fetched.length > 0) return this.fetched;
    this.loading = true;
    this.error = false;
    try {
      this.fetched = await backend.listLeagues(game);
      this.loadedFor = game;
    } catch {
      this.fetched = [];
      this.error = true;
    } finally {
      this.loading = false;
    }
    return this.fetched;
  }

  /** Ligas não são compartilhadas entre PoE1 e PoE2: ao trocar o jogo, vai para a liga atual. */
  async changeGame(game: GameVersion): Promise<void> {
    const current = settings.current;
    if (!current || current.game === game) return;
    const list = await this.load(game, true);
    await settings.update(list[0] && !list.includes(current.league) ? { game, league: list[0] } : { game });
  }

  async changeLeague(league: string): Promise<void> {
    await settings.update({ league });
  }
}

export const leagues = new LeaguesStore();
