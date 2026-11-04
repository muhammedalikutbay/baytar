import { test } from 'node:test';
import assert from 'node:assert/strict';
// @ts-expect-error Node's native TypeScript runner requires the explicit extension.
import { visitDefaults, isPastClinicalRecord, weightSeries } from './clinical.ts';
test('new visit defaults preserve today and never round into the future', () => {
  assert.deepEqual(visitDefaults(new Date(2026, 10, 4, 23, 59)), { date: '2026-11-04', time: '23:55' });
  assert.deepEqual(visitDefaults(new Date(2026, 10, 4, 9, 12)), { date: '2026-11-04', time: '09:10' });
});
test('clinical history excludes future dates and future times today', () => {
  const now = new Date(2026, 10, 4, 12, 0);
  assert.equal(isPastClinicalRecord('2026-11-03', '23:00', now), true);
  assert.equal(isPastClinicalRecord('2026-11-04', null, now), true);
  assert.equal(isPastClinicalRecord('2026-11-04', '13:00', now), false);
  assert.equal(isPastClinicalRecord('2026-11-05', null, now), false);
});
test('weight chart sorts dates, spaces by elapsed time and reports the latest change', () => {
  const series = weightSeries([{ date: '2026-11-04', kg: 4 }, { date: '2026-11-01', kg: 4.5 }, { date: '2026-11-02', kg: 4.2 }])!;
  assert.equal(series.last.kg, 4);
  assert.ok(Math.abs(series.change! + 0.2) < 0.00001);
  assert.ok(Math.abs((series.points[1].x - series.points[0].x) * 2 - (series.points[2].x - series.points[1].x)) < 0.00001);
  assert.equal(weightSeries([]), null);
  assert.equal(weightSeries([{ date: 'invalid', kg: 4 }]), null);
  assert.equal(weightSeries([{ date: '2026-11-04', kg: 4 }])!.change, null);
});

test('latest weight respects same-day visit time and stable record order', () => {
  const series = weightSeries([{ date: '2026-11-04', time: '16:00', id: 3, kg: 4.2 }, { date: '2026-11-04', time: '08:00', id: 2, kg: 4 }])!;
  assert.equal(series.last.kg, 4.2);
  assert.ok(Math.abs(series.change! - 0.2) < 0.00001);
});
