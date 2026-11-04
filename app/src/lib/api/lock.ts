import { call } from './client';
export const status = () => call('lock_status', {});
export const setPin = (pin: string, currentPin?: string) => call('lock_set_pin', { pin, currentPin: currentPin ?? null });
export const disable = (pin: string) => call('lock_disable', { pin });
export const verify = (pin: string) => call('lock_verify', { pin });
export const setIdle = (minutes: number) => call('lock_set_idle', { minutes });

/** Backend rejections are string codes: `wait:<seconds>`, `wrong_pin`, `invalid_pin`, ... */
export function errorCode(error: unknown): { code: string; waitSeconds: number } {
  const raw = typeof error === 'string' ? error : (error as { message?: string } | null)?.message ?? 'internal';
  if (raw.startsWith('wait:')) return { code: 'wait', waitSeconds: Number(raw.slice(5)) || 0 };
  return { code: raw, waitSeconds: 0 };
}