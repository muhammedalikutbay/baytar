import { test } from 'node:test';
import assert from 'node:assert/strict';
// @ts-expect-error Node's native TypeScript runner requires the explicit extension.
import { renderTemplate, unknownPlaceholders, PLACEHOLDERS } from './template.ts';

const values = { sahip: 'Ayşe Yılmaz', hasta: 'Pamuk', asi: 'Kuduz', tarih: '5 Kasım 2026', saat: '09:30', tutar: '1.250,00 ₺', klinik: 'Örnek Klinik', telefon: '+90 212 555 11 22' };

test('every supported placeholder is replaced, repeated ones too', () => {
  const template = PLACEHOLDERS.map((name: string) => `{${name}}`).join(' | ') + ' {sahip}';
  assert.equal(renderTemplate(template, values), 'Ayşe Yılmaz | Pamuk | Kuduz | 5 Kasım 2026 | 09:30 | 1.250,00 ₺ | Örnek Klinik | +90 212 555 11 22 Ayşe Yılmaz');
});

test('missing values leave no gaps or dangling punctuation spacing', () => {
  assert.equal(renderTemplate('Merhaba {sahip}, {hasta} için {asi} aşısı {tarih} .', { sahip: 'Ali', tarih: '1 Aralık 2026' }), 'Merhaba Ali, için aşısı 1 Aralık 2026.');
  assert.equal(renderTemplate('Teşekkürler.\n{klinik} {telefon}', { klinik: '', telefon: null }), 'Teşekkürler.');
  assert.equal(renderTemplate('Saat: {saat}.', {}), 'Saat:.');
});

test('unknown tokens are kept and reported, braces in values are not re-expanded', () => {
  assert.equal(renderTemplate('{sahip} {yanlis} {constructor}', { sahip: '{hasta}', hasta: 'Pamuk' }), '{hasta} {yanlis} {constructor}');
  assert.deepEqual(unknownPlaceholders('{sahip} {yanlis} {yanlis} {x y} {Sahip}'), ['{yanlis}', '{Sahip}']);
  assert.deepEqual(unknownPlaceholders('Merhaba {sahip}'), []);
});

test('line breaks are normalized and blank runs collapse', () => {
  assert.equal(renderTemplate('Merhaba\r\n\r\n\r\n\r\n{sahip}  \n  Sağlıklı günler', { sahip: 'Ali' }), 'Merhaba\n\nAli\nSağlıklı günler');
  assert.equal(renderTemplate('  ', {}), '');
});
