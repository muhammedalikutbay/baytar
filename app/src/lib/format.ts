import { tr } from '$lib/i18n/tr';

export function todayDate(): string {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

export function dateAfterDays(value: string, days: number): string {
  const date = parseDate(value);
  if (!date) return '';
  date.setUTCDate(date.getUTCDate() + days);
  return date.toISOString().slice(0, 10);
}

export function dateAfterMonths(value: string, months: number): string {
  const date = parseDate(value);
  if (!date) return '';
  const day = date.getUTCDate();
  date.setUTCDate(1);
  date.setUTCMonth(date.getUTCMonth() + months);
  const end = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 0)).getUTCDate();
  date.setUTCDate(Math.min(day, end));
  return date.toISOString().slice(0, 10);
}

export function daysUntil(value: string, today = todayDate()): number | null {
  const due = parseDate(value), start = parseDate(today);
  return due && start ? Math.round((due.getTime() - start.getTime()) / 86400000) : null;
}

export function whatsappPhone(value: string | null | undefined): string | null {
  let digits = value?.replace(/\D/g, '') ?? '';
  if (digits.startsWith('0090')) digits = digits.slice(2);
  if (digits.length === 11 && digits.startsWith('0')) digits = digits.slice(1);
  if (digits.length === 10) digits = '90' + digits;
  return /^90[1-9]\d{9}$/.test(digits) ? digits : null;
}

export function formatPhone(value: string | null | undefined): string {
  if (!value) return '';
  let digits = value.replace(/\D/g, '');
  if (digits.length === 12 && digits.startsWith('90')) digits = '0' + digits.slice(2);
  if (digits.length === 10) digits = '0' + digits;
  return digits.length === 11 && digits.startsWith('0')
    ? `${digits.slice(0, 4)} ${digits.slice(4, 7)} ${digits.slice(7, 9)} ${digits.slice(9)}` : value;
}

function parseDate(value: string): Date | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return null;
  const date = new Date(value + 'T00:00:00Z');
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value ? date : null;
}

export function formatDate(value: string | null | undefined): string {
  if (!value) return '';
  const date = parseDate(value);
  return date ? new Intl.DateTimeFormat('tr-TR', { day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' }).format(date) : value;
}

export function formatAge(value: string | null | undefined, estimate = false, today = new Date()): string {
  if (!value) return tr.unknown;
  const birth = parseDate(value);
  if (!birth) return tr.unknown;
  const current = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
  if (birth.getTime() > current) return tr.unknown;
  let months = (today.getFullYear() - birth.getUTCFullYear()) * 12 + today.getMonth() - birth.getUTCMonth();
  if (today.getDate() < birth.getUTCDate()) months--;
  const age = months >= 12 ? `${Math.floor(months / 12)} ${tr.yearsOld}`
    : months > 0 ? `${months} ${tr.monthsOld}` : `${Math.floor((current - birth.getTime()) / 86400000)} ${tr.daysOld}`;
  return (estimate ? '~' : '') + age;
}
