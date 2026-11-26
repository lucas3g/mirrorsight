import { settings } from '$lib/stores/settings.svelte';

/**
 * Idiomas da interface — os mesmos dos READMEs do projeto.
 * Espelho de `UI_LANGUAGES` (crates/domain/src/settings.rs).
 */
export const LOCALES = [
  { code: 'en', label: 'English' },
  { code: 'pt-BR', label: 'Português (Brasil)' },
  { code: 'es', label: 'Español' },
  { code: 'de', label: 'Deutsch' },
  { code: 'zh-CN', label: '简体中文' },
  { code: 'ja', label: '日本語' },
  { code: 'ko', label: '한국어' },
] as const;

export type Locale = (typeof LOCALES)[number]['code'];

const CODES = LOCALES.map((l) => l.code) as readonly string[];
const FALLBACK: Locale = 'en';

function isLocale(code: string): code is Locale {
  return CODES.includes(code);
}

/** "pt-PT" → "pt-BR", "zh-TW"/"zh" → "zh-CN", "de-AT" → "de"… */
function matchSystem(tag: string): Locale | null {
  if (isLocale(tag)) return tag;
  const base = tag.toLowerCase().split('-')[0];
  return LOCALES.find((l) => l.code.toLowerCase().split('-')[0] === base)?.code ?? null;
}

/** Preferência salva (`null` = automático) → idioma efetivo. */
export function resolveLocale(pref: string | null | undefined): Locale {
  if (pref && isLocale(pref)) return pref;
  const tags = typeof navigator === 'undefined' ? [] : (navigator.languages ?? [navigator.language]);
  for (const tag of tags) {
    const hit = tag ? matchSystem(tag) : null;
    if (hit) return hit;
  }
  return FALLBACK;
}

class I18nStore {
  /** Idioma efetivo da interface; reativo (deriva das configurações). */
  get locale(): Locale {
    return resolveLocale(settings.current?.ui_language);
  }
}

export const i18n = new I18nStore();

/** Um texto por idioma; `pt-BR` define o formato e os demais precisam bater. */
export type Translations<T> = { 'pt-BR': T } & { [K in Exclude<Locale, 'pt-BR'>]: NoInfer<T> };

/**
 * Declara as mensagens de uma área. Devolve um objeto que lê sempre do idioma atual
 * (reativo em templates/`$derived`): `{m.title}`, `{m.found(3)}`.
 * Mensagens com parâmetro são funções — plural e ordem das palavras ficam por idioma.
 */
export function defineMessages<T extends object>(dict: Translations<T>): T {
  return new Proxy({} as T, {
    get(_target, key) {
      const table = (dict[i18n.locale] ?? dict[FALLBACK]) as Record<PropertyKey, unknown>;
      return table[key] ?? (dict[FALLBACK] as Record<PropertyKey, unknown>)[key];
    },
  });
}
