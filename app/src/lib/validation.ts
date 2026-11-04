import { parsePhoneNumberFromString, type CountryCode } from 'libphonenumber-js/min';

export function toE164(raw: string, country: CountryCode = 'TR'): string {
  const text = raw.trim().replace(/^00/, '+');
  if (!/^[+\d\s().-]*$/.test(text)) return '';
  const phone = parsePhoneNumberFromString(text, country);
  return phone?.isValid() ? phone.number : '';
}
export function isValidPhone(raw: string, country: CountryCode = 'TR'): boolean { return !!toE164(raw, country); }
export function normalizeEmail(raw: string): string {
  const [local, ...domain] = raw.trim().split('@');
  return domain.length ? local + '@' + domain.join('@').toLowerCase() : local;
}
export function isValidEmail(raw: string): boolean { return /^[^\s@]+@[^\s@.]+(?:\.[^\s@.]+)+$/.test(raw.trim()); }
export function isValidTckn(raw: string): boolean {
  if (!/^[1-9]\d{10}$/.test(raw)) return false;
  const d = [...raw].map(Number);
  const odd = d[0] + d[2] + d[4] + d[6] + d[8], even = d[1] + d[3] + d[5] + d[7];
  return ((odd * 7 - even) % 10 + 10) % 10 === d[9] && d.slice(0, 10).reduce((a, b) => a + b, 0) % 10 === d[10];
}
export function isValidVkn(raw: string): boolean {
  if (!/^\d{10}$/.test(raw) || /^0+$/.test(raw)) return false;
  const d = [...raw].map(Number);
  const sum = d.slice(0, 9).reduce((sum, digit, i) => {
    const n = (digit + 9 - i) % 10;
    return sum + (n === 0 ? 0 : ((n * 2 ** (9 - i)) % 9 || 9));
  }, 0);
  return (10 - sum % 10) % 10 === d[9];
}
export function normalizeMicrochip(raw: string): string { return raw.replace(/\D/g, ''); }
// No grouping or scientific notation in clinical decimals.
export function parseDecimal(raw: string): number | null {
  const text = raw.trim();
  if (!text) return null;
  if (!/^[+-]?(?:\d+(?:[.,]\d+)?|[.,]\d+)$/.test(text)) return NaN;
  const value = Number(text.replace(',', '.'));
  return Number.isFinite(value) ? value : NaN;
}
export function validateForm(form: HTMLFormElement): boolean {
  const invalid = form.querySelector<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>('input:invalid, select:invalid, textarea:invalid');
  if (!invalid) return true;
  invalid.dispatchEvent(new Event('blur'));
  invalid.focus();
  invalid.reportValidity();
  return false;
}

// Accept compact and punctuated local times; null marks invalid input, empty stays optional.
export function parseTime(raw: string): string | null {
  const text = raw.trim();
  if (!text) return '';
  let hour: string, minute: string;
  const separated = /^(\d{1,2})[:.](\d{1,2})$/.exec(text);
  if (separated) { hour = separated[1]; minute = separated[2].padEnd(2, '0'); }
  else if (/^\d{1,4}$/.test(text)) {
    hour = text.length <= 2 ? text : text.slice(0, -2);
    minute = text.length <= 2 ? '00' : text.slice(-2);
  } else return null;
  if (Number(hour) > 23 || Number(minute) > 59) return null;
  return hour.padStart(2, '0') + ':' + minute;
}
