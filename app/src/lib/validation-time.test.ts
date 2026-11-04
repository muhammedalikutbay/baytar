import { test } from 'node:test';
import assert from 'node:assert/strict';
// @ts-expect-error Node's native TypeScript runner requires the explicit extension.
import { parseTime } from './validation.ts';

test('normalizes compact and punctuated times', () => {
  for (const [raw, expected] of Object.entries({ '9': '09:00', '930': '09:30', '0930': '09:30', '9.30': '09:30', '9:3': '09:30', '21': '21:00', '0': '00:00', '2359': '23:59', ' 07:15 ': '07:15', '': '' })) assert.equal(parseTime(raw), expected);
});
test('rejects invalid times without clamping or discarding characters', () => {
  for (const raw of ['24', '2400', '960', '9:6', '-1', '12:345', '12345', '9:', '9am', '09/30']) assert.equal(parseTime(raw), null, raw);
});
