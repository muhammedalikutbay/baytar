import * as api from '$lib/api/lock';
import type { LockStatus } from '$lib/api/types';

const CHECK_INTERVAL_MS = 15_000;

/** App lock state shared by the layout, the lock screen and the settings card. */
class LockStore {
  status = $state<LockStatus>({ enabled: false, idleMinutes: 0, waitSeconds: 0 });
  locked = $state(false);
  ready = $state(false);
  #lastActivity = Date.now();

  async refresh() {
    try { this.status = await api.status(); }
    catch (error) { console.error('Failed to read lock status', error); }
    return this.status;
  }
  /** Reads the status once at start; the app is locked right away when a PIN is set. */
  async init() {
    await this.refresh();
    this.locked = this.status.enabled;
    this.ready = true;
  }
  lock() { if (this.status.enabled) this.locked = true; }
  unlock() { this.locked = false; this.#lastActivity = Date.now(); }

  #check = () => {
    const minutes = this.status.idleMinutes;
    if (this.locked || !this.status.enabled || minutes <= 0) return;
    if (Date.now() - this.#lastActivity >= minutes * 60_000) this.locked = true;
  };
  /** Starts idle tracking. A minimised window keeps its last activity time, so the elapsed time counts. */
  watch() {
    const touch = () => { if (!this.locked) this.#lastActivity = Date.now(); };
    const visibility = () => { if (document.visibilityState === 'visible') this.#check(); };
    const events = ['pointerdown', 'pointermove', 'keydown', 'wheel', 'touchstart'] as const;
    for (const name of events) window.addEventListener(name, touch, { passive: true });
    document.addEventListener('visibilitychange', visibility);
    window.addEventListener('focus', this.#check);
    const timer = setInterval(this.#check, CHECK_INTERVAL_MS);
    return () => {
      for (const name of events) window.removeEventListener(name, touch);
      document.removeEventListener('visibilitychange', visibility);
      window.removeEventListener('focus', this.#check);
      clearInterval(timer);
    };
  }
}
export const lock = new LockStore();