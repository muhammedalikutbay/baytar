import { isTauri } from '@tauri-apps/api/core';
import { check as checkForUpdate, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { backupBeforeUpdate, getAll } from '$lib/api/settings';
import { tr } from '$lib/i18n/tr';

type CheckResult = 'idle' | 'current' | 'available' | 'error';
type Phase = 'idle' | 'backup' | 'download' | 'install' | 'restart';

function createUpdater() {
  let update = $state.raw<Update | null>(null);
  let checking = $state(false);
  let result = $state<CheckResult>('idle');
  let phase = $state<Phase>('idle');
  let banner = $state(false);
  let installError = $state('');
  let downloaded = $state(0);
  let total = $state<number | undefined>();
  let installed = $state(false);
  let startupAttempted = false;
  let hasChecked = false;
  const busy = $derived(phase !== 'idle');
  const notes = $derived(update?.body?.trim().slice(0, 300) ?? '');
  const progress = $derived(
    phase === 'backup' ? tr.updateBackingUp
      : phase === 'download' ? (total
        ? tr.updateDownloadingPercent.replace('{percent}', String(Math.min(100, Math.floor(downloaded / total * 100))))
        : tr.updateDownloading)
      : phase === 'install' ? tr.updateInstalling
      : phase === 'restart' ? tr.updateRestarting : ''
  );

  async function check({ silent = false }: { silent?: boolean } = {}) {
    if (typeof window === 'undefined' || !isTauri() || checking || busy || installed) return;
    // An explicit check also satisfies this session's scheduled startup check.
    if (silent && hasChecked) return;
    checking = true;
    try {
      if (silent && (await getAll()).update_check_enabled === '0') return;
      hasChecked = true;
      const next = await checkForUpdate({ timeout: 15000 });
      const previous = update;
      update = next;
      result = next ? 'available' : 'current';
      banner = next !== null;
      installError = '';
      if (previous) {
        await previous.close().catch(() => console.error('Failed to release previous update resource.'));
      }
    } catch {
      console.error('Update check failed.');
      if (!silent) result = 'error';
    } finally {
      checking = false;
    }
  }

  async function install() {
    if (typeof window === 'undefined' || !isTauri() || !update || checking || busy) return;
    banner = true;
    installError = '';
    try {
      if (!installed) {
        phase = 'backup';
        // Never start the installer unless a durable database snapshot succeeded.
        await backupBeforeUpdate();
        phase = 'download';
        downloaded = 0;
        total = undefined;
        await update.downloadAndInstall(event => {
          if (event.event === 'Started') total = event.data.contentLength;
          else if (event.event === 'Progress') downloaded += event.data.chunkLength;
          else if (event.event === 'Finished') phase = 'install';
        });
        installed = true;
      }
      // Windows exits into its installer; macOS/Linux return here and need relaunch.
      phase = 'restart';
      await relaunch();
    } catch {
      console.error('Update installation or restart failed.');
      installError = phase === 'backup' ? tr.updateBackupError
        : phase === 'restart' ? tr.updateRestartError : tr.updateInstallError;
    } finally {
      phase = 'idle';
    }
  }

  return {
    get checking() { return checking; },
    get result() { return result; },
    get version() { return update?.version ?? ''; },
    get notes() { return notes; },
    get banner() { return banner; },
    get busy() { return busy; },
    get progress() { return progress; },
    get error() { return installError; },
    get actionLabel() { return installed ? tr.updateRestart : tr.updateNow; },
    check,
    install,
    dismiss() { if (!busy) banner = false; },
    init() {
      if (typeof window === 'undefined' || !isTauri() || startupAttempted) return () => {};
      const timer = window.setTimeout(() => {
        startupAttempted = true;
        void check({ silent: true });
      }, 3500);
      return () => window.clearTimeout(timer);
    }
  };
}

export const updater = createUpdater();
