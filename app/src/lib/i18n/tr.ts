export const tr = {
  appName: 'Baytar',
  tagline: 'Veteriner hasta takibi',
  version: 'Sürüm',
  versionLoading: 'Sürüm yükleniyor…',
  versionError: 'Sürüm bilgisi alınamadı.'
} as const satisfies Record<string, string>;

export type TranslationKey = keyof typeof tr;

export function t<K extends TranslationKey>(key: K): (typeof tr)[K] {
  return tr[key];
}
