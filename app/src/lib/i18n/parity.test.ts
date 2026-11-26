import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { runInNewContext } from 'node:vm';

// Read only the source object: importing tr.ts would also initialise the browser registry.
const source = await readFile(new URL('./tr.ts', import.meta.url), 'utf8');
const start = source.indexOf('export const trSource = ') + 'export const trSource = '.length;
const end = source.indexOf('} satisfies Record<string, string>;', start) + 1;
assert.ok(start > 0 && end > start, 'Turkish source dictionary must be present');
const turkish = runInNewContext(`(${source.slice(start, end)})`) as Record<string, string>;
const tokens = (value: string) => [...new Set(value.match(/\{[^{}]+\}/g) ?? [])].sort();
const directory = new URL('./locales/', import.meta.url);
const files = (await readdir(directory)).filter(file => file.endsWith('.ts')).sort();
assert.ok(files.includes('en.ts'), 'English dictionary must be present');
for (const file of files) {
  test(`${file}: keys, non-empty strings and placeholders match Turkish`, async () => {
    const code = file.slice(0, -3);
    const module = await import(new URL(file, directory).href) as Record<string, Record<string, string>>;
    const dictionary = module[code];
    assert.ok(dictionary && typeof dictionary === 'object', `Expected named export ${code}`);
    const missing = Object.keys(turkish).filter(key => !(key in dictionary));
    const extra = Object.keys(dictionary).filter(key => !(key in turkish));
    assert.deepEqual({ missing, extra }, { missing: [], extra: [] }, `${code}: missing and extra keys`);
    for (const [key, value] of Object.entries(dictionary)) {
      assert.ok(typeof value === 'string' && value.trim(), `${code}.${key}: expected a non-empty string`);
      assert.deepEqual(tokens(value), tokens(turkish[key]), `${code}.${key}: placeholders differ`);
    }
  });
}
