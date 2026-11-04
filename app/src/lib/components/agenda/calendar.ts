import { parseDate, addDateDays } from '$lib/format';

// Anchor calendar repeats to the original date, clamping only the target month.
export function addMonths(date: string, months: number): string {
  const start = parseDate(date);
  if (!start || !Number.isSafeInteger(months)) return '';
  const target = new Date(start);
  target.setUTCDate(1);
  target.setUTCMonth(target.getUTCMonth() + months);
  const end = new Date(target);
  end.setUTCMonth(end.getUTCMonth() + 1);
  end.setUTCDate(0);
  target.setUTCDate(Math.min(start.getUTCDate(), end.getUTCDate()));
  const year = target.getUTCFullYear();
  return year >= 1 && year <= 9999 ? target.toISOString().slice(0, 10) : '';
}

export function seriesDates(date: string, count: number, days: number, months?: number): string[] {
  return Array.from({ length: count }, (_, index) => months ? addMonths(date, index * months) : addDateDays(date, index * days));
}
