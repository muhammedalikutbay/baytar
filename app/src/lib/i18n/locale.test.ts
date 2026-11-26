import { test } from 'node:test';
import assert from 'node:assert/strict';

test('locale precedence, persistence, reloads and storage recovery', async () => {
  const globals = ['window', 'document', 'navigator', 'localStorage', 'history'] as const;
  const original = new Map(globals.map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  let sequence = 0;
  async function boot(query: string, saved: string | null, languages: string[], blockedStorage = false) {
    const storage = new Map<string, string>(saved === null ? [] : [['baytar.locale', saved]]);
    const html = { lang: '' };
    const location = { href: `https://baytar.test/settings${query}`, reloads: 0, reload() { this.reloads++; } };
    const values = {
      window: { location }, document: { documentElement: html }, navigator: { languages, language: languages[0] ?? 'en' },
      localStorage: {
        getItem(key: string) { if (blockedStorage) throw new Error('Storage unavailable'); return storage.get(key) ?? null; },
        setItem(key: string, value: string) { if (blockedStorage) throw new Error('Storage unavailable'); storage.set(key, value); }
      },
      history: { replaceState(_state: unknown, _title: string, url: URL) { location.href = url.href; } }
    };
    for (const key of globals) Object.defineProperty(globalThis, key, { configurable: true, value: values[key] });
    const module = await import(new URL(`./index.ts?test=${++sequence}`, import.meta.url).href) as typeof import('./index');
    return { module, storage, location, html };
  }
  try {
    let context = await boot('?lang=fr', 'tr', ['de-DE']);
    assert.equal(context.module.locale, 'fr');
    assert.equal(context.module.intlLocale(), 'fr-FR');
    assert.equal(context.html.lang, 'fr');
    assert.equal(context.storage.get('baytar.locale'), 'fr');
    await context.module.setLocale('es');
    assert.equal(context.storage.get('baytar.locale'), 'es');
    assert.equal(new URL(context.location.href).searchParams.get('lang'), 'es');
    assert.equal(context.location.reloads, 1);

    context = await boot('?lang=unsupported', 'tr', ['de-DE']);
    assert.equal(context.module.locale, 'tr');
    assert.equal(await context.module.restoreLocale({ ui_locale: 'es' }), false);
    context = await boot('', null, ['ja-JP', 'de-AT', 'fr-CA']);
    assert.equal(context.module.locale, 'de');
    assert.equal(await context.module.restoreLocale({ ui_locale: 'es' }), true);
    assert.equal(context.storage.get('baytar.locale'), 'es');
    assert.equal(context.location.reloads, 1);

    context = await boot('', 'invalid', ['ja-JP']);
    assert.equal(context.module.locale, 'en');
    await context.module.setLocale('invalid');
    assert.equal(context.location.reloads, 0);

    context = await boot('', null, ['fr-FR'], true);
    assert.equal(await context.module.restoreLocale({ ui_locale: 'tr' }), true);
    assert.equal(new URL(context.location.href).searchParams.get('lang'), 'tr');
    context = await boot('?lang=tr', null, ['fr-FR'], true);
    assert.equal(context.module.locale, 'tr');
    assert.equal(await context.module.restoreLocale({ ui_locale: 'tr' }), false);
    assert.equal(context.location.reloads, 0);
  } finally {
    for (const key of globals) {
      const descriptor = original.get(key);
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  }
});
