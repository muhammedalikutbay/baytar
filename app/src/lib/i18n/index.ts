/** Locale selection is synchronous so module-level labels share one dictionary. */
export const locales = [
  { code: 'tr', name: 'Türkçe', intl: 'tr-TR' },
  { code: 'en', name: 'English', intl: 'en-GB' },
  { code: 'de', name: 'Deutsch', intl: 'de-DE' },
  { code: 'es', name: 'Español', intl: 'es-ES' },
  { code: 'fr', name: 'Français', intl: 'fr-FR' }
] as const;
export type Locale = (typeof locales)[number]['code'];
export const isLocale = (code: unknown): code is Locale => locales.some(item => item.code === code);
const browser = typeof window !== 'undefined';
export function readPreference(key: string): string | null {
  try { return browser ? localStorage.getItem(key) : null; } catch { return null; }
}
export function cachePreference(key: string, value: string): void {
  try { if (browser) localStorage.setItem(key, value); } catch { /* Settings remain the durable copy. */ }
}
const query = browser ? new URL(window.location.href).searchParams.get('lang') : null;
const saved = readPreference('baytar.locale');
const detected = browser ? navigator.languages.map(language => language.split('-')[0].toLowerCase()).find(isLocale) : undefined;
export const locale: Locale = isLocale(query) ? query : isLocale(saved) ? saved : detected ?? 'en';
if (browser) {
  document.documentElement.lang = locale;
  if (isLocale(query)) cachePreference('baytar.locale', query);
}
export const intlLocale = (): string => locales.find(item => item.code === locale)!.intl;

function reloadWithLocale(code: Locale): void {
  const url = new URL(window.location.href);
  if (url.searchParams.has('lang') || readPreference('baytar.locale') !== code) {
    url.searchParams.set('lang', code);
    history.replaceState(null, '', url);
  }
  window.location.reload();
}

async function persistLocale(code: Locale): Promise<void> {
  if (browser && '__TAURI_INTERNALS__' in window) {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('settings_set', { key: 'ui_locale', value: code });
  }
}
export async function setLocale(code: string): Promise<void> {
  if (!isLocale(code) || !browser) return;
  await persistLocale(code);
  cachePreference('baytar.locale', code);
  // Keep screenshot overrides consistent; the URL also works when storage is unavailable.
  reloadWithLocale(code);
}

/** Restore the durable setting after a WebView reset, before mounting the workspace. */
export async function restoreLocale(values: Record<string, string>): Promise<boolean> {
  if (isLocale(query)) { await persistLocale(query); return false; }
  if (!isLocale(saved) && isLocale(values.ui_locale)) {
    cachePreference('baytar.locale', values.ui_locale);
    if (values.ui_locale !== locale) { reloadWithLocale(values.ui_locale); return true; }
  }
  return false;
}
