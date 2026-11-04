import { test } from 'node:test';
import assert from 'node:assert/strict';
// @ts-expect-error Node's native TypeScript runner requires the explicit extension.
import { toE164, isValidPhone, normalizeEmail, isValidEmail, isValidTckn, isValidVkn, normalizeMicrochip, parseDecimal } from './validation.ts';

test('Turkish national, local and international phone inputs normalize to E.164', () => {
  for (const value of ['0555 111 22 33', '5551112233', '+90 555 111 22 33', '00905551112233']) {
    assert.equal(toE164(value), '+905551112233');
  }
  assert.equal(toE164('+49 1512 3456789'), '+4915123456789');
  assert.equal(toE164('004915123456789'), '+4915123456789');
  assert.equal(toE164('020 7946 0018', 'GB'), '+442079460018');
  for (const value of ['', '555', '055511122334', '+905551112233999', '555abc1112233']) assert.equal(isValidPhone(value), false);
});

test('email normalizes only its domain and rejects malformed addresses', () => {
  assert.equal(normalizeEmail('  Local.Tag@EXAMPLE.COM  '), 'Local.Tag@example.com');
  for (const value of ['vet@example.com', 'a+b@clinic.com.tr']) assert.equal(isValidEmail(value), true);
  for (const value of ['', 'a@@b.com', 'a@b', 'a b@c.com', 'a@b..com', '@b.com']) assert.equal(isValidEmail(value), false);
});

test('TCKN requires all checksum digits and a nonzero leading digit', () => {
  // Generated synthetic checksum fixture.
  assert.equal(isValidTckn('12345678950'), true);
  for (const value of ['12345678951', '02345678950', '1234567895', '123456789500', '1234567895a']) assert.equal(isValidTckn(value), false);
});

test('VKN validates the weighted checksum', () => {
  assert.equal(isValidVkn('1234567890'), true);
  for (const value of ['1234567891', '0000000000', '123456789', '12345678901', '123456789a']) assert.equal(isValidVkn(value), false);
});

test('microchips preserve legacy lengths while removing formatting', () => {
  assert.equal(normalizeMicrochip('900 123 456 789 012'), '900123456789012');
  assert.equal(normalizeMicrochip('old-12345'), '12345');
});

test('decimals accept comma and dot, reject grouping, letters and overflow', () => {
  assert.equal(parseDecimal(' 4,25 '), 4.25);
  assert.equal(parseDecimal('4.25'), 4.25);
  assert.equal(parseDecimal('-.5'), -0.5);
  assert.equal(parseDecimal(''), null);
  assert.equal(parseDecimal('0'), 0);
  for (const value of ['1.000,50', '4kg', '1e3', '1,2,3', '.', '9'.repeat(400)]) assert.ok(Number.isNaN(parseDecimal(value)));
});
