import type { ApiError, DoseInput, DoseRange, DoseResult, DoseWarning } from './types';

// Mirrors core dose arithmetic for the browser preview. Contains no dose data.
export function calculateMock(input: DoseInput): DoseResult {
  function invalid(field?: string): never {
    throw { kind: 'invalid', message: 'Expected positive finite values and an ordered range.', ...(field ? { field } : {}) } satisfies ApiError;
  }
  for (const field of ['weightKg', 'mgPerKgMin', 'mgPerKgMax', 'concentrationMgPerMl', 'strengthMgPerUnit'] as const) {
    const value = input[field];
    if (value != null && (!Number.isFinite(value) || value <= 0)) invalid(field);
  }
  if (!Number.isFinite(input.weightKg) || input.weightKg <= 0 || input.weightKg > 1500) invalid('weightKg');
  if (!Number.isFinite(input.mgPerKgMin) || input.mgPerKgMin <= 0) invalid('mgPerKgMin');
  if (input.mgPerKgMax != null && input.mgPerKgMax < input.mgPerKgMin) invalid('mgPerKgMax');
  function range(min: number, max: number | null): DoseRange {
    if (!Number.isFinite(min) || min <= 0 || (max !== null && (!Number.isFinite(max) || max <= 0))) invalid();
    return { min, max };
  }
  const mg = range(input.weightKg * input.mgPerKgMin, input.mgPerKgMax == null ? null : input.weightKg * input.mgPerKgMax);
  const divide = (value: number | null) => value == null ? null : range(mg.min / value, mg.max === null ? null : mg.max / value);
  const ml = divide(input.concentrationMgPerMl);
  const units = divide(input.strengthMgPerUnit);
  const warnings: DoseWarning[] = [];
  if (ml && ml.min < 0.05) warnings.push('verySmallVolume');
  const impractical = (v: number) => v < 0.25 || Math.abs(v % 1 - Math.round((v % 1) * 4) / 4) > 1e-9;
  if (units && (impractical(units.min) || (units.max !== null && impractical(units.max)))) warnings.push('impracticalUnitCount');
  return { mg, ml, units, warnings };
}
