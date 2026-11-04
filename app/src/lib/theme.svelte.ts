export type ThemePreference = 'system' | 'light' | 'dark';
export type ResolvedTheme = Exclude<ThemePreference, 'system'>;

const storageKey = 'baytar.theme';

function parse(value: string | null): ThemePreference {
  return value === 'light' || value === 'dark' ? value : 'system';
}

function createTheme() {
  let preference = $state<ThemePreference>('system');
  let systemDark = $state(false);
  const resolved = $derived<ResolvedTheme>(preference === 'system' ? (systemDark ? 'dark' : 'light') : preference);

  function apply() {
    if (typeof document !== 'undefined') document.documentElement.dataset.theme = resolved;
  }

  return {
    get preference() { return preference; },
    get resolved() { return resolved; },
    set(value: ThemePreference) {
      preference = value;
      apply();
      try { localStorage.setItem(storageKey, value); } catch { /* Keep working without persistence. */ }
    },
    // Initialize once from the root layout; clean up during HMR and unmounts.
    init() {
      const media = window.matchMedia('(prefers-color-scheme: dark)');
      systemDark = media.matches;
      try { preference = parse(localStorage.getItem(storageKey)); } catch { /* Use system. */ }
      apply();
      const onSystemChange = (event: MediaQueryListEvent) => { systemDark = event.matches; apply(); };
      const onStorage = (event: StorageEvent) => {
        if (event.key === storageKey || event.key === null) { preference = parse(event.newValue); apply(); }
      };
      media.addEventListener('change', onSystemChange);
      window.addEventListener('storage', onStorage);
      return () => {
        media.removeEventListener('change', onSystemChange);
        window.removeEventListener('storage', onStorage);
      };
    }
  };
}

export const theme = createTheme();
