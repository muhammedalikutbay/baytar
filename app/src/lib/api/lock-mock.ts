import type { ApiError, LockStatus, LockVerifyResult } from './types';
interface Context { fail(kind: ApiError['kind'], message: string, field?: string): never }
/** Browser-only mock. The PIN lives in memory (demo PIN 1234 once enabled by the user), never persisted. */
export function createLockMock(ctx: Context) {
  let pin: string | null = null, idle = 0, failures = 0, blockedUntil = 0;
  const reject = (code: string): never => { throw code; };
  const wait = () => Math.max(0, Math.ceil((blockedUntil - Date.now()) / 1000));
  const validPin = (value: string) => /^\d{4,8}$/.test(value);
  function check(value: string | null | undefined): void {
    if (wait() > 0) reject(`wait:${wait()}`);
    if (value !== pin) { fail(); reject(wait() > 0 ? `wait:${wait()}` : 'wrong_pin'); }
    failures = 0;
  }
  function fail() { failures++; if (failures >= 5) blockedUntil = Date.now() + Math.min(30_000 * 2 ** (failures - 5), 300_000); }
  const handlers: Record<string, (args: unknown) => unknown> = {
    lock_status: (): LockStatus => ({ enabled: pin !== null, idleMinutes: idle, waitSeconds: wait() }),
    lock_set_pin: args => {
      const { pin: next, currentPin } = args as { pin: string; currentPin?: string | null };
      if (!validPin(next)) reject('invalid_pin');
      if (pin !== null) check(currentPin);
      pin = next; return null;
    },
    lock_disable: args => { check((args as { pin: string }).pin); pin = null; return null; },
    lock_verify: (args): LockVerifyResult => {
      if (pin === null) reject('not_enabled');
      if (wait() > 0) return { ok: false, waitSeconds: wait() };
      if ((args as { pin: string }).pin === pin) { failures = 0; return { ok: true, waitSeconds: 0 }; }
      fail(); return { ok: false, waitSeconds: wait() };
    },
    lock_set_idle: args => {
      const { minutes } = args as { minutes: number };
      if (!Number.isInteger(minutes) || minutes < 0 || minutes > 240) reject('invalid_idle');
      idle = minutes; return null;
    }
  };
  void ctx;
  return { has: (command: string) => command in handlers, call: (command: string, args: unknown) => handlers[command](args) };
}