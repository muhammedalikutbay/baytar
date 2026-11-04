import { addDateDays, dateAfterMonths, daysUntil, parseDate } from '$lib/format';

export type PeriodKind = 'month' | 'last3' | 'year' | 'lastYear' | 'custom';
export interface DateRange { from: string; to: string }

/** The longest custom range the page asks for (the backend allows at most 120 months). */
export const MAX_RANGE_DAYS = 3650;

const firstOfMonth = (day: string) => day.slice(0, 8) + '01';

export function periodRange(kind: PeriodKind, today: string, custom: DateRange): DateRange {
  const year = today.slice(0, 4);
  switch (kind) {
    case 'month': return { from: firstOfMonth(today), to: today };
    case 'last3': return { from: dateAfterMonths(firstOfMonth(today), -2), to: today };
    case 'year': return { from: `${year}-01-01`, to: today };
    case 'lastYear': return { from: `${Number(year) - 1}-01-01`, to: `${Number(year) - 1}-12-31` };
    case 'custom': return custom;
  }
}

/** Why a range cannot be reported on, or null when it is usable. */
export function rangeProblem(range: DateRange): 'invalid' | 'order' | 'long' | null {
  if (!parseDate(range.from) || !parseDate(range.to)) return 'invalid';
  if (range.from > range.to) return 'order';
  return (daysUntil(range.to, range.from) ?? 0) > MAX_RANGE_DAYS ? 'long' : null;
}

/** The period right before `range`; calendar periods shift by whole months so they compare like for like. */
export function previousRange(kind: PeriodKind, range: DateRange): DateRange {
  const shift = kind === 'custom' ? 0 : { month: 1, last3: 3, year: 12, lastYear: 12 }[kind];
  if (shift) return { from: dateAfterMonths(range.from, -shift), to: dateAfterMonths(range.to, -shift) };
  const days = (daysUntil(range.to, range.from) ?? 0) + 1;
  return { from: addDateDays(range.from, -days), to: addDateDays(range.from, -1) };
}
