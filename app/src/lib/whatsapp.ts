import { isTauri } from '@tauri-apps/api/core';
import { getAll, openWhatsapp } from '$lib/api/settings';
import { tr } from '$lib/i18n/tr';
export type WhatsappMode = 'auto' | 'app' | 'web';
export const whatsappModes = ['auto', 'app', 'web'] as const;
export const whatsappModeLabel = { auto: tr.whatsappAuto, app: tr.whatsappApp, web: tr.whatsappWeb };
export function parseWhatsappMode(value?: string): WhatsappMode {
  return value === 'app' || value === 'web' ? value : 'auto';
}
export function whatsappUrl(phone: string, text: string, mode: WhatsappMode = 'auto'): string {
  if (!/^\+?[1-9]\d{6,14}$/.test(phone)) throw new Error('Invalid E.164 phone');
  const digits = phone.replace(/^\+/, '');
  const encoded = encodeURIComponent(text);
  if (mode === 'app') return 'whatsapp://send?phone=' + digits + '&text=' + encoded;
  if (mode === 'web') return 'https://web.whatsapp.com/send?phone=' + digits + '&text=' + encoded;
  return 'https://wa.me/' + digits + '?text=' + encoded;
}
export async function sendWhatsapp(phone: string, text: string, mode?: WhatsappMode): Promise<void> {
  const selected = mode ?? parseWhatsappMode((await getAll()).whatsapp_mode);
  const url = whatsappUrl(phone, text, selected);
  if (isTauri()) await openWhatsapp(url);
  else window.open(url, '_blank', 'noopener,noreferrer');
}
